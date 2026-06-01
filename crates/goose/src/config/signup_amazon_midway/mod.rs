//! Amazon-internal authentication via the `ada` CLI.
//!
//! This is the third onboarding path (alongside IDC device flow and the
//! default AWS credential chain). It targets Amazon employees who already
//! authenticate to Midway via `mwinit` and use `ada` to mint temporary AWS
//! credentials from Isengard or Conduit.
//!
//! Strategy:
//!   1. Detect `ada` in PATH.
//!   2. Probe `ada credentials print --account=X --role=Y --provider=...`
//!      with the user-supplied account+role to confirm Midway is fresh and
//!      the role is reachable.
//!   3. Write a profile to `~/.aws/credentials` that uses
//!      `credential_process = ada credentials print ...`. The AWS SDK calls
//!      `ada` on every API request, so credentials refresh automatically.
//!   4. Persist `AWS_PROFILE=<profile>` in the goose config.
//!
//! No static AWS credentials ever touch the goose config — everything flows
//! through `ada credentials print` at request time.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use utoipa::ToSchema;

use crate::config::Config;
use crate::providers::bedrock::{BEDROCK_DEFAULT_MODEL, BEDROCK_PROVIDER_NAME};

/// Amazon-internal credential providers supported by `ada`.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AdaProvider {
    Isengard,
    Conduit,
}

impl AdaProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Isengard => "isengard",
            Self::Conduit => "conduit",
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct MidwayLoginRequest {
    pub account_id: String,
    pub role_name: String,
    pub provider: AdaProvider,
    /// Optional region override; defaults to us-west-2 (Amazon convention).
    pub region: Option<String>,
    /// Optional profile name to write into ~/.aws/credentials. Defaults to
    /// `goose-midway`.
    pub profile_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct MidwayLoginResponse {
    pub success: bool,
    pub message: String,
    /// AWS account/role the credentials prove access to (set on success).
    pub identity: Option<String>,
}

/// Returns true if the `ada` CLI is on PATH.
pub fn ada_is_installed() -> bool {
    which::which("ada").is_ok()
}

/// Probe the AWS default credential chain by calling sts:GetCallerIdentity.
/// Used by the "Default AWS credentials" onboarding option.
pub async fn probe_default_aws_credentials() -> Result<String> {
    use aws_config::BehaviorVersion;
    use aws_sdk_sts::Client;
    use smithy_transport_reqwest::ReqwestHttpClient;

    let cfg = aws_config::defaults(BehaviorVersion::latest())
        .http_client(ReqwestHttpClient::new())
        .load()
        .await;
    let sts = Client::new(&cfg);
    let out = sts
        .get_caller_identity()
        .send()
        .await
        .context("sts:GetCallerIdentity failed")?;
    Ok(out.arn().unwrap_or_default().to_string())
}

/// Path to ~/.aws/credentials honoring AWS_SHARED_CREDENTIALS_FILE.
fn aws_credentials_path() -> PathBuf {
    if let Ok(p) = std::env::var("AWS_SHARED_CREDENTIALS_FILE") {
        return PathBuf::from(p);
    }
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "~".to_string());
    PathBuf::from(home).join(".aws").join("credentials")
}

/// Run `ada credentials print --once` once to verify the user can actually
/// assume the requested role. The output is JSON; we just need the
/// AccessKeyId field to confirm success.
pub fn probe_ada_credentials(req: &MidwayLoginRequest) -> Result<()> {
    if !ada_is_installed() {
        return Err(anyhow!(
            "`ada` is not installed or not on PATH. Run `toolbox install ada` first."
        ));
    }

    let output = Command::new("ada")
        .arg("credentials")
        .arg("print")
        .arg(format!("--account={}", req.account_id))
        .arg(format!("--role={}", req.role_name))
        .arg(format!("--provider={}", req.provider.as_str()))
        .output()
        .context("failed to invoke ada")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // ada often prompts the user to run mwinit; surface that verbatim.
        return Err(anyhow!("ada credentials probe failed:\n{}", stderr.trim()));
    }

    // Sanity-check the JSON shape so misconfigured ada output fails clearly.
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value =
        serde_json::from_str(&stdout).context("ada returned non-JSON output")?;
    if parsed.get("AccessKeyId").is_none() {
        return Err(anyhow!(
            "ada output did not contain AccessKeyId; got: {}",
            stdout.trim()
        ));
    }
    Ok(())
}

/// Append (or replace) a profile in ~/.aws/credentials that uses
/// `credential_process = ada credentials print ...` so the SDK refreshes
/// credentials automatically on every request.
pub fn write_credential_process_profile(req: &MidwayLoginRequest) -> Result<String> {
    let profile_name = req
        .profile_name
        .clone()
        .unwrap_or_else(|| "goose-midway".to_string());

    let credentials_path = aws_credentials_path();
    if let Some(parent) = credentials_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create {} for AWS credentials", parent.display())
        })?;
    }

    let existing = std::fs::read_to_string(&credentials_path).unwrap_or_default();

    // Strip any existing block for this profile name.
    let header = format!("[{}]", profile_name);
    let mut filtered = String::new();
    let mut skipping = false;
    for line in existing.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            skipping = trimmed == header;
        }
        if !skipping {
            filtered.push_str(line);
            filtered.push('\n');
        }
    }
    while filtered.ends_with("\n\n") {
        filtered.pop();
    }
    if !filtered.is_empty() && !filtered.ends_with('\n') {
        filtered.push('\n');
    }

    let region = req.region.as_deref().unwrap_or("us-west-2");
    let new_block = format!(
        "\n[{name}]\ncredential_process = ada credentials print --account={account} --role={role} --provider={provider}\nregion = {region}\n",
        name = profile_name,
        account = req.account_id,
        role = req.role_name,
        provider = req.provider.as_str(),
        region = region,
    );

    let final_contents = format!("{}{}", filtered, new_block);
    std::fs::write(&credentials_path, final_contents).with_context(|| {
        format!(
            "failed to write AWS credentials at {}",
            credentials_path.display()
        )
    })?;

    Ok(profile_name)
}

/// Persist the chosen Midway / ada profile into the goose config and set
/// Bedrock as the active provider.
pub fn configure_midway(
    config: &Config,
    req: &MidwayLoginRequest,
    profile_name: &str,
) -> Result<()> {
    let region = req.region.as_deref().unwrap_or("us-west-2");

    // Clear any IDC-style static credentials from a prior login so the AWS
    // SDK falls through to the profile we just wrote.
    let _ = config.delete_secret("AWS_ACCESS_KEY_ID");
    let _ = config.delete_secret("AWS_SECRET_ACCESS_KEY");
    let _ = config.delete_secret("AWS_SESSION_TOKEN");

    config.set_param("AWS_PROFILE", profile_name)?;
    config.set_param("AWS_REGION", region)?;

    config.set_param("AWS_MIDWAY_ACCOUNT_ID", &req.account_id)?;
    config.set_param("AWS_MIDWAY_ROLE_NAME", &req.role_name)?;
    config.set_param("AWS_MIDWAY_PROVIDER", req.provider.as_str())?;

    crate::config::set_active_provider(config, BEDROCK_PROVIDER_NAME, BEDROCK_DEFAULT_MODEL)?;
    Ok(())
}

/// End-to-end Midway login: probe → write profile → configure goose.
pub fn complete_midway_login(req: &MidwayLoginRequest) -> Result<MidwayLoginResponse> {
    if let Err(e) = probe_ada_credentials(req) {
        return Ok(MidwayLoginResponse {
            success: false,
            message: e.to_string(),
            identity: None,
        });
    }

    let profile_name = match write_credential_process_profile(req) {
        Ok(name) => name,
        Err(e) => {
            return Ok(MidwayLoginResponse {
                success: false,
                message: format!("Failed to write AWS credentials profile: {}", e),
                identity: None,
            });
        }
    };

    let config = Config::global();
    if let Err(e) = configure_midway(config, req, &profile_name) {
        return Ok(MidwayLoginResponse {
            success: false,
            message: format!("Failed to persist goose config: {}", e),
            identity: None,
        });
    }

    Ok(MidwayLoginResponse {
        success: true,
        message: format!(
            "Signed in via Midway. Profile `{}` will refresh credentials automatically.",
            profile_name
        ),
        identity: Some(format!("{}/{}", req.account_id, req.role_name)),
    })
}

#[cfg(test)]
mod tests;

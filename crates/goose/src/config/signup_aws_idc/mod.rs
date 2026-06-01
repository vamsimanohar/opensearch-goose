//! AWS IAM Identity Center (IDC) authentication via the OIDC device authorization grant.
//!
//! Used by the OpenSearch Goose distribution to let users sign in with their corporate
//! IDC credentials and have temporary AWS credentials propagate to:
//!   - the Bedrock model provider (LLM)
//!   - any AWS-backed MCP extensions (OpenSearch, AMP, CloudWatch, ...)
//!
//! Flow (RFC 8628 device authorization grant):
//!   1. RegisterClient   -> dynamic OAuth client (cached for ~90 days)
//!   2. StartDeviceAuthorization -> verification URL + user code
//!   3. CreateToken (poll) -> SSO access token
//!   4. ListAccounts / ListAccountRoles -> user picks account/role
//!   5. GetRoleCredentials -> temporary AWS credentials
//!
//! The credentials are stored as goose config secrets so the Bedrock provider and
//! MCP extensions pick them up via the standard AWS env-var chain.

use anyhow::{anyhow, Context, Result};
use aws_config::Region;
use aws_sdk_sso::Client as SsoClient;
use aws_sdk_ssooidc::Client as SsoOidcClient;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use utoipa::ToSchema;

use crate::config::paths::Paths;
use crate::config::Config;
use crate::providers::bedrock::{BEDROCK_DEFAULT_MODEL, BEDROCK_PROVIDER_NAME};

/// User-Agent client name passed to RegisterClient.
const CLIENT_NAME: &str = "opensearch-goose";

/// IDC supports refresh tokens when this scope is requested.
const REFRESH_SCOPE: &str = "sso:account:access";

fn cache_dir() -> PathBuf {
    Paths::in_config_dir("aws-idc")
}

fn client_cache_path() -> PathBuf {
    cache_dir().join("client.json")
}

fn token_cache_path() -> PathBuf {
    cache_dir().join("sso_token.json")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedClient {
    client_id: String,
    client_secret: String,
    /// Unix epoch seconds.
    client_secret_expires_at: i64,
    region: String,
}

impl CachedClient {
    fn is_valid(&self) -> bool {
        self.client_secret_expires_at > Utc::now().timestamp() + 60
    }

    fn load(region: &str) -> Option<Self> {
        let raw = fs::read_to_string(client_cache_path()).ok()?;
        let cached: Self = serde_json::from_str(&raw).ok()?;
        if cached.region == region && cached.is_valid() {
            Some(cached)
        } else {
            None
        }
    }

    fn save(&self) -> Result<()> {
        fs::create_dir_all(cache_dir())?;
        fs::write(client_cache_path(), serde_json::to_string(self)?)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CachedToken {
    access_token: String,
    refresh_token: Option<String>,
    /// Unix epoch seconds.
    expires_at: i64,
    start_url: String,
    region: String,
}

impl CachedToken {
    fn is_valid(&self) -> bool {
        self.expires_at > Utc::now().timestamp() + 60
    }

    fn load(start_url: &str, region: &str) -> Option<Self> {
        let raw = fs::read_to_string(token_cache_path()).ok()?;
        let cached: Self = serde_json::from_str(&raw).ok()?;
        if cached.start_url == start_url && cached.region == region && cached.is_valid() {
            Some(cached)
        } else {
            None
        }
    }

    fn save(&self) -> Result<()> {
        fs::create_dir_all(cache_dir())?;
        fs::write(token_cache_path(), serde_json::to_string(self)?)?;
        Ok(())
    }
}

/// Returned from `StartDeviceAuthorization`. The UI shows these to the user.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DeviceAuthInfo {
    pub verification_uri: String,
    pub verification_uri_complete: Option<String>,
    pub user_code: String,
    pub device_code: String,
    pub expires_in: i32,
    pub interval: i32,
}

/// One AWS account the user has access to.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct IdcAccount {
    pub account_id: String,
    pub account_name: String,
    pub email_address: Option<String>,
}

/// Final temporary credentials returned from `GetRoleCredentials`.
#[derive(Debug, Clone)]
pub struct IdcCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    /// Unix epoch milliseconds (matches AWS SDK).
    pub expiration_ms: i64,
}

/// Server-side state held between the multi-step setup endpoints.
#[derive(Default)]
pub struct AwsIdcSession {
    pub start_url: Option<String>,
    pub region: Option<String>,
    pub access_token: Option<String>,
    /// Set after the user picks an account in the UI.
    pub selected_account_id: Option<String>,
    pub selected_account_name: Option<String>,
}

pub type SharedAwsIdcSession = Arc<Mutex<AwsIdcSession>>;

pub fn new_session() -> SharedAwsIdcSession {
    Arc::new(Mutex::new(AwsIdcSession::default()))
}

async fn build_aws_config(region: &str) -> aws_config::SdkConfig {
    use smithy_transport_reqwest::ReqwestHttpClient;
    aws_config::defaults(aws_config::BehaviorVersion::latest())
        .http_client(ReqwestHttpClient::new())
        .region(Region::new(region.to_string()))
        .no_credentials()
        .load()
        .await
}

async fn build_ssooidc_client(region: &str) -> SsoOidcClient {
    let cfg = build_aws_config(region).await;
    SsoOidcClient::new(&cfg)
}

async fn build_sso_client(region: &str) -> SsoClient {
    let cfg = build_aws_config(region).await;
    SsoClient::new(&cfg)
}

/// Step 1+2: register client (cached) and start device authorization.
///
/// Returns the verification URL + user code that the UI shows the user.
/// The caller must then call [`poll_for_token`] until the user completes auth.
pub async fn start_device_auth(start_url: &str, region: &str) -> Result<DeviceAuthInfo> {
    let oidc = build_ssooidc_client(region).await;

    let client = match CachedClient::load(region) {
        Some(c) => c,
        None => {
            let resp = oidc
                .register_client()
                .client_name(CLIENT_NAME)
                .client_type("public")
                .scopes(REFRESH_SCOPE)
                .send()
                .await
                .context("RegisterClient failed")?;

            let cached = CachedClient {
                client_id: resp.client_id().unwrap_or_default().to_string(),
                client_secret: resp.client_secret().unwrap_or_default().to_string(),
                client_secret_expires_at: resp.client_secret_expires_at(),
                region: region.to_string(),
            };
            cached.save().ok();
            cached
        }
    };

    let resp = oidc
        .start_device_authorization()
        .client_id(&client.client_id)
        .client_secret(&client.client_secret)
        .start_url(start_url)
        .send()
        .await
        .context("StartDeviceAuthorization failed")?;

    let verify_url = resp.verification_uri().unwrap_or_default().to_string();
    let user_code = resp.user_code().unwrap_or_default().to_string();

    // Best-effort: copy to clipboard and open browser. Same UX as oauth_device_flow.
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        let _ = clipboard.set_text(&user_code);
    }
    let open_url = resp
        .verification_uri_complete()
        .unwrap_or(&verify_url)
        .to_string();
    if let Err(e) = webbrowser::open(&open_url) {
        tracing::warn!("Failed to open browser: {}", e);
    }

    Ok(DeviceAuthInfo {
        verification_uri: verify_url,
        verification_uri_complete: resp.verification_uri_complete().map(|s| s.to_string()),
        user_code,
        device_code: resp.device_code().unwrap_or_default().to_string(),
        expires_in: resp.expires_in(),
        interval: resp.interval(),
    })
}

/// Step 3: poll the token endpoint until the user authorizes (or device code expires).
///
/// Returns the SSO access token. Caches it and returns it for the caller to
/// stash in the session.
pub async fn poll_for_token(
    start_url: &str,
    region: &str,
    device_code: &str,
    interval_secs: u64,
    expires_in_secs: u64,
) -> Result<String> {
    let oidc = build_ssooidc_client(region).await;
    let client = CachedClient::load(region)
        .ok_or_else(|| anyhow!("no cached IDC client; call start_device_auth first"))?;

    let deadline = std::time::Instant::now() + Duration::from_secs(expires_in_secs);
    let mut interval = Duration::from_secs(interval_secs.max(1));

    loop {
        if std::time::Instant::now() >= deadline {
            return Err(anyhow!("timed out waiting for user authorization"));
        }
        tokio::time::sleep(interval).await;

        let result = oidc
            .create_token()
            .client_id(&client.client_id)
            .client_secret(&client.client_secret)
            .grant_type("urn:ietf:params:oauth:grant-type:device_code")
            .device_code(device_code)
            .send()
            .await;

        match result {
            Ok(resp) => {
                let access_token = resp
                    .access_token()
                    .ok_or_else(|| anyhow!("no access_token in CreateToken response"))?
                    .to_string();
                let refresh_token = resp.refresh_token().map(|s| s.to_string());
                let expires_in = resp.expires_in();
                let cached = CachedToken {
                    access_token: access_token.clone(),
                    refresh_token,
                    expires_at: Utc::now().timestamp() + expires_in as i64,
                    start_url: start_url.to_string(),
                    region: region.to_string(),
                };
                cached.save().ok();
                return Ok(access_token);
            }
            Err(err) => {
                let msg = format!("{err:?}");
                if msg.contains("AuthorizationPendingException") {
                    continue;
                }
                if msg.contains("SlowDownException") {
                    interval += Duration::from_secs(5);
                    continue;
                }
                if msg.contains("ExpiredTokenException") {
                    return Err(anyhow!(
                        "device code expired before authorization completed"
                    ));
                }
                return Err(anyhow!("CreateToken failed: {}", err));
            }
        }
    }
}

/// Step 4a: list accounts the user has access to.
pub async fn list_accounts(region: &str, access_token: &str) -> Result<Vec<IdcAccount>> {
    let sso = build_sso_client(region).await;

    let mut next_token: Option<String> = None;
    let mut all = Vec::new();
    loop {
        let mut req = sso
            .list_accounts()
            .access_token(access_token)
            .max_results(50);
        if let Some(ref t) = next_token {
            req = req.next_token(t);
        }
        let resp = req.send().await.context("ListAccounts failed")?;
        for a in resp.account_list() {
            all.push(IdcAccount {
                account_id: a.account_id().unwrap_or_default().to_string(),
                account_name: a.account_name().unwrap_or_default().to_string(),
                email_address: a.email_address().map(|s| s.to_string()),
            });
        }
        match resp.next_token() {
            Some(t) if !t.is_empty() => next_token = Some(t.to_string()),
            _ => break,
        }
    }
    Ok(all)
}

/// Step 4b: list role names available in a given account.
pub async fn list_account_roles(
    region: &str,
    access_token: &str,
    account_id: &str,
) -> Result<Vec<String>> {
    let sso = build_sso_client(region).await;

    let mut next_token: Option<String> = None;
    let mut all = Vec::new();
    loop {
        let mut req = sso
            .list_account_roles()
            .access_token(access_token)
            .account_id(account_id)
            .max_results(50);
        if let Some(ref t) = next_token {
            req = req.next_token(t);
        }
        let resp = req.send().await.context("ListAccountRoles failed")?;
        for r in resp.role_list() {
            all.push(r.role_name().unwrap_or_default().to_string());
        }
        match resp.next_token() {
            Some(t) if !t.is_empty() => next_token = Some(t.to_string()),
            _ => break,
        }
    }
    Ok(all)
}

/// Step 5: get temporary AWS credentials for the chosen account+role.
pub async fn get_role_credentials(
    region: &str,
    access_token: &str,
    account_id: &str,
    role_name: &str,
) -> Result<IdcCredentials> {
    let sso = build_sso_client(region).await;

    let resp = sso
        .get_role_credentials()
        .access_token(access_token)
        .account_id(account_id)
        .role_name(role_name)
        .send()
        .await
        .context("GetRoleCredentials failed")?;

    let creds = resp
        .role_credentials()
        .ok_or_else(|| anyhow!("no roleCredentials in GetRoleCredentials response"))?;

    Ok(IdcCredentials {
        access_key_id: creds.access_key_id().unwrap_or_default().to_string(),
        secret_access_key: creds.secret_access_key().unwrap_or_default().to_string(),
        session_token: creds.session_token().unwrap_or_default().to_string(),
        expiration_ms: creds.expiration(),
    })
}

/// Persist credentials + IDC metadata into the goose config.
///
/// After this returns, the Bedrock provider's `set_aws_env_vars` path will
/// inject these as process env vars, and any extension whose `env_keys`
/// includes the AWS_* keys will receive them via `merge_environments`.
pub fn configure_aws_idc(
    config: &Config,
    creds: &IdcCredentials,
    start_url: &str,
    region: &str,
    account_id: &str,
    account_name: &str,
    role_name: &str,
) -> Result<()> {
    config.set_secret("AWS_ACCESS_KEY_ID", &creds.access_key_id)?;
    config.set_secret("AWS_SECRET_ACCESS_KEY", &creds.secret_access_key)?;
    config.set_secret("AWS_SESSION_TOKEN", &creds.session_token)?;

    config.set_param("AWS_REGION", region)?;
    config.set_param("AWS_IDC_START_URL", start_url)?;
    config.set_param("AWS_IDC_REGION", region)?;
    config.set_param("AWS_IDC_ACCOUNT_ID", account_id)?;
    config.set_param("AWS_IDC_ACCOUNT_NAME", account_name)?;
    config.set_param("AWS_IDC_ROLE_NAME", role_name)?;
    config.set_param("AWS_IDC_CREDENTIALS_EXPIRATION_MS", creds.expiration_ms)?;

    crate::config::set_active_provider(config, BEDROCK_PROVIDER_NAME, BEDROCK_DEFAULT_MODEL)?;
    Ok(())
}

/// Best-effort silent refresh: if cached SSO token is still valid, fetch fresh
/// role credentials for the previously-selected account/role and update the
/// config secrets. Returns Ok(true) if credentials were refreshed.
pub async fn refresh_idc_credentials_if_possible(config: &Config) -> Result<bool> {
    let start_url: String = config
        .get_param("AWS_IDC_START_URL")
        .map_err(|_| anyhow!("not signed in via IDC"))?;
    let region: String = config
        .get_param("AWS_IDC_REGION")
        .map_err(|_| anyhow!("missing AWS_IDC_REGION"))?;
    let account_id: String = config
        .get_param("AWS_IDC_ACCOUNT_ID")
        .map_err(|_| anyhow!("missing AWS_IDC_ACCOUNT_ID"))?;
    let role_name: String = config
        .get_param("AWS_IDC_ROLE_NAME")
        .map_err(|_| anyhow!("missing AWS_IDC_ROLE_NAME"))?;
    let account_name: String = config
        .get_param("AWS_IDC_ACCOUNT_NAME")
        .unwrap_or_else(|_| account_id.clone());

    let token = CachedToken::load(&start_url, &region)
        .ok_or_else(|| anyhow!("cached SSO token is missing or expired"))?;

    let creds = get_role_credentials(&region, &token.access_token, &account_id, &role_name).await?;
    configure_aws_idc(
        config,
        &creds,
        &start_url,
        &region,
        &account_id,
        &account_name,
        &role_name,
    )?;
    Ok(true)
}

#[cfg(test)]
mod tests;

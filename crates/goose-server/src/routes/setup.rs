use crate::routes::errors::ErrorResponse;
use crate::state::AppState;
use axum::{
    extract::Query,
    routing::{get, post},
    Json, Router,
};
use goose::config::signup_nanogpt::{complete_nanogpt_auth, configure_nanogpt};
use goose::config::signup_openrouter::OpenRouterAuth;
use goose::config::signup_tetrate::{configure_tetrate, TetrateAuth};
use goose::config::{configure_openrouter, Config};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct SetupResponse {
    pub success: bool,
    pub message: String,
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/handle_openrouter", post(start_openrouter_setup))
        .route("/handle_tetrate", post(start_tetrate_setup))
        .route("/handle_nanogpt", post(start_nanogpt_setup))
        .route("/aws-idc/start", post(aws_idc_start))
        .route("/aws-idc/accounts", get(aws_idc_accounts))
        .route("/aws-idc/roles", get(aws_idc_roles))
        .route("/aws-idc/complete", post(aws_idc_complete))
        .route("/aws-default/probe", post(aws_default_probe))
        .route("/aws-midway/status", get(aws_midway_status))
        .route("/aws-midway/login", post(aws_midway_login))
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/handle_openrouter",
    responses(
        (status = 200, body=SetupResponse)
    ),
)]
pub async fn start_openrouter_setup() -> Result<Json<SetupResponse>, ErrorResponse> {
    let mut auth_flow = OpenRouterAuth::new()
        .map_err(|e| ErrorResponse::internal(format!("Failed to initialize auth flow: {}", e)))?;

    match auth_flow.complete_flow().await {
        Ok(api_key) => {
            let config = Config::global();

            if let Err(e) = configure_openrouter(config, api_key) {
                return Ok(Json(SetupResponse {
                    success: false,
                    message: format!("Failed to configure OpenRouter: {}", e),
                }));
            }

            Ok(Json(SetupResponse {
                success: true,
                message: "OpenRouter setup completed successfully".to_string(),
            }))
        }
        Err(e) => Ok(Json(SetupResponse {
            success: false,
            message: e.to_string(),
        })),
    }
}

#[utoipa::path(
    post,
    path = "/handle_tetrate",
    responses(
        (status = 200, body=SetupResponse)
    ),
)]
pub async fn start_tetrate_setup() -> Result<Json<SetupResponse>, ErrorResponse> {
    let mut auth_flow = TetrateAuth::new()
        .map_err(|e| ErrorResponse::internal(format!("Failed to initialize auth flow: {}", e)))?;

    match auth_flow.complete_flow().await {
        Ok(api_key) => {
            let config = Config::global();

            if let Err(e) = configure_tetrate(config, api_key) {
                return Ok(Json(SetupResponse {
                    success: false,
                    message: format!("Failed to configure Tetrate Agent Router Service: {}", e),
                }));
            }

            Ok(Json(SetupResponse {
                success: true,
                message: "Tetrate Agent Router Service setup completed successfully".to_string(),
            }))
        }
        Err(e) => Ok(Json(SetupResponse {
            success: false,
            message: e.to_string(),
        })),
    }
}

#[utoipa::path(
    post,
    path = "/handle_nanogpt",
    responses(
        (status = 200, body=SetupResponse)
    ),
)]
pub async fn start_nanogpt_setup() -> Result<Json<SetupResponse>, ErrorResponse> {
    match complete_nanogpt_auth().await {
        Ok(api_key) => {
            let config = Config::global();

            if let Err(e) = configure_nanogpt(config, api_key) {
                return Ok(Json(SetupResponse {
                    success: false,
                    message: format!("Failed to configure NanoGPT: {}", e),
                }));
            }

            Ok(Json(SetupResponse {
                success: true,
                message: "NanoGPT setup completed successfully".to_string(),
            }))
        }
        Err(e) => Ok(Json(SetupResponse {
            success: false,
            message: e.to_string(),
        })),
    }
}

// ──────────────────────────────────────────────────────────────────────────
// AWS IAM Identity Center (IDC) login flow
//
// Multi-step because the user picks an account+role mid-flow:
//   POST /aws-idc/start    — runs device auth, polls until user authenticates
//   GET  /aws-idc/accounts — list accounts the user has access to
//   GET  /aws-idc/roles    — list roles for a chosen account
//   POST /aws-idc/complete — gets temporary credentials, persists them
//
// The structs are always compiled so the OpenAPI schema is consistent across
// build configurations. The handler bodies are only functional when the
// `aws-providers` feature is enabled; otherwise they return a friendly error.
// ──────────────────────────────────────────────────────────────────────────

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AwsIdcStartRequest {
    pub start_url: String,
    pub region: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AwsIdcStartResponse {
    pub success: bool,
    pub message: String,
    /// Number of accounts available after sign-in. UI uses this to skip the
    /// picker when there is exactly one.
    pub account_count: Option<usize>,
    /// Auto-populated when there is exactly one account.
    pub auto_account_id: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct AwsIdcCompleteRequest {
    pub account_id: String,
    pub account_name: Option<String>,
    pub role_name: String,
}

#[derive(Deserialize)]
pub struct AwsIdcRolesQuery {
    pub account_id: String,
}

#[cfg(feature = "aws-providers")]
mod aws_idc_impl {
    use super::*;
    use goose::config::signup_aws_idc::{
        configure_aws_idc, get_role_credentials, list_account_roles, list_accounts, new_session,
        poll_for_token, start_device_auth, AwsIdcSession, IdcAccount as IdcAccountInner,
        SharedAwsIdcSession,
    };
    use once_cell::sync::Lazy;

    static IDC_SESSION: Lazy<SharedAwsIdcSession> = Lazy::new(new_session);

    pub async fn start(req: AwsIdcStartRequest) -> Result<AwsIdcStartResponse, ErrorResponse> {
        let device = match start_device_auth(&req.start_url, &req.region).await {
            Ok(d) => d,
            Err(e) => {
                return Ok(AwsIdcStartResponse {
                    success: false,
                    message: format!("Failed to start device authorization: {}", e),
                    account_count: None,
                    auto_account_id: None,
                });
            }
        };

        tracing::info!(
            "AWS IDC: visit {} and enter code {}",
            device.verification_uri,
            device.user_code
        );

        let access_token = match poll_for_token(
            &req.start_url,
            &req.region,
            &device.device_code,
            device.interval as u64,
            device.expires_in as u64,
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                return Ok(AwsIdcStartResponse {
                    success: false,
                    message: format!("Authentication failed: {}", e),
                    account_count: None,
                    auto_account_id: None,
                });
            }
        };

        {
            let mut session = IDC_SESSION.lock().await;
            session.start_url = Some(req.start_url.clone());
            session.region = Some(req.region.clone());
            session.access_token = Some(access_token.clone());
        }

        let accounts = list_accounts(&req.region, &access_token)
            .await
            .unwrap_or_default();
        let auto = if accounts.len() == 1 {
            Some(accounts[0].account_id.clone())
        } else {
            None
        };

        Ok(AwsIdcStartResponse {
            success: true,
            message: "Authenticated. Pick an account.".to_string(),
            account_count: Some(accounts.len()),
            auto_account_id: auto,
        })
    }

    pub async fn accounts() -> Result<Vec<IdcAccountInner>, ErrorResponse> {
        let session = IDC_SESSION.lock().await;
        let access_token = session.access_token.clone().ok_or_else(|| {
            ErrorResponse::bad_request("Not authenticated. Call /aws-idc/start first.".to_string())
        })?;
        let region = session
            .region
            .clone()
            .ok_or_else(|| ErrorResponse::bad_request("Missing region in session".to_string()))?;
        drop(session);

        list_accounts(&region, &access_token)
            .await
            .map_err(|e| ErrorResponse::internal(format!("ListAccounts failed: {}", e)))
    }

    pub async fn roles(account_id: String) -> Result<Vec<String>, ErrorResponse> {
        let session = IDC_SESSION.lock().await;
        let access_token = session.access_token.clone().ok_or_else(|| {
            ErrorResponse::bad_request("Not authenticated. Call /aws-idc/start first.".to_string())
        })?;
        let region = session
            .region
            .clone()
            .ok_or_else(|| ErrorResponse::bad_request("Missing region in session".to_string()))?;
        drop(session);

        {
            let mut session = IDC_SESSION.lock().await;
            session.selected_account_id = Some(account_id.clone());
        }

        list_account_roles(&region, &access_token, &account_id)
            .await
            .map_err(|e| ErrorResponse::internal(format!("ListAccountRoles failed: {}", e)))
    }

    pub async fn complete(req: AwsIdcCompleteRequest) -> Result<SetupResponse, ErrorResponse> {
        let session = IDC_SESSION.lock().await;
        let access_token = session.access_token.clone().ok_or_else(|| {
            ErrorResponse::bad_request("Not authenticated. Call /aws-idc/start first.".to_string())
        })?;
        let start_url = session.start_url.clone().ok_or_else(|| {
            ErrorResponse::bad_request("Missing start_url in session".to_string())
        })?;
        let region = session
            .region
            .clone()
            .ok_or_else(|| ErrorResponse::bad_request("Missing region in session".to_string()))?;
        drop(session);

        let creds =
            match get_role_credentials(&region, &access_token, &req.account_id, &req.role_name)
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    return Ok(SetupResponse {
                        success: false,
                        message: format!("GetRoleCredentials failed: {}", e),
                    });
                }
            };

        let account_name = req.account_name.unwrap_or_else(|| req.account_id.clone());
        let config = Config::global();
        if let Err(e) = configure_aws_idc(
            config,
            &creds,
            &start_url,
            &region,
            &req.account_id,
            &account_name,
            &req.role_name,
        ) {
            return Ok(SetupResponse {
                success: false,
                message: format!("Failed to persist credentials: {}", e),
            });
        }

        {
            let mut session = IDC_SESSION.lock().await;
            *session = AwsIdcSession::default();
        }

        Ok(SetupResponse {
            success: true,
            message: format!(
                "Signed in as {} ({}/{})",
                account_name, req.account_id, req.role_name
            ),
        })
    }
}

#[cfg(not(feature = "aws-providers"))]
fn aws_idc_disabled<T>() -> Result<T, ErrorResponse> {
    Err(ErrorResponse::internal(
        "AWS IDC support not compiled into this build (enable feature `aws-providers`)".to_string(),
    ))
}

#[utoipa::path(
    post,
    path = "/aws-idc/start",
    request_body = AwsIdcStartRequest,
    responses(
        (status = 200, body = AwsIdcStartResponse)
    ),
)]
pub async fn aws_idc_start(
    Json(req): Json<AwsIdcStartRequest>,
) -> Result<Json<AwsIdcStartResponse>, ErrorResponse> {
    #[cfg(feature = "aws-providers")]
    {
        aws_idc_impl::start(req).await.map(Json)
    }
    #[cfg(not(feature = "aws-providers"))]
    {
        let _ = req;
        aws_idc_disabled()
    }
}

#[utoipa::path(
    get,
    path = "/aws-idc/accounts",
    responses(
        (status = 200, description = "Accounts the authenticated user has access to")
    ),
)]
pub async fn aws_idc_accounts() -> Result<Json<serde_json::Value>, ErrorResponse> {
    #[cfg(feature = "aws-providers")]
    {
        let list = aws_idc_impl::accounts().await?;
        Ok(Json(serde_json::to_value(list)?))
    }
    #[cfg(not(feature = "aws-providers"))]
    {
        aws_idc_disabled()
    }
}

#[utoipa::path(
    get,
    path = "/aws-idc/roles",
    params(
        ("account_id" = String, Query, description = "AWS account id")
    ),
    responses(
        (status = 200, description = "Role names available for the given account")
    ),
)]
pub async fn aws_idc_roles(
    Query(q): Query<AwsIdcRolesQuery>,
) -> Result<Json<Vec<String>>, ErrorResponse> {
    #[cfg(feature = "aws-providers")]
    {
        aws_idc_impl::roles(q.account_id).await.map(Json)
    }
    #[cfg(not(feature = "aws-providers"))]
    {
        let _ = q;
        aws_idc_disabled()
    }
}

#[utoipa::path(
    post,
    path = "/aws-idc/complete",
    request_body = AwsIdcCompleteRequest,
    responses(
        (status = 200, body = SetupResponse)
    ),
)]
pub async fn aws_idc_complete(
    Json(req): Json<AwsIdcCompleteRequest>,
) -> Result<Json<SetupResponse>, ErrorResponse> {
    #[cfg(feature = "aws-providers")]
    {
        aws_idc_impl::complete(req).await.map(Json)
    }
    #[cfg(not(feature = "aws-providers"))]
    {
        let _ = req;
        aws_idc_disabled()
    }
}

// ──────────────────────────────────────────────────────────────────────────
// AWS default credential chain probe
// ──────────────────────────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct AwsDefaultProbeResponse {
    pub success: bool,
    pub message: String,
    /// Stripped form of the IAM caller arn, e.g. "arn:aws:sts::123:assumed-role/Admin/...".
    pub identity: Option<String>,
}

#[utoipa::path(
    post,
    path = "/aws-default/probe",
    responses(
        (status = 200, body = AwsDefaultProbeResponse)
    ),
)]
pub async fn aws_default_probe() -> Result<Json<AwsDefaultProbeResponse>, ErrorResponse> {
    #[cfg(feature = "aws-providers")]
    {
        match goose::config::signup_amazon_midway::probe_default_aws_credentials().await {
            Ok(arn) => {
                let config = Config::global();
                let _ = config.delete_secret("AWS_ACCESS_KEY_ID");
                let _ = config.delete_secret("AWS_SECRET_ACCESS_KEY");
                let _ = config.delete_secret("AWS_SESSION_TOKEN");
                if let Err(e) = goose::config::set_active_provider(
                    config,
                    goose::providers::bedrock::BEDROCK_PROVIDER_NAME,
                    goose::providers::bedrock::BEDROCK_DEFAULT_MODEL,
                ) {
                    return Ok(Json(AwsDefaultProbeResponse {
                        success: false,
                        message: format!("Failed to set Bedrock as active provider: {}", e),
                        identity: Some(arn),
                    }));
                }
                Ok(Json(AwsDefaultProbeResponse {
                    success: true,
                    message: "AWS default credentials are valid.".to_string(),
                    identity: Some(arn),
                }))
            }
            Err(e) => Ok(Json(AwsDefaultProbeResponse {
                success: false,
                message: format!("{:#}", e),
                identity: None,
            })),
        }
    }
    #[cfg(not(feature = "aws-providers"))]
    {
        Err(ErrorResponse::internal(
            "AWS support not compiled into this build (enable feature `aws-providers`)".to_string(),
        ))
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Amazon-internal Midway flow (uses the `ada` CLI)
// ──────────────────────────────────────────────────────────────────────────

#[derive(Serialize, ToSchema)]
pub struct MidwayStatusResponse {
    /// True when `ada` is installed and on PATH.
    pub ada_installed: bool,
    pub message: String,
}

#[utoipa::path(
    get,
    path = "/aws-midway/status",
    responses(
        (status = 200, body = MidwayStatusResponse)
    ),
)]
pub async fn aws_midway_status() -> Result<Json<MidwayStatusResponse>, ErrorResponse> {
    #[cfg(feature = "aws-providers")]
    {
        let installed = goose::config::signup_amazon_midway::ada_is_installed();
        Ok(Json(MidwayStatusResponse {
            ada_installed: installed,
            message: if installed {
                "ada is installed.".to_string()
            } else {
                "ada is not installed. Run `toolbox install ada` then try again.".to_string()
            },
        }))
    }
    #[cfg(not(feature = "aws-providers"))]
    {
        Ok(Json(MidwayStatusResponse {
            ada_installed: false,
            message: "AWS support not compiled into this build (enable feature `aws-providers`)"
                .to_string(),
        }))
    }
}

// Local OpenAPI-friendly mirrors of the goose::config::signup_amazon_midway
// types. Defined here (not via `pub use`) because utoipa's schema derive needs
// the type names to resolve at the path it sees in the macro input.

#[derive(Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum AdaProvider {
    Isengard,
    Conduit,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct MidwayLoginRequest {
    pub account_id: String,
    pub role_name: String,
    pub provider: AdaProvider,
    pub region: Option<String>,
    pub profile_name: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct MidwayLoginResponse {
    pub success: bool,
    pub message: String,
    pub identity: Option<String>,
}

#[cfg(feature = "aws-providers")]
fn into_goose_login_request(
    req: MidwayLoginRequest,
) -> goose::config::signup_amazon_midway::MidwayLoginRequest {
    use goose::config::signup_amazon_midway as g;
    g::MidwayLoginRequest {
        account_id: req.account_id,
        role_name: req.role_name,
        provider: match req.provider {
            AdaProvider::Isengard => g::AdaProvider::Isengard,
            AdaProvider::Conduit => g::AdaProvider::Conduit,
        },
        region: req.region,
        profile_name: req.profile_name,
    }
}

#[utoipa::path(
    post,
    path = "/aws-midway/login",
    request_body = MidwayLoginRequest,
    responses(
        (status = 200, body = MidwayLoginResponse)
    ),
)]
pub async fn aws_midway_login(
    Json(req): Json<MidwayLoginRequest>,
) -> Result<Json<MidwayLoginResponse>, ErrorResponse> {
    #[cfg(feature = "aws-providers")]
    {
        let goose_req = into_goose_login_request(req);
        match goose::config::signup_amazon_midway::complete_midway_login(&goose_req) {
            Ok(resp) => Ok(Json(MidwayLoginResponse {
                success: resp.success,
                message: resp.message,
                identity: resp.identity,
            })),
            Err(e) => Ok(Json(MidwayLoginResponse {
                success: false,
                message: format!("{:#}", e),
                identity: None,
            })),
        }
    }
    #[cfg(not(feature = "aws-providers"))]
    {
        let _ = req;
        Err(ErrorResponse::internal(
            "AWS support not compiled into this build (enable feature `aws-providers`)".to_string(),
        ))
    }
}

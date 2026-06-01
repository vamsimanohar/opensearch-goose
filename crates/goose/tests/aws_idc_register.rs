//! Live AWS test for IDC RegisterClient.
//!
//! This test makes a real network call to ssooidc.us-east-1.amazonaws.com to
//! verify our SDK wiring works. It uses no credentials (RegisterClient is
//! anonymous-callable for public clients) so it should succeed in any
//! environment with network access. Run with:
//!
//!     cargo test -p goose --features aws-providers --test aws_idc_register \
//!       -- --ignored --nocapture
//!
//! Marked `#[ignore]` so it doesn't run by default in CI.

#![cfg(feature = "aws-providers")]

use aws_config::{BehaviorVersion, Region};
use aws_sdk_ssooidc::Client;
use smithy_transport_reqwest::ReqwestHttpClient;

#[tokio::test]
#[ignore]
async fn register_client_against_aws() {
    let cfg = aws_config::defaults(BehaviorVersion::latest())
        .http_client(ReqwestHttpClient::new())
        .region(Region::new("us-east-1"))
        .no_credentials()
        .load()
        .await;
    let client = Client::new(&cfg);

    let resp = client
        .register_client()
        .client_name("opensearch-goose-test")
        .client_type("public")
        .scopes("sso:account:access")
        .send()
        .await
        .expect("RegisterClient should succeed without credentials");

    assert!(!resp.client_id().unwrap_or_default().is_empty());
    assert!(!resp.client_secret().unwrap_or_default().is_empty());
    assert!(resp.client_secret_expires_at() > 0);
    eprintln!(
        "RegisterClient OK: client_id={} expires_at={}",
        resp.client_id().unwrap(),
        resp.client_secret_expires_at()
    );
}

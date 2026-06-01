//! Live test for the AWS default credential chain probe. Requires AWS
//! credentials on the host (env vars, ~/.aws/credentials, etc.).
//!
//! Run with:
//!     cargo test -p goose --features aws-providers --test aws_default_probe -- --ignored --nocapture

#![cfg(feature = "aws-providers")]

use goose::config::signup_amazon_midway::probe_default_aws_credentials;

#[tokio::test]
#[ignore]
async fn default_chain_returns_caller_identity() {
    let arn = probe_default_aws_credentials()
        .await
        .expect("default credentials should resolve");
    assert!(arn.starts_with("arn:aws:"), "expected an ARN, got: {}", arn);
    eprintln!("Default credentials resolve to: {}", arn);
}

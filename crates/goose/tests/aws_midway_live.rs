//! Live test that exercises the Midway/ada flow end-to-end against a real
//! Amazon-internal AWS account. Requires:
//!   - `ada` on PATH (toolbox install ada)
//!   - `mwinit -o` already run
//!
//! Run with:
//!     cargo test -p goose --features aws-providers --test aws_midway_live -- --ignored --nocapture
//!
//! Marked `#[ignore]` so it never runs in CI.

#![cfg(feature = "aws-providers")]

use goose::config::signup_amazon_midway::{probe_ada_credentials, AdaProvider, MidwayLoginRequest};

#[test]
#[ignore]
fn probe_real_isengard_account() {
    let req = MidwayLoginRequest {
        account_id: "439024109009".into(),
        role_name: "Admin".into(),
        provider: AdaProvider::Isengard,
        region: Some("us-west-2".into()),
        profile_name: None,
    };
    probe_ada_credentials(&req).expect("ada should mint credentials for the test account");
}

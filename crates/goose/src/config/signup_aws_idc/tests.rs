use super::*;
use serial_test::serial;
use tempfile::TempDir;

fn setup_test_dirs() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");
    std::env::set_var("XDG_CONFIG_HOME", dir.path());
    std::env::set_var("HOME", dir.path());
    dir
}

#[test]
#[serial]
fn cached_client_round_trips() {
    let _dir = setup_test_dirs();
    let cached = CachedClient {
        client_id: "abc".into(),
        client_secret: "secret".into(),
        client_secret_expires_at: Utc::now().timestamp() + 3600,
        region: "us-east-1".into(),
    };
    cached.save().unwrap();

    let loaded = CachedClient::load("us-east-1").expect("cached client should load");
    assert_eq!(loaded.client_id, "abc");
    assert_eq!(loaded.client_secret, "secret");
}

#[test]
#[serial]
fn cached_client_rejects_wrong_region() {
    let _dir = setup_test_dirs();
    let cached = CachedClient {
        client_id: "abc".into(),
        client_secret: "secret".into(),
        client_secret_expires_at: Utc::now().timestamp() + 3600,
        region: "us-east-1".into(),
    };
    cached.save().unwrap();

    assert!(CachedClient::load("eu-west-1").is_none());
}

#[test]
#[serial]
fn cached_client_rejects_expired() {
    let _dir = setup_test_dirs();
    let cached = CachedClient {
        client_id: "abc".into(),
        client_secret: "secret".into(),
        // Already expired.
        client_secret_expires_at: Utc::now().timestamp() - 1,
        region: "us-east-1".into(),
    };
    cached.save().unwrap();

    assert!(CachedClient::load("us-east-1").is_none());
}

#[test]
#[serial]
fn cached_token_round_trips() {
    let _dir = setup_test_dirs();
    let cached = CachedToken {
        access_token: "tok".into(),
        refresh_token: Some("refresh".into()),
        expires_at: Utc::now().timestamp() + 3600,
        start_url: "https://x.awsapps.com/start".into(),
        region: "us-east-1".into(),
    };
    cached.save().unwrap();

    let loaded = CachedToken::load("https://x.awsapps.com/start", "us-east-1")
        .expect("cached token should load");
    assert_eq!(loaded.access_token, "tok");
    assert_eq!(loaded.refresh_token.as_deref(), Some("refresh"));
}

#[test]
#[serial]
fn cached_token_rejects_wrong_start_url() {
    let _dir = setup_test_dirs();
    let cached = CachedToken {
        access_token: "tok".into(),
        refresh_token: None,
        expires_at: Utc::now().timestamp() + 3600,
        start_url: "https://a.awsapps.com/start".into(),
        region: "us-east-1".into(),
    };
    cached.save().unwrap();

    assert!(CachedToken::load("https://b.awsapps.com/start", "us-east-1").is_none());
}

#[test]
fn aws_idc_session_starts_empty() {
    let session = AwsIdcSession::default();
    assert!(session.access_token.is_none());
    assert!(session.start_url.is_none());
    assert!(session.region.is_none());
    assert!(session.selected_account_id.is_none());
}

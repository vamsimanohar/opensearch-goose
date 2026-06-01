use super::*;
use serial_test::serial;
use tempfile::TempDir;

fn req(profile_name: Option<&str>) -> MidwayLoginRequest {
    MidwayLoginRequest {
        account_id: "123456789012".into(),
        role_name: "Admin".into(),
        provider: AdaProvider::Isengard,
        region: Some("us-west-2".into()),
        profile_name: profile_name.map(|s| s.to_string()),
    }
}

#[test]
#[serial]
fn writes_new_profile_to_credentials_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("credentials");
    std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", &path);

    let name = write_credential_process_profile(&req(None)).unwrap();
    assert_eq!(name, "goose-midway");

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("[goose-midway]"));
    assert!(contents.contains(
        "credential_process = ada credentials print --account=123456789012 --role=Admin --provider=isengard"
    ));
    assert!(contents.contains("region = us-west-2"));

    std::env::remove_var("AWS_SHARED_CREDENTIALS_FILE");
}

#[test]
#[serial]
fn replaces_existing_profile_in_place() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("credentials");
    std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", &path);

    // Pre-seed with an unrelated profile and a stale goose-midway block.
    std::fs::write(
        &path,
        "[other]\naws_access_key_id=stay\n\n[goose-midway]\ncredential_process = stale\nregion = us-east-1\n",
    )
    .unwrap();

    write_credential_process_profile(&req(None)).unwrap();
    let contents = std::fs::read_to_string(&path).unwrap();

    // [other] survives.
    assert!(contents.contains("[other]"));
    assert!(contents.contains("aws_access_key_id=stay"));
    // The stale block is gone, replaced with the new one.
    assert!(!contents.contains("credential_process = stale"));
    assert_eq!(contents.matches("[goose-midway]").count(), 1);
    assert!(contents.contains(
        "credential_process = ada credentials print --account=123456789012 --role=Admin --provider=isengard"
    ));

    std::env::remove_var("AWS_SHARED_CREDENTIALS_FILE");
}

#[test]
#[serial]
fn honors_custom_profile_name() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("credentials");
    std::env::set_var("AWS_SHARED_CREDENTIALS_FILE", &path);

    let name = write_credential_process_profile(&req(Some("my-bedrock-profile"))).unwrap();
    assert_eq!(name, "my-bedrock-profile");

    let contents = std::fs::read_to_string(&path).unwrap();
    assert!(contents.contains("[my-bedrock-profile]"));

    std::env::remove_var("AWS_SHARED_CREDENTIALS_FILE");
}

#[test]
fn ada_provider_serializes_lowercase() {
    assert_eq!(AdaProvider::Isengard.as_str(), "isengard");
    assert_eq!(AdaProvider::Conduit.as_str(), "conduit");
}

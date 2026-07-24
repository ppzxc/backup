use backup::commands::update::{is_newer_version, parse_version};

#[test]
fn test_parse_version() {
    assert_eq!(parse_version("v0.1.5"), Some((0, 1, 5)));
    assert_eq!(parse_version("0.1.5"), Some((0, 1, 5)));
    assert_eq!(parse_version("v1.2.3-rc1"), Some((1, 2, 3)));
    assert_eq!(parse_version("invalid"), None);
}

#[test]
fn test_is_newer_version() {
    assert!(is_newer_version("0.1.5", "v0.1.6"));
    assert!(is_newer_version("0.1.5", "0.2.0"));
    assert!(is_newer_version("0.1.5", "1.0.0"));
    assert!(!is_newer_version("0.1.5", "v0.1.5"));
    assert!(!is_newer_version("0.1.5", "0.1.4"));
}

#[test]
fn test_execute_update_check_with_mock_runner_already_up_to_date() {
    use backup::runner::executor::{CommandOutput, MockExecutor};
    use backup::commands::update::execute_update_check_with_runner;

    let mock = MockExecutor::new();
    let json_body = r#"{"tag_name":"v0.1.5","assets":[{"name":"backup-v0.1.5-x86_64-unknown-linux-musl.tar.gz","browser_download_url":"https://example.com/asset.tar.gz"}]}"#;
    mock.push_output("curl", CommandOutput {
        status_code: 0,
        stdout: json_body.into(),
        stderr: "".into(),
    });

    let msg = execute_update_check_with_runner("0.1.5", &mock).unwrap();
    assert!(msg.contains("Already up to date"), "최신 버전인 경우 Already up to date 메시지가 반환되어야 합니다");
}


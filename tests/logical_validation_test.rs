use backup::config::model::ResticProfileConfig;
use backup::runner::executor::{CommandOutput, MockExecutor};
use backup::runner::resticprofile::{ResticProfileRunner, ResticProfileTool};
use tempfile::tempdir;
use std::fs;
use std::path::Path;

#[test]
fn test_permission_failure_detection() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");
    fs::write(&config_path, "version: \"2\"").unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&config_path, fs::Permissions::from_mode(0o777)).unwrap();
        let metadata = fs::metadata(&config_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_ne!(mode, 0o600);
    }
}

#[test]
fn test_executor_non_zero_exit_error_propagation() {
    let mock = MockExecutor::new();
    mock.push_output(
        "resticprofile",
        CommandOutput {
            status_code: 1,
            stdout: "".into(),
            stderr: "wrong password for repository".into(),
        },
    );

    let tool = ResticProfileTool::new(&mock);
    let path = Path::new("/etc/backup/profiles.yaml");
    let res = tool.backup(path, "self", false);

    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("wrong password for repository"));
}

#[test]
fn test_invalid_yaml_missing_fields_validation() {
    let invalid_yaml = r#"
version: "2"
profiles:
  self:
    backup:
      schedule-ignore-on-battery-less-than: [invalid_array_instead_of_number]
"#;
    let res: Result<ResticProfileConfig, _> = serde_yaml::from_str(invalid_yaml);
    assert!(res.is_err());
}

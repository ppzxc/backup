use backup::commands::update::{is_newer_version, parse_version};
mod support;

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
    use crate::support::MockExecutor;
    use backup::commands::update::execute_update_check_with_runner;
    use backup::runner::executor::CommandOutput;

    let mock = MockExecutor::new();
    let json_body = r#"{"tag_name":"v0.1.5","assets":[{"name":"backup-v0.1.5-x86_64-unknown-linux-musl.tar.gz","browser_download_url":"https://example.com/asset.tar.gz"}]}"#;
    mock.push_output(
        "curl",
        CommandOutput {
            status_code: 0,
            stdout: json_body.into(),
            stderr: "".into(),
        },
    );

    let msg = execute_update_check_with_runner("0.1.5", &mock).unwrap();
    assert!(
        msg.contains("Already up to date"),
        "최신 버전인 경우 Already up to date 메시지가 반환되어야 합니다"
    );
}

#[test]
fn update_propagates_release_lookup_failure() {
    use crate::support::MockExecutor;
    use backup::commands::update::execute_update_check_with_runner;
    use backup::runner::executor::CommandOutput;

    let mock = MockExecutor::new();
    mock.push_output(
        "curl",
        CommandOutput {
            status_code: 22,
            stdout: String::new(),
            stderr: "network unavailable".into(),
        },
    );

    let error = execute_update_check_with_runner("0.1.5", &mock).unwrap_err();
    assert!(error.to_string().contains("Failed to fetch release info"));
}

#[test]
fn update_replaces_only_after_staged_binary_is_executable() {
    use backup::commands::update::perform_self_replace_at_path_with_runner;
    use backup::runner::executor::{CommandOutput, CommandRunner};
    use std::path::Path;

    struct StagingRunner;
    impl CommandRunner for StagingRunner {
        fn run(&self, program: &str, args: &[&str]) -> anyhow::Result<CommandOutput> {
            match program {
                "curl" => std::fs::write(args[3], "archive")?,
                "tar" => {
                    let binary = Path::new(args[3]).join("backup");
                    std::fs::write(binary, "new binary")?;
                }
                "chmod" => {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::set_permissions(args[1], std::fs::Permissions::from_mode(0o755))?;
                }
                _ => anyhow::bail!("unexpected command {program}"),
            }
            Ok(CommandOutput {
                status_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let current = temp.path().join("backup");
    std::fs::write(&current, "old binary").unwrap();
    let runner = StagingRunner;

    perform_self_replace_at_path_with_runner(
        "https://example.invalid/update.tar.gz",
        &current,
        temp.path(),
        &runner,
    )
    .unwrap();

    assert_eq!(std::fs::read_to_string(current).unwrap(), "new binary");
}

#[test]
fn update_download_failure_preserves_existing_binary() {
    use backup::commands::update::perform_self_replace_at_path_with_runner;
    use backup::runner::executor::{CommandOutput, CommandRunner};

    struct FailingRunner;
    impl CommandRunner for FailingRunner {
        fn run(&self, _: &str, _: &[&str]) -> anyhow::Result<CommandOutput> {
            Ok(CommandOutput {
                status_code: 1,
                stdout: String::new(),
                stderr: "download failed".into(),
            })
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let current = temp.path().join("backup");
    std::fs::write(&current, "old binary").unwrap();

    let error = perform_self_replace_at_path_with_runner(
        "https://example.invalid/update.tar.gz",
        &current,
        temp.path(),
        &FailingRunner,
    )
    .unwrap_err();

    assert!(error.to_string().contains("download"));
    assert_eq!(std::fs::read_to_string(current).unwrap(), "old binary");
}

#[test]
fn update_non_executable_staging_preserves_existing_binary() {
    use backup::commands::update::perform_self_replace_at_path_with_runner;
    use backup::runner::executor::{CommandOutput, CommandRunner};
    use std::path::Path;

    struct NonExecutableRunner;
    impl CommandRunner for NonExecutableRunner {
        fn run(&self, program: &str, args: &[&str]) -> anyhow::Result<CommandOutput> {
            match program {
                "curl" => std::fs::write(args[3], "archive")?,
                "tar" => std::fs::write(Path::new(args[3]).join("backup"), "new binary")?,
                "chmod" => {}
                _ => anyhow::bail!("unexpected command {program}"),
            }
            Ok(CommandOutput {
                status_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
    }

    let temp = tempfile::tempdir().unwrap();
    let current = temp.path().join("backup");
    std::fs::write(&current, "old binary").unwrap();

    let error = perform_self_replace_at_path_with_runner(
        "https://example.invalid/update.tar.gz",
        &current,
        temp.path(),
        &NonExecutableRunner,
    )
    .unwrap_err();

    assert!(error.to_string().contains("not executable"));
    assert_eq!(std::fs::read_to_string(current).unwrap(), "old binary");
}

mod support;
use backup::runner::executor::{CommandOutput, CommandRunner};
use backup::runner::rclone::{RcloneRunner, RcloneTool};
use backup::runner::restic::{ResticRunner, ResticTool};
use backup::runner::resticprofile::{ResticProfileRunner, ResticProfileTool};
use std::path::Path;
use support::{MockExecutor, MockRcloneRunner, MockResticRunner};

#[test]
fn test_mock_executor_recording() {
    let mock = MockExecutor::new();
    assert_eq!(mock.call_count("restic"), 0);

    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "restic 0.16.0".into(),
            stderr: "".into(),
        },
    );

    assert_eq!(mock.call_count("restic"), 0);

    let res = mock.run("restic", &["version"]).unwrap();
    assert_eq!(res.status_code, 0);
    assert_eq!(res.stdout, "restic 0.16.0");
    assert_eq!(mock.call_count("restic"), 1);

    let _ = mock.run("restic", &["snapshots"]).unwrap();
    assert_eq!(mock.call_count("restic"), 2);

    let calls = mock.get_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "restic");
    assert_eq!(calls[0].1, vec!["version"]);
    assert_eq!(calls[1].1, vec!["snapshots"]);
}

#[test]
fn test_system_executor_run_success_and_invalid_program() {
    use backup::runner::executor::SystemExecutor;

    let executor = SystemExecutor;
    let res = executor.run("echo", &["hello"]).unwrap();
    assert_eq!(res.status_code, 0);
    assert!(res.stdout.contains("hello"));

    let err = executor.run("non_existent_binary_12345", &[]);
    assert!(err.is_err());
}

#[test]
fn test_system_executor_run_with_timeout_times_out() {
    use backup::runner::executor::SystemExecutor;
    let executor = SystemExecutor;
    let res = executor
        .run_with_timeout("sleep", &["10"], &[], std::time::Duration::from_millis(200))
        .unwrap();
    assert_eq!(res.status_code, -1);
    assert!(res.stderr.contains("timed out"));
}

#[test]
fn test_restic_tool_with_mock_executor() {
    let mock = MockExecutor::new();
    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "repo init success".into(),
            stderr: "".into(),
        },
    );
    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "backup success".into(),
            stderr: "".into(),
        },
    );
    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "snapshots listed".into(),
            stderr: "".into(),
        },
    );

    let restic_tool = ResticTool::new(&mock);

    let init_res = restic_tool.init_repo("s3:bucket", "secret123").unwrap();
    assert_eq!(init_res, "repo init success");

    let backup_res = restic_tool
        .backup_paths(
            "s3:bucket",
            "secret123",
            &["/home/user".to_string(), "/var/data".to_string()],
            &["*.tmp".to_string()],
        )
        .unwrap();
    assert_eq!(backup_res, "backup success");

    let snapshots_res = restic_tool
        .list_snapshots("s3:bucket", "secret123")
        .unwrap();
    assert_eq!(snapshots_res, "snapshots listed");

    let calls = mock.get_calls();
    assert_eq!(calls.len(), 3);
    assert_eq!(calls[0].0, "restic");
    assert_eq!(calls[0].1[0], "-r");
    assert_eq!(calls[0].1[1], "s3:bucket");
    assert_eq!(calls[0].1[4], "init");

    assert_eq!(calls[1].1[4], "backup");
    assert_eq!(calls[1].1[5], "/home/user");
    assert_eq!(calls[1].1[6], "/var/data");
    assert_eq!(calls[1].1[7], "--exclude");
    assert_eq!(calls[1].1[8], "*.tmp");

    assert_eq!(calls[2].1[4], "snapshots");
}

#[test]
fn restic_tool_requests_snapshot_json_for_concrete_selection() {
    let mock = MockExecutor::new();
    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "[]".into(),
            stderr: String::new(),
        },
    );

    let restic_tool = ResticTool::new(&mock);
    assert!(
        restic_tool
            .list_snapshot_infos("s3:bucket", "secret")
            .unwrap()
            .is_empty()
    );
    let calls = mock.get_calls();
    assert_eq!(
        &calls[0].1[4..],
        &["snapshots".to_string(), "--json".to_string()]
    );
}

#[test]
fn restic_database_stream_forwards_credentials_only_through_environment() {
    let mock = MockExecutor::new();
    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "streamed".into(),
            stderr: String::new(),
        },
    );
    let restic = ResticTool::new(&mock);

    restic
        .backup_command_with_env(
            "local:/repo",
            "repository-password",
            "app.sql",
            "pg_dump",
            &["--dbname=app".into()],
            &[("PGPASSWORD", "database-password")],
        )
        .unwrap();

    assert_eq!(mock.get_calls()[0].1.last(), Some(&"--dbname=app".into()));
    assert_eq!(
        mock.get_environment_calls(),
        vec![vec![("PGPASSWORD".into(), "database-password".into())]]
    );
}

#[test]
fn restic_database_stream_applies_the_reserved_backup_profile_tag() {
    let mock = MockExecutor::new();
    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "streamed".into(),
            stderr: String::new(),
        },
    );
    let restic = ResticTool::new(&mock);

    restic
        .backup_command_with_env_and_tag(
            "local:/repo",
            "repository-password",
            "app.sql",
            "pg_dump",
            &[],
            "backup-profile:database",
            &[],
        )
        .unwrap();

    let args = &mock.get_calls()[0].1;
    let tag_index = args.iter().position(|arg| arg == "--tag").unwrap();
    assert_eq!(args[tag_index + 1], "backup-profile:database");
}

#[test]
fn test_rclone_tool_with_mock_executor() {
    let mock = MockExecutor::new();
    mock.push_output(
        "rclone",
        CommandOutput {
            status_code: 0,
            stdout: "dir1\ndir2".into(),
            stderr: "".into(),
        },
    );
    mock.push_output(
        "rclone",
        CommandOutput {
            status_code: 0,
            stdout: "remote1:\nremote2:".into(),
            stderr: "".into(),
        },
    );

    let rclone_tool = RcloneTool::new(&mock);
    let lsd_res = rclone_tool.check_connectivity("syno:").unwrap();
    assert_eq!(lsd_res, "dir1\ndir2");

    let list_res = rclone_tool.list_remotes().unwrap();
    assert_eq!(list_res, "remote1:\nremote2:");

    let calls = mock.get_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].1, vec!["lsd", "syno:"]);
    assert_eq!(calls[1].1, vec!["listremotes"]);
}

#[test]
fn test_mock_restic_runner() {
    let runner = MockResticRunner::new(0, "repository initialized");
    let output = runner.init_repo("s3:bucket", "secret").unwrap();
    assert!(output.contains("repository initialized"));

    let backup_output = runner
        .backup_paths("s3:bucket", "secret", &["/data".to_string()], &[])
        .unwrap();
    assert!(backup_output.contains("repository initialized"));

    let snapshots_output = runner.list_snapshots("s3:bucket", "secret").unwrap();
    assert!(snapshots_output.contains("repository initialized"));
}

#[test]
fn test_mock_rclone_runner() {
    let runner = MockRcloneRunner::new(0, "remote_ok");
    let output = runner.check_connectivity("remote:bucket").unwrap();
    assert!(output.contains("remote_ok"));

    let remotes = runner.list_remotes().unwrap();
    assert!(remotes.contains("remote_ok"));
}

#[test]
fn test_resticprofile_tool_with_mock_executor() {
    let mock = MockExecutor::new();
    mock.push_output(
        "resticprofile",
        CommandOutput {
            status_code: 0,
            stdout: "profile backup ok".into(),
            stderr: "".into(),
        },
    );
    mock.push_output(
        "resticprofile",
        CommandOutput {
            status_code: 0,
            stdout: "schedule enabled".into(),
            stderr: "".into(),
        },
    );

    let tool = ResticProfileTool::new(&mock);
    let path = Path::new("/etc/backup/profiles.yaml");

    let backup_res = tool.backup(path, "self", false).unwrap();
    assert_eq!(backup_res, "profile backup ok");

    let sched_res = tool.schedule_enable(path).unwrap();
    assert_eq!(sched_res, "schedule enabled");

    let calls = mock.get_calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].0, "resticprofile");
    assert_eq!(
        calls[0].1,
        vec![
            "--config",
            "/etc/backup/profiles.yaml",
            "--name",
            "self",
            "backup"
        ]
    );
    assert_eq!(
        calls[1].1,
        vec!["--config", "/etc/backup/profiles.yaml", "schedule", "--all"]
    );
}

#[test]
fn resticprofile_init_treats_an_existing_repository_as_idempotent() {
    let mock = MockExecutor::new();
    mock.push_output(
        "resticprofile",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "Fatal: create key in repository failed: repository master key and config already initialized".into(),
        },
    );
    let tool = ResticProfileTool::new(&mock);

    let output = tool.init(Path::new("/nonexistent/profiles.yaml"), "primary");

    assert!(
        output.is_ok(),
        "an existing repository is already initialized"
    );
}

#[test]
fn resticprofile_init_does_not_mask_unrelated_already_exists_errors() {
    let mock = MockExecutor::new();
    mock.push_output(
        "resticprofile",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "permission denied: path already exists but cannot be opened".into(),
        },
    );
    let tool = ResticProfileTool::new(&mock);

    let error = tool
        .init(Path::new("/nonexistent/profiles.yaml"), "primary")
        .unwrap_err();

    assert!(error.to_string().contains("permission denied"));
}

#[test]
fn resticprofile_profile_commands_share_validated_sidecar_environment() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("profiles.yaml");
    std::fs::write(
        &config_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: s3:s3.example/bucket\n    env:\n      AWS_ACCESS_KEY_ID: '{{ .Env.BACKUP_PRIMARY_AWS_ACCESS_KEY_ID }}'\n      AWS_SECRET_ACCESS_KEY: '{{ .Env.BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY }}'\n  archive:\n    inherit: primary\n    backup:\n      source: ['/data']\n    copy:\n      profile: secondary\n      repository: s3:s3.example/secondary\n  secondary:\n    repository: s3:s3.example/secondary\n    env:\n      AWS_ACCESS_KEY_ID: '{{ .Env.BACKUP_SECONDARY_AWS_ACCESS_KEY_ID }}'\n      AWS_SECRET_ACCESS_KEY: '{{ .Env.BACKUP_SECONDARY_AWS_SECRET_ACCESS_KEY }}'\n",
    )
    .unwrap();
    for (name, value) in [
        ("primary-aws-access-key-id", "primary-access"),
        ("primary-aws-secret-access-key", "primary-secret"),
        ("secondary-aws-access-key-id", "secondary-access"),
        ("secondary-aws-secret-access-key", "secondary-secret"),
    ] {
        let path = directory.path().join(name);
        std::fs::write(&path, value).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
    }

    let mock = MockExecutor::new();
    let tool = ResticProfileTool::new(&mock);
    tool.backup(&config_path, "archive", false).unwrap();
    tool.init(&config_path, "archive").unwrap();
    tool.copy(&config_path, "archive", false).unwrap();

    let environments = mock.get_environment_calls();
    assert_eq!(environments.len(), 3);
    assert_eq!(environments[0], environments[1]);
    assert_eq!(
        environments[0],
        vec![
            (
                "BACKUP_PRIMARY_AWS_ACCESS_KEY_ID".into(),
                "primary-access".into()
            ),
            (
                "BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY".into(),
                "primary-secret".into()
            ),
            (
                "BACKUP_SECONDARY_AWS_ACCESS_KEY_ID".into(),
                "secondary-access".into()
            ),
            (
                "BACKUP_SECONDARY_AWS_SECRET_ACCESS_KEY".into(),
                "secondary-secret".into()
            ),
        ]
    );
    assert!(environments[2].contains(&("AWS_ACCESS_KEY_ID".into(), "secondary-access".into())));
    assert!(environments[2].contains(&("AWS_SECRET_ACCESS_KEY".into(), "secondary-secret".into())));
}

#[test]
fn resticprofile_rejects_s3_sidecars_with_insecure_permissions_before_launching() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("profiles.yaml");
    std::fs::write(
        &config_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: s3:s3.example/bucket\n    env:\n      AWS_ACCESS_KEY_ID: '{{ .Env.BACKUP_PRIMARY_AWS_ACCESS_KEY_ID }}'\n      AWS_SECRET_ACCESS_KEY: '{{ .Env.BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY }}'\n  archive:\n    inherit: primary\n    backup:\n      source: ['/data']\n",
    )
    .unwrap();
    for name in ["primary-aws-access-key-id", "primary-aws-secret-access-key"] {
        let path = directory.path().join(name);
        std::fs::write(&path, "credential").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    }

    let mock = MockExecutor::new();
    let error = ResticProfileTool::new(&mock)
        .backup(&config_path, "archive", false)
        .unwrap_err();

    assert!(error.to_string().contains("0600"));
    assert_eq!(mock.call_count("resticprofile"), 0);
}

#[test]
fn resticprofile_rejects_missing_s3_sidecar_before_launching() {
    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("profiles.yaml");
    std::fs::write(
        &config_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: s3:s3.example/bucket\n    env:\n      AWS_ACCESS_KEY_ID: '{{ .Env.BACKUP_PRIMARY_AWS_ACCESS_KEY_ID }}'\n      AWS_SECRET_ACCESS_KEY: '{{ .Env.BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY }}'\n  archive:\n    inherit: primary\n    backup:\n      source: ['/data']\n",
    )
    .unwrap();
    let mock = MockExecutor::new();
    assert!(
        ResticProfileTool::new(&mock)
            .backup(&config_path, "archive", false)
            .is_err()
    );
    assert_eq!(mock.call_count("resticprofile"), 0);
}

#[test]
fn resticprofile_sftp_backend_does_not_require_s3_sidecars() {
    let directory = tempfile::tempdir().unwrap();
    let config_path = directory.path().join("profiles.yaml");
    std::fs::write(
        &config_path,
        "version: '2'\nprofiles:\n  primary:\n    repository: sftp:user@host:/repo\n  archive:\n    inherit: primary\n    backup:\n      source: ['/data']\n",
    )
    .unwrap();
    let mock = MockExecutor::new();
    ResticProfileTool::new(&mock)
        .backup(&config_path, "archive", false)
        .unwrap();
    assert_eq!(mock.call_count("resticprofile"), 1);
    assert_eq!(
        mock.get_environment_calls(),
        vec![Vec::<(String, String)>::new()]
    );
}

#[test]
fn test_resticprofile_tool_all_methods() {
    let mock = MockExecutor::new();
    for _ in 0..5 {
        mock.push_output(
            "resticprofile",
            CommandOutput {
                status_code: 0,
                stdout: "cmd_success".into(),
                stderr: "".into(),
            },
        );
    }

    let tool = ResticProfileTool::new(&mock);
    let path = Path::new("/etc/backup/profiles.yaml");

    assert_eq!(tool.schedule_disable(path).unwrap(), "cmd_success");
    assert_eq!(tool.schedule_status(path).unwrap(), "cmd_success");
    assert_eq!(tool.list_snapshots(path, "self").unwrap(), "cmd_success");
    assert_eq!(tool.prune(path, "self").unwrap(), "cmd_success");
    assert_eq!(tool.check(path, "self").unwrap(), "cmd_success");

    let calls = mock.get_calls();
    assert_eq!(calls.len(), 5);
    assert_eq!(
        calls[0].1,
        vec![
            "--config",
            "/etc/backup/profiles.yaml",
            "unschedule",
            "--all"
        ]
    );
    assert_eq!(
        calls[1].1,
        vec!["--config", "/etc/backup/profiles.yaml", "status", "--all"]
    );
    assert_eq!(
        calls[2].1,
        vec![
            "--config",
            "/etc/backup/profiles.yaml",
            "--name",
            "self",
            "snapshots"
        ]
    );
    assert_eq!(
        calls[3].1,
        vec![
            "--config",
            "/etc/backup/profiles.yaml",
            "--name",
            "self",
            "prune"
        ]
    );
    assert_eq!(
        calls[4].1,
        vec![
            "--config",
            "/etc/backup/profiles.yaml",
            "--name",
            "self",
            "check"
        ]
    );
}

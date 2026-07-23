use backup::runner::executor::{CommandOutput, CommandRunner, MockExecutor};
use backup::runner::rclone::{MockRcloneRunner, RcloneRunner, RcloneTool};
use backup::runner::restic::{MockResticRunner, ResticRunner, ResticTool};

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


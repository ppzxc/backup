use backup::runner::executor::{CommandOutput, CommandRunner, MockExecutor};
use backup::runner::rclone::{MockRcloneRunner, RcloneRunner};
use backup::runner::restic::{MockResticRunner, ResticRunner};

#[test]
fn test_mock_executor_recording() {
    let mock = MockExecutor::new();
    mock.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: "restic 0.16.0".into(),
            stderr: "".into(),
        },
    );

    let res = mock.run("restic", &["version"]).unwrap();
    assert_eq!(res.status_code, 0);
    assert_eq!(res.stdout, "restic 0.16.0");
    assert_eq!(mock.call_count("restic"), 1);
    let calls = mock.get_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "restic");
    assert_eq!(calls[0].1, vec!["version"]);
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

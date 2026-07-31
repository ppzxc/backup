use backup::commands::schedule::{
    execute_schedule_disable, execute_schedule_enable, execute_schedule_status,
};
use backup::runner::resticprofile::MockResticProfileRunner;
use tempfile::tempdir;

#[test]
fn test_execute_schedule_commands() {
    let mock = MockResticProfileRunner::new(0, "scheduled successfully");
    let temp = tempdir().unwrap();
    let path = temp.path().join("profiles.yml");

    let res_enable = execute_schedule_enable(&path, &mock).unwrap();
    assert_eq!(res_enable, "scheduled successfully");

    let res_disable = execute_schedule_disable(&path, &mock).unwrap();
    assert_eq!(res_disable, "scheduled successfully");

    let res_status = execute_schedule_status(&path, &mock).unwrap();
    assert_eq!(res_status, "scheduled successfully");
    let calls = mock.calls.lock().unwrap();
    assert!(
        calls
            .iter()
            .all(|(_, actual_path)| actual_path == &path.to_string_lossy())
    );
}

#[test]
fn schedule_enable_propagates_runner_failure() {
    let temp = tempdir().unwrap();
    let runner = MockResticProfileRunner::new(1, "systemd unavailable");

    let error = execute_schedule_enable(&temp.path().join("profiles.yml"), &runner).unwrap_err();

    assert!(error.to_string().contains("systemd unavailable"));
}

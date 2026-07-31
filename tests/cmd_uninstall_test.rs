use backup::commands::uninstall::{
    execute_uninstall_plan, perform_uninstall, perform_uninstall_at_paths,
};
use backup::commands::update::execute_update_check;

use backup::runner::resticprofile::MockResticProfileRunner;
use std::path::Path;

#[test]
fn test_uninstall_plan() {
    let plan = execute_uninstall_plan();
    assert!(plan.contains("/usr/local/sbin/backup"));
    assert!(plan.contains("/etc/backup"));
    assert!(plan.contains("resticprofile unschedule"));
}

#[test]
fn test_perform_uninstall_with_yes() {
    let runner = MockResticProfileRunner::new(0, "unscheduled");
    let res =
        perform_uninstall(Path::new("/etc/backup/profiles.yaml"), &runner, true, false).unwrap();
    assert!(res.contains("Uninstalled"));
    let calls = runner.calls.lock().unwrap();
    assert!(calls.is_empty());
}

#[test]
fn test_uninstall_uses_profiles_override_for_scheduler_cleanup() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_path = temp_dir.path().join("environment/config.yml");
    let profiles_path = temp_dir.path().join("profiles/profiles.yml");
    std::fs::create_dir_all(profiles_path.parent().unwrap()).unwrap();
    std::fs::write(&profiles_path, "version: '2'\nprofiles: {}\n").unwrap();

    let runner = MockResticProfileRunner::new(0, "unscheduled");
    perform_uninstall_at_paths(&config_path, &profiles_path, &runner, true, false).unwrap();

    let calls = runner.calls.lock().unwrap();
    assert_eq!(
        calls.as_slice(),
        [(
            "schedule_disable".into(),
            profiles_path.to_string_lossy().into()
        )]
    );
}

#[test]
fn test_perform_uninstall_with_purge() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_dir = temp_dir.path().join("backup");
    std::fs::create_dir_all(&config_dir).unwrap();
    let config_file = config_dir.join("profiles.yaml");
    std::fs::write(&config_file, "dummy").unwrap();

    let runner = MockResticProfileRunner::new(0, "unscheduled");
    let res = perform_uninstall(&config_file, &runner, true, true).unwrap();
    assert!(res.contains("Uninstalled"));
    assert!(!config_dir.exists());
}

#[test]
fn test_perform_uninstall_non_interactive_without_yes_fails() {
    let runner = MockResticProfileRunner::new(0, "unscheduled");
    let res = perform_uninstall(
        Path::new("/etc/backup/profiles.yaml"),
        &runner,
        false,
        false,
    );
    assert!(res.is_err());
}

#[test]
fn test_update_check() {
    let result = execute_update_check("1.0.0").unwrap();
    assert!(result.contains("1.0.0"));
    assert!(result.contains("up to date"));
}

use backup::commands::uninstall::{
    UninstallTargets, execute_uninstall_plan, perform_uninstall, perform_uninstall_at_path,
    perform_uninstall_with_executor_at_path_and_targets,
};
use backup::commands::update::execute_update_check;

mod support;
use std::path::Path;
use support::MockResticProfileRunner;

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
    let profiles_path = temp_dir.path().join("profiles/profiles.yml");
    std::fs::create_dir_all(profiles_path.parent().unwrap()).unwrap();
    std::fs::write(&profiles_path, "version: '2'\nprofiles: {}\n").unwrap();

    let runner = MockResticProfileRunner::new(0, "unscheduled");
    perform_uninstall_at_path(&profiles_path, &runner, true, false).unwrap();

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

#[test]
fn purge_removes_only_the_selected_configuration_scope() {
    let temp = tempfile::tempdir().unwrap();
    let scope = temp.path().join("scope");
    let reports = scope.join("reports");
    let cache = scope.join("cache");
    let systemd = temp.path().join("systemd");
    std::fs::create_dir_all(&reports).unwrap();
    std::fs::create_dir_all(&cache).unwrap();
    std::fs::create_dir_all(&systemd).unwrap();
    std::fs::write(reports.join("audit.json"), "report").unwrap();
    std::fs::write(cache.join("index"), "cache").unwrap();
    std::fs::write(scope.join("keep.txt"), "unrelated").unwrap();
    std::fs::write(systemd.join("backup.timer"), "owned").unwrap();
    std::fs::write(systemd.join("backup.timer.bak"), "unrelated").unwrap();
    std::fs::write(systemd.join("other.timer"), "unrelated").unwrap();
    let password = scope.join("primary-password");
    std::fs::write(&password, "uninstall-password").unwrap();
    let profiles = scope.join("profiles.yaml");
    std::fs::write(
        &profiles,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: true\n    enableAnnualDrDrillReport: true\nprofiles:\n  primary:\n    repository: /tmp/repository\n    password-file: primary-password\n",
            reports.display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&profiles, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::set_permissions(&password, std::fs::Permissions::from_mode(0o600)).unwrap();
    }
    let binary = temp.path().join("bin/backup");
    std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
    std::fs::write(&binary, "binary").unwrap();

    let scheduler = support::MockResticProfileRunner::new(0, "unscheduled");
    let executor = support::MockExecutor::new();
    let targets = UninstallTargets {
        binary_path: binary.clone(),
        systemd_dir: systemd.clone(),
    };

    perform_uninstall_with_executor_at_path_and_targets(
        &profiles, &scheduler, &executor, true, true, &targets,
    )
    .unwrap();

    assert!(!binary.exists());
    assert!(!profiles.exists());
    assert!(!password.exists());
    assert!(!reports.exists());
    assert!(!cache.exists());
    assert!(scope.exists());
    assert!(scope.join("keep.txt").exists());
    assert!(!systemd.join("backup.timer").exists());
    assert!(systemd.join("backup.timer.bak").exists());
    assert!(systemd.join("other.timer").exists());
    assert_eq!(
        scheduler.calls.lock().unwrap().as_slice(),
        [("schedule_disable".into(), profiles.to_string_lossy().into())]
    );
}

#[test]
fn scheduler_cleanup_failure_preserves_binary_and_configuration() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    std::fs::write(&profiles, "version: '2'\nprofiles: {}\n").unwrap();
    let binary = temp.path().join("backup");
    std::fs::write(&binary, "binary").unwrap();
    let scheduler = support::MockResticProfileRunner::new(1, "scheduler failed");
    let executor = support::MockExecutor::new();
    let targets = UninstallTargets {
        binary_path: binary.clone(),
        systemd_dir: temp.path().join("systemd"),
    };

    let error = perform_uninstall_with_executor_at_path_and_targets(
        &profiles, &scheduler, &executor, true, true, &targets,
    )
    .unwrap_err();

    assert!(error.to_string().contains("scheduler failed"));
    assert!(binary.exists());
    assert!(profiles.exists());
}

#[test]
fn purge_without_yes_fails_before_any_mutation() {
    let temp = tempfile::tempdir().unwrap();
    let profiles = temp.path().join("profiles.yaml");
    std::fs::write(&profiles, "version: '2'\nprofiles: {}\n").unwrap();
    let binary = temp.path().join("backup");
    std::fs::write(&binary, "binary").unwrap();
    let scheduler = support::MockResticProfileRunner::new(0, "unscheduled");
    let executor = support::MockExecutor::new();
    let targets = UninstallTargets {
        binary_path: binary.clone(),
        systemd_dir: temp.path().join("systemd"),
    };

    let error = perform_uninstall_with_executor_at_path_and_targets(
        &profiles, &scheduler, &executor, false, true, &targets,
    )
    .unwrap_err();

    assert!(error.to_string().contains("requires --yes"));
    assert!(binary.exists());
    assert!(profiles.exists());
    assert!(scheduler.calls.lock().unwrap().is_empty());
}

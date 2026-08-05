use backup::commands::restore::execute_restore;
use backup::commands::snapshots::execute_snapshots;
use backup::config::model::*;
mod support;
use secrecy::SecretString;
use support::MockResticRunner;

#[test]
fn restore_storage_defaults_to_primary_and_requires_an_enabled_secondary() {
    use backup::commands::restore::{RestoreStorage, select_storage};
    let mut config = BackupConfig::default();
    config.storage.primary.repository = "primary-repository".into();
    assert_eq!(
        select_storage(&config, RestoreStorage::Primary).unwrap().0,
        "primary-repository"
    );
    assert!(
        select_storage(&config, RestoreStorage::Secondary)
            .unwrap_err()
            .to_string()
            .contains("not configured")
    );
}

#[test]
fn test_execute_snapshots() {
    let mock_runner = MockResticRunner::new(0, "ID        Date\n12345678  2026-07-23");
    let config = BackupConfig {
        version: "1.0".into(),
        profile: "test".into(),
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets: vec!["/tmp".into()],
            excludes: vec![],
        },
        retention: RetentionPolicy {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "sftp".into(),
                repository: "rclone:syno:/backup".into(),
                password: SecretString::new("secret".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
        audit: AuditConfig::default(),
    };
    let result = execute_snapshots(&config, &mock_runner).unwrap();
    assert!(result.contains("12345678"));
}

#[test]
fn restore_rejects_successful_runner_that_produces_no_output() {
    let config = BackupConfig::default();
    let dir = tempfile::tempdir().unwrap();
    let error = execute_restore(
        &config,
        &MockResticRunner::new(0, "restored"),
        "12345678",
        dir.path().to_str().unwrap(),
        false,
    )
    .unwrap_err();
    assert!(error.to_string().contains("no non-empty output"));
}

#[test]
fn restore_validation_accepts_nonempty_database_sql_dump() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("nested")).unwrap();
    std::fs::write(
        dir.path().join("nested/database.sql"),
        "-- PostgreSQL database dump\nCREATE TABLE items (id int);",
    )
    .unwrap();
    backup::commands::restore::validate_restored_output(dir.path(), true).unwrap();
}

#[test]
fn restore_rejects_nonempty_target_without_force() {
    let target = tempfile::tempdir().unwrap();
    std::fs::write(target.path().join("existing.txt"), "keep").unwrap();

    let error = execute_restore(
        &BackupConfig::default(),
        &MockResticRunner::new(0, "restored"),
        "latest",
        target.path().to_str().unwrap(),
        false,
    )
    .unwrap_err();

    assert!(error.to_string().contains("pass --force"));
}

#[test]
fn restore_force_rejects_success_without_new_output() {
    let target = tempfile::tempdir().unwrap();
    std::fs::write(target.path().join("existing.txt"), "keep").unwrap();

    let error = execute_restore(
        &BackupConfig::default(),
        &MockResticRunner::new(0, "restored"),
        "latest",
        target.path().to_str().unwrap(),
        true,
    )
    .unwrap_err();

    assert!(error.to_string().contains("no new output"));
    assert_eq!(
        std::fs::read_to_string(target.path().join("existing.txt")).unwrap(),
        "keep"
    );
}

#[test]
fn restore_propagates_runner_failure_after_explicit_force() {
    let target = tempfile::tempdir().unwrap();
    std::fs::write(target.path().join("existing.txt"), "replace").unwrap();

    let error = execute_restore(
        &BackupConfig::default(),
        &MockResticRunner::new(1, "repository unavailable"),
        "latest",
        target.path().to_str().unwrap(),
        true,
    )
    .unwrap_err();

    assert!(error.to_string().contains("repository unavailable"));
}

use backup::commands::restore::execute_restore;
use backup::commands::snapshots::execute_snapshots;
use backup::config::model::*;
use backup::runner::restic::MockResticRunner;
use secrecy::SecretString;

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
fn test_execute_restore() {
    let config = BackupConfig::default();
    let dir = tempfile::tempdir().unwrap();
    let result = execute_restore(
        &config,
        &MockResticRunner::new(0, "restored"),
        "12345678",
        dir.path().to_str().unwrap(),
        false,
    )
    .unwrap();
    assert_eq!(result, "restored");
}

use backup::commands::run::{execute_run, execute_run_profile};
use backup::commands::status::execute_status;
use backup::config::model::*;
use backup::runner::restic::MockResticRunner;
use backup::runner::resticprofile::MockResticProfileRunner;
use secrecy::SecretString;
use std::path::Path;

#[test]
fn test_execute_run() {
    let mock_runner = MockResticRunner::new(0, "backup complete");
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
    };
    let result = execute_run(&config, &mock_runner).unwrap();
    assert!(result.contains("backup complete"));
}

#[test]
fn test_execute_run_profile() {
    let mock_runner = MockResticProfileRunner::new(0, "resticprofile backup complete");
    let config_path = Path::new("/etc/backup/profiles.yaml");
    let result = execute_run_profile(config_path, "self", false, &mock_runner).unwrap();
    assert!(result.contains("resticprofile backup complete"));
}

#[test]
fn test_pipeline_engine_flag_combinations() {
    use backup::commands::run::{PipelineEngine, PipelineOptions};
    let mock_runner = MockResticProfileRunner::new(0, "profile_run_ok");
    let engine = PipelineEngine::new(&mock_runner);
    let config_path = Path::new("/etc/backup/profiles.yaml");

    let opts = PipelineOptions {
        skip_database: true,
        skip_secondary_sync: true,
        skip_retention: true,
        dry_run: false,
    };
    let result = engine.execute(config_path, "self", &opts).unwrap();
    assert!(!result.contains("[Pipeline] Executed Database"));
    assert!(!result.contains("Secondary storage sync"));
    assert!(result.contains("profile_run_ok"));
}

#[test]
fn test_execute_status() {
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
    };
    let result = execute_status(&config).unwrap();
    assert!(result.contains("Profile: test"));
    assert!(result.contains("Backend: sftp"));
    assert!(result.contains("Repository: rclone:syno:/backup"));
}

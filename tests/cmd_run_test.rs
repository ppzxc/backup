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
fn test_execute_status_dynamic() {
    use backup::commands::status::execute_status_with_runner;
    use backup::runner::executor::{CommandOutput, MockExecutor};

    let config = BackupConfig {
        version: "1.0".into(),
        profile: "log".into(),
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets: vec!["/var/log".into()],
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
                repository: "rclone:syno_backup:/backup".into(),
                password: SecretString::new("secret".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
    };

    let mock_executor = MockExecutor::new();
    let json_output = r#"[
        {
            "id": "01012723",
            "time": "2026-07-24T17:31:02+09:00",
            "paths": ["/var/log"],
            "hostname": "funa1.nanoit.kr"
        }
    ]"#;
    mock_executor.push_output(
        "restic",
        CommandOutput {
            status_code: 0,
            stdout: json_output.into(),
            stderr: "".into(),
        },
    );

    let status_res = execute_status_with_runner(&config, &mock_executor, Some("log")).unwrap();
    assert!(status_res.contains("Profile: log"));
    assert!(status_res.contains("Backend: sftp"));
    assert!(status_res.contains("Repository: rclone:syno_backup:/backup"));
    assert!(status_res.contains("Targets: [\"/var/log\"]"));
    assert!(status_res.contains("Latest Snapshot: 01012723"));
    assert!(status_res.contains("Snapshot Time: 2026-07-24T17:31:02+09:00"));
}

#[test]
fn test_execute_status_fallback_on_error() {
    use backup::commands::status::execute_status_with_runner;
    use backup::runner::executor::{CommandOutput, MockExecutor};

    let config = BackupConfig {
        version: "1.0".into(),
        profile: "default".into(),
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets: vec!["/data".into()],
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
                repository: "rclone:syno_backup:/backup".into(),
                password: SecretString::new("secret".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
    };

    let mock_executor = MockExecutor::new();
    mock_executor.push_output(
        "restic",
        CommandOutput {
            status_code: 1,
            stdout: "".into(),
            stderr: "repository does not exist".into(),
        },
    );

    let status_res = execute_status_with_runner(&config, &mock_executor, None).unwrap();
    assert!(status_res.contains("Profile: default"));
    assert!(status_res.contains("[WARN] Failed to fetch snapshots"));
}

#[test]
fn test_execute_status_from_profiles_config() {
    use backup::commands::status::execute_status_from_profiles_config;
    use backup::runner::resticprofile::MockResticProfileRunner;
    use tempfile::NamedTempFile;

    let yaml_content = r#"version: '2'
profiles:
  default:
    repository: s3:https://59.25.177.53:39000/backup/ns0327/log
  log:
    inherit: default
    backup:
      source:
      - /var/log
"#;
    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), yaml_content).unwrap();

    let mock_table = "ID        Time                 Host        Tags        Paths\n------------------------------------------------------------------\nabc12345  2026-07-24 17:40:00  funa1                   /var/log";

    let mock_runner = MockResticProfileRunner::new(0, mock_table);
    let status_res = execute_status_from_profiles_config(temp_file.path(), Some("log"), &mock_runner).unwrap();

    assert!(status_res.contains("Profile: log"));
    assert!(status_res.contains("Repository: s3:https://59.25.177.53:39000/backup/ns0327/log"));
    assert!(status_res.contains("Targets: [\"/var/log\"]"));
    assert!(status_res.contains("abc12345"));
}

#[test]
fn test_execute_run_profile() {
    use backup::commands::run::PipelineOptions;
    let mock_runner = MockResticProfileRunner::new(0, "resticprofile backup complete");
    let config_path = Path::new("/etc/backup/profiles.yaml");
    let opts = PipelineOptions {
        dry_run: false,
        skip_database: false,
        skip_secondary_sync: false,
        skip_retention: false,
    };
    let result = execute_run_profile(config_path, "self", &opts, &mock_runner).unwrap();
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

#[test]
fn test_backend_migrate() {
    use backup::commands::backend::execute_backend_migrate;
    use backup::runner::rclone::MockRcloneRunner;
    let mock = MockRcloneRunner::new(0, "sync ok");
    let res = execute_backend_migrate(&mock, "primary:backup", "secondary:backup").unwrap();
    assert!(res.contains("Backend snapshot migration completed"));
}


use backup::commands::database::execute_database_backup;
use backup::commands::run::{execute_run, execute_run_profile};
use backup::commands::status::execute_status;
use backup::config::model::*;
mod support;
use secrecy::SecretString;
use std::path::Path;
use support::{MockResticProfileRunner, MockResticRunner};

#[test]
fn execution_reports_capture_failures_without_exposing_storage_passwords() {
    use backup::commands::run::{ExecutionReport, write_execution_report};
    let directory = tempfile::tempdir().unwrap();
    let mut config = BackupConfig::default();
    config.storage.primary.password = SecretString::new("top-secret-password".into());
    config.reports.output_dir = directory.path().to_string_lossy().into_owned();
    let report = ExecutionReport::failure(
        "default",
        "secondary-sync",
        "copy failed: top-secret-password",
    );

    let path = write_execution_report(&config, report).unwrap();
    let contents = std::fs::read_to_string(path).unwrap();
    assert!(contents.contains("secondary-sync"));
    assert!(contents.contains("******"));
    assert!(!contents.contains("top-secret-password"));
}

#[test]
fn execution_reports_mask_s3_and_database_credentials() {
    use backup::commands::run::{ExecutionReport, write_execution_report};
    let directory = tempfile::tempdir().unwrap();
    let mut config = BackupConfig::default();
    config.reports.output_dir = directory.path().to_string_lossy().into_owned();
    config.storage.primary.s3 = Some(S3Config {
        endpoint: "https://s3.example".into(),
        access_key_id: SecretString::new("access-key".into()),
        secret_access_key: SecretString::new("s3-secret".into()),
    });
    config.backup.backup_type = BackupType::DbStream {
        db_type: DatabaseType::Postgres,
        connection_url: Some("postgres://user:db-secret@host/db".into()),
    };
    let path = write_execution_report(
        &config,
        ExecutionReport::failure(
            "default",
            "database",
            "access-key s3-secret postgres://user:db-secret@host/db",
        ),
    )
    .unwrap();
    let contents = std::fs::read_to_string(path).unwrap();
    for secret in [
        "access-key",
        "s3-secret",
        "postgres://user:db-secret@host/db",
    ] {
        assert!(!contents.contains(secret));
    }
}

#[test]
fn execution_reports_record_the_primary_snapshot_id() {
    use backup::commands::run::ExecutionReport;

    let report = ExecutionReport::success("default", "snapshot abc123 saved\n".into(), None, None);

    assert_eq!(report.snapshot_id.as_deref(), Some("abc123"));
}

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
        audit: AuditConfig::default(),
    };
    let result = execute_run(&config, &mock_runner).unwrap();
    assert!(result.contains("backup complete"));
}

#[test]
fn test_database_stream_uses_database_adapter() {
    let mut config = BackupConfig::default();
    config.backup.backup_type = BackupType::DbStream {
        db_type: DatabaseType::Postgres,
        connection_url: Some("postgres://postgres:secret@localhost:5432/app".into()),
    };
    let output =
        execute_database_backup(&config, &MockResticRunner::new(0, "streamed"), false).unwrap();
    assert_eq!(output, "streamed");
}

#[test]
fn database_stream_rejects_missing_connection_url() {
    let mut config = BackupConfig::default();
    config.backup.backup_type = BackupType::DbStream {
        db_type: DatabaseType::Postgres,
        connection_url: None,
    };

    let error =
        execute_database_backup(&config, &MockResticRunner::new(0, "unused"), false).unwrap_err();

    assert!(error.to_string().contains("connection URL"));
}

#[test]
fn mysql_database_stream_uses_portable_dump_arguments() {
    let mut config = BackupConfig::default();
    config.backup.backup_type = BackupType::DbStream {
        db_type: DatabaseType::Mysql,
        connection_url: Some("mysql://root:secret@db:3306/app".into()),
    };
    let runner = MockResticRunner::new(0, "streamed");

    execute_database_backup(&config, &runner, false).unwrap();

    let calls = runner.command_calls.lock().unwrap();
    assert_eq!(calls[0].0, "mysqldump");
    assert!(!calls[0].1.contains(&"--skip-generated-columns".to_string()));
}

#[test]
fn database_stream_rejects_url_without_database_name() {
    let mut config = BackupConfig::default();
    config.backup.backup_type = BackupType::DbStream {
        db_type: DatabaseType::Mysql,
        connection_url: Some("mysql://root:secret@db:3306/".into()),
    };

    let error =
        execute_database_backup(&config, &MockResticRunner::new(0, "unused"), false).unwrap_err();

    assert!(error.to_string().contains("database name"));
}

#[test]
fn database_stream_propagates_restic_failure() {
    let mut config = BackupConfig::default();
    config.backup.backup_type = BackupType::DbStream {
        db_type: DatabaseType::Postgres,
        connection_url: Some("postgres://postgres:secret@db:5432/app".into()),
    };

    let error = execute_database_backup(
        &config,
        &MockResticRunner::new(1, "repository unavailable"),
        false,
    )
    .unwrap_err();

    assert!(error.to_string().contains("repository unavailable"));
}

#[test]
fn test_execute_status_dynamic() {
    use crate::support::MockExecutor;
    use backup::commands::status::execute_status_with_runner;
    use backup::runner::executor::CommandOutput;

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
        audit: AuditConfig::default(),
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
    use crate::support::MockExecutor;
    use backup::commands::status::execute_status_with_runner;
    use backup::runner::executor::CommandOutput;

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
        audit: AuditConfig::default(),
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
    let status_res =
        execute_status_from_profiles_config(temp_file.path(), Some("log"), &mock_runner).unwrap();

    assert!(status_res.contains("Profile: log"));
    assert!(status_res.contains("Repository: s3:https://59.25.177.53:39000/backup/ns0327/log"));
    assert!(status_res.contains("Targets: [\"/var/log\"]"));
    assert!(status_res.contains("abc12345"));
}

#[test]
fn test_execute_status_from_profiles_config_all_profiles() {
    use backup::commands::status::execute_status_from_profiles_config;
    use tempfile::NamedTempFile;

    let yaml_content = r#"version: '2'
profiles:
  default:
    insecure-tls: true
  primary:
    repository: s3:https://59.25.177.53:39000/backup/ns0327/log
  log:
    inherit: primary
    backup:
      source:
      - /var/log
"#;
    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), yaml_content).unwrap();

    let mock_runner = MockResticProfileRunner::new(0, "mock snapshot output");
    let status_res =
        execute_status_from_profiles_config(temp_file.path(), None, &mock_runner).unwrap();

    assert!(!status_res.contains("Profile: default"));
    assert!(!status_res.contains("Profile: primary"));
    assert!(status_res.contains("Profile: log"));
    assert!(status_res.contains("Repository: s3:https://59.25.177.53:39000/backup/ns0327/log"));
}

#[test]
fn test_execute_status_from_profiles_config_no_active_profiles() {
    use backup::commands::status::execute_status_from_profiles_config;
    use tempfile::NamedTempFile;

    let yaml_content = r#"version: '2'
profiles:
  default:
    insecure-tls: true
  primary:
    repository: s3:https://59.25.177.53:39000/backup/ns0327/log
"#;
    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), yaml_content).unwrap();

    let mock_runner = MockResticProfileRunner::new(0, "mock snapshot output");
    let status_res =
        execute_status_from_profiles_config(temp_file.path(), None, &mock_runner).unwrap();

    assert_eq!(
        status_res,
        "No active backup profiles found in configuration."
    );
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
    let calls = mock_runner.calls.lock().unwrap();
    assert_eq!(
        calls
            .iter()
            .map(|(call, _)| call.as_str())
            .collect::<Vec<_>>(),
        ["backup"]
    );
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
        audit: AuditConfig::default(),
    };
    let result = execute_status(&config).unwrap();
    assert!(result.contains("Profile: test"));
    assert!(result.contains("Backend: sftp"));
    assert!(result.contains("Repository: rclone:syno:/backup"));
}

#[test]
fn test_copy() {
    use backup::commands::copy::execute_copy;
    use std::path::Path;
    let mock = MockResticProfileRunner::new(0, "copy ok");
    let res = execute_copy(
        &mock,
        Path::new("/etc/backup/profiles.yaml"),
        "default",
        false,
    )
    .unwrap();
    assert!(res.contains("Snapshot copy completed for profile [default]"));
}

use backup::commands::setup::{create_default_config_file, run_setup_with_prompter, SetupParams, SetupPrompter};
use backup::config::model::*;
use backup::i18n::Language;
use secrecy::SecretString;
use tempfile::tempdir;

#[test]
fn test_create_default_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("config.yml");
    create_default_config_file(&config_path, "host1", "/data", "s3:bucket", "secret").unwrap();
    assert!(config_path.exists());

    let config = BackupConfig::load_from_path(&config_path).unwrap();
    assert_eq!(config.profile, "host1");
    assert_eq!(config.storage.primary.sftp.as_ref().unwrap().key_file.as_deref(), Some("/etc/backup/id_rsa"));
    assert!(config.reports.enable_daily_reports);
    assert!(config.reports.enable_annual_dr_drill_report);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let parent_perms = std::fs::metadata(dir.path()).unwrap().permissions();
        assert_eq!(parent_perms.mode() & 0o777, 0o700);
        let file_perms = std::fs::metadata(&config_path).unwrap().permissions();
        assert_eq!(file_perms.mode() & 0o777, 0o600);
    }
}

struct MockPrompter {
    params: SetupParams,
}

impl SetupPrompter for MockPrompter {
    fn prompt_setup_params(&self, _lang_opt: Option<Language>) -> anyhow::Result<SetupParams> {
        if self.params.primary_storage.backend == "sftp" {
            let key = self.params.primary_storage.sftp.as_ref().and_then(|s| s.key_file.as_deref()).unwrap_or("");
            if key.trim().is_empty() {
                anyhow::bail!("ISMS Compliance Error: SFTP requires SSH key_file path");
            }
        }
        if secrecy::ExposeSecret::expose_secret(&self.params.primary_storage.password).len() < 12 {
            anyhow::bail!("ISMS Compliance Error: Password must be at least 12 characters long.");
        }
        Ok(SetupParams {
            profile: self.params.profile.clone(),
            backup_type: self.params.backup_type.clone(),
            targets: self.params.targets.clone(),
            excludes: self.params.excludes.clone(),
            retention: self.params.retention.clone(),
            primary_storage: self.params.primary_storage.clone(),
            secondary_storage: self.params.secondary_storage.clone(),
            reports: self.params.reports.clone(),
        })
    }
}

#[test]
fn test_setup_with_prompter_success() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profile_db.yaml");

    let params = SetupParams {
        profile: "profile-db".into(),
        backup_type: BackupType::DbStream {
            db_type: "postgres".into(),
            connection_url: Some("postgresql://user:pass@localhost:5432/mydb".into()),
            dump_command: None,
        },
        targets: vec!["db-stream:postgres".into()],
        excludes: vec![],
        retention: RetentionPolicy { keep_daily: 180, keep_weekly: 12, keep_monthly: 24 },
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:backup@remote:/storage".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: Some(SftpConfig {
                host: "remote".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some("/etc/backup/id_rsa".into()),
            }),
            s3: None,
        },
        secondary_storage: Some(SecondaryStorageTarget {
            enabled: true,
            backend: "s3".into(),
            repository: "s3:offsite-bucket".into(),
            password: SecretString::new("secondary_pass_123".into()),
        }),
        reports: ReportsConfig {
            output_dir: "/var/log/backup/reports".into(),
            enable_daily_reports: true,
            enable_annual_dr_drill_report: true,
        },
    };

    let prompter = MockPrompter { params };
    run_setup_with_prompter(&config_path, &prompter, false, Some(Language::En)).unwrap();

    let loaded = BackupConfig::load_from_path(&config_path).unwrap();
    assert_eq!(loaded.profile, "profile-db");
    assert_eq!(loaded.retention.keep_daily, 180);
    assert!(loaded.storage.secondary.as_ref().unwrap().enabled);
    assert!(loaded.reports.enable_annual_dr_drill_report);
}

#[test]
fn test_setup_engine_validation_rules() {
    use backup::commands::setup::SetupEngine;

    let mut params = SetupParams {
        profile: "test".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/data".into()],
        excludes: vec![],
        retention: RetentionPolicy { keep_daily: 30, keep_weekly: 4, keep_monthly: 12 },
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:host:/var/backups".into(),
            password: SecretString::new("short_pass".into()),
            sftp: Some(SftpConfig {
                host: "host".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some("/etc/backup/id_rsa".into()),
            }),
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig::default(),
    };

    // Password < 12 characters failure
    let err = SetupEngine::validate_and_build(params.clone()).unwrap_err();
    assert!(err.to_string().contains("ISMS Compliance Error: Password must be at least 12 characters long."));

    // Fix password
    params.primary_storage.password = SecretString::new("valid_password_123".into());

    // SFTP key empty failure
    params.primary_storage.sftp.as_mut().unwrap().key_file = Some("".into());
    let err = SetupEngine::validate_and_build(params.clone()).unwrap_err();
    assert!(err.to_string().contains("ISMS Compliance Error: SFTP requires SSH key_file path"));

    // Valid setup build
    params.primary_storage.sftp.as_mut().unwrap().key_file = Some("/etc/backup/id_rsa".into());
    let config = SetupEngine::validate_and_build(params).unwrap();
    assert_eq!(config.profile, "test");
}

#[test]
fn test_run_setup_dependencies_with_mock_runner() {
    use backup::commands::setup::run_setup_dependencies_with_runner;
    use backup::runner::executor::{CommandOutput, MockExecutor};

    let mock = MockExecutor::new();
    mock.push_output("which", CommandOutput {
        status_code: 0,
        stdout: "/usr/bin/restic\n".into(),
        stderr: "".into(),
    });
    mock.push_output("which", CommandOutput {
        status_code: 1,
        stdout: "".into(),
        stderr: "not found".into(),
    });
    mock.push_output("sh", CommandOutput {
        status_code: 0,
        stdout: "".into(),
        stderr: "".into(),
    });
    mock.push_output("which", CommandOutput {
        status_code: 0,
        stdout: "/usr/bin/resticprofile\n".into(),
        stderr: "".into(),
    });

    let report = run_setup_dependencies_with_runner(&mock).unwrap();
    assert!(report.contains("restic: OK (/usr/bin/restic)"));
    assert!(report.contains("rclone: MISSING -> Installing from"));
    assert!(report.contains("resticprofile: OK (/usr/bin/resticprofile)"));
}

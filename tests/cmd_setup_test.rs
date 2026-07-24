use backup::commands::setup::{create_default_config_file, run_setup_with_prompter, SetupParams, SetupPrompter};
use backup::config::model::*;
use backup::i18n::Language;
use secrecy::SecretString;
use tempfile::tempdir;

#[test]
fn test_create_default_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");
    create_default_config_file(&config_path, "host1", "/var/log", "sftp:backup@192.168.1.100:/backup", "secret_pass_123").unwrap();
    assert!(config_path.exists());

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("host1"));
    assert!(content.contains("sftp:backup@192.168.1.100:/backup"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
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
    let config_dir = dir.path();
    let config_path = config_dir.join("profiles.yaml");

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
            repository: "sftp:backup@192.168.1.100:/storage".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: Some(SftpConfig {
                host: "192.168.1.100".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some("/etc/backup/id_ed25519".into()),
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
            output_dir: "/data/backup/reports".into(),
            enable_daily_reports: true,
            enable_annual_dr_drill_report: true,
        },
    };

    let prompter = MockPrompter { params };
    run_setup_with_prompter(&config_path, &prompter, false, Some(Language::En)).unwrap();

    assert!(config_path.exists());
    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("profile-db"));
    assert!(content.contains("sftp:backup@192.168.1.100:/storage"));
    assert!(content.contains("keep-daily: 180"));
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

#[test]
fn test_generate_secure_password_length_and_complexity() {
    use backup::commands::setup::generate_secure_password;
    let pwd = generate_secure_password();
    assert_eq!(pwd.len(), 32, "자동 생성 비밀번호 길이는 32자여야 합니다");
    assert!(pwd.chars().any(|c| c.is_ascii_uppercase()), "대문자가 포함되어야 합니다");
    assert!(pwd.chars().any(|c| c.is_ascii_lowercase()), "소문자가 포함되어야 합니다");
    assert!(pwd.chars().any(|c| c.is_ascii_digit()), "숫자가 포함되어야 합니다");
}

#[test]
fn test_resolve_encryption_keyfile_uses_existing_file() {
    use backup::commands::setup::resolve_encryption_keyfile;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let enc_path = dir.path().join("enc");
    std::fs::write(&enc_path, "existing_secret_password_12345\n").unwrap();

    let pwd = resolve_encryption_keyfile(&enc_path).unwrap();
    assert_eq!(pwd, "existing_secret_password_12345");
}

#[test]
fn test_save_encryption_keyfile_permission_600() {
    use backup::commands::setup::save_encryption_keyfile;
    use tempfile::tempdir;

    let dir = tempdir().unwrap();
    let enc_path = dir.path().join("sub/enc");
    save_encryption_keyfile(&enc_path, "generated_secret_password_12345").unwrap();

    assert!(enc_path.exists());
    let content = std::fs::read_to_string(&enc_path).unwrap();
    assert_eq!(content.trim(), "generated_secret_password_12345");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::metadata(&enc_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o600, "enc 키파일 권한은 600이어야 합니다");
        let dir_perms = std::fs::metadata(enc_path.parent().unwrap()).unwrap().permissions();
        assert_eq!(dir_perms.mode() & 0o777, 0o700, "etc/backup 디렉터리 권한은 700이어야 합니다");
    }
}


/// lang_opt이 None일 때 Language::detect()로 언어를 자동 감지하여 프롬프트 없이 진행하는지 검증.
/// setup 내부 prompter가 받은 lang_opt이 Some(..)이어야 합니다.
#[test]
fn test_setup_auto_detects_language_when_lang_opt_none() {
    use std::sync::{Arc, Mutex};

    struct CapturingPrompter {
        received_lang: Arc<Mutex<Option<Language>>>,
    }
    impl SetupPrompter for CapturingPrompter {
        fn prompt_setup_params(&self, lang_opt: Option<Language>) -> anyhow::Result<SetupParams> {
            *self.received_lang.lock().unwrap() = lang_opt;
            anyhow::bail!("capture_only") // 언어 캡처가 목적이므로 에러로 조기 종료
        }
    }

    let received = Arc::new(Mutex::new(None::<Language>));
    let prompter = CapturingPrompter { received_lang: Arc::clone(&received) };
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");

    // lang_opt = None으로 호출 → run_setup_with_prompter 내부에서 detect()로 채워 Some(..)으로 전달
    let _ = run_setup_with_prompter(&config_path, &prompter, false, None);

    let captured = received.lock().unwrap();
    assert!(
        captured.is_some(),
        "lang_opt이 None이면 Language::detect()로 채워 Some(..)을 prompter에 전달해야 합니다"
    );
}

#[test]
fn test_setup_auto_enables_schedule() {
    use backup::commands::setup::run_setup_with_prompter_and_runner;
    use backup::runner::resticprofile::MockResticProfileRunner;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");

    let params = SetupParams {
        profile: "default".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:backup@192.168.1.100:/storage".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: Some(SftpConfig {
                host: "192.168.1.100".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some("/etc/backup/id_ed25519".into()),
            }),
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig::default(),
    };

    let prompter = MockPrompter { params };
    let runner = MockResticProfileRunner::new(0, "scheduled successfully");

    run_setup_with_prompter_and_runner(&config_path, &prompter, false, Some(Language::En), &runner).unwrap();

    assert!(config_path.exists());
    let mock_calls = runner.calls.lock().unwrap();
    assert_eq!(mock_calls.len(), 1);
    assert_eq!(mock_calls[0].0, "schedule_enable");
}

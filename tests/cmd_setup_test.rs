mod support;
use backup::commands::setup::{
    SetupEngine, SetupParams, SetupPrompter, create_default_profiles_file, discard_pending_setup,
    pending_setup_profiles_path, promote_pending_setup, render_sftp_diagnostic_summary,
    resolve_reused_sftp_config, run_setup_with_prompter, run_setup_with_prompter_and_runners,
};
use backup::config::model::*;
use backup::config::profile_resolver::ProfileResolver;
use backup::i18n::Language;
use secrecy::SecretString;
use support::{MockExecutor, MockResticProfileRunner};
use tempfile::tempdir;

#[test]
fn test_create_default_profiles_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");
    create_default_profiles_file(
        &config_path,
        "host1",
        "/var/log",
        "sftp:backup@192.168.1.100:/backup",
        "secret_pass_123",
    )
    .unwrap();
    assert!(config_path.exists());

    let content = std::fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("host1"));
    assert!(content.contains("sftp:backup@192.168.1.100:/backup"));
    assert!(content.contains("backup-profile:host1"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let file_perms = std::fs::metadata(&config_path).unwrap().permissions();
        assert_eq!(file_perms.mode() & 0o777, 0o600);
    }
}

#[test]
fn setup_reserves_an_exact_backup_profile_tag_without_dropping_user_tags() {
    let dir = tempdir().unwrap();
    let profiles = dir.path().join("profiles.yaml");
    std::fs::write(
        &profiles,
        r#"
version: "2"
profiles:
  primary:
    repository: /primary
  default:
    backup:
      tag: ["inherited-user-tag"]
  daily:
    inherit: default
    backup:
      source: [/data]
      tag: ["user-tag", "backup-profile:stale"]
"#,
    )
    .unwrap();

    BackupConfig {
        profile: "daily".into(),
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets: vec!["/data".into()],
            excludes: vec![],
        },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "local".into(),
                repository: "/primary".into(),
                password: SecretString::new("primary-password".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
        retention: RetentionPolicy::standard_defaults(),
        reports: ReportsConfig::default(),
        audit: AuditConfig::default(),
        version: "2".into(),
    }
    .save_to_profiles_path(&profiles)
    .unwrap();

    let saved = ResticProfileConfig::load_from_path(&profiles).unwrap();
    let tags = saved
        .profiles
        .get("daily")
        .and_then(|profile| profile.backup.as_ref())
        .and_then(|backup| backup.tag.as_ref())
        .unwrap();
    assert_eq!(tags, &["user-tag", "daily", "backup-profile:daily"]);
    assert_eq!(
        ProfileResolver::resolve_backup_tags(&saved, "daily").unwrap(),
        vec![
            "user-tag",
            "daily",
            "backup-profile:daily",
            "inherited-user-tag"
        ]
    );
    assert!(!tags.iter().any(|tag| tag == "backup-profile:stale"));
}

#[test]
fn setup_preserves_inherited_user_tags_when_assigning_the_exact_profile_tag() {
    let dir = tempdir().unwrap();
    let profiles = dir.path().join("profiles.yaml");
    std::fs::write(
        &profiles,
        r#"
version: "2"
profiles:
  primary:
    repository: /primary
  default:
    backup:
      source: [/data]
      tag: ["inherited-user-tag"]
  daily:
    inherit: default
"#,
    )
    .unwrap();

    BackupConfig {
        profile: "daily".into(),
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets: vec!["/data".into()],
            excludes: vec![],
        },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "local".into(),
                repository: "/primary".into(),
                password: SecretString::new("primary-password".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
        retention: RetentionPolicy::standard_defaults(),
        reports: ReportsConfig::default(),
        audit: AuditConfig::default(),
        version: "2".into(),
    }
    .save_to_profiles_path(&profiles)
    .unwrap();

    let saved = ResticProfileConfig::load_from_path(&profiles).unwrap();
    let tags = saved
        .profiles
        .get("daily")
        .and_then(|profile| profile.backup.as_ref())
        .and_then(|backup| backup.tag.as_ref())
        .unwrap();
    assert_eq!(tags, &["daily", "backup-profile:daily"]);
    assert_eq!(
        ProfileResolver::resolve_backup_tags(&saved, "daily").unwrap(),
        vec![
            "daily",
            "backup-profile:daily",
            "inherited-user-tag",
            "default",
            "backup-profile:default",
        ]
    );
}

struct MockPrompter {
    params: SetupParams,
}

impl SetupPrompter for MockPrompter {
    fn prompt_setup_params(
        &self,
        _lang_opt: Option<Language>,
        _config_dir: &std::path::Path,
        _profiles_path: &std::path::Path,
    ) -> anyhow::Result<SetupParams> {
        if self.params.primary_storage.backend == "sftp" {
            let key = self
                .params
                .primary_storage
                .sftp
                .as_ref()
                .and_then(|s| s.key_file.as_deref())
                .unwrap_or("");
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
            audit: self.params.audit.clone(),
        })
    }

    fn prompt_confirm_save_on_init_failure(&self, _msg: &str) -> anyhow::Result<bool> {
        Ok(false)
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
            db_type: DatabaseType::Postgres,
            connection_url: Some("postgresql://user:pass@localhost:5432/mydb".into()),
        },
        targets: vec!["db-stream:postgres".into()],
        excludes: vec![],
        retention: RetentionPolicy {
            keep_daily: 180,
            keep_weekly: 12,
            keep_monthly: 24,
        },
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:backup@192.168.1.100:/storage".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: Some(SftpConfig {
                host: "192.168.1.100".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some(config_dir.join("id_ed25519").to_string_lossy().into_owned()),
                additional_args: Vec::new(),
            }),
            s3: None,
        },
        secondary_storage: Some(SecondaryStorageTarget {
            enabled: true,
            backend: "s3".into(),
            repository: "s3:offsite-bucket".into(),
            password: SecretString::new("secondary_pass_123".into()),
            sftp: None,
            s3: None,
        }),
        reports: ReportsConfig {
            output_dir: config_dir.join("reports").to_string_lossy().into_owned(),
            enable_daily_reports: true,
            enable_annual_dr_drill_report: true,
        },
        audit: AuditConfig {
            system_manager: Some("홍길동".into()),
            security_officer: Some("김보안".into()),
            restore_drill_rto_minutes: None,
            restore_drill_timeout_minutes: None,
            restore_drill_work_dir: None,
        },
    };

    let prompter = MockPrompter { params };
    let runner = MockResticProfileRunner::new(0, "initialized");
    let scheduler = support::MockScheduler::new(0, "scheduled");
    run_setup_with_prompter_and_runners(
        &config_path,
        &prompter,
        false,
        Some(Language::En),
        &runner,
        &scheduler,
    )
    .unwrap();

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
        retention: RetentionPolicy {
            keep_daily: 30,
            keep_weekly: 4,
            keep_monthly: 12,
        },
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:host:/var/backups".into(),
            password: SecretString::new("short_pass".into()),
            sftp: Some(SftpConfig {
                host: "host".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some("/etc/backup/id_rsa".into()),
                additional_args: Vec::new(),
            }),
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig::default(),
        audit: AuditConfig::default(),
    };

    // Password < 12 characters failure
    let err = SetupEngine::validate_and_build(params.clone()).unwrap_err();
    assert!(
        err.to_string()
            .contains("ISMS Compliance Error: Password must be at least 12 characters long.")
    );

    // Fix password
    params.primary_storage.password = SecretString::new("valid_password_123".into());

    // SFTP key empty failure
    params.primary_storage.sftp.as_mut().unwrap().key_file = Some("".into());
    let err = SetupEngine::validate_and_build(params.clone()).unwrap_err();
    assert!(
        err.to_string()
            .contains("ISMS Compliance Error: SFTP requires SSH key_file path")
    );

    // Valid setup build
    params.primary_storage.sftp.as_mut().unwrap().key_file = Some("/etc/backup/id_rsa".into());
    let config = SetupEngine::validate_and_build(params).unwrap();
    assert_eq!(config.profile, "test");
}

#[test]
fn test_setup_engine_run_backend_init_failure_masks_secrets_in_non_interactive_mode() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");

    let params = SetupParams {
        profile: "test-s3".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        },
        primary_storage: StorageTarget {
            backend: "s3".into(),
            repository: "s3:https://59.25.177.53:3900/backup/log".into(),
            password: SecretString::new("super_secret_password_123".into()),
            sftp: None,
            s3: Some(S3Config {
                endpoint: "https://59.25.177.53:3900".into(),
                access_key_id: SecretString::new("access_key_id_xyz".into()),
                secret_access_key: SecretString::new("super_secret_s3_key_12345".into()),
            }),
        },
        secondary_storage: None,
        reports: ReportsConfig {
            output_dir: dir.path().join("reports").to_string_lossy().into_owned(),
            ..ReportsConfig::default()
        },
        audit: AuditConfig::default(),
    };

    let config = SetupEngine::validate_and_build(params.clone()).unwrap();
    config.save_to_profiles_path(&config_path).unwrap();

    let prompter = MockPrompter { params };
    let failing_runner = MockResticProfileRunner::new(
        1,
        "connection timeout to https://59.25.177.53:3900 with secret super_secret_s3_key_12345",
    );
    let scheduler = support::MockScheduler::new(0, "scheduled");

    let err = run_setup_with_prompter_and_runners(
        &config_path,
        &prompter,
        true,
        Some(Language::En),
        &failing_runner,
        &scheduler,
    )
    .unwrap_err();

    let err_str = err.to_string();
    assert!(err_str.contains("******"));
    assert!(!err_str.contains("super_secret_s3_key_12345"));
    assert!(!err_str.contains("super_secret_password_123"));
}

struct ConfirmSaveMockPrompter {
    params: SetupParams,
}

impl SetupPrompter for ConfirmSaveMockPrompter {
    fn prompt_setup_params(
        &self,
        _lang_opt: Option<Language>,
        _config_dir: &std::path::Path,
        _profiles_path: &std::path::Path,
    ) -> anyhow::Result<SetupParams> {
        Ok(self.params.clone())
    }

    fn prompt_confirm_save_on_init_failure(&self, _msg: &str) -> anyhow::Result<bool> {
        Ok(true)
    }
}

#[test]
fn test_setup_engine_saves_config_when_user_confirms_save_on_init_failure() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");
    std::fs::write(&config_path, "previous live configuration").unwrap();

    let params = SetupParams {
        profile: "test-save-on-fail".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy::standard_defaults(),
        primary_storage: StorageTarget {
            backend: "s3".into(),
            repository: "s3:https://59.25.177.53:3900/backup/log".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: None,
            s3: Some(S3Config {
                endpoint: "https://59.25.177.53:3900".into(),
                access_key_id: SecretString::new("access_key".into()),
                secret_access_key: SecretString::new("secret_key".into()),
            }),
        },
        secondary_storage: None,
        reports: ReportsConfig {
            output_dir: dir.path().join("reports").to_string_lossy().into_owned(),
            ..ReportsConfig::default()
        },
        audit: AuditConfig::default(),
    };

    let prompter = ConfirmSaveMockPrompter { params };
    let failing_runner = MockResticProfileRunner::new(1, "s3 endpoint connection timeout");
    let scheduler = support::MockScheduler::new(0, "scheduled");

    let res = run_setup_with_prompter_and_runners(
        &config_path,
        &prompter,
        false,
        Some(Language::Ko),
        &failing_runner,
        &scheduler,
    );

    assert!(res.is_ok());
    assert_eq!(
        std::fs::read_to_string(&config_path).unwrap(),
        "previous live configuration"
    );
    let pending_profiles = pending_setup_profiles_path(&config_path);
    assert!(pending_profiles.exists());
    assert!(
        std::fs::read_to_string(&pending_profiles)
            .unwrap()
            .contains("test-save-on-fail")
    );
    assert!(scheduler.calls.lock().unwrap().is_empty());

    assert!(promote_pending_setup(&config_path).unwrap());
    assert!(
        std::fs::read_to_string(&config_path)
            .unwrap()
            .contains("test-save-on-fail")
    );
    assert!(!pending_profiles.exists());
    discard_pending_setup(&config_path).unwrap();
}

struct MutatingFailurePrompter;

impl SetupPrompter for MutatingFailurePrompter {
    fn prompt_setup_params(
        &self,
        _lang_opt: Option<Language>,
        config_dir: &std::path::Path,
        _profiles_path: &std::path::Path,
    ) -> anyhow::Result<SetupParams> {
        std::fs::write(config_dir.join("id_ed25519"), "new-key").unwrap();
        std::fs::write(config_dir.join("known_hosts"), "new-host-key").unwrap();
        anyhow::bail!("operator cancelled setup")
    }
}

#[test]
fn setup_cancellation_restores_preexisting_sftp_artifacts() {
    let dir = tempdir().unwrap();
    let profiles = dir.path().join("profiles.yaml");
    std::fs::write(&profiles, "previous configuration").unwrap();
    std::fs::write(dir.path().join("id_ed25519"), "old-key").unwrap();
    std::fs::write(dir.path().join("known_hosts"), "old-host-key\n").unwrap();

    let error = run_setup_with_prompter_and_runners(
        &profiles,
        &MutatingFailurePrompter,
        false,
        Some(Language::En),
        &MockResticProfileRunner::new(0, "unused"),
        &support::MockScheduler::new(0, "unused"),
    )
    .unwrap_err();

    assert!(error.to_string().contains("operator cancelled setup"));
    assert_eq!(
        std::fs::read_to_string(dir.path().join("id_ed25519")).unwrap(),
        "old-key"
    );
    assert_eq!(
        std::fs::read_to_string(dir.path().join("known_hosts")).unwrap(),
        "old-host-key\n"
    );
    assert_eq!(
        std::fs::read_to_string(&profiles).unwrap(),
        "previous configuration"
    );
}

#[test]
fn setup_cancellation_removes_new_sftp_artifacts() {
    let dir = tempdir().unwrap();
    let profiles = dir.path().join("profiles.yaml");

    let _ = run_setup_with_prompter_and_runners(
        &profiles,
        &MutatingFailurePrompter,
        false,
        Some(Language::En),
        &MockResticProfileRunner::new(0, "unused"),
        &support::MockScheduler::new(0, "unused"),
    );

    assert!(!dir.path().join("id_ed25519").exists());
    assert!(!dir.path().join("known_hosts").exists());
    assert!(!profiles.exists());
}

#[test]
fn test_run_setup_dependencies_with_mock_runner() {
    use backup::commands::setup::run_setup_dependencies_with_runner;
    use backup::runner::executor::CommandOutput;

    let mock = MockExecutor::new();
    mock.push_output(
        "which",
        CommandOutput {
            status_code: 0,
            stdout: "/usr/bin/restic\n".into(),
            stderr: "".into(),
        },
    );
    mock.push_output(
        "which",
        CommandOutput {
            status_code: 1,
            stdout: "".into(),
            stderr: "not found".into(),
        },
    );
    mock.push_output(
        "sh",
        CommandOutput {
            status_code: 0,
            stdout: "".into(),
            stderr: "".into(),
        },
    );
    mock.push_output(
        "which",
        CommandOutput {
            status_code: 0,
            stdout: "/usr/bin/rclone\n".into(),
            stderr: "".into(),
        },
    );
    mock.push_output(
        "which",
        CommandOutput {
            status_code: 0,
            stdout: "/usr/bin/resticprofile\n".into(),
            stderr: "".into(),
        },
    );

    let report = run_setup_dependencies_with_runner(&mock).unwrap();
    assert!(report.contains("restic: OK (/usr/bin/restic)"));
    assert!(report.contains("rclone: MISSING -> Installing from"));
    assert!(report.contains("resticprofile: OK (/usr/bin/resticprofile)"));
}

#[test]
fn setup_dependencies_attempts_all_missing_binaries_and_aggregates_install_failures() {
    use backup::commands::setup::run_setup_dependencies_with_runner_at_dir;
    use backup::runner::executor::CommandOutput;

    let directory = tempdir().unwrap();
    let runner = MockExecutor::new();
    for response in [
        (1, "", "missing"),
        (0, "/tmp/restic\n", ""),
        (1, "", "missing"),
        (0, "/tmp/rclone\n", ""),
        (1, "", "missing"),
    ] {
        runner.push_output(
            "which",
            CommandOutput {
                status_code: response.0,
                stdout: response.1.into(),
                stderr: response.2.into(),
            },
        );
    }
    for _ in 0..2 {
        runner.push_output(
            "sh",
            CommandOutput {
                status_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            },
        );
    }
    runner.push_output(
        "sh",
        CommandOutput {
            status_code: 1,
            stdout: String::new(),
            stderr: "download failed".into(),
        },
    );
    let error = run_setup_dependencies_with_runner_at_dir(
        &runner,
        &directory.path().join("bin"),
        Language::En,
    )
    .unwrap_err();

    assert!(error.to_string().contains("resticprofile"));
    assert_eq!(runner.call_count("sh"), 3);
    assert_eq!(runner.call_count("which"), 5);
    assert_eq!(runner.call_count("restic"), 1);
    assert_eq!(runner.call_count("rclone"), 1);
}

#[test]
fn test_generate_secure_password_length_and_complexity() {
    use backup::commands::setup::generate_secure_password;
    let pwd = generate_secure_password();
    assert_eq!(pwd.len(), 32, "자동 생성 비밀번호 길이는 32자여야 합니다");
    assert!(
        pwd.chars().any(|c| c.is_ascii_uppercase()),
        "대문자가 포함되어야 합니다"
    );
    assert!(
        pwd.chars().any(|c| c.is_ascii_lowercase()),
        "소문자가 포함되어야 합니다"
    );
    assert!(
        pwd.chars().any(|c| c.is_ascii_digit()),
        "숫자가 포함되어야 합니다"
    );
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
        assert_eq!(
            perms.mode() & 0o777,
            0o600,
            "enc 키파일 권한은 600이어야 합니다"
        );
        let dir_perms = std::fs::metadata(enc_path.parent().unwrap())
            .unwrap()
            .permissions();
        assert_eq!(
            dir_perms.mode() & 0o777,
            0o700,
            "etc/backup 디렉터리 권한은 700이어야 합니다"
        );
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
        fn prompt_setup_params(
            &self,
            lang_opt: Option<Language>,
            _config_dir: &std::path::Path,
            _profiles_path: &std::path::Path,
        ) -> anyhow::Result<SetupParams> {
            *self.received_lang.lock().unwrap() = lang_opt;
            anyhow::bail!("capture_only") // 언어 캡처가 목적이므로 에러로 조기 종료
        }
    }

    let received = Arc::new(Mutex::new(None::<Language>));
    let prompter = CapturingPrompter {
        received_lang: Arc::clone(&received),
    };
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");

    // lang_opt = None으로 호출해도 setup은 명시적인 기본 언어를 전달해야 한다.
    let _ = run_setup_with_prompter(&config_path, &prompter, false, None);

    let captured = received.lock().unwrap();
    assert!(
        captured.is_some(),
        "lang_opt이 None이어도 명시적인 기본 언어를 prompter에 전달해야 합니다"
    );
}

#[test]
fn test_setup_does_not_enable_schedule_outside_the_isolated_e2e_runner() {
    use backup::commands::setup::run_setup_with_prompter_and_runners;

    let dir = tempdir().unwrap();
    let config_path = dir.path().join("profiles.yaml");

    let params = SetupParams {
        profile: "default".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        },
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:backup@192.168.1.100:/storage".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: Some(SftpConfig {
                host: "192.168.1.100".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some(dir.path().join("id_ed25519").to_string_lossy().into_owned()),
                additional_args: Vec::new(),
            }),
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig {
            output_dir: dir.path().join("reports").to_string_lossy().into_owned(),
            ..ReportsConfig::default()
        },
        audit: AuditConfig::default(),
    };

    let prompter = MockPrompter { params };
    let runner = MockResticProfileRunner::new(0, "initialized successfully");
    let scheduler = support::MockScheduler::new(0, "scheduled successfully");

    run_setup_with_prompter_and_runners(
        &config_path,
        &prompter,
        false,
        Some(Language::En),
        &runner,
        &scheduler,
    )
    .unwrap();

    assert!(config_path.exists());
    let mock_calls = runner.calls.lock().unwrap();
    assert!(mock_calls.iter().any(|(call, _)| call == "init"));
    assert_eq!(scheduler.calls.lock().unwrap().as_slice(), ["enable"]);
}

#[test]
fn setup_keeps_the_existing_configuration_when_backend_initialization_fails() {
    let dir = tempdir().unwrap();
    let profiles = dir.path().join("profiles.yaml");
    std::fs::write(&profiles, "previous configuration").unwrap();
    let params = SetupParams {
        profile: "default".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy::standard_defaults(),
        primary_storage: StorageTarget {
            backend: "s3".into(),
            repository: "s3:bucket/new".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: None,
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig {
            output_dir: dir.path().join("reports").to_string_lossy().into_owned(),
            ..ReportsConfig::default()
        },
        audit: AuditConfig::default(),
    };
    let runner = MockResticProfileRunner::new(1, "repository unreachable");
    let scheduler = support::MockScheduler::new(0, "scheduled");

    let error = run_setup_with_prompter_and_runners(
        &profiles,
        &MockPrompter { params },
        false,
        Some(Language::En),
        &runner,
        &scheduler,
    )
    .unwrap_err();

    assert!(error.to_string().contains("repository unreachable"));
    assert_eq!(
        std::fs::read_to_string(&profiles).unwrap(),
        "previous configuration"
    );
    assert!(scheduler.calls.lock().unwrap().is_empty());
}

#[test]
fn setup_restores_the_existing_configuration_when_schedule_registration_fails() {
    let dir = tempdir().unwrap();
    let profiles = dir.path().join("profiles.yaml");
    std::fs::write(&profiles, "previous configuration").unwrap();
    let params = SetupParams {
        profile: "default".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy::standard_defaults(),
        primary_storage: StorageTarget {
            backend: "s3".into(),
            repository: "s3:bucket/new".into(),
            password: SecretString::new("secure_password_123".into()),
            sftp: None,
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig {
            output_dir: dir.path().join("reports").to_string_lossy().into_owned(),
            ..ReportsConfig::default()
        },
        audit: AuditConfig::default(),
    };
    let scheduler = support::MockScheduler::new(1, "scheduler unavailable");
    let error = run_setup_with_prompter_and_runners(
        &profiles,
        &MockPrompter { params },
        false,
        Some(Language::En),
        &MockResticProfileRunner::new(0, "initialized"),
        &scheduler,
    )
    .unwrap_err();
    assert!(error.to_string().contains("scheduler unavailable"));
    assert_eq!(
        std::fs::read_to_string(&profiles).unwrap(),
        "previous configuration"
    );
}

#[test]
fn setup_stages_s3_credentials_in_secure_sidecars_for_child_processes() {
    let config = BackupConfig {
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "s3".into(),
                repository: "s3:http://example/bucket".into(),
                password: SecretString::new("secure_password_123".into()),
                sftp: None,
                s3: Some(S3Config {
                    endpoint: "http://example".into(),
                    access_key_id: SecretString::new("test-access".into()),
                    secret_access_key: SecretString::new("test-secret".into()),
                }),
            },
            secondary: None,
        },
        ..BackupConfig::default()
    };
    let directory = tempfile::tempdir().unwrap();
    let profiles = directory.path().join("profiles.yaml");
    config.save_to_profiles_path(&profiles).unwrap();
    let staged = backup::config::model::ResticProfileConfig::load_from_path(&profiles).unwrap();
    let owned_environment = staged.sidecar_environment(directory.path()).unwrap();
    let environment = backup::config::model::borrowed_environment(&owned_environment);
    assert_eq!(
        environment,
        vec![
            (
                "BACKUP_PRIMARY_AWS_ACCESS_KEY_ID".into(),
                "test-access".into()
            ),
            (
                "BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY".into(),
                "test-secret".into()
            ),
        ]
    );
}

#[test]
fn test_database_type_enum() {
    use backup::config::model::DatabaseType;
    use std::str::FromStr;

    assert_eq!(
        DatabaseType::from_str("mysql").unwrap(),
        DatabaseType::Mysql
    );
    assert_eq!(
        DatabaseType::from_str("postgres").unwrap(),
        DatabaseType::Postgres
    );
    assert!(DatabaseType::from_str("invalid").is_err());
}

#[test]
fn test_sftp_params_key_path_validation() {
    let params = SetupParams {
        profile: "sftp-test".into(),
        backup_type: BackupType::Directory,
        targets: vec!["/var/log".into()],
        excludes: vec![],
        retention: RetentionPolicy::standard_defaults(),
        primary_storage: StorageTarget {
            backend: "sftp".into(),
            repository: "sftp:backup@192.168.1.100:/backup".into(),
            password: SecretString::new("password_123456789".into()),
            sftp: Some(SftpConfig {
                host: "192.168.1.100".into(),
                port: 22,
                user: "backup".into(),
                key_file: Some("/etc/backup/id_ed25519".into()),
                additional_args: Vec::new(),
            }),
            s3: None,
        },
        secondary_storage: None,
        reports: ReportsConfig {
            output_dir: "/data/backup/reports".into(),
            enable_daily_reports: true,
            enable_annual_dr_drill_report: true,
        },
        audit: AuditConfig {
            system_manager: Some("Admin".into()),
            security_officer: Some("CISO".into()),
            restore_drill_rto_minutes: None,
            restore_drill_timeout_minutes: None,
            restore_drill_work_dir: None,
        },
    };

    let config = SetupEngine::validate_and_build(params).expect("Validation should pass");
    assert_eq!(config.storage.primary.backend, "sftp");
    assert_eq!(
        config.storage.primary.sftp.unwrap().key_file.unwrap(),
        "/etc/backup/id_ed25519"
    );
}

#[test]
fn test_format_sftp_repository_url() {
    use backup::commands::setup::format_sftp_repository_url;

    // Standard port 22 with absolute path
    assert_eq!(
        format_sftp_repository_url("backup", "192.168.1.100", 22, "/backup/data"),
        "sftp:backup@192.168.1.100:/backup/data"
    );

    // Custom port 49382 with absolute path
    assert_eq!(
        format_sftp_repository_url("backup_restic", "59.25.177.53", 49382, "/backup/ns0327/log"),
        "sftp://backup_restic@59.25.177.53:49382//backup/ns0327/log"
    );

    // Custom port 2222 with relative path
    assert_eq!(
        format_sftp_repository_url("user", "host.com", 2222, "relative/path"),
        "sftp://user@host.com:2222/relative/path"
    );
}

#[test]
fn test_verify_sftp_connection_success_and_failure() {
    use backup::commands::setup::verify_sftp_connection;
    use backup::runner::executor::CommandOutput;

    let expected_args = [
        "-i",
        "/etc/backup/id_ed25519",
        "-o",
        "IdentitiesOnly=yes",
        "-o",
        "BatchMode=yes",
        "-o",
        "StrictHostKeyChecking=accept-new",
        "-o",
        "UserKnownHostsFile=/etc/backup/known_hosts",
        "-P",
        "49382",
        "-o",
        "ConnectTimeout=5",
        "-b",
        "/dev/null",
        "backup_restic@59.25.177.53",
    ];
    let expected_call = (
        "sftp".to_string(),
        expected_args
            .iter()
            .map(|arg| (*arg).to_string())
            .collect::<Vec<_>>(),
    );
    let mock_success = MockExecutor::new();
    mock_success.push_output(
        "sftp",
        CommandOutput {
            status_code: 0,
            stdout: "".into(),
            stderr: "".into(),
        },
    );
    assert!(
        verify_sftp_connection(
            "backup_restic",
            "59.25.177.53",
            49382,
            "/etc/backup/id_ed25519",
            &mock_success
        )
        .is_ok()
    );
    assert_eq!(mock_success.get_calls(), vec![expected_call.clone()]);

    let mock_failure = MockExecutor::new();
    mock_failure.push_output(
        "sftp",
        CommandOutput {
            status_code: 255,
            stdout: "".into(),
            stderr: "Permission denied (publickey,password).".into(),
        },
    );
    let res = verify_sftp_connection(
        "backup_restic",
        "59.25.177.53",
        49382,
        "/etc/backup/id_ed25519",
        &mock_failure,
    );
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Permission denied"));
    assert_eq!(mock_failure.get_calls(), vec![expected_call.clone()]);

    let mock_empty_failure = MockExecutor::new();
    mock_empty_failure.push_output(
        "sftp",
        CommandOutput {
            status_code: 255,
            stdout: "".into(),
            stderr: " \n\t".into(),
        },
    );
    let res = verify_sftp_connection(
        "backup_restic",
        "59.25.177.53",
        49382,
        "/etc/backup/id_ed25519",
        &mock_empty_failure,
    );
    assert_eq!(res.unwrap_err(), "exit code: 255");
    assert_eq!(mock_empty_failure.get_calls(), vec![expected_call]);
}

#[test]
fn reused_sftp_native_args_recover_only_managed_identity() {
    use std::collections::BTreeMap;

    let dir = tempdir().unwrap();
    let config_dir = dir.path();
    std::fs::write(config_dir.join("id_ed25519"), "managed-key").unwrap();
    let options = BTreeMap::from([(
        "sftp.args".to_string(),
        format!(
            "-i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile={}",
            config_dir.join("id_ed25519").display(),
            config_dir.join("known_hosts").display()
        ),
    )]);
    let config =
        resolve_reused_sftp_config("sftp://backup@host:2222/repo", Some(&options), config_dir)
            .unwrap();
    assert_eq!(config.host, "host");
    assert_eq!(config.port, 2222);
    assert_eq!(config.user, "backup");
    assert_eq!(
        config.key_file.as_deref(),
        Some(config_dir.join("id_ed25519").to_str().unwrap())
    );
}

#[test]
fn reused_sftp_native_args_preserve_safe_additional_options() {
    use std::collections::BTreeMap;

    let dir = tempdir().unwrap();
    let config_dir = dir.path();
    let key = config_dir.join("id_ed25519");
    std::fs::write(&key, "managed-key").unwrap();
    let options = BTreeMap::from([(
        "sftp.args".to_string(),
        format!(
            "-i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile={} -o ConnectTimeout=5",
            key.display(),
            config_dir.join("known_hosts").display()
        ),
    )]);

    let config =
        resolve_reused_sftp_config("sftp://backup@host:2222/repo", Some(&options), config_dir)
            .unwrap();
    assert_eq!(config.additional_args, ["-o", "ConnectTimeout=5"]);
    let rendered = config.sftp_args_for_config_dir(config_dir).unwrap();
    assert!(rendered.contains("-o ConnectTimeout=5"));
}

#[test]
fn reused_sftp_native_args_reject_conflicting_or_duplicate_policy() {
    use std::collections::BTreeMap;

    let dir = tempdir().unwrap();
    let config_dir = dir.path();
    let key = config_dir.join("id_ed25519");
    std::fs::write(&key, "managed-key").unwrap();
    let base = format!(
        "-i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile={}",
        key.display(),
        config_dir.join("known_hosts").display()
    );
    for suffix in [
        " -o BatchMode=yes",
        " -o BatchMode=no",
        " -o batchmode=no",
        &format!(" -i {}", key.display()),
        " -p2222",
        " -lother-user",
        " -F/etc/ssh/config",
        " -D8080",
        " -o PermitLocalCommand=yes",
        &format!(
            " -o UserKnownHostsFile={}",
            dir.path().join("other-known-hosts").display()
        ),
        &format!(" -o IdentityFile={}", key.display()),
    ] {
        let options = BTreeMap::from([("sftp.args".to_string(), format!("{base}{suffix}"))]);
        let error =
            resolve_reused_sftp_config("sftp://backup@host:2222/repo", Some(&options), config_dir)
                .unwrap_err();
        assert!(
            error.to_string().contains("explicit SFTP reconfiguration"),
            "unexpected error: {error}"
        );
    }
}

#[cfg(unix)]
#[test]
fn reused_sftp_native_args_reject_key_symbolic_link() {
    use std::collections::BTreeMap;
    use std::os::unix::fs::symlink;

    let dir = tempdir().unwrap();
    let config_dir = dir.path();
    let outside = dir.path().join("outside-key");
    std::fs::write(&outside, "outside-key").unwrap();
    symlink(&outside, config_dir.join("id_ed25519")).unwrap();
    let options = BTreeMap::from([(
        "sftp.args".to_string(),
        format!(
            "-i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile={}",
            config_dir.join("id_ed25519").display(),
            config_dir.join("known_hosts").display()
        ),
    )]);

    let error =
        resolve_reused_sftp_config("sftp://backup@host:2222/repo", Some(&options), config_dir)
            .unwrap_err();
    assert!(error.to_string().contains("explicit SFTP reconfiguration"));
    assert_eq!(std::fs::read_to_string(outside).unwrap(), "outside-key");
}

#[test]
fn reused_standard_legacy_sftp_command_is_migratable() {
    use std::collections::BTreeMap;

    let dir = tempdir().unwrap();
    let config_dir = dir.path();
    std::fs::write(config_dir.join("id_ed25519"), "managed-key").unwrap();
    let options = BTreeMap::from([(
        "sftp.command".to_string(),
        format!(
            "ssh -o StrictHostKeyChecking=no -i {} -p 2222 backup@host -s sftp",
            config_dir.join("id_ed25519").display()
        ),
    )]);
    let config =
        resolve_reused_sftp_config("sftp://backup@host:2222/repo", Some(&options), config_dir)
            .unwrap();
    assert_eq!(
        config.key_file.as_deref(),
        Some(config_dir.join("id_ed25519").to_str().unwrap())
    );
}

#[test]
fn reused_sftp_nonstandard_legacy_command_requires_reconfiguration() {
    use std::collections::BTreeMap;
    use std::path::Path;

    let options = BTreeMap::from([(
        "sftp.command".to_string(),
        "ssh custom-wrapper --sftp".to_string(),
    )]);
    let error = resolve_reused_sftp_config(
        "sftp://backup@host:2222/repo",
        Some(&options),
        Path::new("/tmp/backup"),
    )
    .unwrap_err();
    assert!(error.to_string().contains("explicit SFTP reconfiguration"));
}

#[test]
fn sftp_initialization_diagnostic_contains_context_without_secrets() {
    use std::collections::BTreeMap;
    use std::path::Path;

    let options = BTreeMap::from([(
        "sftp.args".to_string(),
        "-i /tmp/backup/id_ed25519 -o IdentitiesOnly=yes -o BatchMode=yes".to_string(),
    )]);
    let summary = render_sftp_diagnostic_summary(
        "primary",
        "sftp://backup@host:2222/repo",
        Some(&options),
        Path::new("/tmp/backup/profiles.yaml"),
        "authentication failed for secret-password",
    )
    .unwrap();
    for field in [
        "backend=primary",
        "host=host",
        "port=2222",
        "user=backup",
        "identity=/tmp/backup/id_ed25519",
        "authentication=managed-key-only",
        "known_hosts=/tmp/backup/known_hosts",
    ] {
        assert!(summary.contains(field), "missing diagnostic field {field}");
    }
    assert!(!summary.contains("secret-password"));
}

use backup::config::model::{
    BackupConfig, BackupType, ReportsConfig, ResticProfileConfig, SecondaryStorageTarget,
    StorageConfig, StorageTarget,
};
use secrecy::ExposeSecret;
use std::fs;
use tempfile::tempdir;

#[test]
fn unified_profiles_reject_deprecated_application_execution_settings() {
    use backup::config::model::ResticProfileConfig;

    let yaml = r#"
version: "2"
application:
  profile: legacy
  reports:
    outputDir: /var/reports
    enableDailyReports: true
    enableAnnualDrDrillReport: false
profiles:
  legacy:
    backup: { source: [/data] }
"#;

    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), yaml).unwrap();
    let error = ResticProfileConfig::load_from_path(file.path()).unwrap_err();
    assert!(error.to_string().contains("application.profile"));
    assert!(error.to_string().contains("move"));
}

#[test]
fn unified_profiles_reject_database_target_that_is_not_a_backup_profile() {
    use backup::config::model::ResticProfileConfig;

    let yaml = r#"
version: "2"
application:
  database:
    profile: missing
    type: postgres
    connection-url: ${BACKUP_DATABASE_CONNECTION_URL}
profiles:
  primary:
    repository: s3:bucket
"#;

    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), yaml).unwrap();
    let error = ResticProfileConfig::load_from_path(file.path()).unwrap_err();
    assert!(error.to_string().contains("application.database.profile"));
    assert!(error.to_string().contains("missing"));
}

#[test]
fn backup_run_requires_the_exact_reserved_snapshot_tag() {
    use backup::config::model::ResticProfileConfig;

    let missing = ResticProfileConfig {
        version: "2".into(),
        application: None,
        global: None,
        groups: None,
        profiles: [(
            "daily".into(),
            backup::config::model::ProfileSection {
                backup: Some(backup::config::model::BackupCommandSection {
                    source: Some(vec!["/data".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
    };
    let error = missing
        .validate_reserved_backup_profile_tag("daily")
        .unwrap_err();
    assert!(error.to_string().contains("backup-profile:daily"));

    let tagged = ResticProfileConfig {
        profiles: [(
            "daily".into(),
            backup::config::model::ProfileSection {
                backup: Some(backup::config::model::BackupCommandSection {
                    source: Some(vec!["/data".into()]),
                    tag: Some(vec!["user-tag".into(), "backup-profile:daily".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        )]
        .into_iter()
        .collect(),
        ..missing
    };
    tagged
        .validate_reserved_backup_profile_tag("daily")
        .unwrap();
}

#[test]
fn effective_backup_settings_merge_partial_inheritance_fields() {
    use backup::config::model::ResticProfileConfig;

    let yaml = r#"
version: "2"
profiles:
  default:
    backup:
      exclude: [/parent-cache]
    retention:
      keep-weekly: 5
      keep-monthly: 13
  child:
    inherit: default
    backup:
      source: [/work]
    retention:
      keep-daily: 9
"#;
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), yaml).unwrap();
    let profiles = ResticProfileConfig::load_from_path(file.path()).unwrap();

    let settings = profiles.effective_backup_settings("child").unwrap();
    assert_eq!(settings.source, vec!["/work"]);
    assert_eq!(settings.exclude, vec!["/parent-cache"]);
    assert_eq!(settings.retention.keep_daily, 9);
    assert_eq!(settings.retention.keep_weekly, 5);
    assert_eq!(settings.retention.keep_monthly, 13);
}

#[test]
fn effective_backup_settings_reject_unknown_profile_and_cycles() {
    use backup::config::model::ResticProfileConfig;

    let unknown = ResticProfileConfig {
        version: "2".into(),
        application: None,
        global: None,
        groups: None,
        profiles: Default::default(),
    };
    assert!(
        unknown
            .effective_backup_settings("missing")
            .unwrap_err()
            .to_string()
            .contains("Unknown Backup Profile")
    );

    let yaml = r#"
version: "2"
profiles:
  one:
    inherit: two
  two:
    inherit: one
"#;
    let file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(file.path(), yaml).unwrap();
    let profiles = ResticProfileConfig::load_from_path(file.path()).unwrap();
    assert!(
        profiles
            .effective_backup_settings("one")
            .unwrap_err()
            .to_string()
            .contains("cyclic")
    );
}

#[test]
fn test_parse_yaml_config() {
    let yaml = r#"
version: "1.0"
profile: "host1"
backup:
  targets:
    - "/home/user/data"
  excludes:
    - "/home/user/data/temp"
retention:
  keepDaily: 7
  keepWeekly: 4
  keepMonthly: 12
storage:
  primary:
    backend: "sftp"
    repository: "rclone:syno_backup:/backup/host1"
    password: "testpassword"
    sftp:
      host: "192.168.1.100"
      port: 2222
      user: "backupUser"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.profile, "host1");
    assert_eq!(config.retention.keep_daily, 7);
    assert_eq!(config.backup.targets, vec!["/home/user/data"]);
}

#[test]
fn test_config_redacted() {
    let yaml = r#"
version: "1.0"
profile: "redact-test"
backup:
  targets: ["/data"]
  excludes: []
retention:
  keepDaily: 1
  keepWeekly: 1
  keepMonthly: 1
storage:
  primary:
    backend: "s3"
    repository: "s3:mybucket"
    password: "secret_password"
    s3:
      endpoint: "http://localhost:9000"
      accessKeyId: "minioadmin"
      secretAccessKey: "minioadmin_secret"
  secondary:
    enabled: true
    backend: "sftp"
    repository: "remote:backup"
    password: "sec_password"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    let redacted = config.redacted();

    // Check masked values
    assert_eq!(redacted.storage.primary.password.expose_secret(), "******");
    assert_eq!(
        redacted
            .storage
            .primary
            .s3
            .as_ref()
            .unwrap()
            .secret_access_key
            .expose_secret(),
        "******"
    );
    assert_eq!(
        redacted
            .storage
            .secondary
            .as_ref()
            .unwrap()
            .password
            .expose_secret(),
        "******"
    );

    // Original should remain unchanged
    assert_eq!(
        config.storage.primary.password.expose_secret(),
        "secret_password"
    );
}

#[test]
fn test_config_render() {
    let yaml = r#"
version: "1.0"
profile: "render-test"
backup:
  targets: ["/data"]
  excludes: []
retention:
  keepDaily: 5
  keepWeekly: 2
  keepMonthly: 1
storage:
  primary:
    backend: "sftp"
    repository: "/repo"
    password: "my_secret_pass"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();

    let yaml_rendered = config.render("yaml", false).unwrap();
    assert!(yaml_rendered.contains("render-test"));
    assert!(yaml_rendered.contains("my_secret_pass"));

    let yaml_redacted = config.render("yaml", true).unwrap();
    assert!(yaml_redacted.contains("******"));
    assert!(!yaml_redacted.contains("my_secret_pass"));

    let json_rendered = config.render("json", false).unwrap();
    assert!(json_rendered.contains("\"profile\": \"render-test\""));
    assert!(json_rendered.contains("my_secret_pass"));

    let json_redacted = config.render("json", true).unwrap();
    assert!(json_redacted.contains("******"));
}

#[test]
fn test_config_save_to_path() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sub_dir").join("profiles.yaml");

    let yaml = r#"
version: "1.0"
profile: "save-test"
backup:
  targets: ["/data"]
  excludes: []
retention:
  keepDaily: 1
  keepWeekly: 1
  keepMonthly: 1
storage:
  primary:
    backend: "sftp"
    repository: "/repo"
    password: "pass"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    config.save_to_path(&file_path).unwrap();

    assert!(file_path.exists());
    let saved_content = fs::read_to_string(&file_path).unwrap();
    assert!(saved_content.contains("save-test"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let parent = file_path.parent().unwrap();
        let parent_perms = fs::metadata(parent).unwrap().permissions();
        let file_perms = fs::metadata(&file_path).unwrap().permissions();
        assert_eq!(parent_perms.mode() & 0o777, 0o700);
        assert_eq!(file_perms.mode() & 0o777, 0o600);
    }
}

#[test]
fn test_invalid_yaml_parse_error() {
    let invalid_yaml = "invalid: yaml: [";
    let res: Result<BackupConfig, _> = serde_yaml::from_str(invalid_yaml);
    assert!(res.is_err());
}

#[test]
fn test_resticprofile_config_yaml() {
    use backup::config::model::ResticProfileConfig;

    let yaml = r#"
version: "2"
profiles:
  default:
    repository: "s3:https://s3.amazonaws.com/mybucket"
    password-file: "/etc/backup/restic-password"
  self:
    inherit: "default"
    backup:
      source:
        - "/var/www"
      schedule: "*-*-* 03:00:00"
      schedule-permission: "system"
      schedule-priority: "background"
      schedule-ignore-on-battery-less-than: 20
      run-before: "/usr/local/bin/dump.sh"
      send-after-fail:
        method: "POST"
        url: "https://hooks.slack.com/test"
        body: '{"text":"failed"}'
    prune:
      schedule: "Sun 04:00:00"
      keep-daily: 7
      keep-weekly: 4
      keep-monthly: 12
"#;
    let config: ResticProfileConfig = serde_yaml::from_str(yaml).unwrap();
    assert_eq!(config.version, "2");
    let default_prof = config.profiles.get("default").unwrap();
    assert_eq!(
        default_prof.repository.as_deref(),
        Some("s3:https://s3.amazonaws.com/mybucket")
    );
    let self_prof = config.profiles.get("self").unwrap();
    assert_eq!(self_prof.inherit.as_deref(), Some("default"));

    let backup_sec = self_prof.backup.as_ref().unwrap();
    assert_eq!(
        backup_sec.source.as_ref().unwrap(),
        &vec!["/var/www".to_string()]
    );
    assert_eq!(backup_sec.schedule_ignore_on_battery_less_than, Some(20));
    assert_eq!(
        backup_sec.send_after_fail.as_ref().unwrap().url,
        "https://hooks.slack.com/test"
    );
}

#[test]
fn test_config_save_and_sync() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");

    let yaml = r#"
version: "1.0"
profile: "sync-test"
backup:
  targets: ["/data/web"]
  excludes: []
retention:
  keepDaily: 14
  keepWeekly: 4
  keepMonthly: 12
storage:
  primary:
    backend: "sftp"
    repository: "sftp:user@host:/backups"
    password: "secret_pass_123"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    config.save_and_sync(&config_dir).unwrap();

    let profiles_file = config_dir.join("profiles.yaml");
    assert!(profiles_file.exists());

    let profiles_content = fs::read_to_string(&profiles_file).unwrap();
    assert!(profiles_content.contains("sync-test"));
    assert!(profiles_content.contains("sftp:user@host:/backups"));
    assert!(profiles_content.contains("/data/web"));

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let prof_mode = fs::metadata(&profiles_file).unwrap().permissions().mode() & 0o777;
        assert_eq!(prof_mode, 0o600);
    }
}

#[test]
fn generated_profiles_do_not_contain_plaintext_credentials() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");
    let config: BackupConfig = serde_yaml::from_str(
        r#"
version: "1.0"
profile: "secret-safe"
backup: { targets: ["/data"], excludes: [] }
retention: { keepDaily: 1, keepWeekly: 1, keepMonthly: 1 }
storage:
  primary:
    backend: "s3"
    repository: "s3:https://example.invalid/backup"
    password: "repository-secret"
    s3: { endpoint: "https://example.invalid", accessKeyId: "access-key", secretAccessKey: "aws-secret" }
"#,
    )
    .unwrap();

    config.save_and_sync(&config_dir).unwrap();

    let profiles = fs::read_to_string(config_dir.join("profiles.yaml")).unwrap();
    for secret in ["repository-secret", "access-key", "aws-secret"] {
        assert!(!profiles.contains(secret), "profiles leaked {secret}");
    }
    assert!(profiles.contains("{{ .Env.BACKUP_PRIMARY_AWS_ACCESS_KEY_ID }}"));
    assert!(profiles.contains("{{ .Env.BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY }}"));
    assert!(config_dir.join("primary-password").exists());
}

#[test]
fn test_sftp_option_args_generation_uses_key_only_policy() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");

    let yaml = format!(
        r#"
version: "1.0"
profile: "sftp-test"
backup:
  targets: ["/var/log"]
  excludes: []
retention:
  keepDaily: 7
  keepWeekly: 4
  keepMonthly: 12
storage:
  primary:
    backend: "sftp"
    repository: "sftp://backup_restic@59.25.177.53:49382/backup/ns0327/log"
    password: "secret_pass_123"
    sftp:
      host: "59.25.177.53"
      port: 49382
      user: "backup_restic"
      keyFile: "{}"
  secondary:
    enabled: true
    backend: "sftp"
    repository: "sftp://backup_restic@59.25.177.53:49382/backup/ns0327/sec"
    password: "secret_pass_123"
    sftp:
      host: "59.25.177.53"
      port: 49382
      user: "backup_restic"
      keyFile: "{}"
"#,
        config_dir.join("id_ed25519").display(),
        config_dir.join("id_ed25519_secondary").display()
    );
    let config: BackupConfig = serde_yaml::from_str(&yaml).unwrap();
    config.save_and_sync(&config_dir).unwrap();

    let profiles_file = config_dir.join("profiles.yaml");
    assert!(profiles_file.exists());

    let content = fs::read_to_string(&profiles_file).unwrap();
    assert!(content.contains("option:"));
    assert!(content.contains("sftp.args="));
    assert!(!content.contains("sftp.command:"));
    for key in [
        config_dir.join("id_ed25519").display().to_string(),
        config_dir
            .join("id_ed25519_secondary")
            .display()
            .to_string(),
    ] {
        assert!(content.contains(&key));
    }
    for policy in [
        "IdentitiesOnly=yes",
        "BatchMode=yes",
        "StrictHostKeyChecking=accept-new",
    ] {
        assert!(content.contains(policy), "missing SFTP policy {policy}");
    }
    let known_hosts = config_dir.join("known_hosts");
    assert!(content.contains(&format!("UserKnownHostsFile={}", known_hosts.display())));
    assert!(known_hosts.exists());
    let profiles =
        backup::config::model::ResticProfileConfig::load_from_path(&profiles_file).unwrap();
    let copy_options = profiles
        .profiles
        .get("sftp-test")
        .and_then(|profile| profile.copy.as_ref())
        .and_then(|copy| copy.option.as_ref())
        .expect("SFTP copy section must carry destination auth options");
    assert_eq!(
        copy_options.get("sftp.args"),
        profiles
            .profiles
            .get("secondary")
            .and_then(|profile| profile.option.as_ref())
            .and_then(|options| options.get("sftp.args"))
    );
}

#[cfg(unix)]
#[test]
fn known_hosts_rejects_symbolic_link_escape() {
    use backup::config::model::ensure_sftp_known_hosts_file;
    use std::os::unix::fs::symlink;

    let dir = tempdir().unwrap();
    let outside = dir.path().join("outside-known-hosts");
    let config_dir = dir.path().join("config");
    fs::create_dir(&config_dir).unwrap();
    fs::write(&outside, "trusted host\n").unwrap();
    symlink(&outside, config_dir.join("known_hosts")).unwrap();

    let error = ensure_sftp_known_hosts_file(&config_dir).unwrap_err();
    assert!(error.to_string().contains("symbolic link"));
    assert_eq!(fs::read_to_string(outside).unwrap(), "trusted host\n");
}

#[cfg(unix)]
#[test]
fn known_hosts_is_created_with_private_permissions() {
    use backup::config::model::ensure_sftp_known_hosts_file;
    use std::os::unix::fs::PermissionsExt;

    let dir = tempdir().unwrap();
    let path = ensure_sftp_known_hosts_file(dir.path()).unwrap();
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn sftp_auth_renderer_quotes_config_paths_and_derives_known_hosts() {
    use backup::config::model::SftpAuthPolicy;
    use std::path::Path;

    let profiles_path = Path::new("/tmp/backup config/profiles.yaml");
    let identity = Path::new("/tmp/backup config/id_ed25519");
    let policy = SftpAuthPolicy::for_profiles_path(identity, profiles_path).unwrap();

    let rendered = policy.render_restic_args().unwrap();
    assert!(rendered.contains("-i '/tmp/backup config/id_ed25519'"));
    assert!(rendered.contains("-o IdentitiesOnly=yes"));
    assert!(rendered.contains("-o BatchMode=yes"));
    assert!(rendered.contains("-o StrictHostKeyChecking=accept-new"));
    assert!(rendered.contains("-o 'UserKnownHostsFile=/tmp/backup config/known_hosts'"));
}

#[test]
fn sftp_legacy_command_is_migrated_and_removed() {
    use backup::config::model::ResticProfileConfig;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("backup config");
    fs::create_dir_all(&config_dir).unwrap();
    let profiles_file = config_dir.join("profiles.yaml");
    fs::write(
        &profiles_file,
        r#"
version: "2"
profiles:
  primary:
    repository: "sftp://backup@host:2222/repo"
    option:
      sftp.command: "ssh -o StrictHostKeyChecking=no -i /tmp/backup config/id_ed25519 -p 2222 backup@host -s sftp"
  daily:
    inherit: primary
    backup:
      source: [/data]
"#,
    )
    .unwrap();

    let config_yaml = r#"
version: "1.0"
profile: "daily"
backup: { targets: [/data], excludes: [] }
retention: { keepDaily: 7, keepWeekly: 4, keepMonthly: 12 }
storage:
  primary:
    backend: sftp
    repository: "sftp://backup@host:2222/repo"
    password: "secret_pass_123"
    sftp:
      host: host
      port: 2222
      user: backup
      keyFile: "KEYFILE"
"#
    .replace(
        "KEYFILE",
        &config_dir.join("id_ed25519").display().to_string(),
    );
    let config: BackupConfig = serde_yaml::from_str(&config_yaml).unwrap();
    config.save_to_profiles_path(&profiles_file).unwrap();

    let saved = fs::read_to_string(&profiles_file).unwrap();
    assert!(saved.contains("sftp.args="));
    assert!(!saved.contains("sftp.command:"));
    let parsed = ResticProfileConfig::load_from_path(&profiles_file).unwrap();
    let args = parsed
        .profiles
        .get("primary")
        .and_then(|profile| profile.option.as_ref())
        .and_then(|options| options.get("sftp.args"))
        .unwrap();
    assert!(args.contains("StrictHostKeyChecking=accept-new"));
}

#[test]
fn staged_sftp_profiles_keep_trust_state_beside_live_profiles() {
    let dir = tempdir().unwrap();
    let live_dir = dir.path().join("live config");
    let staged_dir = dir.path().join(".setup");
    let staged_profiles = staged_dir.join("profiles.yaml");
    let config_yaml = r#"
version: "1.0"
profile: daily
backup: { targets: [/data], excludes: [] }
retention: { keepDaily: 7, keepWeekly: 4, keepMonthly: 12 }
storage:
  primary:
    backend: sftp
    repository: sftp:backup@host:/repo
    password: secret_pass_123
    sftp:
      host: host
      port: 22
      user: backup
      keyFile: "KEYFILE"
"#
    .replace(
        "KEYFILE",
        &live_dir.join("id_ed25519").display().to_string(),
    );
    let config: BackupConfig = serde_yaml::from_str(&config_yaml).unwrap();

    config
        .save_to_profiles_path_with_config_dir(&staged_profiles, &live_dir)
        .unwrap();
    let staged = fs::read_to_string(&staged_profiles).unwrap();
    assert!(staged.contains(&format!(
        "UserKnownHostsFile={}",
        live_dir.join("known_hosts").display()
    )));
    assert!(!staged.contains(&staged_dir.join("known_hosts").display().to_string()));
    assert!(live_dir.join("known_hosts").exists());
    assert!(!staged_dir.join("known_hosts").exists());
}

#[test]
fn test_profiles_yaml_single_file_unification_and_merge() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");

    let yaml1 = r#"
version: "1.0"
profile: "log"
backup:
  targets: ["/var/log"]
  excludes: []
retention:
  keepDaily: 7
  keepWeekly: 4
  keepMonthly: 12
storage:
  primary:
    backend: "sftp"
    repository: "sftp:backup@192.168.1.100:/backup"
    password: "secret_pass_123"
"#;
    let config1: BackupConfig = serde_yaml::from_str(yaml1).unwrap();
    config1.save_and_sync(&config_dir).unwrap();

    let profiles_file = config_dir.join("profiles.yaml");

    // profiles.yaml is the sole canonical configuration file. Application settings
    // live under its dedicated top-level section, separate from resticprofile v2 keys.
    assert!(profiles_file.exists(), "profiles.yaml must exist");

    let content1 = fs::read_to_string(&profiles_file).unwrap();
    let document: serde_yaml::Value = serde_yaml::from_str(&content1).unwrap();
    assert_eq!(document["version"].as_str(), Some("2"));
    assert!(
        document["application"].get("version").is_none(),
        "the application namespace must not declare a second configuration version"
    );
    let parsed1: backup::config::model::ResticProfileConfig =
        serde_yaml::from_str(&content1).unwrap();
    let application = parsed1
        .application
        .expect("profiles.yaml must contain the application configuration section");
    assert_eq!(application.reports, ReportsConfig::default());
    assert!(content1.contains("log:"));
    assert!(content1.contains("sftp:backup@192.168.1.100:/backup"));

    // Now save a second profile "db"
    let yaml2 = r#"
version: "1.0"
profile: "db"
backup:
  targets: ["db-stream:mysql"]
  excludes: []
retention:
  keepDaily: 180
  keepWeekly: 12
  keepMonthly: 24
storage:
  primary:
    backend: "s3"
    repository: "s3:https://s3.amazonaws.com/db-backups"
    password: "secret_pass_123"
"#;
    let config2: BackupConfig = serde_yaml::from_str(yaml2).unwrap();
    config2.save_and_sync(&config_dir).unwrap();

    let content2 = fs::read_to_string(&profiles_file).unwrap();
    // Both log and db profiles must exist
    assert!(
        content2.contains("log:"),
        "Original 'log' profile must be preserved"
    );
    assert!(content2.contains("db:"), "New 'db' profile must be merged");
}

#[test]
fn test_profiles_yaml_three_layer_separation() {
    use backup::config::model::ResticProfileConfig;

    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");

    let yaml = r#"
version: "1.0"
profile: "web-data"
backup:
  targets: ["/var/www/html"]
  excludes: []
retention:
  keepDaily: 7
  keepWeekly: 4
  keepMonthly: 12
storage:
  primary:
    backend: "s3"
    repository: "s3:https://s3.amazonaws.com/primary-bucket"
    password: "primary_password_123"
  secondary:
    enabled: true
    backend: "s3"
    repository: "s3:https://s3.amazonaws.com/secondary-bucket"
    password: "secondary_password_123"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    config.save_and_sync(&config_dir).unwrap();

    let profiles_file = config_dir.join("profiles.yaml");
    let restic_config = ResticProfileConfig::load_from_path(&profiles_file).unwrap();

    assert!(restic_config.profiles.contains_key("default"));
    assert!(restic_config.profiles.contains_key("primary"));
    assert!(restic_config.profiles.contains_key("secondary"));
    assert!(restic_config.profiles.contains_key("web-data"));

    let primary_prof = restic_config.profiles.get("primary").unwrap();
    assert_eq!(
        primary_prof.repository.as_deref(),
        Some("s3:https://s3.amazonaws.com/primary-bucket")
    );
    assert_eq!(primary_prof.inherit.as_deref(), Some("default"));

    let secondary_prof = restic_config.profiles.get("secondary").unwrap();
    assert_eq!(
        secondary_prof.repository.as_deref(),
        Some("s3:https://s3.amazonaws.com/secondary-bucket")
    );
    assert_eq!(secondary_prof.inherit.as_deref(), Some("default"));

    let web_prof = restic_config.profiles.get("web-data").unwrap();
    assert_eq!(web_prof.inherit.as_deref(), Some("primary"));
    let copy_sec = web_prof.copy.as_ref().unwrap();
    assert_eq!(copy_sec.profile.as_deref(), Some("secondary"));
    assert_eq!(
        copy_sec.repository.as_deref(),
        Some("s3:https://s3.amazonaws.com/secondary-bucket")
    );
    assert_eq!(copy_sec.password, None);
    assert_eq!(
        copy_sec.password_file.as_deref(),
        Some(
            config_dir
                .join("secondary-password")
                .to_string_lossy()
                .as_ref()
        )
    );
}

#[test]
fn test_restic_profile_config_audit_is_application_metadata() {
    use backup::config::model::ResticProfileConfig;
    let yaml = r#"
version: "2"
application:
  audit:
    system-manager: "홍길동 차장"
    security-officer: "김보안 이사"
global:
  min-memory: 1024
profiles: {}
"#;
    let config: ResticProfileConfig = serde_yaml::from_str(yaml).unwrap();
    let audit = config.application.unwrap().audit;
    assert_eq!(audit.system_manager, Some("홍길동 차장".to_string()));
    assert_eq!(audit.security_officer, Some("김보안 이사".to_string()));
}

#[test]
fn profile_names_includes_primary_when_it_is_a_runnable_profile() {
    use backup::config::model::ResticProfileConfig;

    let config: ResticProfileConfig = serde_yaml::from_str(
        r#"
version: "2"
profiles:
  primary:
    repository: s3:http://example.invalid/backup
    backup: {source: ["/data"]}
"#,
    )
    .unwrap();

    assert_eq!(config.profile_names(), vec!["primary"]);
}

#[test]
fn config_accepts_legacy_snake_case_backup_and_retention_fields() {
    let config: BackupConfig = serde_yaml::from_str(
        r#"
version: "1.0"
profile: e2e
backup:
  backup_type: directory
  targets: ["/data"]
  excludes: []
retention: {keep_daily: 1, keep_weekly: 2, keep_monthly: 3}
storage:
  primary: {backend: local, repository: /tmp/repository, password: test-password}
"#,
    )
    .unwrap();

    assert_eq!(config.retention.keep_daily, 1);
    assert_eq!(config.storage.primary.repository, "/tmp/repository");
}

#[test]
fn config_accepts_legacy_snake_case_database_stream_fields() {
    let config: BackupConfig = serde_yaml::from_str(
        r#"
version: "1.0"
profile: database
backup:
  backupType: !dbStream
    db_type: postgres
    connection_url: postgres://postgres:secret@db:5432/app
  targets: []
  excludes: []
retention: {keep_daily: 1, keep_weekly: 1, keep_monthly: 1}
storage:
  primary: {backend: local, repository: /tmp/repository, password: test-password}
"#,
    )
    .unwrap();

    assert!(matches!(
        config.backup.backup_type,
        BackupType::DbStream { .. }
    ));
}

#[test]
fn test_save_secure_file_permissions() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config_file = temp_dir.path().join("secure.yaml");
    let mut config = BackupConfig::default();
    config.storage.primary.password = secrecy::SecretString::new("valid_secret_pass".into());
    config.save_to_path(&config_file).unwrap();

    let metadata = std::fs::metadata(&config_file).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    }
}

#[test]
fn test_empty_password_validation_error() {
    let mut config = BackupConfig::default();
    config.storage.primary.password = secrecy::SecretString::new("".into());
    assert!(config.validate().is_err());
}

#[test]
fn test_secondary_profile_password_file_or_password_fallback() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");

    let yaml = r#"
version: "1.0"
profile: "sftp_sec_test"
backup:
  targets: ["/var/log"]
  excludes: []
retention:
  keepDaily: 7
  keepWeekly: 4
  keepMonthly: 12
storage:
  primary:
    backend: "s3"
    repository: "s3:https://59.25.177.53:39000/backup/ns0327/log"
    password: "primary_secret_123"
  secondary:
    enabled: true
    backend: "sftp"
    repository: "sftp:backup_restic@59.25.177.53:49382/backup/nbs0327/log"
    password: ""
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    config.save_and_sync(&config_dir).unwrap();

    let profiles_file = config_dir.join("profiles.yaml");
    assert!(profiles_file.exists());

    let content = fs::read_to_string(&profiles_file).unwrap();
    let parsed: backup::config::model::ResticProfileConfig =
        serde_yaml::from_str(&content).unwrap();
    let sec_prof = parsed
        .profiles
        .get("secondary")
        .expect("secondary profile should exist");

    // When no keyfile exists, the secondary profile writes the primary fallback password securely.
    if !config_dir.join("enc").is_file() && !std::path::Path::new("/etc/backup/enc").is_file() {
        assert_eq!(sec_prof.password, None);
        assert_eq!(
            sec_prof.password_file.as_deref(),
            Some(
                config_dir
                    .join("secondary-password")
                    .to_string_lossy()
                    .as_ref()
            )
        );
        assert_eq!(
            fs::read_to_string(config_dir.join("secondary-password")).unwrap(),
            "primary_secret_123"
        );
    } else {
        assert_eq!(sec_prof.password, None);
        assert!(sec_prof.password_file.is_some());
    }
}

#[test]
fn explicit_secondary_repository_password_uses_an_independent_secure_sidecar() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");
    let primary_password = "primary-secret-123";
    let secondary_password = "secondary-secret-456";
    let config = BackupConfig {
        profile: "daily".into(),
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "local".into(),
                repository: "/primary-repository".into(),
                password: secrecy::SecretString::new(primary_password.into()),
                sftp: None,
                s3: None,
            },
            secondary: Some(SecondaryStorageTarget {
                enabled: true,
                backend: "local".into(),
                repository: "/secondary-repository".into(),
                password: secrecy::SecretString::new(secondary_password.into()),
                sftp: None,
                s3: None,
            }),
        },
        ..BackupConfig::default()
    };

    config.save_and_sync(&config_dir).unwrap();
    let profiles = ResticProfileConfig::load_from_path(&config_dir.join("profiles.yaml")).unwrap();
    let primary = profiles.profiles.get("primary").unwrap();
    let secondary = profiles.profiles.get("secondary").unwrap();
    let copy = profiles
        .profiles
        .get("daily")
        .unwrap()
        .copy
        .as_ref()
        .unwrap();

    assert_eq!(
        fs::read_to_string(primary.password_file.as_ref().unwrap()).unwrap(),
        primary_password
    );
    assert_eq!(
        fs::read_to_string(secondary.password_file.as_ref().unwrap()).unwrap(),
        secondary_password
    );
    assert_eq!(copy.password_file, secondary.password_file);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(secondary.password_file.as_ref().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
}

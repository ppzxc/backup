use backup::config::model::BackupConfig;
use secrecy::ExposeSecret;
use std::fs;
use tempfile::tempdir;

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
    let file_path = dir.path().join("sub_dir").join("config.yaml");

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
fn test_sftp_option_command_generation() {
    let dir = tempdir().unwrap();
    let config_dir = dir.path().join("etc_backup");

    let yaml = r#"
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
      keyFile: "/etc/backup/id_ed25519"
  secondary:
    enabled: true
    backend: "sftp"
    repository: "sftp://backup_restic@59.25.177.53:49382/backup/ns0327/sec"
    password: "secret_pass_123"
    sftp:
      host: "59.25.177.53"
      port: 49382
      user: "backup_restic"
      keyFile: "/etc/backup/id_ed25519_secondary"
"#;
    let config: BackupConfig = serde_yaml::from_str(yaml).unwrap();
    config.save_and_sync(&config_dir).unwrap();

    let profiles_file = config_dir.join("profiles.yaml");
    assert!(profiles_file.exists());

    let content = fs::read_to_string(&profiles_file).unwrap();
    assert!(content.contains("option:"));
    assert!(content.contains("sftp.command: ssh -o StrictHostKeyChecking=no -i /etc/backup/id_ed25519 -p 49382 backup_restic@59.25.177.53 -s sftp"));
    assert!(content.contains("sftp.command: ssh -o StrictHostKeyChecking=no -i /etc/backup/id_ed25519_secondary -p 49382 backup_restic@59.25.177.53 -s sftp"));
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

    let env_file = config_dir.join("backup.env");
    let config_yml = config_dir.join("config.yml");
    let profiles_file = config_dir.join("profiles.yaml");

    // Canonical config and derived resticprofile YAML should exist.
    assert!(!env_file.exists(), "backup.env should not be created");
    assert!(config_yml.exists(), "config.yml should be created");
    assert!(profiles_file.exists(), "profiles.yaml must exist");

    let content1 = fs::read_to_string(&profiles_file).unwrap();
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
    assert_eq!(copy_sec.password.as_deref(), Some("secondary_password_123"));
}

#[test]
fn test_restic_profile_config_audit_section() {
    use backup::config::model::ResticProfileConfig;
    let yaml = r#"
version: "2"
audit:
  system-manager: "홍길동 차장"
  security-officer: "김보안 이사"
global:
  min-memory: 1024
profiles: {}
"#;
    let config: ResticProfileConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.audit.is_some());
    let audit = config.audit.unwrap();
    assert_eq!(audit.system_manager, Some("홍길동 차장".to_string()));
    assert_eq!(audit.security_officer, Some("김보안 이사".to_string()));
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

    // Deterministic check: when no enc keyfile exists in temp dir, secondary falls back to primary.password
    if !config_dir.join("enc").is_file() && !std::path::Path::new("/etc/backup/enc").is_file() {
        assert_eq!(sec_prof.password.as_deref(), Some("primary_secret_123"));
        assert_eq!(sec_prof.password_file, None);
    } else {
        assert!(sec_prof.password_file.is_some());
    }
}

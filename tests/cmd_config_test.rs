use backup::commands::config_cmd::{execute_config_export, execute_config_show};
use backup::config::model::*;
use secrecy::SecretString;

#[test]
fn test_config_show_masked() {
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
                password: SecretString::new("secret123".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
    };
    let output = execute_config_show(&config).unwrap();
    assert!(!output.contains("secret123"));
    assert!(output.contains("******"));
}

#[test]
fn test_config_export_yaml_and_json() {
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
                password: SecretString::new("secret123".into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
    };
    let yaml_output = execute_config_export(&config, "yaml").unwrap();
    assert!(yaml_output.contains("secret123"));

    let json_output = execute_config_export(&config, "json").unwrap();
    assert!(json_output.contains("secret123"));
}

#[test]
fn test_config_import_legacy() {
    use backup::commands::config_cmd::execute_config_import_legacy;
    let temp_dir = tempfile::tempdir().unwrap();
    let env_path = temp_dir.path().join("backup.env");
    let target_path = temp_dir.path().join("config.yml");
    std::fs::write(&env_path, "export RESTIC_REPOSITORY=\"sftp:user@host:/repo\"\nexport RESTIC_PASSWORD=\"secret123\"\nexport BACKUP_TARGETS=\"/data\"").unwrap();

    let res = execute_config_import_legacy(&env_path, &target_path).unwrap();
    assert!(res.contains("Imported legacy configuration"));
    assert!(target_path.exists());
}

#[test]
fn test_config_edit() {
    use backup::commands::config_cmd::execute_config_edit;
    let path = std::path::Path::new("/tmp/test_config.yml");
    let res = execute_config_edit(path).unwrap();
    assert!(res.contains("Config file edit session initiated"));
}


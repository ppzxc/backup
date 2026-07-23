use backup::commands::config_cmd::{execute_config_export, execute_config_show};
use backup::config::model::*;
use secrecy::SecretString;

#[test]
fn test_config_show_masked() {
    let config = BackupConfig {
        version: "1.0".into(),
        profile: "test".into(),
        backup: BackupTargets {
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
    };
    let yaml_output = execute_config_export(&config, "yaml").unwrap();
    assert!(yaml_output.contains("secret123"));

    let json_output = execute_config_export(&config, "json").unwrap();
    assert!(json_output.contains("secret123"));
}

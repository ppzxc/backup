use anyhow::Result;
use secrecy::SecretString;
use std::collections::HashMap;
use crate::config::model::*;

pub fn parse_legacy_env(content: &str) -> Result<BackupConfig> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let stripped = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        if let Some((k, v)) = stripped.split_once('=') {
            let val = v.trim_matches('"').trim_matches('\'');
            map.insert(k.trim(), val.to_string());
        }
    }

    let profile = map.get("BACKUP_PROFILE_NAME").cloned().unwrap_or_else(|| "default".into());
    let repo = map.get("RESTIC_REPOSITORY").cloned().unwrap_or_default();
    let pwd = map.get("RESTIC_PASSWORD").cloned().unwrap_or_default();
    let targets_str = map.get("BACKUP_TARGETS").cloned().unwrap_or_default();
    let targets = targets_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    Ok(BackupConfig {
        version: "1.0".to_string(),
        profile,
        backup: BackupTargets {
            targets,
            excludes: vec![],
        },
        retention: RetentionPolicy {
            keep_daily: map.get("KEEP_DAILY").and_then(|v| v.parse().ok()).unwrap_or(7),
            keep_weekly: map.get("KEEP_WEEKLY").and_then(|v| v.parse().ok()).unwrap_or(4),
            keep_monthly: map.get("KEEP_MONTHLY").and_then(|v| v.parse().ok()).unwrap_or(12),
        },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: map.get("RCLONE_CONFIG_SYNO_BACKUP_TYPE").cloned().unwrap_or_else(|| "sftp".into()),
                repository: repo,
                password: SecretString::new(pwd.into()),
                sftp: Some(SftpConfig {
                    host: map.get("RCLONE_CONFIG_SYNO_BACKUP_HOST").cloned().unwrap_or_default(),
                    port: map.get("RCLONE_CONFIG_SYNO_BACKUP_PORT").and_then(|v| v.parse().ok()).unwrap_or(22),
                    user: map.get("RCLONE_CONFIG_SYNO_BACKUP_USER").cloned().unwrap_or_default(),
                    key_file: map.get("RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE").cloned(),
                }),
                s3: None,
            },
            secondary: None,
        },
    })
}

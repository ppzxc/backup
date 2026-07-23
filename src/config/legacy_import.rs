use anyhow::Result;
use secrecy::SecretString;
use std::collections::HashMap;
use crate::config::model::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyEnvKey {
    ProfileName,
    ResticRepository,
    ResticPassword,
    BackupTargetsKey,
    KeepDaily,
    KeepWeekly,
    KeepMonthly,
    StorageType,
    StorageHost,
    StoragePort,
    StorageUser,
    StorageKeyFile,
}

impl LegacyEnvKey {
    pub fn as_str(&self) -> &'static str {
        match self {
            LegacyEnvKey::ProfileName => "BACKUP_PROFILE_NAME",
            LegacyEnvKey::ResticRepository => "RESTIC_REPOSITORY",
            LegacyEnvKey::ResticPassword => "RESTIC_PASSWORD",
            LegacyEnvKey::BackupTargetsKey => "BACKUP_TARGETS",
            LegacyEnvKey::KeepDaily => "KEEP_DAILY",
            LegacyEnvKey::KeepWeekly => "KEEP_WEEKLY",
            LegacyEnvKey::KeepMonthly => "KEEP_MONTHLY",
            LegacyEnvKey::StorageType => "RCLONE_CONFIG_SYNO_BACKUP_TYPE",
            LegacyEnvKey::StorageHost => "RCLONE_CONFIG_SYNO_BACKUP_HOST",
            LegacyEnvKey::StoragePort => "RCLONE_CONFIG_SYNO_BACKUP_PORT",
            LegacyEnvKey::StorageUser => "RCLONE_CONFIG_SYNO_BACKUP_USER",
            LegacyEnvKey::StorageKeyFile => "RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE",
        }
    }
}

pub struct LegacyEnvMap {
    values: HashMap<String, String>,
}

impl LegacyEnvMap {
    pub fn parse(content: &str) -> Self {
        let mut values = HashMap::new();
        for line in content.lines() {
            let trimmed = line.trim();
            let stripped = trimmed.strip_prefix("export ").unwrap_or(trimmed);
            if let Some((k, v)) = stripped.split_once('=') {
                let val = v.trim_matches('"').trim_matches('\'');
                values.insert(k.trim().to_string(), val.to_string());
            }
        }
        Self { values }
    }

    pub fn get(&self, key: LegacyEnvKey) -> Option<&str> {
        self.values.get(key.as_str()).map(|s| s.as_str())
    }

    pub fn get_or_default(&self, key: LegacyEnvKey, default: &str) -> String {
        self.get(key).unwrap_or(default).to_string()
    }

    pub fn get_parsed<T: std::str::FromStr>(&self, key: LegacyEnvKey, default: T) -> T {
        self.get(key).and_then(|v| v.parse().ok()).unwrap_or(default)
    }
}

pub fn parse_legacy_env(content: &str) -> Result<BackupConfig> {
    let env_map = LegacyEnvMap::parse(content);

    let profile = env_map.get_or_default(LegacyEnvKey::ProfileName, "default");
    let repo = env_map.get_or_default(LegacyEnvKey::ResticRepository, "");
    let pwd = env_map.get_or_default(LegacyEnvKey::ResticPassword, "");
    let targets_str = env_map.get_or_default(LegacyEnvKey::BackupTargetsKey, "");
    let targets = targets_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    Ok(BackupConfig {
        version: "1.0".to_string(),
        profile,
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets,
            excludes: vec![],
        },
        retention: RetentionPolicy {
            keep_daily: env_map.get_parsed(LegacyEnvKey::KeepDaily, 7),
            keep_weekly: env_map.get_parsed(LegacyEnvKey::KeepWeekly, 4),
            keep_monthly: env_map.get_parsed(LegacyEnvKey::KeepMonthly, 12),
        },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: env_map.get_or_default(LegacyEnvKey::StorageType, "sftp"),
                repository: repo,
                password: SecretString::new(pwd.into()),
                sftp: Some(SftpConfig {
                    host: env_map.get_or_default(LegacyEnvKey::StorageHost, ""),
                    port: env_map.get_parsed(LegacyEnvKey::StoragePort, 22),
                    user: env_map.get_or_default(LegacyEnvKey::StorageUser, ""),
                    key_file: env_map.get(LegacyEnvKey::StorageKeyFile).map(|s| s.to_string()),
                }),
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
    })
}

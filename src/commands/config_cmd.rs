use anyhow::Result;
use secrecy::SecretString;

use crate::config::model::BackupConfig;

pub fn execute_config_show(config: &BackupConfig) -> Result<String> {
    let mut masked_config = config.clone();
    masked_config.storage.primary.password = SecretString::new("******".into());
    if let Some(ref mut s3) = masked_config.storage.primary.s3 {
        s3.secret_access_key = SecretString::new("******".into());
    }
    if let Some(ref mut sec) = masked_config.storage.secondary {
        sec.password = SecretString::new("******".into());
    }
    let yaml = serde_yaml::to_string(&masked_config)?;
    Ok(yaml)
}

pub fn execute_config_export(config: &BackupConfig, format: &str) -> Result<String> {
    if format == "json" {
        Ok(serde_json::to_string_pretty(config)?)
    } else {
        Ok(serde_yaml::to_string(config)?)
    }
}

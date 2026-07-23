use anyhow::Result;
use crate::config::model::BackupConfig;

pub fn execute_config_show(config: &BackupConfig) -> Result<String> {
    config.render("yaml", true)
}

pub fn execute_config_export(config: &BackupConfig, format: &str) -> Result<String> {
    config.render(format, false)
}


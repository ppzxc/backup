use anyhow::Result;
use std::path::Path;
use crate::config::model::BackupConfig;

pub struct ConfigurationRegistry;

impl ConfigurationRegistry {
    pub fn load(path: &Path) -> Result<BackupConfig> {
        let config = BackupConfig::load_from_path(path)?;
        Ok(config)
    }

    pub fn save_profile(config: &BackupConfig, config_dir: &Path) -> Result<()> {
        config.save_and_sync(config_dir)?;
        Ok(())
    }

    pub fn load_and_validate_config(path: &Path) -> Result<BackupConfig> {
        Self::load(path)
    }

    pub fn save_profile_config(config: &BackupConfig, config_dir: &Path) -> Result<()> {
        Self::save_profile(config, config_dir)
    }
}


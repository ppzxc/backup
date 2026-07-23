use anyhow::Result;
use crate::config::model::BackupConfig;

pub fn execute_status(config: &BackupConfig) -> Result<String> {
    Ok(format!(
        "Profile: {}\nBackend: {}\nRepository: {}\nTargets: {:?}",
        config.profile,
        config.storage.primary.backend,
        config.storage.primary.repository,
        config.backup.targets
    ))
}

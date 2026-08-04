use crate::runner::scheduler::BackupScheduler;
use anyhow::Result;
use std::path::Path;

pub fn execute_schedule_enable<R: BackupScheduler>(
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    runner.enable(config_path)
}

pub fn execute_schedule_disable<R: BackupScheduler>(
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    runner.disable()
}

pub fn execute_schedule_status<R: BackupScheduler>(runner: &R) -> Result<String> {
    match runner.status() {
        Ok(res) => Ok(res),
        Err(err) => Ok(format!(
            "Schedule status: Inactive or scheduler unavailable ({})",
            err
        )),
    }
}

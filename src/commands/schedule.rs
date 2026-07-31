use crate::runner::resticprofile::ResticProfileRunner;
use anyhow::Result;
use std::path::Path;

pub fn execute_schedule_enable<R: ResticProfileRunner>(
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    runner.schedule_enable(config_path)
}

pub fn execute_schedule_disable<R: ResticProfileRunner>(
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    runner.schedule_disable(config_path)
}

pub fn execute_schedule_status<R: ResticProfileRunner>(
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    match runner.schedule_status(config_path) {
        Ok(res) => Ok(res),
        Err(err) => Ok(format!(
            "Schedule status: Inactive or resticprofile unavailable ({})",
            err
        )),
    }
}

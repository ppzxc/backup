use crate::runner::scheduler::{BackupScheduler, SchedulerMode};
use anyhow::Result;
use std::path::Path;

pub fn execute_schedule_enable<R: BackupScheduler + ?Sized>(
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    execute_schedule_enable_with_mode(config_path, runner, SchedulerMode::Auto)
}

pub fn execute_schedule_enable_with_mode<R: BackupScheduler + ?Sized>(
    config_path: &Path,
    runner: &R,
    mode: SchedulerMode,
) -> Result<String> {
    crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    runner.enable_with_mode(config_path, mode)
}

pub fn execute_schedule_disable<R: BackupScheduler + ?Sized>(
    _config_path: &Path,
    runner: &R,
) -> Result<String> {
    execute_schedule_disable_with_mode(_config_path, runner, SchedulerMode::Auto)
}

pub fn execute_schedule_disable_with_mode<R: BackupScheduler + ?Sized>(
    _config_path: &Path,
    runner: &R,
    mode: SchedulerMode,
) -> Result<String> {
    runner.disable_with_mode(mode)
}

pub fn execute_schedule_status<R: BackupScheduler + ?Sized>(runner: &R) -> Result<String> {
    execute_schedule_status_with_mode(runner, SchedulerMode::Auto)
}

pub fn execute_schedule_status_with_mode<R: BackupScheduler + ?Sized>(
    runner: &R,
    mode: SchedulerMode,
) -> Result<String> {
    runner.status_with_mode(mode)
}

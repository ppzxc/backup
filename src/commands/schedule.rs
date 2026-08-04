use crate::runner::scheduler::{BackupScheduler, SchedulerMode, SchedulerSettings};
use anyhow::Result;
use std::path::Path;

pub fn execute_schedule_enable<R: BackupScheduler + ?Sized>(
    config_path: &Path,
    runner: &R,
) -> Result<String> {
    execute_schedule_enable_with_settings(config_path, runner, &SchedulerSettings::auto())
}

pub fn execute_schedule_enable_with_mode<R: BackupScheduler + ?Sized>(
    config_path: &Path,
    runner: &R,
    mode: SchedulerMode,
) -> Result<String> {
    execute_schedule_enable_with_settings(
        config_path,
        runner,
        &SchedulerSettings::new(mode, crate::runner::scheduler::DEFAULT_SCHEDULE_CALENDAR),
    )
}

pub fn execute_schedule_enable_with_settings<R: BackupScheduler + ?Sized>(
    config_path: &Path,
    runner: &R,
    settings: &SchedulerSettings,
) -> Result<String> {
    crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    runner.enable_with_settings(config_path, settings)
}

pub fn execute_schedule_disable<R: BackupScheduler + ?Sized>(
    _config_path: &Path,
    runner: &R,
) -> Result<String> {
    execute_schedule_disable_with_settings(_config_path, runner, &SchedulerSettings::auto())
}

pub fn execute_schedule_disable_with_mode<R: BackupScheduler + ?Sized>(
    _config_path: &Path,
    runner: &R,
    mode: SchedulerMode,
) -> Result<String> {
    execute_schedule_disable_with_settings(
        _config_path,
        runner,
        &SchedulerSettings::new(mode, crate::runner::scheduler::DEFAULT_SCHEDULE_CALENDAR),
    )
}

pub fn execute_schedule_disable_with_settings<R: BackupScheduler + ?Sized>(
    _config_path: &Path,
    runner: &R,
    settings: &SchedulerSettings,
) -> Result<String> {
    runner.disable_with_settings(settings)
}

pub fn execute_schedule_status<R: BackupScheduler + ?Sized>(runner: &R) -> Result<String> {
    execute_schedule_status_with_settings(runner, &SchedulerSettings::auto())
}

pub fn execute_schedule_status_with_mode<R: BackupScheduler + ?Sized>(
    runner: &R,
    mode: SchedulerMode,
) -> Result<String> {
    execute_schedule_status_with_settings(
        runner,
        &SchedulerSettings::new(mode, crate::runner::scheduler::DEFAULT_SCHEDULE_CALENDAR),
    )
}

pub fn execute_schedule_status_with_settings<R: BackupScheduler + ?Sized>(
    runner: &R,
    settings: &SchedulerSettings,
) -> Result<String> {
    runner.status_with_settings(settings)
}

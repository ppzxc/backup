use anyhow::Result;
use std::path::Path;
use secrecy::ExposeSecret;
use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use crate::runner::resticprofile::ResticProfileRunner;

pub fn execute_run<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    runner.backup_paths(repo, pwd, &config.backup.targets, &config.backup.excludes)
}

pub fn execute_run_profile<R: ResticProfileRunner>(
    config_path: &Path,
    profile: &str,
    dry_run: bool,
    runner: &R,
) -> Result<String> {
    runner.backup(config_path, profile, dry_run)
}

use anyhow::Result;
use secrecy::ExposeSecret;
use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;

pub fn execute_snapshots<R: ResticRunner>(config: &BackupConfig, runner: &R) -> Result<String> {
    let repo = &config.storage.primary.repository;
    let pwd = config.storage.primary.password.expose_secret();
    runner.list_snapshots(repo, pwd)
}

use crate::config::model::BackupConfig;
use crate::runner::restic::ResticRunner;
use anyhow::{Result, bail};
use secrecy::ExposeSecret;
use std::path::Path;

pub fn execute_restore<R: ResticRunner>(
    config: &BackupConfig,
    runner: &R,
    snapshot_id: &str,
    target_path: &str,
    force: bool,
) -> Result<String> {
    let target = Path::new(target_path);
    if target.exists() && target.read_dir()?.next().is_some() && !force {
        bail!("Restore target is not empty; pass --force to overwrite");
    }
    runner.restore(
        &config.storage.primary.repository,
        config.storage.primary.password.expose_secret(),
        snapshot_id,
        target_path,
    )
}

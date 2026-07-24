use anyhow::Result;
use std::path::Path;
use crate::runner::resticprofile::ResticProfileRunner;

pub fn execute_copy<R: ResticProfileRunner>(
    runner: &R,
    config_path: &Path,
    profile: &str,
    dry_run: bool,
) -> Result<String> {
    let out = runner.copy(config_path, profile, dry_run)?;
    Ok(format!("Snapshot copy completed for profile [{}]:\n{}", profile, out))
}

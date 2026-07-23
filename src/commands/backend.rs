use anyhow::Result;
use crate::runner::rclone::RcloneRunner;

pub fn execute_backend_migrate<R: RcloneRunner>(runner: &R, source: &str, target: &str) -> Result<String> {
    runner.sync(source, target)?;
    Ok(format!("Backend snapshot migration completed: {} -> {}", source, target))
}

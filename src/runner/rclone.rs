use crate::runner::executor::CommandRunner;
use anyhow::Result;

pub trait RcloneRunner {
    fn check_connectivity(&self, remote: &str) -> Result<String>;
    fn list_remotes(&self) -> Result<String>;
    fn sync(&self, source: &str, target: &str) -> Result<String>;
}

pub struct RcloneTool<'a, E: CommandRunner> {
    executor: &'a E,
}

impl<'a, E: CommandRunner> RcloneTool<'a, E> {
    pub fn new(executor: &'a E) -> Self {
        Self { executor }
    }
}

impl<'a, E: CommandRunner> RcloneRunner for RcloneTool<'a, E> {
    fn check_connectivity(&self, remote: &str) -> Result<String> {
        let output = self.executor.run("rclone", &["lsd", remote])?;
        if output.status_code != 0 {
            anyhow::bail!("rclone connectivity check failed: {}", output.stderr.trim());
        }
        Ok(output.stdout)
    }

    fn list_remotes(&self) -> Result<String> {
        let output = self.executor.run("rclone", &["listremotes"])?;
        Ok(output.stdout)
    }

    fn sync(&self, source: &str, target: &str) -> Result<String> {
        let output = self.executor.run("rclone", &["sync", source, target])?;
        Ok(output.stdout)
    }
}

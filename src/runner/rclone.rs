use anyhow::Result;
use crate::runner::executor::CommandRunner;

pub trait RcloneRunner {
    fn check_connectivity(&self, remote: &str) -> Result<String>;
    fn list_remotes(&self) -> Result<String>;
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
        Ok(output.stdout)
    }

    fn list_remotes(&self) -> Result<String> {
        let output = self.executor.run("rclone", &["listremotes"])?;
        Ok(output.stdout)
    }
}

pub struct MockRcloneRunner {
    pub exit_code: i32,
    pub response: String,
}

impl MockRcloneRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self {
            exit_code,
            response: response.to_string(),
        }
    }
}

impl RcloneRunner for MockRcloneRunner {
    fn check_connectivity(&self, _remote: &str) -> Result<String> {
        Ok(self.response.clone())
    }
    fn list_remotes(&self) -> Result<String> {
        Ok(self.response.clone())
    }
}


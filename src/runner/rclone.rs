use anyhow::Result;
use std::process::Command;

pub trait RcloneRunner {
    fn check_connectivity(&self, remote: &str) -> Result<String>;
    fn list_remotes(&self) -> Result<String>;
}

pub struct SystemRcloneRunner;

impl RcloneRunner for SystemRcloneRunner {
    fn check_connectivity(&self, remote: &str) -> Result<String> {
        let output = Command::new("rclone")
            .arg("lsd")
            .arg(remote)
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn list_remotes(&self) -> Result<String> {
        let output = Command::new("rclone")
            .arg("listremotes")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
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

use anyhow::{Context, Result};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput>;
    fn run_with_env(
        &self,
        program: &str,
        args: &[&str],
        _env: &[(&str, &str)],
    ) -> Result<CommandOutput> {
        self.run(program, args)
    }
}

pub struct SystemExecutor;

impl CommandRunner for SystemExecutor {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        let output = Command::new(program)
            .args(args)
            .output()
            .with_context(|| format!("Failed to execute process: {}", program))?;

        Ok(CommandOutput {
            status_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
    fn run_with_env(
        &self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<CommandOutput> {
        let output = Command::new(program)
            .args(args)
            .envs(env.iter().copied())
            .output()
            .with_context(|| format!("Failed to execute process: {}", program))?;
        Ok(CommandOutput {
            status_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

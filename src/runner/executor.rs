use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;

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
    fn run_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        _timeout: Duration,
    ) -> Result<CommandOutput> {
        self.run_with_env(program, args, env)
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
    fn run_with_timeout(
        &self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
        timeout: Duration,
    ) -> Result<CommandOutput> {
        let mut child = Command::new(program)
            .args(args)
            .envs(env.iter().copied())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute process: {}", program))?;

        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child.wait_with_output()?;
                    return Ok(CommandOutput {
                        status_code: status.code().unwrap_or(-1),
                        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    });
                }
                Ok(None) => {
                    if start.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Ok(CommandOutput {
                            status_code: -1,
                            stdout: String::new(),
                            stderr: format!(
                                "Process execution timed out after {} seconds",
                                timeout.as_secs()
                            ),
                        });
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    anyhow::bail!("Error waiting for process: {}", e);
                }
            }
        }
    }
}

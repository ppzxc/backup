#![allow(dead_code)]

use anyhow::Result;
use backup::runner::executor::{CommandOutput, CommandRunner};
use backup::runner::rclone::RcloneRunner;
use backup::runner::restic::ResticRunner;
use backup::runner::resticprofile::ResticProfileRunner;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct MockExecutor {
    responses: Arc<Mutex<HashMap<String, Vec<CommandOutput>>>>,
    calls: Arc<Mutex<Vec<(String, Vec<String>)>>>,
    environment_calls: Arc<Mutex<Vec<Vec<(String, String)>>>>,
}
impl MockExecutor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push_output(&self, program: &str, output: CommandOutput) {
        self.responses
            .lock()
            .unwrap()
            .entry(program.into())
            .or_default()
            .push(output);
    }
    pub fn call_count(&self, program: &str) -> usize {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|(name, _)| name == program)
            .count()
    }
    pub fn get_calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }
    pub fn get_environment_calls(&self) -> Vec<Vec<(String, String)>> {
        self.environment_calls.lock().unwrap().clone()
    }
}
impl CommandRunner for MockExecutor {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        self.calls.lock().unwrap().push((
            program.into(),
            args.iter().map(|arg| (*arg).into()).collect(),
        ));
        Ok(self
            .responses
            .lock()
            .unwrap()
            .get_mut(program)
            .and_then(|outputs| (!outputs.is_empty()).then(|| outputs.remove(0)))
            .unwrap_or(CommandOutput {
                status_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            }))
    }
    fn run_with_env(
        &self,
        program: &str,
        args: &[&str],
        env: &[(&str, &str)],
    ) -> Result<CommandOutput> {
        self.environment_calls.lock().unwrap().push(
            env.iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect(),
        );
        self.run(program, args)
    }
}

pub struct MockResticRunner {
    pub exit_code: i32,
    pub response: String,
    pub command_calls: Mutex<Vec<(String, Vec<String>, Vec<(String, String)>)>>,
}
impl MockResticRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self {
            exit_code,
            response: response.into(),
            command_calls: Mutex::new(vec![]),
        }
    }
    fn result(&self) -> Result<String> {
        if self.exit_code != 0 {
            anyhow::bail!(
                "mock restic failed with exit code {}: {}",
                self.exit_code,
                self.response
            )
        }
        Ok(self.response.clone())
    }
}
impl ResticRunner for MockResticRunner {
    fn init_repo(&self, _: &str, _: &str) -> Result<String> {
        self.result()
    }
    fn backup_paths(&self, _: &str, _: &str, _: &[String], _: &[String]) -> Result<String> {
        self.result()
    }
    fn list_snapshots(&self, _: &str, _: &str) -> Result<String> {
        self.result()
    }
    fn restore(&self, _: &str, _: &str, _: &str, _: &str) -> Result<String> {
        self.result()
    }
    fn backup_command(&self, _: &str, _: &str, _: &str, _: &str, _: &[String]) -> Result<String> {
        self.result()
    }
    fn backup_command_with_env(
        &self,
        _: &str,
        _: &str,
        _: &str,
        program: &str,
        args: &[String],
        env: &[(&str, &str)],
    ) -> Result<String> {
        self.command_calls.lock().unwrap().push((
            program.into(),
            args.to_vec(),
            env.iter()
                .map(|(key, value)| ((*key).into(), (*value).into()))
                .collect(),
        ));
        self.result()
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
            response: response.into(),
        }
    }
    fn result(&self) -> Result<String> {
        if self.exit_code != 0 {
            anyhow::bail!("mock rclone failed: {}", self.response)
        }
        Ok(self.response.clone())
    }
}
impl RcloneRunner for MockRcloneRunner {
    fn check_connectivity(&self, _: &str) -> Result<String> {
        self.result()
    }
    fn list_remotes(&self) -> Result<String> {
        self.result()
    }
    fn sync(&self, _: &str, _: &str) -> Result<String> {
        self.result()
    }
}

pub struct MockResticProfileRunner {
    pub exit_code: i32,
    pub response: String,
    pub calls: Mutex<Vec<(String, String)>>,
}
impl MockResticProfileRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self {
            exit_code,
            response: response.into(),
            calls: Mutex::new(vec![]),
        }
    }
    fn result(&self, action: &str, path: &Path) -> Result<String> {
        self.calls
            .lock()
            .unwrap()
            .push((action.into(), path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response)
        }
        Ok(self.response.clone())
    }
}
impl ResticProfileRunner for MockResticProfileRunner {
    fn backup(&self, path: &Path, _: &str, _: bool) -> Result<String> {
        self.result("backup", path)
    }
    fn init(&self, path: &Path, _: &str) -> Result<String> {
        self.result("init", path)
    }
    fn schedule_enable(&self, path: &Path) -> Result<String> {
        self.result("schedule_enable", path)
    }
    fn schedule_disable(&self, path: &Path) -> Result<String> {
        self.result("schedule_disable", path)
    }
    fn schedule_status(&self, path: &Path) -> Result<String> {
        self.result("schedule_status", path)
    }
    fn list_snapshots(&self, path: &Path, _: &str) -> Result<String> {
        self.result("list_snapshots", path)
    }
    fn prune(&self, path: &Path, _: &str) -> Result<String> {
        self.result("prune", path)
    }
    fn check(&self, path: &Path, _: &str) -> Result<String> {
        self.result("check", path)
    }
    fn copy(&self, path: &Path, _: &str, _: bool) -> Result<String> {
        self.result("copy", path)
    }
}

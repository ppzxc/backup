use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::Command;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    pub status_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait CommandRunner: Send + Sync {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput>;
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
}

#[derive(Clone, Default)]
pub struct MockExecutor {
    responses: Arc<Mutex<HashMap<String, Vec<CommandOutput>>>>,
    calls: Arc<Mutex<Vec<(String, Vec<String>)>>>,
}

impl MockExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_output(&self, program: &str, output: CommandOutput) {
        let mut map = self.responses.lock().unwrap();
        map.entry(program.to_string()).or_default().push(output);
    }

    pub fn call_count(&self, program: &str) -> usize {
        let calls = self.calls.lock().unwrap();
        calls.iter().filter(|(p, _)| p == program).count()
    }

    pub fn get_calls(&self) -> Vec<(String, Vec<String>)> {
        self.calls.lock().unwrap().clone()
    }
}

impl CommandRunner for MockExecutor {
    fn run(&self, program: &str, args: &[&str]) -> Result<CommandOutput> {
        let mut calls = self.calls.lock().unwrap();
        calls.push((program.to_string(), args.iter().map(|s| s.to_string()).collect()));

        let mut responses = self.responses.lock().unwrap();
        if let Some(list) = responses.get_mut(program) {
            if !list.is_empty() {
                return Ok(list.remove(0));
            }
        }

        Ok(CommandOutput {
            status_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

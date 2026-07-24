use anyhow::Result;
use std::path::Path;
use crate::runner::executor::{CommandOutput, CommandRunner};

pub trait ResticProfileRunner {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String>;
    fn init(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn schedule_enable(&self, config_path: &Path) -> Result<String>;
    fn schedule_disable(&self, config_path: &Path) -> Result<String>;
    fn schedule_status(&self, config_path: &Path) -> Result<String>;
    fn list_snapshots(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn prune(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn check(&self, config_path: &Path, profile: &str) -> Result<String>;
    fn copy(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String>;
}

pub struct ResticProfileTool<'a, E: CommandRunner> {
    executor: &'a E,
}

impl<'a, E: CommandRunner> ResticProfileTool<'a, E> {
    pub fn new(executor: &'a E) -> Self {
        Self { executor }
    }

    fn check_output(&self, output: CommandOutput) -> Result<String> {
        if output.status_code != 0 {
            let err_msg = if !output.stderr.trim().is_empty() {
                output.stderr.trim().to_string()
            } else if !output.stdout.trim().is_empty() {
                output.stdout.trim().to_string()
            } else {
                format!("command exited with status code {}", output.status_code)
            };
            anyhow::bail!(
                "resticprofile failed with exit code {}: {}",
                output.status_code,
                err_msg
            );
        }
        Ok(output.stdout)
    }
}

impl<'a, E: CommandRunner> ResticProfileRunner for ResticProfileTool<'a, E> {
    fn backup(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let mut args = vec!["--config", &config_str, "--name", profile];
        if dry_run {
            args.push("--dry-run");
        }
        args.push("backup");
        let output = self.executor.run("resticprofile", &args)?;
        self.check_output(output)
    }

    fn init(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "--name", profile, "init"])?;
        self.check_output(output)
    }

    fn schedule_enable(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "schedule", "--all"])?;
        self.check_output(output)
    }

    fn schedule_disable(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "unschedule", "--all"])?;
        self.check_output(output)
    }

    fn schedule_status(&self, config_path: &Path) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "status", "--all"])?;
        self.check_output(output)
    }

    fn list_snapshots(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "--name", profile, "snapshots"])?;
        self.check_output(output)
    }

    fn prune(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "--name", profile, "prune"])?;
        self.check_output(output)
    }

    fn check(&self, config_path: &Path, profile: &str) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let output = self.executor.run("resticprofile", &["--config", &config_str, "--name", profile, "check"])?;
        self.check_output(output)
    }

    fn copy(&self, config_path: &Path, profile: &str, dry_run: bool) -> Result<String> {
        let config_str = config_path.to_string_lossy();
        let mut args = vec!["--config", &config_str, "--name", profile];
        if dry_run {
            args.push("--dry-run");
        }
        args.push("copy");
        let output = self.executor.run("resticprofile", &args)?;
        self.check_output(output)
    }
}

pub struct MockResticProfileRunner {
    pub exit_code: i32,
    pub response: String,
    pub calls: std::sync::Mutex<Vec<(String, String)>>,
}

impl MockResticProfileRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self {
            exit_code,
            response: response.to_string(),
            calls: std::sync::Mutex::new(Vec::new()),
        }
    }
}

impl ResticProfileRunner for MockResticProfileRunner {
    fn backup(&self, config_path: &Path, _profile: &str, _dry_run: bool) -> Result<String> {
        self.calls.lock().unwrap().push(("backup".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn init(&self, config_path: &Path, _profile: &str) -> Result<String> {
        self.calls.lock().unwrap().push(("init".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn schedule_enable(&self, config_path: &Path) -> Result<String> {
        self.calls.lock().unwrap().push(("schedule_enable".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn schedule_disable(&self, config_path: &Path) -> Result<String> {
        self.calls.lock().unwrap().push(("schedule_disable".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn schedule_status(&self, config_path: &Path) -> Result<String> {
        self.calls.lock().unwrap().push(("schedule_status".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn list_snapshots(&self, config_path: &Path, _profile: &str) -> Result<String> {
        self.calls.lock().unwrap().push(("list_snapshots".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn prune(&self, config_path: &Path, _profile: &str) -> Result<String> {
        self.calls.lock().unwrap().push(("prune".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn check(&self, config_path: &Path, _profile: &str) -> Result<String> {
        self.calls.lock().unwrap().push(("check".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
    fn copy(&self, config_path: &Path, _profile: &str, _dry_run: bool) -> Result<String> {
        self.calls.lock().unwrap().push(("copy".into(), config_path.to_string_lossy().into()));
        if self.exit_code != 0 {
            anyhow::bail!("mock error: {}", self.response);
        }
        Ok(self.response.clone())
    }
}

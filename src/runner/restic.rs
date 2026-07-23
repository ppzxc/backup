use anyhow::Result;
use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

pub trait ResticRunner {
    fn init_repo(&self, repo: &str, password: &str) -> Result<String>;
    fn backup_paths(
        &self,
        repo: &str,
        password: &str,
        targets: &[String],
        excludes: &[String],
    ) -> Result<String>;
    fn list_snapshots(&self, repo: &str, password: &str) -> Result<String>;
}

pub struct SystemResticRunner;

fn create_temp_password_file(password: &str) -> Result<NamedTempFile> {
    let mut file = NamedTempFile::new()?;
    file.write_all(password.as_bytes())?;
    file.flush()?;
    Ok(file)
}

impl ResticRunner for SystemResticRunner {
    fn init_repo(&self, repo: &str, password: &str) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let output = Command::new("restic")
            .arg("-r")
            .arg(repo)
            .arg("--password-file")
            .arg(pass_file.path())
            .arg("init")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn backup_paths(
        &self,
        repo: &str,
        password: &str,
        targets: &[String],
        excludes: &[String],
    ) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let mut cmd = Command::new("restic");
        cmd.arg("-r")
            .arg(repo)
            .arg("--password-file")
            .arg(pass_file.path())
            .arg("backup");
        for t in targets {
            cmd.arg(t);
        }
        for e in excludes {
            cmd.arg("--exclude").arg(e);
        }
        let output = cmd.output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    fn list_snapshots(&self, repo: &str, password: &str) -> Result<String> {
        let pass_file = create_temp_password_file(password)?;
        let output = Command::new("restic")
            .arg("-r")
            .arg(repo)
            .arg("--password-file")
            .arg(pass_file.path())
            .arg("snapshots")
            .output()?;
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

pub struct MockResticRunner {
    pub exit_code: i32,
    pub response: String,
}

impl MockResticRunner {
    pub fn new(exit_code: i32, response: &str) -> Self {
        Self {
            exit_code,
            response: response.to_string(),
        }
    }
}

impl ResticRunner for MockResticRunner {
    fn init_repo(&self, _repo: &str, _password: &str) -> Result<String> {
        Ok(self.response.clone())
    }
    fn backup_paths(
        &self,
        _repo: &str,
        _password: &str,
        _targets: &[String],
        _excludes: &[String],
    ) -> Result<String> {
        Ok(self.response.clone())
    }
    fn list_snapshots(&self, _repo: &str, _password: &str) -> Result<String> {
        Ok(self.response.clone())
    }
}

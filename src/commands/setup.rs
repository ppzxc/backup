use anyhow::Result;
use std::path::Path;
use secrecy::SecretString;
use crate::config::model::*;

pub trait SetupPrompter {
    fn prompt_setup_params(&self) -> Result<(String, String, String, String)>;
}

pub struct InquirePrompter;

impl SetupPrompter for InquirePrompter {
    fn prompt_setup_params(&self) -> Result<(String, String, String, String)> {
        let profile = inquire::Text::new("Enter Profile Name:")
            .with_default("default")
            .prompt()?;
        let target = inquire::Text::new("Enter Target Directory:")
            .with_default("/data")
            .prompt()?;
        let repo = inquire::Text::new("Enter Repository URI:")
            .prompt()?;
        let password = inquire::Password::new("Enter Encryption Password:")
            .without_confirmation()
            .prompt()?;
        Ok((profile, target, repo, password))
    }
}

pub fn create_default_config_file(path: &Path, profile: &str, target: &str, repo: &str, pwd: &str) -> Result<()> {
    let config = BackupConfig {
        version: "1.0".into(),
        profile: profile.into(),
        backup: BackupTargets { targets: vec![target.into()], excludes: vec![] },
        retention: RetentionPolicy { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "sftp".into(),
                repository: repo.into(),
                password: SecretString::new(pwd.into()),
                sftp: None,
                s3: None,
            },
            secondary: None,
        },
    };
    config.save_to_path(path)
}

pub fn run_setup_with_prompter<P: SetupPrompter>(config_path: &Path, prompter: &P, non_interactive: bool) -> Result<()> {
    use std::io::IsTerminal;
    if !non_interactive && cfg!(not(test)) && std::io::stdin().is_terminal() {
        let (profile, target, repo, password) = prompter.prompt_setup_params()?;
        create_default_config_file(config_path, &profile, &target, &repo, &password)?;
    } else {
        create_default_config_file(config_path, "default", "/data", "rclone:syno_backup:/backup", "default_secret")?;
    }
    Ok(())
}

pub fn run_setup(config_path: &Path) -> Result<()> {
    let prompter = InquirePrompter;
    run_setup_with_prompter(config_path, &prompter, false)
}



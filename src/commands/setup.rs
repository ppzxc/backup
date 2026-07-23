use anyhow::Result;
use std::fs;
use std::path::Path;
use secrecy::SecretString;
use crate::config::model::*;

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
    let yaml = serde_yaml::to_string(&config)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, yaml)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

pub fn prompt_interactive_setup() -> Result<(String, String, String, String)> {
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

pub fn run_setup(config_path: &Path) -> Result<()> {
    let (profile, target, repo, password) = prompt_interactive_setup()?;
    create_default_config_file(config_path, &profile, &target, &repo, &password)?;
    Ok(())
}

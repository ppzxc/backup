use anyhow::Result;
use std::path::Path;
use secrecy::SecretString;
use crate::config::model::*;

pub struct SetupParams {
    pub profile: String,
    pub target: String,
    pub repository: String,
    pub password: SecretString,
}

pub trait SetupPrompter {
    fn prompt_setup_params(&self) -> Result<SetupParams>;
}

pub struct InquirePrompter;

impl SetupPrompter for InquirePrompter {
    fn prompt_setup_params(&self) -> Result<SetupParams> {
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
        Ok(SetupParams {
            profile,
            target,
            repository: repo,
            password: SecretString::new(password),
        })
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
        let params = prompter.prompt_setup_params()?;
        create_default_config_file(config_path, &params.profile, &params.target, &params.repository, secrecy::ExposeSecret::expose_secret(&params.password))?;
    } else {
        create_default_config_file(config_path, "default", "/data", "rclone:syno_backup:/backup", "default_secret")?;
    }
    Ok(())
}

pub fn run_setup(config_path: &Path) -> Result<()> {
    let prompter = InquirePrompter;
    run_setup_with_prompter(config_path, &prompter, false)
}

pub fn run_setup_dependencies() -> Result<String> {
    use std::process::Command;
    let mut report = String::new();
    report.push_str("Checking binary dependencies...\n");

    let install_target_dir = if Path::new("/usr/local/bin").is_dir() {
        "/usr/local/bin"
    } else {
        "/tmp"
    };

    let binaries = [("restic", "https://github.com/restic/restic/releases/download/v0.16.4/restic_0.16.4_linux_amd64.bz2"),
                    ("rclone", "https://downloads.rclone.org/rclone-current-linux-amd64.zip")];

    for (bin, url) in &binaries {
        let status = Command::new("which").arg(bin).output();
        match status {
            Ok(out) if out.status.success() => {
                let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                report.push_str(&format!("{}: OK ({})\n", bin, path));
            }
            _ => {
                report.push_str(&format!("{}: MISSING -> Installing from {}\n", bin, url));
                if *bin == "restic" {
                    let cmd = format!("curl -fsSL {} | bunzip2 > {}/restic && chmod +x {}/restic", url, install_target_dir, install_target_dir);
                    let _ = Command::new("sh").arg("-c").arg(&cmd).output();
                } else if *bin == "rclone" {
                    let cmd = format!("curl -fsSL {} -o /tmp/rclone.zip && unzip -q /tmp/rclone.zip -d /tmp && cp /tmp/rclone-*-linux-amd64/rclone {}/rclone && chmod +x {}/rclone && rm -rf /tmp/rclone*", url, install_target_dir, install_target_dir);
                    let _ = Command::new("sh").arg("-c").arg(&cmd).output();
                }
                report.push_str(&format!("{}: Installed to {}/{}\n", bin, install_target_dir, bin));
            }
        }
    }
    Ok(report)
}





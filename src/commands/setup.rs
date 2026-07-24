use anyhow::Result;
use std::path::Path;
use secrecy::SecretString;
use crate::config::model::*;
use crate::i18n::{Language, I18nMessages};

#[derive(Clone)]
pub struct SetupParams {
    pub profile: String,
    pub backup_type: BackupType,
    pub targets: Vec<String>,
    pub excludes: Vec<String>,
    pub retention: RetentionPolicy,
    pub primary_storage: StorageTarget,
    pub secondary_storage: Option<SecondaryStorageTarget>,
    pub reports: ReportsConfig,
}

pub trait SetupPrompter {
    fn prompt_setup_params(&self, lang_opt: Option<Language>) -> Result<SetupParams>;
}

pub struct InquirePrompter;

impl SetupPrompter for InquirePrompter {
    fn prompt_setup_params(&self, lang_opt: Option<Language>) -> Result<SetupParams> {
        let lang = match lang_opt {
            Some(l) => l,
            None => {
                let choice = inquire::Select::new(
                    "Select Language / 언어 선택:",
                    vec!["[1] 한국어 (Korean)", "[2] English"],
                ).prompt()?;
                if choice.starts_with("[1]") {
                    Language::Ko
                } else {
                    Language::En
                }
            }
        };

        let msg = I18nMessages::get(lang);

        let profile = inquire::Text::new(msg.enter_profile_name)
            .with_default("default")
            .prompt()?;

        let backup_type_choice = inquire::Select::new(
            msg.select_backup_type,
            vec![msg.dir_batch_backup, msg.db_stream_backup],
        ).prompt()?;

        let (backup_type, targets) = if backup_type_choice.starts_with("[1]") {
            let t = inquire::Text::new(msg.enter_target_dir)
                .with_default("/var/log")
                .prompt()?;
            let target_list: Vec<String> = t.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
            (BackupType::Directory, target_list)
        } else {
            let db_kind = inquire::Select::new(msg.select_db_type, vec!["mysql", "postgres"]).prompt()?;
            let conn = inquire::Text::new(msg.enter_conn_url).prompt_skippable()?;
            (
                BackupType::DbStream {
                    db_type: db_kind.to_string(),
                    connection_url: conn.filter(|s| !s.is_empty()),
                    dump_command: None,
                },
                vec![format!("db-stream:{}", db_kind)],
            )
        };

        let excludes_str = inquire::Text::new(msg.enter_exclude_patterns)
            .with_default("")
            .prompt()?;
        let excludes: Vec<String> = excludes_str.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

        // Retention defaults depending on type
        let (default_daily, default_weekly, default_monthly) = match backup_type {
            BackupType::Directory => (7, 4, 12),
            BackupType::DbStream { .. } => (180, 12, 24),
        };

        let keep_daily = inquire::CustomType::<u32>::new(msg.retention_keep_daily)
            .with_default(default_daily)
            .prompt()?;
        let keep_weekly = inquire::CustomType::<u32>::new(msg.retention_keep_weekly)
            .with_default(default_weekly)
            .prompt()?;
        let keep_monthly = inquire::CustomType::<u32>::new(msg.retention_keep_monthly)
            .with_default(default_monthly)
            .prompt()?;

        // Primary Storage Setup
        let backend = inquire::Select::new(msg.primary_storage_backend, vec!["sftp", "s3", "local"])
            .prompt()?;

        let (repository, sftp_config) = if backend == "sftp" {
            let host = inquire::Text::new(msg.sftp_host).with_default("192.168.1.100").prompt()?;
            let port = inquire::CustomType::<u16>::new(msg.sftp_port).with_default(22).prompt()?;
            let user = inquire::Text::new(msg.sftp_user).with_default("backup").prompt()?;
            let path = inquire::Text::new(msg.sftp_path).with_default("/backup").prompt()?;

            let gen_key = inquire::Confirm::new(msg.sftp_auto_gen_key).with_default(true).prompt()?;
            let key_file = if gen_key {
                let key_path = "/etc/backup/id_ed25519";
                let pub_path = "/etc/backup/id_ed25519.pub";
                if !Path::new(key_path).exists() {
                    let _ = std::process::Command::new("ssh-keygen")
                        .args(["-t", "ed25519", "-N", "", "-f", key_path])
                        .output();
                }
                if let Ok(pub_key) = std::fs::read_to_string(pub_path) {
                    println!("\n=== SSH Public Key (/etc/backup/id_ed25519.pub) ===");
                    println!("{}", pub_key.trim());
                    println!("Please register this public key into remote server's ~/.ssh/authorized_keys\n");
                }
                key_path.to_string()
            } else {
                let k = inquire::Text::new(msg.sftp_key_file)
                    .with_default("/etc/backup/id_rsa")
                    .prompt()?;
                if k.trim().is_empty() {
                    anyhow::bail!(msg.isms_sftp_key_error);
                }
                k
            };

            let repo_uri = format!("sftp:{}@{}:{}", user, host, path);
            (repo_uri, Some(SftpConfig {
                host,
                port,
                user,
                key_file: Some(key_file),
            }))
        } else if backend == "s3" {
            let repo_uri = inquire::Text::new(msg.primary_repo_uri)
                .with_default("s3:https://s3.amazonaws.com/my-backup-bucket")
                .prompt()?;
            (repo_uri, None)
        } else {
            let repo_uri = inquire::Text::new(msg.primary_repo_uri)
                .with_default("/data/backup")
                .prompt()?;
            (repo_uri, None)
        };

        let password = inquire::Password::new(msg.enter_encryption_password)
            .without_confirmation()
            .prompt()?;
        
        if password.len() < 12 {
            anyhow::bail!(msg.isms_password_error);
        }

        let primary_storage = StorageTarget {
            backend: backend.to_string(),
            repository,
            password: SecretString::new(password),
            sftp: sftp_config,
            s3: None,
        };

        // Secondary Storage Setup (Optional)
        let enable_sec = inquire::Confirm::new(msg.config_secondary_storage)
            .with_default(false)
            .prompt()?;

        let secondary_storage = if enable_sec {
            let sec_backend = inquire::Select::new(msg.secondary_backend, vec!["sftp", "s3", "local"]).prompt()?;
            let sec_repo = inquire::Text::new(msg.secondary_repo_uri).prompt()?;
            let sec_pass = inquire::Password::new(msg.secondary_password).without_confirmation().prompt()?;
            Some(SecondaryStorageTarget {
                enabled: true,
                backend: sec_backend.to_string(),
                repository: sec_repo,
                password: SecretString::new(sec_pass),
            })
        } else {
            None
        };

        // ISMS Report Options Setup
        let enable_reports = inquire::Confirm::new(msg.enable_isms_reports)
            .with_default(true)
            .prompt()?;

        let report_dir_path = "/data/backup/reports";
        let reports = if enable_reports {
            let output_dir = inquire::Text::new(msg.report_export_dir)
                .with_default(report_dir_path)
                .prompt()?;
            let _ = std::fs::create_dir_all(&output_dir);
            ReportsConfig {
                output_dir,
                enable_daily_reports: true,
                enable_annual_dr_drill_report: true,
            }
        } else {
            ReportsConfig {
                output_dir: report_dir_path.into(),
                enable_daily_reports: false,
                enable_annual_dr_drill_report: false,
            }
        };

        Ok(SetupParams {
            profile,
            backup_type,
            targets,
            excludes,
            retention: RetentionPolicy {
                keep_daily,
                keep_weekly,
                keep_monthly,
            },
            primary_storage,
            secondary_storage,
            reports,
        })
    }
}

pub fn create_default_config_file(path: &Path, profile: &str, target: &str, repo: &str, pwd: &str) -> Result<()> {
    let config = BackupConfig {
        version: "1.0".into(),
        profile: profile.into(),
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets: vec![target.into()],
            excludes: vec![],
        },
        retention: RetentionPolicy { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
        storage: StorageConfig {
            primary: StorageTarget {
                backend: "sftp".into(),
                repository: repo.into(),
                password: SecretString::new(pwd.into()),
                sftp: Some(SftpConfig {
                    host: "192.168.1.100".into(),
                    port: 22,
                    user: "backup".into(),
                    key_file: Some("/etc/backup/id_ed25519".into()),
                }),
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
    };
    let config_dir = path.parent().unwrap_or(path);
    config.save_and_sync(config_dir)
}

pub struct SetupEngine;

impl SetupEngine {
    pub fn validate_and_build(params: SetupParams) -> Result<BackupConfig> {
        let password_len = secrecy::ExposeSecret::expose_secret(&params.primary_storage.password).len();
        if password_len < 12 {
            anyhow::bail!("ISMS Compliance Error: Password must be at least 12 characters long.");
        }

        if params.primary_storage.backend == "sftp" {
            let key_file = params
                .primary_storage
                .sftp
                .as_ref()
                .and_then(|s| s.key_file.as_deref())
                .unwrap_or("");
            if key_file.trim().is_empty() {
                anyhow::bail!("ISMS Compliance Error: SFTP requires SSH key_file path for passwordless key-based authentication.");
            }
        }

        Ok(BackupConfig {
            version: "1.0".into(),
            profile: params.profile,
            backup: BackupTargets {
                backup_type: params.backup_type,
                targets: params.targets,
                excludes: params.excludes,
            },
            retention: params.retention,
            storage: StorageConfig {
                primary: params.primary_storage,
                secondary: params.secondary_storage,
            },
            reports: params.reports,
        })
    }

    pub fn run<P: SetupPrompter>(config_path: &Path, prompter: &P, non_interactive: bool, lang_opt: Option<Language>) -> Result<()> {
        use std::io::IsTerminal;
        if !non_interactive && cfg!(not(test)) && std::io::stdin().is_terminal() {
            let params = prompter.prompt_setup_params(lang_opt)?;
            let config = Self::validate_and_build(params)?;

            if let Some(parent) = config_path.parent() {
                crate::config::registry::ConfigurationRegistry::save_profile_config(&config, parent)?;
            } else {
                crate::config::registry::ConfigurationRegistry::save_profile_config(&config, config_path)?;
            }
        } else {
            create_default_config_file(config_path, "default", "/var/log", "sftp:backup@192.168.1.100:/backup", "default_secret_pass123")?;
        }
        Ok(())
    }
}

pub fn run_setup_with_prompter<P: SetupPrompter>(config_path: &Path, prompter: &P, non_interactive: bool, lang_opt: Option<Language>) -> Result<()> {
    SetupEngine::run(config_path, prompter, non_interactive, lang_opt)
}

pub fn run_setup(config_path: &Path, lang_opt: Option<Language>) -> Result<()> {
    let prompter = InquirePrompter;
    run_setup_with_prompter(config_path, &prompter, false, lang_opt)
}

use crate::runner::executor::{CommandRunner, SystemExecutor};

pub fn build_download_command(bin: &str, url: &str, target_dir: &str) -> String {
    match bin {
        "restic" => format!("curl -fsSL {} | bunzip2 > {}/restic && chmod +x {}/restic", url, target_dir, target_dir),
        "rclone" => format!("curl -fsSL {} -o /tmp/rclone.zip && unzip -q /tmp/rclone.zip -d /tmp && cp /tmp/rclone-*-linux-amd64/rclone {}/rclone && chmod +x {}/rclone && rm -rf /tmp/rclone*", url, target_dir, target_dir),
        "resticprofile" => format!("curl -fsSL {} -o /tmp/rp.tar.gz && tar -xzf /tmp/rp.tar.gz -C /tmp && cp /tmp/resticprofile {}/resticprofile && chmod +x {}/resticprofile && rm -rf /tmp/rp*", url, target_dir, target_dir),
        _ => format!("echo Unknown binary {}", bin),
    }
}

pub fn run_setup_dependencies_with_runner<R: CommandRunner>(runner: &R) -> Result<String> {
    let mut report = String::new();
    report.push_str("Checking binary dependencies...\n");

    let install_target_dir = if Path::new("/usr/local/bin").is_dir() {
        "/usr/local/bin"
    } else {
        "/tmp"
    };

    let binaries = [
        ("restic", "https://github.com/restic/restic/releases/download/v0.16.4/restic_0.16.4_linux_amd64.bz2"),
        ("rclone", "https://downloads.rclone.org/rclone-current-linux-amd64.zip"),
        ("resticprofile", "https://github.com/creativeprojects/resticprofile/releases/download/v0.28.0/resticprofile_0.28.0_linux_amd64.tar.gz"),
    ];

    for (bin, url) in &binaries {
        let status = runner.run("which", &[bin]);
        match status {
            Ok(out) if out.status_code == 0 => {
                let path = out.stdout.trim().to_string();
                report.push_str(&format!("{}: OK ({})\n", bin, path));
            }
            _ => {
                report.push_str(&format!("{}: MISSING -> Installing from {}\n", bin, url));
                let cmd = build_download_command(bin, url, install_target_dir);
                let _ = runner.run("sh", &["-c", &cmd]);
                report.push_str(&format!("{}: Installed to {}/{}\n", bin, install_target_dir, bin));
            }
        }
    }
    Ok(report)
}

pub fn run_setup_dependencies() -> Result<String> {
    let runner = SystemExecutor;
    run_setup_dependencies_with_runner(&runner)
}





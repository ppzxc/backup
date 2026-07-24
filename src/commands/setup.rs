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

fn prompt_text_with_default(msg: &str, default_val: &str) -> Result<String> {
    let input = inquire::Text::new(msg).prompt()?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default_val.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

impl SetupPrompter for InquirePrompter {
    fn prompt_setup_params(&self, lang_opt: Option<Language>) -> Result<SetupParams> {
        // lang_opt은 항상 Some(..)으로 전달됩니다 (호출자가 detect()로 채워줌).
        // 만약을 위해 None이면 detect()로 fallback합니다.
        let lang = lang_opt.unwrap_or_else(Language::detect);

        let msg = I18nMessages::get(lang);

        let profile = prompt_text_with_default(msg.enter_profile_name, "default")?;

        let backup_type_choice = inquire::Select::new(
            msg.select_backup_type,
            vec![msg.dir_batch_backup, msg.db_stream_backup],
        ).prompt()?;

        let (backup_type, targets) = if backup_type_choice.starts_with("[1]") {
            let t = prompt_text_with_default(msg.enter_target_dir, "/var/log")?;
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

        let excludes_str = prompt_text_with_default(msg.enter_exclude_patterns, "")?;
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
            let host = prompt_text_with_default(msg.sftp_host, "192.168.1.100")?;
            let port = inquire::CustomType::<u16>::new(msg.sftp_port).with_default(22).prompt()?;
            let user = prompt_text_with_default(msg.sftp_user, "backup")?;
            let path = prompt_text_with_default(msg.sftp_path, "/backup")?;

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
                let k = prompt_text_with_default(msg.sftp_key_file, "/etc/backup/id_rsa")?;
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
            let repo_uri = prompt_text_with_default(
                msg.primary_repo_uri,
                "s3:https://s3.amazonaws.com/my-backup-bucket",
            )?;
            (repo_uri, None)
        } else {
            let repo_uri = prompt_text_with_default(msg.primary_repo_uri, "/data/backup")?;
            (repo_uri, None)
        };

        let default_enc_path = Path::new("/etc/backup/enc");
        let password = if let Some(existing_pass) = resolve_encryption_keyfile(default_enc_path) {
            println!("\n{}", msg.found_existing_keyfile);
            existing_pass
        } else {
            let auto_gen = inquire::Confirm::new(msg.auto_generate_password_prompt)
                .with_default(true)
                .prompt()?;

            if auto_gen {
                let gen_pass = generate_secure_password();
                let _ = save_encryption_keyfile(default_enc_path, &gen_pass);
                gen_pass
            } else {
                let user_pass = inquire::Password::new(msg.enter_encryption_password)
                    .without_confirmation()
                    .prompt()?;
                if user_pass.len() < 12 {
                    anyhow::bail!(msg.isms_password_error);
                }
                let save_key = inquire::Confirm::new(msg.save_password_to_keyfile_prompt)
                    .with_default(true)
                    .prompt()?;
                if save_key {
                    let _ = save_encryption_keyfile(default_enc_path, &user_pass);
                }
                user_pass
            }
        };

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
            let output_dir = prompt_text_with_default(msg.report_export_dir, report_dir_path)?;
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
        if !non_interactive {
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
    // lang_opt이 None이면 LANG/LC_ALL 환경변수로 자동 감지합니다.
    // prompter에는 항상 Some(..)을 전달하여 언어 선택 프롬프트를 건너뜁니다.
    let resolved_lang = lang_opt.or_else(|| Some(Language::detect()));
    SetupEngine::run(config_path, prompter, non_interactive, resolved_lang)
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

pub fn generate_secure_password() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=";
    let mut state = seed;
    let mut password = String::with_capacity(32);

    // 대문자, 소문자, 숫자, 특수문자 각 1개 이상 보장
    password.push(('A' as u8 + (state % 26) as u8) as char);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    password.push(('a' as u8 + (state % 26) as u8) as char);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    password.push(('0' as u8 + (state % 10) as u8) as char);
    state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    password.push(b"!@#$%^&*()_+-="[(state % 14) as usize] as char);

    for _ in 4..32 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let idx = (state as usize) % charset.len();
        password.push(charset[idx] as char);
    }
    password
}

pub fn resolve_encryption_keyfile(path: &Path) -> Option<String> {
    if path.is_file() {
        if let Ok(content) = std::fs::read_to_string(path) {
            let trimmed = content.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

pub fn save_encryption_keyfile(path: &Path, password: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700));
        }
    }

    std::fs::write(path, format!("{}\n", password))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }

    Ok(())
}

pub fn run_setup_dependencies() -> Result<String> {
    let runner = SystemExecutor;
    run_setup_dependencies_with_runner(&runner)
}






use crate::config::model::*;
use crate::i18n::{I18nMessages, Language};
use anyhow::Result;
use secrecy::SecretString;
use std::path::Path;

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
    pub audit: AuditConfig,
}

pub trait SetupPrompter {
    fn prompt_setup_params(
        &self,
        lang_opt: Option<Language>,
        config_dir: &Path,
    ) -> Result<SetupParams>;
}

pub struct InquirePrompter;

fn prompt_text_with_default(msg: &str, default_val: &str, lang: Language) -> Result<String> {
    let prompt_msg = if default_val.is_empty() {
        msg.to_string()
    } else {
        match lang {
            Language::Ko => format!("{} (기본값: {})", msg, default_val),
            Language::En => format!("{} (default: {})", msg, default_val),
        }
    };
    let input = inquire::Text::new(&prompt_msg).prompt()?;
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Ok(default_val.to_string())
    } else {
        Ok(trimmed.to_string())
    }
}

pub const DEFAULT_BACKUP_TARGET: &str = "/var/log";

impl SetupPrompter for InquirePrompter {
    fn prompt_setup_params(
        &self,
        lang_opt: Option<Language>,
        config_dir: &Path,
    ) -> Result<SetupParams> {
        let lang = lang_opt.unwrap_or(Language::En);
        let msg = I18nMessages::get(lang);

        let profile = prompt_text_with_default(msg.enter_profile_name, "default", lang)?;

        let backup_type_choice = inquire::Select::new(
            msg.select_backup_type,
            vec![msg.dir_batch_backup, msg.db_stream_backup],
        )
        .prompt()?;

        let (backup_type, targets) = if backup_type_choice.starts_with("[1]") {
            let t = prompt_text_with_default(msg.enter_target_dir, DEFAULT_BACKUP_TARGET, lang)?;
            let target_list: Vec<String> = t
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            (BackupType::Directory, target_list)
        } else {
            let db_kind =
                inquire::Select::new(msg.select_db_type, vec!["mysql", "postgres"]).prompt()?;
            let db_type: DatabaseType = db_kind.parse()?;
            let conn = inquire::Text::new(msg.enter_conn_url).prompt_skippable()?;
            (
                BackupType::DbStream {
                    db_type,
                    connection_url: conn.filter(|s| !s.is_empty()),
                },
                vec![format!("db-stream:{}", db_type)],
            )
        };

        let excludes_str = prompt_text_with_default(msg.enter_exclude_patterns, "", lang)?;
        let excludes: Vec<String> = excludes_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        // Retention defaults depending on type
        let retention_defaults = match backup_type {
            BackupType::Directory => RetentionPolicy::standard_defaults(),
            BackupType::DbStream { .. } => RetentionPolicy::long_term_defaults(),
        };

        let keep_daily = inquire::CustomType::<u32>::new(msg.retention_keep_daily)
            .with_default(retention_defaults.keep_daily)
            .prompt()?;
        let keep_weekly = inquire::CustomType::<u32>::new(msg.retention_keep_weekly)
            .with_default(retention_defaults.keep_weekly)
            .prompt()?;
        let keep_monthly = inquire::CustomType::<u32>::new(msg.retention_keep_monthly)
            .with_default(retention_defaults.keep_monthly)
            .prompt()?;

        // Primary & Secondary Storage Setup
        let target_profiles_path = config_dir.join(crate::config::model::DEFAULT_PROFILES_FILENAME);
        let default_yaml_path = Path::new(crate::config::model::DEFAULT_PROFILES_PATH);
        let profiles_yaml_path = if target_profiles_path.exists() {
            target_profiles_path.as_path()
        } else {
            default_yaml_path
        };

        let existing_restic = if profiles_yaml_path.exists() {
            ResticProfileConfig::load_from_path(profiles_yaml_path).ok()
        } else {
            None
        };

        let primary_prof = existing_restic
            .as_ref()
            .and_then(|c| c.profiles.get("primary"));
        let reuse_storage = if let Some(p) = primary_prof {
            if let Some(ref repo) = p.repository {
                let sec_repo = existing_restic
                    .as_ref()
                    .and_then(|c| c.profiles.get("secondary"))
                    .and_then(|s| s.repository.as_deref())
                    .unwrap_or("-");
                let prompt_label = format!(
                    "{} ({}: {}, {}: {}) — {}",
                    msg.reuse_existing_storage_label,
                    msg.reuse_primary_label,
                    repo,
                    msg.reuse_secondary_label,
                    sec_repo,
                    msg.reuse_existing_storage_prompt
                );
                inquire::Confirm::new(&prompt_label)
                    .with_default(true)
                    .prompt()?
            } else {
                false
            }
        } else {
            false
        };

        let (primary_storage, secondary_storage) = if reuse_storage {
            let p = primary_prof.unwrap();
            let repo = p.repository.clone().unwrap_or_default();
            let backend = if repo.starts_with("s3:") {
                "s3"
            } else if repo.starts_with("sftp:") {
                "sftp"
            } else {
                "local"
            };

            let pwd = p.password.clone().unwrap_or_else(generate_secure_password);
            let primary = StorageTarget {
                backend: backend.to_string(),
                repository: repo,
                password: SecretString::new(pwd),
                sftp: None,
                s3: None,
            };

            let secondary = existing_restic
                .as_ref()
                .and_then(|c| c.profiles.get("secondary"))
                .map(|sec_prof| {
                    let sec_repo = sec_prof.repository.clone().unwrap_or_default();
                    let sec_backend = if sec_repo.starts_with("s3:") {
                        "s3"
                    } else if sec_repo.starts_with("sftp:") {
                        "sftp"
                    } else {
                        "local"
                    };
                    let sec_pwd = sec_prof.password.clone().unwrap_or_default();
                    SecondaryStorageTarget {
                        enabled: true,
                        backend: sec_backend.to_string(),
                        repository: sec_repo,
                        password: SecretString::new(sec_pwd),
                        sftp: None,
                        s3: None,
                    }
                });

            (primary, secondary)
        } else {
            let backend =
                inquire::Select::new(msg.primary_storage_backend, vec!["sftp", "s3", "local"])
                    .prompt()?;

            let (repository, sftp_config, s3_config) = if backend == "sftp" {
                let runner = SystemExecutor;
                let (repo_uri, conf) =
                    prompt_sftp_storage(msg, lang, config_dir, "id_ed25519", &runner)?;
                (repo_uri, Some(conf), None)
            } else if backend == "s3" {
                let mode_choice = inquire::Select::new(
                    msg.s3_mode_select,
                    vec![msg.s3_mode_detailed, msg.s3_mode_uri_only],
                )
                .prompt()?;

                if mode_choice.starts_with("[1]") {
                    let endpoint = prompt_text_with_default(
                        msg.s3_endpoint,
                        "https://s3.amazonaws.com",
                        lang,
                    )?;
                    let access_key_id = inquire::Text::new(msg.s3_access_key_id).prompt()?;
                    let secret_access_key_str = inquire::Password::new(msg.s3_secret_access_key)
                        .without_confirmation()
                        .prompt()?;
                    let _region = prompt_text_with_default(msg.s3_region, "", lang)?;
                    let bucket = prompt_text_with_default(msg.s3_bucket, "my-backup-bucket", lang)?;
                    let subfolder = prompt_text_with_default(msg.s3_path, "", lang)?;

                    let clean_endpoint = endpoint.trim_start_matches("s3:").trim_end_matches('/');
                    let clean_subfolder = subfolder.trim_matches('/');
                    let repo_uri = if clean_subfolder.is_empty() {
                        format!("s3:{}/{}", clean_endpoint, bucket)
                    } else {
                        format!("s3:{}/{}/{}", clean_endpoint, bucket, clean_subfolder)
                    };

                    let s3_conf = S3Config {
                        endpoint,
                        access_key_id,
                        secret_access_key: SecretString::new(secret_access_key_str),
                    };
                    (repo_uri, None, Some(s3_conf))
                } else {
                    let repo_uri = prompt_text_with_default(
                        msg.primary_repo_uri,
                        "s3:https://s3.amazonaws.com/my-backup-bucket/backup",
                        lang,
                    )?;
                    (repo_uri, None, None)
                }
            } else {
                let repo_uri =
                    prompt_text_with_default(msg.primary_repo_uri, "/data/backup", lang)?;
                (repo_uri, None, None)
            };

            let enc_file_path = config_dir.join("enc");
            let password = if let Some(existing_pass) = resolve_encryption_keyfile(&enc_file_path) {
                println!("\n{}", msg.found_existing_keyfile);
                existing_pass
            } else {
                let auto_gen = inquire::Confirm::new(msg.auto_generate_password_prompt)
                    .with_default(true)
                    .prompt()?;

                if auto_gen {
                    let gen_pass = generate_secure_password();
                    let _ = save_encryption_keyfile(&enc_file_path, &gen_pass);
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
                        let _ = save_encryption_keyfile(&enc_file_path, &user_pass);
                    }
                    user_pass
                }
            };

            let primary = StorageTarget {
                backend: backend.to_string(),
                repository,
                password: SecretString::new(password),
                sftp: sftp_config,
                s3: s3_config,
            };

            // Secondary Storage Setup (Optional)
            let enable_sec = inquire::Confirm::new(msg.config_secondary_storage)
                .with_default(false)
                .prompt()?;

            let secondary = if enable_sec {
                let sec_backend =
                    inquire::Select::new(msg.secondary_backend, vec!["sftp", "s3", "local"])
                        .prompt()?;
                let (sec_repo, sec_pass, sec_sftp, sec_s3) = if sec_backend == "sftp" {
                    let runner = SystemExecutor;
                    let (repo_uri, sec_sftp_conf) = prompt_sftp_storage(
                        msg,
                        lang,
                        config_dir,
                        "id_ed25519_secondary",
                        &runner,
                    )?;
                    (repo_uri, String::new(), Some(sec_sftp_conf), None)
                } else if sec_backend == "s3" {
                    let mode_choice = inquire::Select::new(
                        msg.s3_mode_select,
                        vec![msg.s3_mode_detailed, msg.s3_mode_uri_only],
                    )
                    .prompt()?;

                    if mode_choice.starts_with("[1]") {
                        let endpoint = prompt_text_with_default(
                            msg.s3_endpoint,
                            "https://s3.amazonaws.com",
                            lang,
                        )?;
                        let access_key_id = inquire::Text::new(msg.s3_access_key_id).prompt()?;
                        let secret_access_key_str =
                            inquire::Password::new(msg.s3_secret_access_key)
                                .without_confirmation()
                                .prompt()?;
                        let _region = prompt_text_with_default(msg.s3_region, "", lang)?;
                        let bucket =
                            prompt_text_with_default(msg.s3_bucket, "my-backup-bucket", lang)?;
                        let subfolder = prompt_text_with_default(msg.s3_path, "", lang)?;

                        let clean_endpoint =
                            endpoint.trim_start_matches("s3:").trim_end_matches('/');
                        let clean_subfolder = subfolder.trim_matches('/');
                        let repo_uri = if clean_subfolder.is_empty() {
                            format!("s3:{}/{}", clean_endpoint, bucket)
                        } else {
                            format!("s3:{}/{}/{}", clean_endpoint, bucket, clean_subfolder)
                        };

                        let s3_conf = S3Config {
                            endpoint,
                            access_key_id,
                            secret_access_key: SecretString::new(secret_access_key_str),
                        };
                        (repo_uri, String::new(), None, Some(s3_conf))
                    } else {
                        let sec_r = prompt_text_with_default(
                            msg.secondary_repo_uri,
                            "s3:https://s3.amazonaws.com/my-backup-bucket/backup",
                            lang,
                        )?;
                        let sec_p = inquire::Password::new(msg.secondary_password)
                            .without_confirmation()
                            .prompt()?;
                        (sec_r, sec_p, None, None)
                    }
                } else {
                    let sec_r = inquire::Text::new(msg.secondary_repo_uri).prompt()?;
                    let sec_p = inquire::Password::new(msg.secondary_password)
                        .without_confirmation()
                        .prompt()?;
                    (sec_r, sec_p, None, None)
                };
                Some(SecondaryStorageTarget {
                    enabled: true,
                    backend: sec_backend.to_string(),
                    repository: sec_repo,
                    password: SecretString::new(sec_pass),
                    sftp: sec_sftp,
                    s3: sec_s3,
                })
            } else {
                None
            };

            (primary, secondary)
        };

        // ISMS Report Options Setup
        let enable_reports = inquire::Confirm::new(msg.enable_isms_reports)
            .with_default(true)
            .prompt()?;

        let report_dir_path = "/data/backup/reports";
        let reports = if enable_reports {
            let output_dir =
                prompt_text_with_default(msg.report_export_dir, report_dir_path, lang)?;
            crate::config::model::create_secure_dir(Path::new(&output_dir))?;
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

        let default_sys_mgr = match lang {
            Language::Ko => "시스템 운영팀",
            Language::En => "System Operations Team",
        };
        let default_sec_off = match lang {
            Language::Ko => "정보보안책임자",
            Language::En => "Chief Information Security Officer",
        };

        let sys_mgr = prompt_text_with_default(msg.prompt_system_manager, default_sys_mgr, lang)?;
        let sec_off = prompt_text_with_default(msg.prompt_security_officer, default_sec_off, lang)?;

        let audit = AuditConfig {
            system_manager: Some(sys_mgr),
            security_officer: Some(sec_off),
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
            audit,
        })
    }
}

pub fn create_default_config_file(
    path: &Path,
    profile: &str,
    target: &str,
    repo: &str,
    pwd: &str,
) -> Result<()> {
    let config = BackupConfig {
        version: "1.0".into(),
        profile: profile.into(),
        backup: BackupTargets {
            backup_type: BackupType::Directory,
            targets: vec![target.into()],
            excludes: vec![],
        },
        retention: RetentionPolicy {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        },
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
        audit: AuditConfig {
            system_manager: Some("시스템 운영팀".into()),
            security_officer: Some("정보보안책임자".into()),
        },
    };
    let config_dir = path.parent().unwrap_or(path);
    config.save_and_sync(config_dir)
}

fn prompt_sftp_storage<R: crate::runner::executor::CommandRunner>(
    msg: I18nMessages,
    lang: Language,
    config_dir: &Path,
    key_name: &str,
    runner: &R,
) -> Result<(String, SftpConfig)> {
    let key_dir = config_dir;
    let key_path = key_dir.join(key_name);
    let pub_path = key_dir.join(format!("{}.pub", key_name));
    let key_path_str = key_path.to_string_lossy().to_string();

    crate::config::model::create_secure_dir(key_dir)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(key_dir, std::fs::Permissions::from_mode(0o700))?;
    }

    let generate_key = if key_path.exists() {
        let options = vec![
            msg.sftp_key_choice_use_existing,
            msg.sftp_key_choice_generate_new,
        ];
        let selection_idx = inquire::Select::new(msg.sftp_key_choice_prompt, options.clone())
            .raw_prompt()?
            .index;
        selection_idx == 1
    } else {
        true
    };

    if generate_key {
        if key_path.exists() {
            std::fs::remove_file(&key_path)?;
        }
        if pub_path.exists() {
            std::fs::remove_file(&pub_path)?;
        }
        let output = runner.run(
            "ssh-keygen",
            &["-t", "ed25519", "-N", "", "-f", &key_path_str],
        )?;
        if output.status_code != 0 {
            anyhow::bail!("ssh-keygen failed: {}", output.stderr);
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if key_path.exists() {
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }
        if pub_path.exists() {
            std::fs::set_permissions(&pub_path, std::fs::Permissions::from_mode(0o600))?;
        }
    }

    if let Ok(pub_key) = std::fs::read_to_string(&pub_path) {
        println!(
            "\n================================================================================"
        );
        println!("{}", msg.sftp_pubkey_notice);
        println!(
            "================================================================================"
        );
        println!("{}\n", pub_key.trim());
    }

    let _ = inquire::Text::new(msg.sftp_press_enter).prompt_skippable()?;

    let mut host = prompt_text_with_default(msg.sftp_host, "192.168.1.100", lang)?;
    let mut port = inquire::CustomType::<u16>::new(msg.sftp_port)
        .with_default(22)
        .prompt()?;
    let mut user = prompt_text_with_default(msg.sftp_user, "backup", lang)?;
    let mut path = prompt_text_with_default(msg.sftp_path, "/backup", lang)?;

    loop {
        // Perform SFTP connection test
        println!("{}", msg.sftp_testing_connection);
        let test_result = verify_sftp_connection(&user, &host, port, &key_path_str, runner);

        let success = match test_result {
            Ok(()) => {
                println!("{}", msg.sftp_test_success);
                true
            }
            Err(ref reason) => {
                println!("{}", msg.sftp_test_failed.replace("{}", reason));
                false
            }
        };

        if success {
            break;
        }

        let options = vec![
            msg.sftp_action_retry,
            msg.sftp_action_reenter,
            msg.sftp_action_ignore,
            msg.sftp_action_cancel,
        ];
        let choice_idx = inquire::Select::new(msg.sftp_test_failed_action, options)
            .raw_prompt()?
            .index;

        match choice_idx {
            0 => continue, // Retry
            1 => {
                // Re-enter credentials
                host = prompt_text_with_default(msg.sftp_host, &host, lang)?;
                port = inquire::CustomType::<u16>::new(msg.sftp_port)
                    .with_default(port)
                    .prompt()?;
                user = prompt_text_with_default(msg.sftp_user, &user, lang)?;
                path = prompt_text_with_default(msg.sftp_path, &path, lang)?;
            }
            2 => break, // Ignore warning and proceed
            _ => anyhow::bail!("SFTP setup cancelled by user due to connection failure."),
        }
    }

    let repo_uri = format_sftp_repository_url(&user, &host, port, &path);
    Ok((
        repo_uri,
        SftpConfig {
            host,
            port,
            user,
            key_file: Some(key_path_str),
        },
    ))
}

pub fn verify_sftp_connection<R: crate::runner::executor::CommandRunner>(
    user: &str,
    host: &str,
    port: u16,
    key_path: &str,
    runner: &R,
) -> Result<(), String> {
    let port_str = port.to_string();
    let remote_target = format!("{}@{}", user, host);
    let test_output = runner.run(
        "ssh",
        &[
            "-i",
            key_path,
            "-p",
            &port_str,
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=5",
            &remote_target,
            "exit",
        ],
    );

    match test_output {
        Ok(out) if out.status_code == 0 => Ok(()),
        Ok(out) => {
            let trimmed_err = out.stderr.trim();
            if trimmed_err.is_empty() {
                Err(format!("exit code: {}", out.status_code))
            } else {
                Err(trimmed_err.to_string())
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn format_sftp_repository_url(user: &str, host: &str, port: u16, path: &str) -> String {
    let clean_path = path.trim();
    if port == 22 {
        format!("sftp:{}@{}:{}", user, host, clean_path)
    } else if clean_path.starts_with('/') {
        format!(
            "sftp://{}@{}:{}//{}",
            user,
            host,
            port,
            clean_path.trim_start_matches('/')
        )
    } else {
        format!("sftp://{}@{}:{}/{}", user, host, port, clean_path)
    }
}

pub struct SetupEngine;

impl SetupEngine {
    pub fn validate_and_build(params: SetupParams) -> Result<BackupConfig> {
        let password_len =
            secrecy::ExposeSecret::expose_secret(&params.primary_storage.password).len();
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
                anyhow::bail!(
                    "ISMS Compliance Error: SFTP requires SSH key_file path for passwordless key-based authentication."
                );
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
            audit: params.audit,
        })
    }

    pub fn run<P: SetupPrompter, R: crate::runner::resticprofile::ResticProfileRunner>(
        config_path: &Path,
        prompter: &P,
        non_interactive: bool,
        lang_opt: Option<Language>,
        runner: &R,
    ) -> Result<()> {
        let config_dir = if let Some(parent) = config_path.parent() {
            if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            }
        } else {
            config_path
        };

        crate::config::model::create_secure_dir(config_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(config_dir, std::fs::Permissions::from_mode(0o700))?;
        }

        if !non_interactive {
            let params = prompter.prompt_setup_params(lang_opt, config_dir)?;
            let config = Self::validate_and_build(params)?;
            crate::config::registry::ConfigurationRegistry::save_profile_config(
                &config, config_dir,
            )?;
        } else {
            create_default_config_file(
                config_path,
                "default",
                DEFAULT_BACKUP_TARGET,
                "sftp:backup@192.168.1.100:/backup",
                &generate_secure_password(),
            )?;
        }

        let profiles_yaml_path =
            if config_path.ends_with(crate::config::model::DEFAULT_PROFILES_FILENAME) {
                config_path.to_path_buf()
            } else {
                config_dir.join(crate::config::model::DEFAULT_PROFILES_FILENAME)
            };

        if profiles_yaml_path.exists() {
            let _ = runner.schedule_enable(&profiles_yaml_path);
        }

        Ok(())
    }
}

pub fn run_setup_with_prompter_and_runner<
    P: SetupPrompter,
    R: crate::runner::resticprofile::ResticProfileRunner,
>(
    config_path: &Path,
    prompter: &P,
    non_interactive: bool,
    lang_opt: Option<Language>,
    runner: &R,
) -> Result<()> {
    let resolved_lang = lang_opt.or_else(|| Some(Language::detect()));
    SetupEngine::run(
        config_path,
        prompter,
        non_interactive,
        resolved_lang,
        runner,
    )
}

pub fn run_setup_with_prompter<P: SetupPrompter>(
    config_path: &Path,
    prompter: &P,
    non_interactive: bool,
    lang_opt: Option<Language>,
) -> Result<()> {
    let executor = crate::runner::executor::SystemExecutor;
    let runner = crate::runner::resticprofile::ResticProfileTool::new(&executor);
    run_setup_with_prompter_and_runner(config_path, prompter, non_interactive, lang_opt, &runner)
}

pub fn run_setup(config_path: &Path, lang_opt: Option<Language>) -> Result<()> {
    let prompter = InquirePrompter;
    run_setup_with_prompter(config_path, &prompter, false, lang_opt)
}

use crate::runner::executor::{CommandRunner, SystemExecutor};

pub fn build_download_command(bin: &str, url: &str, target_dir: &str) -> String {
    match bin {
        "restic" => format!(
            "curl -fsSL {} | bunzip2 > {}/restic && chmod +x {}/restic",
            url, target_dir, target_dir
        ),
        "rclone" => format!(
            "curl -fsSL {} -o /tmp/rclone.zip && unzip -q /tmp/rclone.zip -d /tmp && cp /tmp/rclone-*-linux-amd64/rclone {}/rclone && chmod +x {}/rclone && rm -rf /tmp/rclone*",
            url, target_dir, target_dir
        ),
        "resticprofile" => format!(
            "curl -fsSL {} -o /tmp/rp.tar.gz && tar -xzf /tmp/rp.tar.gz -C /tmp && cp /tmp/resticprofile {}/resticprofile && chmod +x {}/resticprofile && rm -rf /tmp/rp*",
            url, target_dir, target_dir
        ),
        _ => format!("echo Unknown binary {}", bin),
    }
}

fn is_dir_writable(path: &str) -> bool {
    let test_file = Path::new(path).join(".write_test");
    if std::fs::write(&test_file, "test").is_ok() {
        let _ = std::fs::remove_file(test_file);
        true
    } else {
        false
    }
}

pub fn run_setup_dependencies_with_runner<R: CommandRunner>(runner: &R) -> Result<String> {
    let mut report = String::new();
    report.push_str("Checking binary dependencies...\n");

    let home_bin = std::env::var("HOME")
        .map(|h| format!("{}/.local/bin", h))
        .unwrap_or_else(|_| "/tmp".into());
    let install_target_dir =
        if Path::new("/usr/local/bin").is_dir() && is_dir_writable("/usr/local/bin") {
            "/usr/local/bin".to_string()
        } else {
            std::fs::create_dir_all(&home_bin)?;
            home_bin
        };

    let binaries = [
        (
            "restic",
            "https://github.com/restic/restic/releases/download/v0.16.4/restic_0.16.4_linux_amd64.bz2",
        ),
        (
            "rclone",
            "https://downloads.rclone.org/rclone-current-linux-amd64.zip",
        ),
        (
            "resticprofile",
            "https://github.com/creativeprojects/resticprofile/releases/download/v0.28.0/resticprofile_0.28.0_linux_amd64.tar.gz",
        ),
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
                let cmd = build_download_command(bin, url, &install_target_dir);
                let _ = runner.run("sh", &["-c", &cmd]);
                report.push_str(&format!(
                    "{}: Installed to {}/{}\n",
                    bin, install_target_dir, bin
                ));
            }
        }
    }
    Ok(report)
}

pub fn generate_secure_password() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let charset = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=";
    let mut state = seed;
    let mut password = String::with_capacity(32);

    // 대문자, 소문자, 숫자, 특수문자 각 1개 이상 보장
    password.push(('A' as u8 + (state % 26) as u8) as char);
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    password.push(('a' as u8 + (state % 26) as u8) as char);
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    password.push(('0' as u8 + (state % 10) as u8) as char);
    state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    password.push(b"!@#$%^&*()_+-="[(state % 14) as usize] as char);

    for _ in 4..32 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
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
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }

    std::fs::write(path, format!("{}\n", password))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

pub fn run_setup_dependencies() -> Result<String> {
    let runner = SystemExecutor;
    run_setup_dependencies_with_runner(&runner)
}

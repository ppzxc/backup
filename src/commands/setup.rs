use crate::config::model::*;
use crate::i18n::{I18nMessages, Language};
use anyhow::Result;
use secrecy::{ExposeSecret, SecretString};
use std::io::Write;
use std::path::{Path, PathBuf};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupInitFailureDecision {
    Save,
    Cancel,
    InputInterrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetupCancellationKind {
    Explicit,
    Sftp,
    InitializationFailure,
    InputInterrupted,
}

#[derive(Debug)]
pub struct SetupCancellationError {
    pub kind: SetupCancellationKind,
    pub diagnostic: String,
}

impl SetupCancellationError {
    pub fn new(kind: SetupCancellationKind, diagnostic: impl Into<String>) -> Self {
        Self {
            kind,
            diagnostic: diagnostic.into(),
        }
    }
}

impl std::fmt::Display for SetupCancellationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.diagnostic.is_empty() {
            match self.kind {
                SetupCancellationKind::Sftp => "SFTP setup cancelled by user".fmt(formatter),
                SetupCancellationKind::InputInterrupted => {
                    "setup input interrupted (Ctrl-C)".fmt(formatter)
                }
                SetupCancellationKind::Explicit => "setup cancelled by user".fmt(formatter),
                SetupCancellationKind::InitializationFailure => {
                    "setup cancelled after repository initialization failure".fmt(formatter)
                }
            }
        } else {
            self.diagnostic.fmt(formatter)
        }
    }
}

impl std::error::Error for SetupCancellationError {}

/// Destination for notices that help an operator understand setup progress or choose the next
/// action. It is deliberately separate from structured system diagnostics.
pub trait SetupNoticeSink {
    fn notice(&mut self, message: &str);
}

#[derive(Default)]
pub struct SetupNoticeCollector {
    notices: Vec<String>,
}

impl SetupNoticeCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_output(self) -> String {
        self.notices.join("\n")
    }
}

impl SetupNoticeSink for SetupNoticeCollector {
    fn notice(&mut self, message: &str) {
        self.notices.push(message.to_owned());
    }
}

/// Renderer used by the interactive wizard. It writes notices through the wizard's terminal
/// output path rather than through the global structured logger.
pub struct TuiSetupNoticeRenderer<W: Write = std::io::Stdout> {
    writer: W,
}

impl<W: Write> TuiSetupNoticeRenderer<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub fn into_inner(self) -> W {
        self.writer
    }
}

impl TuiSetupNoticeRenderer<std::io::Stdout> {
    pub fn stdout() -> Self {
        Self::new(std::io::stdout())
    }
}

impl<W: Write> SetupNoticeSink for TuiSetupNoticeRenderer<W> {
    fn notice(&mut self, message: &str) {
        let _ = writeln!(self.writer, "{message}");
        let _ = self.writer.flush();
    }
}

pub struct SetupRunOptions<'a> {
    scheduler_settings: &'a crate::runner::scheduler::SchedulerSettings,
    notices: &'a mut dyn SetupNoticeSink,
}

impl<'a> SetupRunOptions<'a> {
    pub fn new(
        scheduler_settings: &'a crate::runner::scheduler::SchedulerSettings,
        notices: &'a mut dyn SetupNoticeSink,
    ) -> Self {
        Self {
            scheduler_settings,
            notices,
        }
    }
}

#[derive(Default)]
struct NoopSetupNoticeSink;

impl SetupNoticeSink for NoopSetupNoticeSink {
    fn notice(&mut self, _message: &str) {}
}

pub trait SetupPrompter {
    fn prompt_setup_params(
        &self,
        lang_opt: Option<Language>,
        config_dir: &Path,
        profiles_path: &Path,
        notices: &mut dyn SetupNoticeSink,
    ) -> Result<SetupParams>;

    fn prompt_confirm_save_on_init_failure(&self, _msg: &str) -> Result<bool> {
        Ok(false)
    }

    /// Typed replacement for the legacy boolean prompt. The default keeps existing test and
    /// third-party prompters source-compatible while production prompt implementations can
    /// distinguish save, explicit cancellation, and Ctrl-C interruption.
    fn prompt_init_failure_decision(&self, msg: &str) -> Result<SetupInitFailureDecision> {
        Ok(if self.prompt_confirm_save_on_init_failure(msg)? {
            SetupInitFailureDecision::Save
        } else {
            SetupInitFailureDecision::Cancel
        })
    }
}

pub struct InquirePrompter {
    restore_drill_overrides: crate::cli::RestoreDrillSetupOverrides,
}

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
pub const PENDING_SETUP_DIRNAME: &str = ".setup-pending";

impl InquirePrompter {
    pub fn with_restore_drill_overrides(
        restore_drill_overrides: crate::cli::RestoreDrillSetupOverrides,
    ) -> Self {
        Self {
            restore_drill_overrides,
        }
    }
}

impl Default for InquirePrompter {
    fn default() -> Self {
        Self {
            restore_drill_overrides: crate::cli::RestoreDrillSetupOverrides::default(),
        }
    }
}

impl SetupPrompter for InquirePrompter {
    fn prompt_init_failure_decision(&self, msg: &str) -> Result<SetupInitFailureDecision> {
        match inquire::Confirm::new(msg).with_default(false).prompt() {
            Ok(true) => Ok(SetupInitFailureDecision::Save),
            Ok(false) | Err(inquire::error::InquireError::OperationCanceled) => {
                Ok(SetupInitFailureDecision::Cancel)
            }
            Err(inquire::error::InquireError::OperationInterrupted) => {
                Ok(SetupInitFailureDecision::InputInterrupted)
            }
            Err(error) => Err(error.into()),
        }
    }

    fn prompt_setup_params(
        &self,
        lang_opt: Option<Language>,
        config_dir: &Path,
        profiles_path: &Path,
        notices: &mut dyn SetupNoticeSink,
    ) -> Result<SetupParams> {
        let result = (|| -> Result<SetupParams> {
            let lang = lang_opt.unwrap_or(Language::En);
            let msg = I18nMessages::get(lang);

            let profile = prompt_text_with_default(msg.enter_profile_name, "default", lang)?;

            let backup_type_choice = inquire::Select::new(
                msg.select_backup_type,
                vec![msg.dir_batch_backup, msg.db_stream_backup],
            )
            .prompt()?;

            let (backup_type, targets) = if backup_type_choice.starts_with("[1]") {
                let t =
                    prompt_text_with_default(msg.enter_target_dir, DEFAULT_BACKUP_TARGET, lang)?;
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
            let existing_restic = if profiles_path.exists() {
                ResticProfileConfig::load_from_path(profiles_path).ok()
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
                let sftp = if backend == "sftp" {
                    Some(resolve_reused_sftp_config(
                        &repo,
                        p.option.as_ref(),
                        config_dir,
                    )?)
                } else {
                    None
                };
                let primary = StorageTarget {
                    backend: backend.to_string(),
                    repository: repo,
                    password: SecretString::new(pwd),
                    sftp,
                    s3: None,
                };

                let secondary = existing_restic
                    .as_ref()
                    .and_then(|c| c.profiles.get("secondary"))
                    .map(|sec_prof| -> Result<SecondaryStorageTarget> {
                        let sec_repo = sec_prof.repository.clone().unwrap_or_default();
                        let sec_backend = if sec_repo.starts_with("s3:") {
                            "s3"
                        } else if sec_repo.starts_with("sftp:") {
                            "sftp"
                        } else {
                            "local"
                        };
                        let sec_pwd = sec_prof.password.clone().unwrap_or_default();
                        let sftp = if sec_backend == "sftp" {
                            Some(resolve_reused_sftp_config(
                                &sec_repo,
                                sec_prof.option.as_ref(),
                                config_dir,
                            )?)
                        } else {
                            None
                        };
                        Ok(SecondaryStorageTarget {
                            enabled: true,
                            backend: sec_backend.to_string(),
                            repository: sec_repo,
                            password: SecretString::new(sec_pwd),
                            password_source: SecondaryPasswordSource::ReusePrimary,
                            sftp,
                            s3: None,
                        })
                    })
                    .transpose()?;

                (primary, secondary)
            } else {
                let backend =
                    inquire::Select::new(msg.primary_storage_backend, vec!["sftp", "s3", "local"])
                        .prompt()?;

                let (repository, sftp_config, s3_config) = if backend == "sftp" {
                    let runner = SystemExecutor;
                    let (repo_uri, conf) =
                        prompt_sftp_storage(msg, lang, config_dir, "id_ed25519", &runner, notices)?;
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
                            access_key_id: SecretString::new(access_key_id),
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
                let password = if let Some(existing_pass) =
                    resolve_encryption_keyfile(&enc_file_path)
                {
                    notices.notice(msg.found_existing_keyfile);
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
                    let (sec_repo, sec_pass, sec_password_source, sec_sftp, sec_s3) = if sec_backend
                        == "sftp"
                    {
                        let runner = SystemExecutor;
                        let (repo_uri, sec_sftp_conf) = prompt_sftp_storage(
                            msg,
                            lang,
                            config_dir,
                            "id_ed25519_secondary",
                            &runner,
                            notices,
                        )?;
                        let key_choice = inquire::Select::new(
                            match lang {
                                Language::Ko => "2차 restic 저장소 키를 선택하세요:",
                                Language::En => "Choose the secondary restic repository key:",
                            },
                            match lang {
                                Language::Ko => {
                                    vec!["1차 저장소 키 재사용", "기존 2차 저장소 키 입력"]
                                }
                                Language::En => vec![
                                    "Reuse primary repository key",
                                    "Enter existing secondary repository key",
                                ],
                            },
                        )
                        .raw_prompt()?
                        .index;
                        let (password, password_source) = if key_choice == 0 {
                            (
                                primary.password.clone(),
                                SecondaryPasswordSource::ReusePrimary,
                            )
                        } else {
                            (
                                SecretString::new(
                                    inquire::Password::new(msg.secondary_password)
                                        .without_confirmation()
                                        .prompt()?,
                                ),
                                SecondaryPasswordSource::Explicit,
                            )
                        };
                        (
                            repo_uri,
                            password,
                            password_source,
                            Some(sec_sftp_conf),
                            None,
                        )
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
                            let access_key_id =
                                inquire::Text::new(msg.s3_access_key_id).prompt()?;
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
                                access_key_id: SecretString::new(access_key_id),
                                secret_access_key: SecretString::new(secret_access_key_str),
                            };
                            (
                                repo_uri,
                                primary.password.clone(),
                                SecondaryPasswordSource::ReusePrimary,
                                None,
                                Some(s3_conf),
                            )
                        } else {
                            let sec_r = prompt_text_with_default(
                                msg.secondary_repo_uri,
                                "s3:https://s3.amazonaws.com/my-backup-bucket/backup",
                                lang,
                            )?;
                            let sec_p = inquire::Password::new(msg.secondary_password)
                                .without_confirmation()
                                .prompt()?;
                            (
                                sec_r,
                                SecretString::new(sec_p),
                                SecondaryPasswordSource::Explicit,
                                None,
                                None,
                            )
                        }
                    } else {
                        let sec_r = inquire::Text::new(msg.secondary_repo_uri).prompt()?;
                        let sec_p = inquire::Password::new(msg.secondary_password)
                            .without_confirmation()
                            .prompt()?;
                        (
                            sec_r,
                            SecretString::new(sec_p),
                            SecondaryPasswordSource::Explicit,
                            None,
                            None,
                        )
                    };
                    Some(SecondaryStorageTarget {
                        enabled: true,
                        backend: sec_backend.to_string(),
                        repository: sec_repo,
                        password: sec_pass,
                        password_source: sec_password_source,
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

            let sys_mgr =
                prompt_text_with_default(msg.prompt_system_manager, default_sys_mgr, lang)?;
            let sec_off =
                prompt_text_with_default(msg.prompt_security_officer, default_sec_off, lang)?;

            let audit = AuditConfig {
                system_manager: Some(sys_mgr),
                security_officer: Some(sec_off),
                restore_drill_rto_minutes: self.restore_drill_overrides.rto_minutes,
                restore_drill_timeout_minutes: self.restore_drill_overrides.timeout_minutes,
                restore_drill_work_dir: self
                    .restore_drill_overrides
                    .work_dir
                    .as_deref()
                    .map(|path| path.to_string_lossy().into_owned()),
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
        })();
        let result = result.map_err(classify_prompt_error);
        if result.as_ref().is_err_and(|error| {
            error
                .downcast_ref::<SetupCancellationError>()
                .is_some_and(|error| error.kind == SetupCancellationKind::Explicit)
        }) {
            notices.notice(I18nMessages::get(lang_opt.unwrap_or(Language::En)).setup_cancelled);
        }
        result
    }
}

fn classify_prompt_error(error: anyhow::Error) -> anyhow::Error {
    let Some(prompt_error) = error.downcast_ref::<inquire::error::InquireError>() else {
        return error;
    };
    let kind = match prompt_error {
        inquire::error::InquireError::OperationCanceled => SetupCancellationKind::Explicit,
        inquire::error::InquireError::OperationInterrupted => {
            SetupCancellationKind::InputInterrupted
        }
        _ => return error,
    };
    SetupCancellationError::new(kind, prompt_error.to_string()).into()
}

pub fn create_default_profiles_file(
    profiles_path: &Path,
    profile: &str,
    target: &str,
    repo: &str,
    pwd: &str,
) -> Result<()> {
    let mut config = default_application_config(profile, target, repo, pwd);
    if let Some(sftp) = config.storage.primary.sftp.as_mut() {
        let config_dir = profiles_path.parent().unwrap_or_else(|| Path::new("."));
        sftp.key_file = Some(config_dir.join("id_ed25519").to_string_lossy().into_owned());
    }
    config.save_to_profiles_path(profiles_path)
}

fn default_application_config(profile: &str, target: &str, repo: &str, pwd: &str) -> BackupConfig {
    BackupConfig {
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
                    additional_args: Vec::new(),
                }),
                s3: None,
            },
            secondary: None,
        },
        reports: ReportsConfig::default(),
        audit: AuditConfig {
            system_manager: Some("시스템 운영팀".into()),
            security_officer: Some("정보보안책임자".into()),
            restore_drill_rto_minutes: None,
            restore_drill_timeout_minutes: None,
            restore_drill_work_dir: None,
        },
    }
}

fn prompt_sftp_storage<R: crate::runner::executor::CommandRunner>(
    msg: I18nMessages,
    lang: Language,
    config_dir: &Path,
    key_name: &str,
    runner: &R,
    notices: &mut dyn SetupNoticeSink,
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
        generate_sftp_keypair(&key_path, &pub_path, &key_path_str, runner)?;
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

    crate::config::model::ensure_sftp_known_hosts_file(config_dir)?;

    if let Ok(pub_key) = std::fs::read_to_string(&pub_path) {
        notices.notice(&format!(
            "{}\n{}\n{}\n{}",
            "================================================================================",
            msg.sftp_pubkey_notice,
            "================================================================================",
            pub_key.trim()
        ));
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
        notices.notice(msg.sftp_testing_connection);
        let test_result = verify_sftp_connection_with_config_dir(
            &user,
            &host,
            port,
            &key_path_str,
            config_dir,
            runner,
        );

        let success = match test_result {
            Ok(()) => {
                notices.notice(msg.sftp_test_success);
                true
            }
            Err(ref reason) => {
                let notice = msg.sftp_test_failed.replace("{}", reason);
                notices.notice(&notice);
                false
            }
        };

        if success {
            break;
        }

        let options = vec![
            msg.sftp_action_retry,
            msg.sftp_action_reenter,
            msg.sftp_action_change_key,
            msg.sftp_action_ignore,
            msg.sftp_action_cancel,
        ];
        let choice_idx =
            match inquire::Select::new(msg.sftp_test_failed_action, options).raw_prompt() {
                Ok(choice) => choice.index,
                Err(inquire::error::InquireError::OperationCanceled) => {
                    notices.notice(msg.setup_cancelled);
                    return Err(SetupCancellationError::new(
                        SetupCancellationKind::Sftp,
                        "SFTP setup cancelled by user",
                    )
                    .into());
                }
                Err(error) => return Err(anyhow::Error::new(error)),
            };

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
            2 => {
                let key_options = vec![
                    msg.sftp_key_choice_use_existing,
                    msg.sftp_key_choice_generate_new,
                ];
                let selection_idx = inquire::Select::new(msg.sftp_key_choice_prompt, key_options)
                    .raw_prompt()?
                    .index;
                if selection_idx == 1 {
                    generate_sftp_keypair(&key_path, &pub_path, &key_path_str, runner)?;
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if key_path.exists() {
                        std::fs::set_permissions(
                            &key_path,
                            std::fs::Permissions::from_mode(0o600),
                        )?;
                    }
                    if pub_path.exists() {
                        std::fs::set_permissions(
                            &pub_path,
                            std::fs::Permissions::from_mode(0o600),
                        )?;
                    }
                }
                if let Ok(pub_key) = std::fs::read_to_string(&pub_path) {
                    notices.notice(&format!(
                        "{}\n{}\n{}\n{}",
                        "================================================================================",
                        msg.sftp_pubkey_notice,
                        "================================================================================",
                        pub_key.trim()
                    ));
                }
                let _ = inquire::Text::new(msg.sftp_press_enter).prompt_skippable()?;
            }
            3 => break, // Ignore warning and proceed
            _ => {
                notices.notice(msg.setup_cancelled);
                return Err(SetupCancellationError::new(
                    SetupCancellationKind::Sftp,
                    "SFTP setup cancelled by user due to connection failure",
                )
                .into());
            }
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
            additional_args: Vec::new(),
        },
    ))
}

fn generate_sftp_keypair<R: crate::runner::executor::CommandRunner>(
    key_path: &Path,
    pub_path: &Path,
    key_path_str: &str,
    runner: &R,
) -> Result<()> {
    if key_path.exists() {
        std::fs::remove_file(key_path)?;
    }
    if pub_path.exists() {
        std::fs::remove_file(pub_path)?;
    }
    let output = runner.run(
        "ssh-keygen",
        &["-t", "ed25519", "-N", "", "-f", key_path_str],
    )?;
    if output.status_code != 0 {
        anyhow::bail!("ssh-keygen failed: {}", output.stderr);
    }
    Ok(())
}

/// Reconstructs the wizard-owned SFTP authentication configuration when an
/// operator elects to reuse an existing Backend Profile. Native `sftp.args`
/// is preferred; the old full command is accepted only in the exact shape
/// emitted by the former wizard.
pub fn resolve_reused_sftp_config(
    repository: &str,
    options: Option<&std::collections::BTreeMap<String, String>>,
    config_dir: &Path,
) -> Result<SftpConfig> {
    let (user, host, port) = parse_sftp_repository(repository)?;
    let options = options.ok_or_else(|| {
        anyhow::anyhow!(
            "existing SFTP authentication is unavailable; explicit SFTP reconfiguration is required"
        )
    })?;

    if let Some(args) = options.get("sftp.args") {
        if options.contains_key("sftp.command") {
            anyhow::bail!(
                "existing SFTP configuration contains both sftp.args and sftp.command; explicit SFTP reconfiguration is required"
            );
        }
        let tokens = tokenize_sftp_arguments(args)?;
        let key_file = identity_from_sftp_tokens(&tokens).ok_or_else(|| {
            anyhow::anyhow!(
                "existing sftp.args has no managed identity; explicit SFTP reconfiguration is required"
            )
        })?;
        let additional_args = validate_reused_sftp_args(&tokens, Path::new(key_file), config_dir)?;
        return Ok(SftpConfig {
            host,
            port,
            user,
            key_file: Some(key_file.to_string()),
            additional_args,
        });
    }

    let command = options.get("sftp.command").ok_or_else(|| {
        anyhow::anyhow!(
            "existing SFTP authentication is unavailable; explicit SFTP reconfiguration is required"
        )
    })?;
    let tokens = tokenize_sftp_arguments(command)?;
    if tokens.len() != 10
        || tokens[0] != "ssh"
        || tokens[1] != "-o"
        || tokens[2] != "StrictHostKeyChecking=no"
        || tokens[3] != "-i"
        || tokens[5] != "-p"
        || tokens[8] != "-s"
        || tokens[9] != "sftp"
    {
        anyhow::bail!(
            "existing sftp.command is nonstandard; explicit SFTP reconfiguration is required"
        );
    }
    let key_file = Path::new(&tokens[4]);
    let command_port = tokens[6].parse::<u16>().map_err(|_| {
        anyhow::anyhow!(
            "existing sftp.command has an invalid port; explicit SFTP reconfiguration is required"
        )
    })?;
    if command_port != port || tokens[7] != format!("{user}@{host}") {
        anyhow::bail!(
            "existing sftp.command does not match its repository URI; explicit SFTP reconfiguration is required"
        );
    }
    if !is_managed_sftp_key_path(key_file, config_dir) {
        anyhow::bail!(
            "existing SFTP identity is outside the managed configuration directory; explicit SFTP reconfiguration is required"
        );
    }
    if !is_regular_sftp_key_file(&resolve_sftp_key_path(key_file, config_dir)) {
        anyhow::bail!(
            "existing managed SFTP identity is missing; explicit SFTP reconfiguration is required"
        );
    }

    Ok(SftpConfig {
        host,
        port,
        user,
        key_file: Some(tokens[4].clone()),
        additional_args: Vec::new(),
    })
}

fn is_regular_sftp_key_file(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_file())
        .unwrap_or(false)
}

fn resolve_sftp_key_path(path: &Path, config_dir: &Path) -> std::path::PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    }
}

fn validate_reused_sftp_args(
    tokens: &[String],
    key_file: &Path,
    config_dir: &Path,
) -> Result<Vec<String>> {
    if !is_managed_sftp_key_path(key_file, config_dir) {
        anyhow::bail!(
            "existing SFTP identity is outside the managed configuration directory; explicit SFTP reconfiguration is required"
        );
    }
    if !is_regular_sftp_key_file(&resolve_sftp_key_path(key_file, config_dir)) {
        anyhow::bail!(
            "existing managed SFTP identity is missing; explicit SFTP reconfiguration is required"
        );
    }
    let policy = SftpAuthPolicy::for_config_dir(key_file, config_dir)?;
    let known_hosts = policy
        .known_hosts_file()
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("SFTP known_hosts path must be valid UTF-8"))?;
    let identity_indices = tokens
        .windows(2)
        .enumerate()
        .filter_map(|(index, window)| (window[0] == "-i").then_some(index))
        .collect::<Vec<_>>();
    if identity_indices.len() != 1
        || tokens.get(identity_indices[0] + 1).map(String::as_str)
            != Some(key_file.to_string_lossy().as_ref())
    {
        anyhow::bail!(
            "existing sftp.args must contain exactly one -i option for the managed identity; explicit SFTP reconfiguration is required"
        );
    }

    let options = parse_sftp_option_values(tokens)?;
    let required = [
        ("IdentitiesOnly", "yes"),
        ("BatchMode", "yes"),
        ("StrictHostKeyChecking", "accept-new"),
        ("UserKnownHostsFile", known_hosts),
    ];
    for (key, expected_value) in required {
        let matches = options
            .iter()
            .filter(|(option_key, _)| sftp_option_key_is(option_key, key))
            .collect::<Vec<_>>();
        if matches.len() != 1 || matches[0].1 != expected_value {
            anyhow::bail!(
                "existing sftp.args does not enforce one unambiguous managed key-only policy; explicit SFTP reconfiguration is required"
            );
        }
    }

    let mut additional_args = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        if token == "-i" {
            index += 2;
            continue;
        }
        if token == "-o" {
            let value = tokens.get(index + 1).ok_or_else(|| {
                anyhow::anyhow!(
                    "existing sftp.args has an incomplete -o option; explicit SFTP reconfiguration is required"
                )
            })?;
            let (key, _) = split_sftp_option(value)?;
            if sftp_option_key_is(key, "IdentityFile") {
                anyhow::bail!(
                    "existing sftp.args contains an additional IdentityFile; explicit SFTP reconfiguration is required"
                );
            }
            if required
                .iter()
                .any(|(required_key, _)| sftp_option_key_is(required_key, key))
            {
                index += 2;
                continue;
            }
            validate_safe_sftp_option(key)?;
            additional_args.extend([token.clone(), value.clone()]);
            index += 2;
            continue;
        }
        if let Some(value) = token.strip_prefix("-o") {
            let (key, _) = split_sftp_option(value)?;
            if sftp_option_key_is(key, "IdentityFile") {
                anyhow::bail!(
                    "existing sftp.args contains an additional IdentityFile; explicit SFTP reconfiguration is required"
                );
            }
            if required
                .iter()
                .any(|(required_key, _)| sftp_option_key_is(required_key, key))
            {
                index += 1;
                continue;
            }
            validate_safe_sftp_option(key)?;
            additional_args.push(token.clone());
            index += 1;
            continue;
        }
        if token.starts_with('-') {
            if [
                "-i", "-p", "-l", "-F", "-S", "-A", "-a", "-J", "-D", "-P", "-W", "-w", "-R", "-L",
                "-O",
            ]
            .iter()
            .any(|prefix| token == prefix || token.starts_with(prefix))
                || token == "--"
            {
                anyhow::bail!(
                    "existing sftp.args contains a host or authentication override; explicit SFTP reconfiguration is required"
                );
            }
            if !matches!(token.as_str(), "-4" | "-6" | "-C" | "-q" | "-T") {
                anyhow::bail!(
                    "existing sftp.args contains an unsupported SSH flag; explicit SFTP reconfiguration is required"
                );
            }
            additional_args.push(token.clone());
            index += 1;
            continue;
        }
        anyhow::bail!(
            "existing sftp.args contains a positional argument; explicit SFTP reconfiguration is required"
        );
    }
    Ok(additional_args)
}

fn identity_from_sftp_tokens(tokens: &[String]) -> Option<&str> {
    tokens
        .windows(2)
        .find_map(|window| (window[0] == "-i").then_some(window[1].as_str()))
        .or_else(|| {
            tokens.iter().find_map(|token| {
                token
                    .strip_prefix("IdentityFile=")
                    .or_else(|| token.strip_prefix("-oIdentityFile="))
            })
        })
}

fn split_sftp_option(value: &str) -> Result<(&str, &str)> {
    value.split_once('=').ok_or_else(|| {
        anyhow::anyhow!(
            "existing sftp.args contains an invalid -o option; explicit SFTP reconfiguration is required"
        )
    })
}

fn parse_sftp_option_values(tokens: &[String]) -> Result<Vec<(&str, &str)>> {
    let mut values = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index] == "-o" {
            let value = tokens.get(index + 1).ok_or_else(|| {
                anyhow::anyhow!(
                    "existing sftp.args has an incomplete -o option; explicit SFTP reconfiguration is required"
                )
            })?;
            values.push(split_sftp_option(value)?);
            index += 2;
        } else if let Some(value) = tokens[index].strip_prefix("-o") {
            values.push(split_sftp_option(value)?);
            index += 1;
        } else {
            index += 1;
        }
    }
    Ok(values)
}

fn validate_safe_sftp_option(key: &str) -> Result<()> {
    if ![
        "ConnectTimeout",
        "ConnectionAttempts",
        "ServerAliveInterval",
        "ServerAliveCountMax",
        "TCPKeepAlive",
        "Compression",
        "CompressionLevel",
        "IPQoS",
        "AddressFamily",
        "KexAlgorithms",
        "HostKeyAlgorithms",
        "Ciphers",
        "MACs",
        "RekeyLimit",
    ]
    .iter()
    .any(|safe_key| sftp_option_key_is(key, safe_key))
    {
        anyhow::bail!(
            "existing sftp.args contains an unsupported SSH option ({key}); explicit SFTP reconfiguration is required"
        );
    }
    Ok(())
}

fn sftp_option_key_is(actual: &str, expected: &str) -> bool {
    actual.eq_ignore_ascii_case(expected)
}

fn parse_sftp_repository(repository: &str) -> Result<(String, String, u16)> {
    if !repository.starts_with("sftp:") {
        anyhow::bail!("not an SFTP repository URI: {repository}");
    }

    if repository.starts_with("sftp://") {
        let parsed = url::Url::parse(repository)?;
        let user = parsed.username().to_string();
        let host = parsed
            .host_str()
            .ok_or_else(|| anyhow::anyhow!("SFTP repository URI has no host"))?
            .to_string();
        let port = parsed.port().unwrap_or(22);
        if user.is_empty() {
            anyhow::bail!("SFTP repository URI has no user");
        }
        return Ok((user, host, port));
    }

    let authority = repository["sftp:".len()..]
        .split_once('/')
        .map(|(authority, _)| authority)
        .unwrap_or(&repository["sftp:".len()..]);
    let (user, host_port) = authority
        .split_once('@')
        .ok_or_else(|| anyhow::anyhow!("SFTP repository URI has no user"))?;
    let (host, port) = if host_port.starts_with('[') {
        let end = host_port
            .find(']')
            .ok_or_else(|| anyhow::anyhow!("invalid bracketed SFTP host"))?;
        let host = &host_port[1..end];
        let port = host_port
            .get(end + 1..)
            .and_then(|value| value.strip_prefix(':'))
            .map(str::parse)
            .transpose()?
            .unwrap_or(22);
        (host.to_string(), port)
    } else if let Some((host, port)) = host_port.rsplit_once(':') {
        if port.is_empty() {
            (host.to_string(), 22)
        } else if let Ok(port) = port.parse::<u16>() {
            (host.to_string(), port)
        } else {
            (host_port.to_string(), 22)
        }
    } else {
        (host_port.to_string(), 22)
    };
    if host.is_empty() {
        anyhow::bail!("SFTP repository URI has no host");
    }
    Ok((user.to_string(), host, port))
}

pub fn verify_sftp_connection<R: crate::runner::executor::CommandRunner>(
    user: &str,
    host: &str,
    port: u16,
    key_path: &str,
    runner: &R,
) -> Result<(), String> {
    let config_dir = Path::new(key_path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    verify_sftp_connection_with_config_dir(user, host, port, key_path, config_dir, runner)
}

pub fn verify_sftp_connection_with_config_dir<R: crate::runner::executor::CommandRunner>(
    user: &str,
    host: &str,
    port: u16,
    key_path: &str,
    config_dir: &Path,
    runner: &R,
) -> Result<(), String> {
    let port_str = port.to_string();
    let remote_target = format!("{}@{}", user, host);
    let policy = SftpAuthPolicy::for_config_dir(Path::new(key_path), config_dir)
        .map_err(|error| error.to_string())?;
    let auth_args = policy
        .argument_tokens()
        .map_err(|error| error.to_string())?;
    let mut args = auth_args;
    args.extend([
        "-P".into(),
        port_str,
        "-o".into(),
        "ConnectTimeout=5".into(),
        "-b".into(),
        "/dev/null".into(),
        remote_target,
    ]);
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let test_output = runner.run("sftp", &arg_refs);

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

        if let Some(secondary) = &params.secondary_storage {
            if secondary.enabled && secondary.password.expose_secret().len() < 12 {
                anyhow::bail!(
                    "ISMS Compliance Error: Secondary password must be at least 12 characters long."
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

    pub fn run<
        P: SetupPrompter,
        R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
        S: crate::runner::scheduler::BackupScheduler + ?Sized,
    >(
        profiles_path: &Path,
        prompter: &P,
        non_interactive: bool,
        lang_opt: Option<Language>,
        runner: &R,
        scheduler: &S,
    ) -> Result<()> {
        let mut notices = NoopSetupNoticeSink;
        Self::run_with_options(
            profiles_path,
            prompter,
            non_interactive,
            lang_opt,
            runner,
            scheduler,
            SetupRunOptions::new(
                &crate::runner::scheduler::SchedulerSettings::auto(),
                &mut notices,
            ),
        )
    }

    pub fn run_with_scheduler_settings<
        P: SetupPrompter,
        R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
        S: crate::runner::scheduler::BackupScheduler + ?Sized,
    >(
        profiles_path: &Path,
        prompter: &P,
        non_interactive: bool,
        lang_opt: Option<Language>,
        runner: &R,
        scheduler: &S,
        scheduler_settings: &crate::runner::scheduler::SchedulerSettings,
    ) -> Result<()> {
        let mut notices = NoopSetupNoticeSink;
        Self::run_with_options(
            profiles_path,
            prompter,
            non_interactive,
            lang_opt,
            runner,
            scheduler,
            SetupRunOptions::new(scheduler_settings, &mut notices),
        )
    }

    pub fn run_with_options<
        P: SetupPrompter,
        R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
        S: crate::runner::scheduler::BackupScheduler + ?Sized,
    >(
        profiles_path: &Path,
        prompter: &P,
        non_interactive: bool,
        lang_opt: Option<Language>,
        runner: &R,
        scheduler: &S,
        options: SetupRunOptions<'_>,
    ) -> Result<()> {
        let SetupRunOptions {
            scheduler_settings,
            notices,
        } = options;
        let config_dir = if let Some(parent) = profiles_path.parent() {
            if parent.as_os_str().is_empty() {
                Path::new(".")
            } else {
                parent
            }
        } else {
            profiles_path
        };

        let language = lang_opt.unwrap_or(Language::En);
        if non_interactive {
            return Self::run_existing_profiles_setup(
                profiles_path,
                runner,
                scheduler,
                scheduler_settings,
                language,
                notices,
            );
        }

        // Capture the live state before prompting. Key generation, encryption
        // sidecars, and host-key trust may all happen during prompting.
        let mut previous = LiveConfigSnapshot::capture(profiles_path)?;
        crate::config::model::create_secure_dir(config_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(config_dir, std::fs::Permissions::from_mode(0o700))?;
        }
        let params =
            match prompter.prompt_setup_params(lang_opt, config_dir, profiles_path, notices) {
                Ok(params) => params,
                Err(error) => {
                    previous.restore()?;
                    return Err(error);
                }
            };
        let config = match Self::validate_and_build(params) {
            Ok(config) => config,
            Err(error) => {
                previous.restore()?;
                return Err(error);
            }
        };

        previous.track_directory(Path::new(&config.reports.output_dir))?;
        if let Err(error) = std::fs::create_dir_all(&config.reports.output_dir) {
            previous.restore()?;
            return Err(error.into());
        }
        let staged_dir = tempfile::Builder::new()
            .prefix(".setup-")
            .tempdir_in(config_dir)
            .map_err(anyhow::Error::from)?;
        let staged_profiles = staged_dir
            .path()
            .join(crate::config::model::DEFAULT_PROFILES_FILENAME);
        if let Err(error) =
            config.save_to_profiles_path_with_config_dir(&staged_profiles, config_dir)
        {
            drop(staged_dir);
            previous.restore()?;
            return Err(error);
        }
        let staged =
            match crate::config::model::ResticProfileConfig::load_from_path(&staged_profiles) {
                Ok(staged) => staged,
                Err(error) => {
                    drop(staged_dir);
                    previous.restore()?;
                    return Err(error);
                }
            };
        let msg = crate::i18n::I18nMessages::get(language);

        notices.notice(msg.initializing_backend_repo);

        let init_result = initialize_backend_targets(
            &staged_profiles,
            &staged.backend_initialization_targets()?,
            runner,
            true,
        );

        let mut initialization_failed = false;
        if let Err(error) = init_result {
            initialization_failed = true;
            let mut err_msg = error.to_string();
            let mut secrets = vec![config.storage.primary.password.expose_secret()];
            if let Some(s3) = &config.storage.primary.s3 {
                secrets.push(s3.access_key_id.expose_secret());
                secrets.push(s3.secret_access_key.expose_secret());
            }
            if let Some(secondary) = &config.storage.secondary {
                secrets.push(secondary.password.expose_secret());
                if let Some(s3) = &secondary.s3 {
                    secrets.push(s3.access_key_id.expose_secret());
                    secrets.push(s3.secret_access_key.expose_secret());
                }
            }
            for secret in secrets {
                let trimmed = secret.trim();
                if !trimmed.is_empty() {
                    err_msg = err_msg.replace(trimmed, "******");
                }
            }
            err_msg = append_sftp_diagnostics(err_msg, &staged, profiles_path);
            tracing::error!("{}", err_msg);

            // `init` is idempotent for an existing repository and does not
            // prove that the selected key can decrypt it. Do not offer to
            // retain a setup that is known to use the wrong credential.
            if is_repository_credential_failure(&err_msg) {
                drop(staged_dir);
                previous.restore()?;
                return Err(SetupCancellationError::new(
                    SetupCancellationKind::InitializationFailure,
                    err_msg,
                )
                .into());
            }

            if !non_interactive {
                match prompter.prompt_init_failure_decision(msg.backend_init_failed_save_prompt)? {
                    SetupInitFailureDecision::Save => {}
                    SetupInitFailureDecision::Cancel => {
                        drop(staged_dir);
                        previous.restore()?;
                        return Err(SetupCancellationError::new(
                            SetupCancellationKind::InitializationFailure,
                            err_msg,
                        )
                        .into());
                    }
                    SetupInitFailureDecision::InputInterrupted => {
                        drop(staged_dir);
                        previous.restore()?;
                        return Err(SetupCancellationError::new(
                            SetupCancellationKind::InputInterrupted,
                            "setup input interrupted (Ctrl-C)",
                        )
                        .into());
                    }
                }
            } else {
                let prefix = match language {
                    Language::Ko => "비대화형 설정의 저장소 초기화에 실패했습니다",
                    Language::En => "Non-interactive setup failed repository initialization",
                };
                return Err(anyhow::anyhow!("{prefix}: {err_msg}"));
            }
        }

        if initialization_failed {
            if let Err(error) = save_pending_setup(&config, profiles_path) {
                drop(staged_dir);
                previous.restore()?;
                discard_pending_setup(profiles_path)?;
                return Err(error);
            }
            // Keep retryable configuration and authentication artifacts, but
            // never activate a scheduler that can reach an uninitialized
            // Backend Adapter. The live configuration remains untouched.
            return Ok(());
        }

        if let Err(error) =
            crate::config::registry::ConfigurationRegistry::save_profile_config_to_path(
                &config,
                profiles_path,
            )
        {
            drop(staged_dir);
            previous.restore()?;
            return Err(error);
        }
        discard_pending_setup(profiles_path)?;
        if let Err(error) = scheduler.enable_preserving_state(profiles_path, scheduler_settings) {
            drop(staged_dir);
            previous.restore()?;
            return Err(error);
        }

        Ok(())
    }

    fn run_existing_profiles_setup<
        R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
        S: crate::runner::scheduler::BackupScheduler + ?Sized,
    >(
        profiles_path: &Path,
        runner: &R,
        scheduler: &S,
        scheduler_settings: &crate::runner::scheduler::SchedulerSettings,
        language: Language,
        notices: &mut dyn SetupNoticeSink,
    ) -> Result<()> {
        if !profiles_path.is_file() {
            let message = match language {
                Language::Ko => {
                    "비대화형 설정에는 실제 대상·저장소·자격 증명이 포함된 기존 profiles.yaml이 필요합니다"
                }
                Language::En => {
                    "Non-interactive setup requires an existing unified profiles.yaml with real target, repository, and credentials"
                }
            };
            anyhow::bail!(message);
        }
        let profiles = crate::config::model::ResticProfileConfig::load_from_path(profiles_path)?;
        let profile_names = profiles.profile_names();
        if profile_names.is_empty() {
            let message = match language {
                Language::Ko => "비대화형 설정에는 하나 이상의 Backup Profile이 필요합니다",
                Language::En => "Non-interactive setup requires at least one Backup Profile",
            };
            anyhow::bail!(message);
        }

        let targets = profiles.backend_initialization_targets()?;
        let msg = crate::i18n::I18nMessages::get(language);
        notices.notice(msg.initializing_backend_repo);
        let init_result = initialize_backend_targets(profiles_path, &targets, runner, false);
        if let Err(error) = init_result {
            let message = append_sftp_diagnostics(
                redact_existing_profile_error(
                    error.to_string(),
                    &profiles,
                    profiles_path.parent().unwrap_or_else(|| Path::new(".")),
                ),
                &profiles,
                profiles_path,
            );
            tracing::error!("{}", message);
            let prefix = match language {
                Language::Ko => "비대화형 설정의 저장소 초기화에 실패했습니다",
                Language::En => "Non-interactive setup failed repository initialization",
            };
            anyhow::bail!("{prefix}: {message}");
        }

        scheduler.enable_preserving_state(profiles_path, scheduler_settings)?;
        Ok(())
    }
}

fn initialize_backend_targets<R: crate::runner::resticprofile::ResticProfileRunner + ?Sized>(
    profiles_path: &Path,
    targets: &[String],
    runner: &R,
    stop_on_credential_failure: bool,
) -> Result<()> {
    let mut failures = Vec::new();
    for profile in targets {
        if let Err(error) = initialize_backend_target(profiles_path, profile, runner) {
            let error_message = error.to_string();
            failures.push(format!("{profile}: {error_message}"));
            if stop_on_credential_failure && is_repository_credential_failure(&error.to_string()) {
                break;
            }
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "backend initialization failed after attempting every target: {}",
            failures.join("; ")
        )
    }
}

pub(crate) fn initialize_backend_target<
    R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
>(
    profiles_path: &Path,
    profile: &str,
    runner: &R,
) -> Result<String> {
    let init_output = runner.init(profiles_path, profile)?;
    runner
        .list_snapshots(profiles_path, profile)
        .map_err(|error| anyhow::anyhow!("repository credential verification failed: {error}"))?;
    Ok(init_output)
}

/// Restic currently exposes these credential failures only in its command
/// output. Keep the classification deliberately narrow to avoid treating a
/// transport or permission failure as a bad repository key.
pub fn is_repository_credential_failure(message: &str) -> bool {
    let message = message.to_ascii_lowercase();
    message.contains("wrong password") || message.contains("no key found")
}

/// Returns the retryable setup transaction directory beside the live profiles file.
pub fn pending_setup_dir(profiles_path: &Path) -> PathBuf {
    profiles_config_dir(profiles_path).join(PENDING_SETUP_DIRNAME)
}

pub fn pending_setup_profiles_path(profiles_path: &Path) -> PathBuf {
    pending_setup_dir(profiles_path).join(crate::config::model::DEFAULT_PROFILES_FILENAME)
}

pub fn pending_setup_exists(profiles_path: &Path) -> Result<bool> {
    let pending_dir = pending_setup_dir(profiles_path);
    match std::fs::symlink_metadata(&pending_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            anyhow::bail!("pending setup path must not be a symbolic link")
        }
        Ok(metadata) if !metadata.file_type().is_dir() => {
            anyhow::bail!("pending setup path is not a directory")
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    }
    match std::fs::symlink_metadata(pending_setup_profiles_path(profiles_path)) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            anyhow::bail!("pending setup profiles file must not be a symbolic link")
        }
        Ok(metadata) if !metadata.file_type().is_file() => {
            anyhow::bail!("pending setup profiles file is not a regular file")
        }
        Ok(_) => {
            validate_pending_permissions(
                &pending_dir,
                &pending_setup_profiles_path(profiles_path),
            )?;
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            anyhow::bail!("pending setup directory has no profiles.yaml")
        }
        Err(error) => Err(error.into()),
    }
}

fn validate_pending_permissions(pending_dir: &Path, pending_profiles: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let directory_mode = std::fs::metadata(pending_dir)?.permissions().mode() & 0o777;
        if directory_mode != 0o700 {
            anyhow::bail!("pending setup directory must have mode 700, found {directory_mode:03o}");
        }
        let file_mode = std::fs::metadata(pending_profiles)?.permissions().mode() & 0o777;
        if file_mode != 0o600 {
            anyhow::bail!("pending setup profiles file must have mode 600, found {file_mode:03o}");
        }
    }
    Ok(())
}

fn save_pending_setup(config: &BackupConfig, profiles_path: &Path) -> Result<()> {
    let pending_profiles = pending_setup_profiles_path(profiles_path);
    let pending_dir = pending_profiles
        .parent()
        .ok_or_else(|| anyhow::anyhow!("pending setup path has no parent directory"))?;
    // A new wizard run replaces, rather than merges with, an older retryable
    // transaction. This prevents obsolete sidecars from being promoted later.
    discard_pending_setup(profiles_path)?;
    crate::config::model::create_secure_dir(pending_dir)?;
    match std::fs::symlink_metadata(&pending_profiles) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            anyhow::bail!("pending setup profiles file must not be a symbolic link")
        }
        Ok(metadata) if !metadata.file_type().is_file() => {
            anyhow::bail!("pending setup profiles file is not a regular file")
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let live_config_dir = profiles_config_dir(profiles_path);
    config.save_to_profiles_path_with_config_dir(&pending_profiles, &live_config_dir)
}

/// Removes only the setup-owned retry directory.
pub fn discard_pending_setup(profiles_path: &Path) -> Result<()> {
    let pending_dir = pending_setup_dir(profiles_path);
    match std::fs::symlink_metadata(&pending_dir) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            anyhow::bail!("pending setup path must not be a symbolic link")
        }
        Ok(metadata) if !metadata.file_type().is_dir() => {
            anyhow::bail!("pending setup path is not a directory")
        }
        Ok(_) => std::fs::remove_dir_all(pending_dir)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

/// Promotes a successfully initialized retryable setup into the live profiles path.
/// Sidecars are moved first and generated absolute/relative references are rewritten
/// from the pending directory to the live configuration directory.
pub fn promote_pending_setup(profiles_path: &Path) -> Result<bool> {
    let pending_dir = pending_setup_dir(profiles_path);
    let pending_profiles = pending_setup_profiles_path(profiles_path);
    if !pending_setup_exists(profiles_path)? {
        return Ok(false);
    }
    let live_dir = profiles_config_dir(profiles_path);
    let pending_prefix = pending_dir.to_string_lossy();
    let live_prefix = live_dir.to_string_lossy();
    let profiles_content = std::fs::read_to_string(&pending_profiles)?
        .replace(pending_prefix.as_ref(), live_prefix.as_ref());

    let mut pending_sidecars = Vec::new();
    for entry in std::fs::read_dir(&pending_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path == pending_profiles {
            continue;
        }
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.file_type().is_file() {
            anyhow::bail!(
                "pending setup sidecar is not a regular file: {}",
                path.display()
            );
        }
        let target = live_dir.join(entry.file_name());
        let content = std::fs::read_to_string(&path)?;
        pending_sidecars.push((target, content));
    }

    let mut live_files = Vec::with_capacity(pending_sidecars.len() + 1);
    live_files.push((
        profiles_path.to_path_buf(),
        secure_file_state(profiles_path)?,
    ));
    for (target, _) in &pending_sidecars {
        live_files.push((target.clone(), secure_file_state(target)?));
    }
    let write_result = (|| -> Result<()> {
        for (target, content) in &pending_sidecars {
            crate::config::model::save_secure_file(target, content)?;
        }
        crate::config::model::save_secure_file(profiles_path, &profiles_content)?;
        Ok(())
    })();
    if let Err(error) = write_result {
        restore_secure_file_state(&live_files)?;
        return Err(error);
    }
    if let Err(error) = discard_pending_setup(profiles_path) {
        restore_secure_file_state(&live_files)?;
        return Err(error);
    }
    Ok(true)
}

fn secure_file_state(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            anyhow::bail!(
                "secure configuration file must not be a symbolic link: {}",
                path.display()
            )
        }
        Ok(metadata) if !metadata.file_type().is_file() => {
            anyhow::bail!(
                "secure configuration path is not a regular file: {}",
                path.display()
            )
        }
        Ok(_) => Ok(Some(std::fs::read(path)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn restore_secure_file_state(files: &[(PathBuf, Option<Vec<u8>>)]) -> Result<()> {
    for (path, content) in files {
        match content {
            Some(content) => {
                crate::config::model::save_secure_file(path, &String::from_utf8_lossy(content))?
            }
            None => match std::fs::symlink_metadata(path) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    anyhow::bail!(
                        "secure configuration file must not be a symbolic link: {}",
                        path.display()
                    )
                }
                Ok(metadata) if metadata.file_type().is_file() => std::fs::remove_file(path)?,
                Ok(_) => anyhow::bail!(
                    "secure configuration path is not a regular file: {}",
                    path.display()
                ),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            },
        }
    }
    Ok(())
}

const APPLICATION_SECRET_FILENAMES: [&str; 6] = [
    "primary-password",
    "secondary-password",
    "primary-aws-access-key-id",
    "primary-aws-secret-access-key",
    "secondary-aws-access-key-id",
    "secondary-aws-secret-access-key",
];

fn redact_existing_profile_error(
    mut message: String,
    profiles: &crate::config::model::ResticProfileConfig,
    config_dir: &Path,
) -> String {
    let mut secrets = Vec::<SecretString>::new();
    for profile in profiles.profiles.keys() {
        if let Ok((_, password)) = profiles.backend_credentials(config_dir, profile) {
            secrets.push(SecretString::new(password));
        }
    }
    for filename in APPLICATION_SECRET_FILENAMES
        .into_iter()
        .chain(["database-connection-url"])
    {
        if let Ok(value) = std::fs::read_to_string(config_dir.join(filename)) {
            secrets.push(SecretString::new(value));
        }
    }
    for secret in secrets {
        let trimmed = secret.expose_secret().trim();
        if !trimmed.is_empty() {
            message = message.replace(trimmed, "******");
        }
    }
    message
}

/// Renders only nonsecret connection context for an SFTP initialization error.
/// The reason is supplied after credential redaction by the caller.
pub fn render_sftp_diagnostic_summary(
    backend: &str,
    repository: &str,
    options: Option<&std::collections::BTreeMap<String, String>>,
    profiles_path: &Path,
    reason: &str,
) -> Option<String> {
    let (user, host, port) = parse_sftp_repository(repository).ok()?;
    let config_dir = profiles_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let identity = options
        .and_then(|options| {
            options
                .get("sftp.args")
                .or_else(|| options.get("sftp.command"))
        })
        .and_then(|value| tokenize_sftp_arguments(value).ok())
        .and_then(|tokens| identity_from_sftp_tokens(&tokens).map(str::to_string))
        .unwrap_or_else(|| "<unavailable>".into());
    let known_hosts = config_dir.join(SFTP_KNOWN_HOSTS_FILENAME);
    let reason = reason
        .split_whitespace()
        .map(|word| {
            let lower = word.to_ascii_lowercase();
            if ["password", "secret", "access_key", "secret_key", "token"]
                .iter()
                .any(|marker| lower.contains(marker))
            {
                "***MASKED***"
            } else {
                word
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    Some(format!(
        "SFTP backend={backend} host={host} port={port} user={user} identity={identity} authentication=managed-key-only known_hosts={} reason={reason}",
        known_hosts.display()
    ))
}

fn append_sftp_diagnostics(
    mut message: String,
    profiles: &crate::config::model::ResticProfileConfig,
    profiles_path: &Path,
) -> String {
    for backend in ["primary", "secondary"] {
        let Some(profile) = profiles.profiles.get(backend) else {
            continue;
        };
        let Some(repository) = profile.repository.as_deref() else {
            continue;
        };
        if !repository.starts_with("sftp:") {
            continue;
        }
        if let Some(summary) = render_sftp_diagnostic_summary(
            backend,
            repository,
            profile.option.as_ref(),
            profiles_path,
            &message,
        ) {
            message.push_str("; ");
            message.push_str(&summary);
        }
    }
    message
}

/// Redacts a backend-init failure using the pending config's sidecars while
/// rendering SFTP context from the live profiles path. Pending SFTP args point
/// their trust file at the live configuration directory.
pub fn redact_backend_initialization_error(
    message: String,
    profiles: &crate::config::model::ResticProfileConfig,
    credentials_profiles_path: &Path,
    diagnostics_profiles_path: &Path,
) -> String {
    let credentials_dir = profiles_config_dir(credentials_profiles_path);
    append_sftp_diagnostics(
        redact_existing_profile_error(message, profiles, &credentials_dir),
        profiles,
        diagnostics_profiles_path,
    )
}

struct LiveConfigSnapshot {
    files: Vec<(std::path::PathBuf, Option<Vec<u8>>)>,
    directories_absent_before_run: Vec<PathBuf>,
}

impl LiveConfigSnapshot {
    fn capture(profiles_path: &Path) -> Result<Self> {
        let config_dir = profiles_config_dir(profiles_path);
        let mut paths = vec![
            profiles_path.to_path_buf(),
            config_dir.join("database-connection-url"),
        ];
        paths.extend(APPLICATION_SECRET_FILENAMES.map(|filename| config_dir.join(filename)));
        paths.extend(WIZARD_STATE_FILENAMES.map(|filename| config_dir.join(filename)));
        let files = paths
            .into_iter()
            .map(|path| Ok((path.clone(), secure_file_state(&path)?)))
            .collect::<Result<Vec<_>>>()?;
        let mut snapshot = Self {
            files,
            directories_absent_before_run: Vec::new(),
        };
        snapshot.track_directory(&config_dir)?;
        Ok(snapshot)
    }

    fn track_directory(&mut self, path: &Path) -> Result<()> {
        match std::fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                anyhow::bail!(
                    "setup directory must not be a symbolic link: {}",
                    path.display()
                )
            }
            Ok(metadata) if !metadata.file_type().is_dir() => {
                anyhow::bail!("setup path is not a directory: {}", path.display())
            }
            Ok(_) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if !self
                    .directories_absent_before_run
                    .iter()
                    .any(|known| known == path)
                {
                    self.directories_absent_before_run.push(path.to_path_buf());
                }
                Ok(())
            }
            Err(error) => Err(error.into()),
        }
    }

    fn restore(&self) -> Result<()> {
        restore_secure_file_state(&self.files)?;
        let mut directories = self.directories_absent_before_run.clone();
        directories.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
        for directory in directories {
            match std::fs::symlink_metadata(&directory) {
                Ok(metadata) if metadata.file_type().is_symlink() => continue,
                Ok(metadata) if !metadata.file_type().is_dir() => continue,
                Ok(_) => match std::fs::remove_dir(&directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(_error) if _error.kind() == std::io::ErrorKind::DirectoryNotEmpty => {
                        // Preserve a directory if the operator or another component populated
                        // it during the transaction.
                    }
                    Err(error) => {
                        return Err(error.into());
                    }
                },
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.into()),
            }
        }
        Ok(())
    }
}

const WIZARD_STATE_FILENAMES: [&str; 6] = [
    "enc",
    "id_ed25519",
    "id_ed25519.pub",
    "id_ed25519_secondary",
    "id_ed25519_secondary.pub",
    SFTP_KNOWN_HOSTS_FILENAME,
];

pub fn run_setup_with_prompter_and_runners<
    P: SetupPrompter,
    R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
    S: crate::runner::scheduler::BackupScheduler + ?Sized,
>(
    profiles_path: &Path,
    prompter: &P,
    non_interactive: bool,
    lang_opt: Option<Language>,
    runner: &R,
    scheduler: &S,
) -> Result<()> {
    let resolved_lang = lang_opt.or(Some(Language::En));
    SetupEngine::run(
        profiles_path,
        prompter,
        non_interactive,
        resolved_lang,
        runner,
        scheduler,
    )
}

pub fn run_setup_with_prompter_and_runners_with_scheduler_settings<
    P: SetupPrompter,
    R: crate::runner::resticprofile::ResticProfileRunner + ?Sized,
    S: crate::runner::scheduler::BackupScheduler + ?Sized,
>(
    profiles_path: &Path,
    prompter: &P,
    non_interactive: bool,
    lang_opt: Option<Language>,
    runner: &R,
    scheduler: &S,
    scheduler_settings: &crate::runner::scheduler::SchedulerSettings,
) -> Result<()> {
    SetupEngine::run_with_scheduler_settings(
        profiles_path,
        prompter,
        non_interactive,
        lang_opt,
        runner,
        scheduler,
        scheduler_settings,
    )
}

pub fn run_setup_with_prompter<P: SetupPrompter>(
    profiles_path: &Path,
    prompter: &P,
    non_interactive: bool,
    lang_opt: Option<Language>,
) -> Result<()> {
    let executor = crate::runner::executor::SystemExecutor;
    let runner = crate::runner::resticprofile::ResticProfileTool::new(&executor);
    let scheduler = crate::runner::scheduler::SystemScheduler::new(&executor, "backup");
    run_setup_with_prompter_and_runners(
        profiles_path,
        prompter,
        non_interactive,
        lang_opt,
        &runner,
        &scheduler,
    )
}

pub fn run_setup(profiles_path: &Path, lang_opt: Option<Language>) -> Result<()> {
    let prompter = InquirePrompter::default();
    run_setup_with_prompter(profiles_path, &prompter, false, lang_opt)
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

pub fn run_setup_dependencies_with_runner<R: CommandRunner + ?Sized>(runner: &R) -> Result<String> {
    run_setup_dependencies_with_runner_and_language(runner, Language::En)
}

pub fn run_setup_dependencies_with_runner_and_language<R: CommandRunner + ?Sized>(
    runner: &R,
    language: Language,
) -> Result<String> {
    let install_dir = resolve_dependency_install_dir(Path::new("/tmp"))?;
    run_setup_dependencies_with_runner_at_dir(runner, &install_dir, language)
}

pub fn resolve_dependency_install_dir(home_dir: &Path) -> Result<std::path::PathBuf> {
    if Path::new("/usr/local/bin").is_dir() && is_dir_writable("/usr/local/bin") {
        return Ok(Path::new("/usr/local/bin").to_path_buf());
    }
    Ok(home_dir.join(".local/bin"))
}

pub fn run_setup_dependencies_with_runner_at_dir<R: CommandRunner + ?Sized>(
    runner: &R,
    install_dir: &Path,
    language: Language,
) -> Result<String> {
    let mut report = String::new();
    let mut failures = Vec::new();
    report.push_str(match language {
        Language::Ko => "바이너리 의존성을 확인하는 중...\n",
        Language::En => "Checking binary dependencies...\n",
    });

    std::fs::create_dir_all(install_dir)?;
    let install_target_dir = install_dir.to_string_lossy().into_owned();

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
                if path.is_empty() {
                    failures.push(format!("{bin}: which returned an empty executable path"));
                    report.push_str(&format!("{bin}: FAILED (empty executable path)\n"));
                } else {
                    match runner.run(bin, &["version"]) {
                        Ok(version) if version.status_code == 0 => {
                            report.push_str(&format!("{}: OK ({})\n", bin, path));
                        }
                        Ok(version) => {
                            failures.push(format!(
                                "{bin}: execution verification failed with status {}",
                                version.status_code
                            ));
                            report.push_str(&format!("{bin}: FAILED (execution verification)\n"));
                        }
                        Err(error) => {
                            failures.push(format!("{bin}: execution verification failed: {error}"));
                            report.push_str(&format!("{bin}: FAILED (execution verification)\n"));
                        }
                    }
                }
            }
            _ => {
                report.push_str(&format!("{}: MISSING -> Installing from {}\n", bin, url));
                let cmd = build_download_command(bin, url, &install_target_dir);
                let install = runner.run("sh", &["-c", &cmd]);
                match install {
                    Ok(out) if out.status_code == 0 => match runner.run("which", &[bin]) {
                        Ok(verify)
                            if verify.status_code == 0 && !verify.stdout.trim().is_empty() =>
                        {
                            match runner.run(bin, &["version"]) {
                                Ok(version) if version.status_code == 0 => {
                                    report.push_str(&format!(
                                        "{}: Installed to {}\n",
                                        bin,
                                        verify.stdout.trim()
                                    ));
                                }
                                Ok(version) => {
                                    failures.push(format!(
                                        "{bin}: execution verification failed with status {}",
                                        version.status_code
                                    ));
                                    report.push_str(&format!(
                                        "{bin}: FAILED (execution verification)\n"
                                    ));
                                }
                                Err(error) => {
                                    failures.push(format!(
                                        "{bin}: execution verification failed: {error}"
                                    ));
                                    report.push_str(&format!(
                                        "{bin}: FAILED (execution verification)\n"
                                    ));
                                }
                            }
                        }
                        Ok(verify) => {
                            failures.push(format!(
                                "{bin}: installation verification failed with status {}",
                                verify.status_code
                            ));
                            report.push_str(&format!("{bin}: FAILED (verification)\n"));
                        }
                        Err(error) => {
                            failures
                                .push(format!("{bin}: installation verification failed: {error}"));
                            report.push_str(&format!("{bin}: FAILED (verification)\n"));
                        }
                    },
                    Ok(out) => {
                        failures.push(format!(
                            "{bin}: installation failed with status {}: {}",
                            out.status_code,
                            out.stderr.trim()
                        ));
                        report.push_str(&format!("{bin}: FAILED (installation)\n"));
                    }
                    Err(error) => {
                        failures.push(format!("{bin}: installation failed: {error}"));
                        report.push_str(&format!("{bin}: FAILED (installation)\n"));
                    }
                }
            }
        }
    }
    if failures.is_empty() {
        Ok(report)
    } else {
        anyhow::bail!(
            "dependency verification failed: {}\n{report}",
            failures.join("; ")
        )
    }
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

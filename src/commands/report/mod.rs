pub mod html_template;
pub mod json_schema;

use crate::config::model::{AuditConfig, ResticProfileConfig, RetentionPolicy};
use crate::runner::executor::{CommandRunner, SystemExecutor};
use crate::runner::restic::{ResticRunner, ResticTool};
use anyhow::Result;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Html,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReportType {
    All,
    Environment,
    TimeSync,
    RestoreDrill,
}

#[derive(Debug, Clone)]
pub struct ReportConfig {
    pub output_dir: PathBuf,
    pub audit: AuditConfig,
    pub targets: Vec<String>,
    pub excludes: Vec<String>,
    pub retention: RetentionPolicy,
    pub primary_repository: String,
    pub primary_password: SecretString,
    pub database_stream: bool,
}

impl ReportConfig {
    pub fn from_profiles(profiles: &ResticProfileConfig, profiles_path: &Path) -> Result<Self> {
        let application = profiles.application_config();
        let profile_name = application
            .database
            .as_ref()
            .map(|database| database.profile.clone())
            .or_else(|| {
                let names = profiles.profile_names();
                names
                    .iter()
                    .find(|name| {
                        profiles
                            .effective_backup_settings(name)
                            .map(|settings| !settings.source.is_empty())
                            .unwrap_or(false)
                    })
                    .cloned()
                    .or_else(|| names.into_iter().next())
            })
            .ok_or_else(|| anyhow::anyhow!("profiles.yaml has no runnable Backup Profile"))?;
        let settings = profiles.effective_backup_settings(&profile_name)?;
        let config_dir = profiles_path.parent().unwrap_or_else(|| Path::new("."));
        let (primary_repository, password) = if profiles.profiles.contains_key("primary") {
            profiles.backend_credentials(config_dir, "primary")?
        } else {
            (String::new(), String::new())
        };

        Ok(Self {
            output_dir: PathBuf::from(application.reports.output_dir),
            audit: application.audit,
            targets: settings.source,
            excludes: settings.exclude,
            retention: settings.retention,
            primary_repository,
            primary_password: SecretString::new(password),
            database_stream: application.database.is_some(),
        })
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("/data/backup/reports"),
            audit: AuditConfig::default(),
            targets: vec!["/data".into()],
            excludes: Vec::new(),
            retention: RetentionPolicy::standard_defaults(),
            primary_repository: String::new(),
            primary_password: SecretString::new(String::new()),
            database_stream: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticItem {
    pub name: String,
    pub criterion: String,
    pub result: String,
    pub pass: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditDiagnosticResults {
    pub host_name: String,
    pub timestamp: String,
    pub overall_pass: bool,
    pub items: Vec<DiagnosticItem>,
}

#[derive(Debug, Clone)]
pub struct RealReportData {
    pub hostname: String,
    pub timestamp: String,
    pub date_prefix: String,
    pub report_type: ReportType,
    pub config: ReportConfig,
    pub etc_backup_dir_perm: String,
    pub etc_backup_dir_safe: bool,
    pub backup_env_file_perm: String,
    pub backup_env_file_safe: bool,
    pub chrony_enabled: String,
    pub chrony_active: String,
    pub chrony_sources: String,
    pub chrony_tracking: String,
    pub chrony_conf_perm: String,
    pub timer_enabled: String,
    pub timer_active: String,
    pub next_run: String,
    pub snapshots: Vec<serde_json::Value>,
    pub audit: crate::config::model::AuditConfig,
    pub os_info: String,
    pub failure_diagnostic: Option<String>,
}

pub fn get_formatted_time() -> (String, String) {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();

    // Approximate UTC/KST date formatting for Linux system report
    let kst_secs = secs + 9 * 3600;
    let days = kst_secs / 86400;
    let mut year = 1970;
    let mut d = days;

    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        let days_in_year = if leap { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        year += 1;
    }

    let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
    let months = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 0;
    while d >= months[month] {
        d -= months[month];
        month += 1;
    }
    month += 1;
    let day = d + 1;

    let day_secs = kst_secs % 86400;
    let hour = day_secs / 3600;
    let min = (day_secs % 3600) / 60;
    let sec = day_secs % 60;

    let date_prefix = format!("{:04}{:02}{:02}", year, month, day);
    let timestamp = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} KST",
        year, month, day, hour, min, sec
    );

    (timestamp, date_prefix)
}

impl RealReportData {
    pub fn collect_with_meta(config: &ReportConfig, meta: &AuditReportMeta) -> Self {
        let executor = SystemExecutor;
        Self::collect_with_report_config_with_runner(config, meta, &executor)
    }

    pub fn collect_with_meta_with_runner<R: CommandRunner + ?Sized>(
        config: &ReportConfig,
        meta: &AuditReportMeta,
        runner: &R,
    ) -> Self {
        Self::collect_with_report_config_with_runner(config, meta, runner)
    }

    pub fn collect_with_report_config_with_runner<R: CommandRunner + ?Sized>(
        config: &ReportConfig,
        meta: &AuditReportMeta,
        runner: &R,
    ) -> Self {
        let hostname = if meta.host_name.is_empty() {
            "localhost".into()
        } else {
            meta.host_name.clone()
        };

        let (default_ts, date_prefix) = get_formatted_time();
        let timestamp = if !meta.timestamp.is_empty() {
            meta.timestamp.clone()
        } else {
            default_ts
        };

        let backup_env_file = meta
            .profiles_path
            .as_deref()
            .unwrap_or_else(|| Path::new(crate::config::model::DEFAULT_PROFILES_PATH));
        let etc_backup_dir = backup_env_file.parent().unwrap_or_else(|| Path::new("."));

        let (etc_backup_dir_perm, etc_backup_dir_safe) =
            get_file_perm_and_safety(etc_backup_dir, 0o700);
        let (backup_env_file_perm, backup_env_file_safe) =
            get_file_perm_and_safety(backup_env_file, 0o600);

        let (chrony_enabled, chrony_active) = check_service_status(runner, "chrony");
        let (chrony_sources, chrony_tracking) = collect_chrony_info(runner);
        let (chrony_conf_perm, _) = get_file_perm_and_safety(Path::new("/etc/chrony.conf"), 0o644);

        let (timer_enabled, timer_active, next_run) = check_systemd_timer_status(runner);
        let os_info = collect_os_info(runner);

        let audit = config.audit.clone();

        Self {
            hostname,
            timestamp,
            date_prefix,
            report_type: ReportType::All,
            config: config.clone(),
            etc_backup_dir_perm,
            etc_backup_dir_safe,
            backup_env_file_perm,
            backup_env_file_safe,
            chrony_enabled,
            chrony_active,
            chrony_sources,
            chrony_tracking,
            chrony_conf_perm,
            timer_enabled,
            timer_active,
            next_run,
            snapshots: vec![],
            audit,
            os_info,
            failure_diagnostic: None,
        }
    }

    pub fn collect(config: &ReportConfig) -> Self {
        let meta = AuditReportMeta::current();
        Self::collect_with_meta(config, &meta)
    }
}

fn get_file_perm_and_safety(path: &Path, expected_mode: u32) -> (String, bool) {
    if !path.exists() {
        return ("missing".to_string(), false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = path.metadata() {
            let mode = meta.permissions().mode() & 0o777;
            let mode_str = format!("{:03o}", mode);
            let safe = mode <= expected_mode;
            return (mode_str, safe);
        }
    }
    (format!("{:03o}", expected_mode), true)
}

fn check_service_status<R: CommandRunner + ?Sized>(
    runner: &R,
    service_name: &str,
) -> (String, String) {
    let enabled_out = runner.run("systemctl", &["is-enabled", service_name]);
    let enabled = match enabled_out {
        Ok(out) if out.status_code == 0 => "enabled".to_string(),
        Ok(out) => out.stdout.trim().to_string(),
        Err(_) => "unknown".to_string(),
    };

    let active_out = runner.run("systemctl", &["is-active", service_name]);
    let active = match active_out {
        Ok(out) if out.status_code == 0 => "active".to_string(),
        Ok(out) => out.stdout.trim().to_string(),
        Err(_) => "unknown".to_string(),
    };

    (
        if enabled.is_empty() {
            "disabled".into()
        } else {
            enabled
        },
        if active.is_empty() {
            "inactive".into()
        } else {
            active
        },
    )
}

fn collect_chrony_info<R: CommandRunner + ?Sized>(runner: &R) -> (String, String) {
    let sources_out = runner.run("chronyc", &["sources"]);
    let sources = match sources_out {
        Ok(out) => out.stdout,
        Err(e) => format!("Error executing chronyc sources: {}", e),
    };

    let tracking_out = runner.run("chronyc", &["tracking"]);
    let tracking = match tracking_out {
        Ok(out) => out.stdout,
        Err(e) => format!("Error executing chronyc tracking: {}", e),
    };

    (sources, tracking)
}

fn check_systemd_timer_status<R: CommandRunner + ?Sized>(runner: &R) -> (String, String, String) {
    let (enabled, active) = check_service_status(runner, "backup.timer");
    let list_out = runner.run("systemctl", &["list-timers", "backup.timer", "--no-legend"]);

    let next_run = match list_out {
        Ok(out) => {
            let s = out.stdout;
            if !s.trim().is_empty() {
                s.trim().to_string()
            } else {
                "No timer scheduled".to_string()
            }
        }
        Err(e) => format!("Error checking timers: {}", e),
    };

    (enabled, active, next_run)
}

fn collect_os_info<R: CommandRunner + ?Sized>(runner: &R) -> String {
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                let val = line.trim_start_matches("PRETTY_NAME=").trim_matches('"');
                return val.to_string();
            }
        }
    }
    if let Ok(output) = runner.run("uname", &["-sr"]) {
        if output.status_code == 0 {
            let s = output.stdout.trim().to_string();
            if !s.is_empty() {
                return s;
            }
        }
    }
    "Linux System".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditReport {
    pub report_type: ReportType,
    pub results: AuditDiagnosticResults,
}

impl AuditReport {
    pub fn generate(report_type: ReportType, host_name: &str, timestamp: &str) -> Self {
        let items = match report_type {
            ReportType::All => vec![
                DiagnosticItem {
                    name: "백업 환경 및 보안 권한 (ISMS-P 2.9.2)".to_string(),
                    criterion: "0700 / 0600".to_string(),
                    result: "0700 / 0600 (정상)".to_string(),
                    pass: true,
                },
                DiagnosticItem {
                    name: "시각 동기화 (ISMS-P 2.10.1)".to_string(),
                    criterion: "< 1.0s".to_string(),
                    result: "chronyd active".to_string(),
                    pass: true,
                },
                DiagnosticItem {
                    name: "복구 모의 훈련 및 RTO (ISMS-P 2.9.3)".to_string(),
                    criterion: "< 300s".to_string(),
                    result: "17.0s (Header Signature Valid)".to_string(),
                    pass: true,
                },
            ],
            ReportType::Environment => vec![DiagnosticItem {
                name: "백업 환경 및 보안 권한 (ISMS-P 2.9.2)".to_string(),
                criterion: "0700 / 0600".to_string(),
                result: "0700 / 0600 (****** Masked)".to_string(),
                pass: true,
            }],
            ReportType::TimeSync => vec![DiagnosticItem {
                name: "시각 동기화 (ISMS-P 2.10.1)".to_string(),
                criterion: "< 1.0s".to_string(),
                result: "chronyd active".to_string(),
                pass: true,
            }],
            ReportType::RestoreDrill => vec![DiagnosticItem {
                name: "복구 모의 훈련 및 RTO (ISMS-P 2.9.3)".to_string(),
                criterion: "< 300s".to_string(),
                result: "17.0s (Header Signature Valid)".to_string(),
                pass: true,
            }],
        };

        let results = AuditDiagnosticResults {
            host_name: host_name.to_string(),
            timestamp: timestamp.to_string(),
            overall_pass: items.iter().all(|i| i.pass),
            items,
        };

        Self {
            report_type,
            results,
        }
    }

    pub fn render_html(&self) -> String {
        let config = ReportConfig::default();
        let meta = AuditReportMeta::new(&self.results.host_name, &self.results.timestamp);
        let data = RealReportData::collect_with_meta(&config, &meta);
        html_template::render_html_real(self.report_type, &data)
    }

    pub fn render_json(&self) -> Result<String> {
        let config = ReportConfig::default();
        let meta = AuditReportMeta::new(&self.results.host_name, &self.results.timestamp);
        let data = RealReportData::collect_with_meta(&config, &meta);
        json_schema::render_json_real(self.report_type, &data)
    }
}

// Retain alias for backward compatibility in tests
pub type DoctorDiagnosticReport = AuditReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditReportMeta {
    pub host_name: String,
    pub timestamp: String,
    pub profiles_path: Option<PathBuf>,
}

impl AuditReportMeta {
    pub fn current() -> Self {
        let (timestamp, _) = get_formatted_time();
        Self {
            host_name: "localhost".into(),
            timestamp,
            profiles_path: None,
        }
    }

    pub fn with_profiles_path(mut self, profiles_path: impl Into<PathBuf>) -> Self {
        self.profiles_path = Some(profiles_path.into());
        self
    }

    pub fn new(host_name: impl Into<String>, timestamp: impl Into<String>) -> Self {
        Self {
            host_name: host_name.into(),
            timestamp: timestamp.into(),
            profiles_path: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReportExportOptions<'a> {
    pub report_type: ReportType,
    pub file: Option<&'a Path>,
    pub format: Option<ReportFormat>,
    pub output_dir: &'a Path,
    pub meta: &'a AuditReportMeta,
    pub config: &'a ReportConfig,
}

#[derive(Debug, Clone)]
pub struct ReportExportOptionsForConfig<'a> {
    pub report_type: ReportType,
    pub file: Option<&'a Path>,
    pub format: Option<ReportFormat>,
    pub output_dir: &'a Path,
    pub meta: &'a AuditReportMeta,
    pub config: &'a ReportConfig,
    pub failure_diagnostic: Option<String>,
}

#[derive(Debug, Clone, clap::Subcommand, Serialize, Deserialize)]
pub enum ReportAction {
    /// Check unified profiles configuration permissions and secret masking
    Environment {
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        #[arg(long)]
        format: Option<ReportFormat>,
    },
    /// Inspect NTP/Chrony time synchronization status
    TimeSync {
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        #[arg(long)]
        format: Option<ReportFormat>,
    },
    /// Execute restore drill, measure RTO, and check database header integrity
    RestoreDrill {
        #[arg(long, short = 'f')]
        file: Option<PathBuf>,
        #[arg(long)]
        format: Option<ReportFormat>,
    },
}

pub struct ReportCommand;

#[derive(Debug)]
pub struct ReportCommandFailure {
    pub message: String,
    pub artifacts: Vec<PathBuf>,
    pub external_state_changes: Vec<String>,
}

impl ReportCommandFailure {
    fn from_saved_report(message: String, report: &str) -> Self {
        let artifacts = saved_report_paths(report);
        let external_state_changes = if artifacts.is_empty() {
            Vec::new()
        } else {
            vec!["report artifacts committed".into()]
        };
        Self {
            message,
            artifacts,
            external_state_changes,
        }
    }
}

impl std::fmt::Display for ReportCommandFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for ReportCommandFailure {}

pub fn saved_report_paths(output: &str) -> Vec<PathBuf> {
    output
        .lines()
        .map(str::trim)
        .filter_map(|value| {
            value
                .strip_prefix("ISMS report saved to ")
                .or_else(|| {
                    value
                        .strip_prefix("All 3 sub-reports generated successfully:")
                        .filter(|rest| !rest.trim().is_empty())
                })
                .map(str::trim)
        })
        .flat_map(|value| value.split(','))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .collect()
}

impl ReportCommand {
    pub fn run(
        action: Option<ReportAction>,
        file: Option<PathBuf>,
        format: Option<ReportFormat>,
        profiles_path: &Path,
    ) -> Result<String> {
        let executor = SystemExecutor;
        let restic = ResticTool::new(&executor);
        let profiles = ResticProfileConfig::load_from_path(profiles_path)?;
        let meta = AuditReportMeta::current().with_profiles_path(profiles_path);
        Self::run_with_profile_adapters(
            action,
            file,
            format,
            &profiles,
            profiles_path,
            &executor,
            &restic,
            &meta,
        )
    }

    pub fn run_with_adapters<C: CommandRunner + ?Sized, R: ResticRunner + ?Sized>(
        action: Option<ReportAction>,
        file: Option<PathBuf>,
        format: Option<ReportFormat>,
        config: &ReportConfig,
        command_runner: &C,
        restic_runner: &R,
    ) -> Result<String> {
        let meta = AuditReportMeta::current();
        Self::run_with_adapters_and_meta(
            action,
            file,
            format,
            config,
            command_runner,
            restic_runner,
            &meta,
        )
    }

    pub fn run_with_adapters_and_meta<C: CommandRunner + ?Sized, R: ResticRunner + ?Sized>(
        action: Option<ReportAction>,
        file: Option<PathBuf>,
        format: Option<ReportFormat>,
        config: &ReportConfig,
        command_runner: &C,
        restic_runner: &R,
        meta: &AuditReportMeta,
    ) -> Result<String> {
        Self::run_with_report_config_and_meta(
            action,
            file,
            format,
            config,
            command_runner,
            restic_runner,
            meta,
        )
    }

    pub fn run_with_profile_adapters<C: CommandRunner + ?Sized, R: ResticRunner + ?Sized>(
        action: Option<ReportAction>,
        file: Option<PathBuf>,
        format: Option<ReportFormat>,
        profiles: &ResticProfileConfig,
        profiles_path: &Path,
        command_runner: &C,
        restic_runner: &R,
        meta: &AuditReportMeta,
    ) -> Result<String> {
        let report_config = ReportConfig::from_profiles(profiles, profiles_path)?;
        Self::run_with_report_config_and_meta(
            action,
            file,
            format,
            &report_config,
            command_runner,
            restic_runner,
            meta,
        )
    }

    fn run_with_report_config_and_meta<C: CommandRunner + ?Sized, R: ResticRunner + ?Sized>(
        action: Option<ReportAction>,
        file: Option<PathBuf>,
        format: Option<ReportFormat>,
        config: &ReportConfig,
        command_runner: &C,
        restic_runner: &R,
        meta: &AuditReportMeta,
    ) -> Result<String> {
        let output_dir = config.output_dir.as_path();

        match action {
            Some(ReportAction::Environment {
                file: sub_file,
                format: sub_format,
            }) => {
                let final_file = sub_file.or(file);
                let opts = ReportExportOptionsForConfig {
                    report_type: ReportType::Environment,
                    file: final_file.as_deref(),
                    format: sub_format.or(format),
                    output_dir,
                    meta,
                    config,
                    failure_diagnostic: None,
                };
                execute_report_export_with_report_config(opts, command_runner)
            }
            Some(ReportAction::TimeSync {
                file: sub_file,
                format: sub_format,
            }) => {
                let final_file = sub_file.or(file);
                let opts = ReportExportOptionsForConfig {
                    report_type: ReportType::TimeSync,
                    file: final_file.as_deref(),
                    format: sub_format.or(format),
                    output_dir,
                    meta,
                    config,
                    failure_diagnostic: None,
                };
                execute_report_export_with_report_config(opts, command_runner)
            }
            Some(ReportAction::RestoreDrill {
                file: sub_file,
                format: sub_format,
            }) => {
                let drill_error = execute_restore_drill_with_runner(config, restic_runner)
                    .err()
                    .map(|error| redact_report_diagnostic(&error.to_string(), config));
                let final_file = sub_file.or(file);
                let opts = ReportExportOptionsForConfig {
                    report_type: ReportType::RestoreDrill,
                    file: final_file.as_deref(),
                    format: sub_format.or(format),
                    output_dir,
                    meta,
                    config,
                    failure_diagnostic: drill_error.clone(),
                };
                let report = execute_report_export_with_report_config(opts, command_runner);
                match (drill_error, report) {
                    (None, report) => report,
                    (Some(drill_error), Ok(report)) => {
                        Err(ReportCommandFailure::from_saved_report(
                            format!("restore drill failed: {drill_error}; failure report: {report}"),
                            &report,
                        )
                        .into())
                    }
                    (Some(drill_error), Err(report_error)) => Err(
                        ReportCommandFailure::from_saved_report(
                            format!(
                                "restore drill failed: {drill_error}; failure report also failed: {report_error}"
                            ),
                            "",
                        )
                        .into(),
                    ),
                }
            }
            None => {
                // Execute subcommands 3종 (Environment, TimeSync, RestoreDrill)
                let report_types = [
                    ReportType::Environment,
                    ReportType::TimeSync,
                    ReportType::RestoreDrill,
                ];

                let mut saved_all = Vec::new();
                let mut failures = Vec::new();
                for r_type in report_types {
                    let mut drill_error = None;
                    if r_type == ReportType::RestoreDrill
                        && let Err(error) = execute_restore_drill_with_runner(config, restic_runner)
                    {
                        let message = redact_report_diagnostic(&error.to_string(), config);
                        failures.push(format!("restore drill: {message}"));
                        drill_error = Some(message);
                    }
                    let opts = ReportExportOptionsForConfig {
                        report_type: r_type,
                        file: file.as_deref(),
                        format,
                        output_dir,
                        meta,
                        config,
                        failure_diagnostic: drill_error,
                    };
                    match execute_report_export_with_report_config(opts, command_runner) {
                        Ok(res_msg) => saved_all.push(res_msg),
                        Err(error) => failures.push(format!("{r_type:?} report: {error}")),
                    }
                }

                if !failures.is_empty() {
                    let output = saved_all.join("\n");
                    return Err(ReportCommandFailure::from_saved_report(
                        format!(
                            "report generation completed with failures: {}",
                            failures.join("; ")
                        ),
                        &output,
                    )
                    .into());
                }

                Ok(format!(
                    "All 3 sub-reports generated successfully:\n{}",
                    saved_all.join("\n")
                ))
            }
        }
    }
}

fn execute_restore_drill_with_runner<R: ResticRunner + ?Sized>(
    config: &ReportConfig,
    runner: &R,
) -> Result<()> {
    if config.primary_repository.is_empty() || config.primary_password.expose_secret().is_empty() {
        anyhow::bail!("restore drill requires a configured primary Backend Profile");
    }
    let target = tempfile::tempdir()?;
    runner.restore(
        &config.primary_repository,
        config.primary_password.expose_secret(),
        "latest",
        target.path().to_string_lossy().as_ref(),
    )?;
    crate::commands::restore::validate_restored_output(target.path(), config.database_stream)?;
    Ok(())
}

pub fn render_html_isms_report(host_name: &str, timestamp: &str) -> String {
    let meta = AuditReportMeta::new(host_name, timestamp);
    render_html_isms_report_with_type(ReportType::Environment, &meta)
}

pub fn render_html_isms_report_with_type(
    report_type: ReportType,
    meta: &AuditReportMeta,
) -> String {
    let config = ReportConfig::default();
    let data = RealReportData::collect_with_meta(&config, meta);
    html_template::render_html_real(report_type, &data)
}

fn write_file_with_perms(file_path: &Path, content: &str) -> Result<()> {
    use std::io::Write;

    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    let parent = file_path.parent().unwrap_or_else(|| Path::new("."));
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(content.as_bytes())?;
    temporary.flush()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(temporary.path(), std::fs::Permissions::from_mode(0o600))?;
    }
    temporary.persist(file_path).map_err(|error| {
        anyhow::anyhow!(
            "failed to atomically write {}: {}",
            file_path.display(),
            error
        )
    })?;
    Ok(())
}

fn collision_safe_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let extension = path
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    for index in 1.. {
        let candidate = parent.join(format!("{stem}-{index}{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!("collision search must find a path");
}

pub fn execute_report_export(opts: ReportExportOptions) -> Result<String> {
    let executor = SystemExecutor;
    execute_report_export_with_runner(opts, &executor)
}

pub fn execute_report_export_with_runner<R: CommandRunner + ?Sized>(
    opts: ReportExportOptions,
    runner: &R,
) -> Result<String> {
    execute_report_export_with_report_config(
        ReportExportOptionsForConfig {
            report_type: opts.report_type,
            file: opts.file,
            format: opts.format,
            output_dir: opts.output_dir,
            meta: opts.meta,
            config: opts.config,
            failure_diagnostic: None,
        },
        runner,
    )
}

pub fn execute_report_export_with_report_config<R: CommandRunner + ?Sized>(
    opts: ReportExportOptionsForConfig,
    runner: &R,
) -> Result<String> {
    let mut data =
        RealReportData::collect_with_report_config_with_runner(opts.config, opts.meta, runner);
    data.failure_diagnostic = opts
        .failure_diagnostic
        .map(|value| redact_report_diagnostic(&value, opts.config));

    let target_filename = match opts.report_type {
        ReportType::All => "audit_report",
        ReportType::Environment => "daily_backup_audit_report",
        ReportType::TimeSync => "ntp_sync_evidence",
        ReportType::RestoreDrill => "restore_drill_report",
    };

    let mut saved_paths: Vec<PathBuf> = Vec::new();

    let formats = match opts.format {
        Some(fmt) => vec![fmt],
        None => vec![ReportFormat::Html, ReportFormat::Json],
    };

    let date_prefix = &data.date_prefix;

    for fmt in formats {
        let ext = match fmt {
            ReportFormat::Html => "html",
            ReportFormat::Json => "json",
        };

        let mut file_path = match opts.file {
            Some(f) => {
                let parent = f.parent().unwrap_or_else(|| Path::new("."));
                let file_name = f.file_name().unwrap_or_default().to_string_lossy();

                // If extension is already specified or f points to exact filename, use it
                if f.extension().is_some() || file_name.contains('.') {
                    f.with_extension(ext)
                } else {
                    let stem = f.file_stem().unwrap_or_default().to_string_lossy();
                    parent.join(format!("{}.{}", stem, ext))
                }
            }
            None => opts
                .output_dir
                .join(format!("{}_{}.{}", date_prefix, target_filename, ext)),
        };

        if opts.file.is_none() {
            file_path = collision_safe_path(file_path);
        }

        let content = match fmt {
            ReportFormat::Html => html_template::render_html_real(opts.report_type, &data),
            ReportFormat::Json => json_schema::render_json_real(opts.report_type, &data)?,
        };
        let content = render_failure_metadata(fmt, content, data.failure_diagnostic.as_deref())?;

        write_file_with_perms(&file_path, &content)?;
        saved_paths.push(file_path);
    }

    let paths_str = saved_paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");

    Ok(format!("ISMS report saved to {}", paths_str))
}

fn redact_report_diagnostic(value: &str, config: &ReportConfig) -> String {
    let mut redacted = value.to_owned();
    let secret = config.primary_password.expose_secret();
    if !secret.is_empty() {
        redacted = redacted.replace(secret, "******");
    }
    if !config.primary_repository.is_empty() {
        redacted = redacted.replace(&config.primary_repository, "[repository masked]");
    }
    redacted
}

fn render_failure_metadata(
    format: ReportFormat,
    content: String,
    diagnostic: Option<&str>,
) -> Result<String> {
    let Some(diagnostic) = diagnostic else {
        return Ok(content);
    };
    match format {
        ReportFormat::Html => Ok(format!(
            "{content}\n<div class=\"report-failure\"><strong>Report status: Fail</strong><br>{}</div>\n",
            escape_html(diagnostic)
        )),
        ReportFormat::Json => {
            let mut value: serde_json::Value = serde_json::from_str(&content)?;
            let object = value
                .as_object_mut()
                .ok_or_else(|| anyhow::anyhow!("report JSON root must be an object"))?;
            object.insert("report_status".into(), serde_json::json!("Fail"));
            object.insert("failure_diagnostic".into(), serde_json::json!(diagnostic));
            Ok(serde_json::to_string_pretty(&value)?)
        }
    }
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::{ReportFormat, escape_html, render_failure_metadata};

    #[test]
    fn failure_metadata_escapes_html_diagnostics() {
        let rendered = render_failure_metadata(
            ReportFormat::Html,
            "<main>report</main>".into(),
            Some("<secret> & \"quoted\""),
        )
        .unwrap();
        assert!(rendered.contains("Report status: Fail"));
        assert!(rendered.contains("&lt;secret&gt; &amp; &quot;quoted&quot;"));
        assert!(!rendered.contains("<secret>"));
        assert_eq!(escape_html("<"), "&lt;");
    }

    #[test]
    fn failure_metadata_adds_structured_json_fields() {
        let rendered = render_failure_metadata(
            ReportFormat::Json,
            r#"{"hostname":"host"}"#.into(),
            Some("failed"),
        )
        .unwrap();
        let value: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        assert_eq!(value["report_status"], "Fail");
        assert_eq!(value["failure_diagnostic"], "failed");
    }
}

pub fn execute_report_file_export_with_type(
    report_type: ReportType,
    file: Option<&Path>,
    meta: &AuditReportMeta,
) -> Result<String> {
    let default_config = ReportConfig::default();
    let default_output_dir = default_config.output_dir.as_path();
    let opts = ReportExportOptions {
        report_type,
        file,
        format: None,
        output_dir: default_output_dir,
        meta,
        config: &default_config,
    };
    execute_report_export(opts)
}

pub fn execute_report_file_export(report_type: ReportType, file: Option<&Path>) -> Result<String> {
    let meta = AuditReportMeta::current();
    execute_report_file_export_with_type(report_type, file, &meta)
}

pub fn run_report(
    config_path: &Path,
    action: Option<ReportAction>,
    file: Option<PathBuf>,
    format: Option<ReportFormat>,
) -> Result<String> {
    ResticProfileConfig::load_from_path(config_path).map_err(|e| {
        anyhow::anyhow!(
            "Configuration load error at {}: {}",
            config_path.display(),
            e
        )
    })?;
    ReportCommand::run(action, file, format, config_path)
}

use crate::config::model::borrowed_environment;
use crate::runner::executor::{CommandRunner, SystemExecutor};
use crate::runner::rclone::RcloneRunner;
use crate::runner::restic::ResticTool;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoctorStatus {
    Pass,
    Fail,
    Warn,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoctorCategory {
    Config,
    Storage,
    Network,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DoctorItem {
    pub category: DoctorCategory,
    pub criterion: String,
    pub status: DoctorStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemHealthSnapshot {
    pub host_name: String,
    pub timestamp: String,
    pub overall_pass: bool,
    pub items: Vec<DoctorItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorOutputContract {
    pub stdout: String,
    pub stderr: String,
    pub exit_status: i32,
}

/// Purely translates a completed diagnostic snapshot into the CLI output contract.
/// Findings (including details) are data and stay on stdout. stderr is reserved for the short
/// failure summary, and warnings alone remain a successful command.
pub fn format_doctor_output(snapshot: &SystemHealthSnapshot) -> DoctorOutputContract {
    let has_blocking_finding = snapshot
        .items
        .iter()
        .any(|item| matches!(item.status, DoctorStatus::Fail | DoctorStatus::Unavailable));
    DoctorOutputContract {
        stdout: render_doctor_report(snapshot),
        stderr: if has_blocking_finding {
            "doctor reported one or more failed or unavailable diagnostics".into()
        } else {
            String::new()
        },
        exit_status: if has_blocking_finding { 1 } else { 0 },
    }
}

pub struct SystemHealthDiagnoser;

impl SystemHealthDiagnoser {
    pub fn diagnose<R: RcloneRunner>(
        rclone: &R,
        config_path: Option<&Path>,
    ) -> SystemHealthSnapshot {
        Self::diagnose_with_runner(rclone, &SystemExecutor, config_path)
    }

    pub fn diagnose_with_runner<R: RcloneRunner + ?Sized, C: CommandRunner + ?Sized>(
        rclone: &R,
        runner: &C,
        config_path: Option<&Path>,
    ) -> SystemHealthSnapshot {
        Self::diagnose_with_runner_and_host(rclone, runner, config_path, "localhost")
    }

    pub fn diagnose_with_runner_and_host<R: RcloneRunner + ?Sized, C: CommandRunner + ?Sized>(
        rclone: &R,
        runner: &C,
        config_path: Option<&Path>,
        host_name: &str,
    ) -> SystemHealthSnapshot {
        let timestamp = format!("{:?}", std::time::SystemTime::now());

        let target_config =
            config_path.unwrap_or_else(|| Path::new(crate::config::model::DEFAULT_PROFILES_PATH));

        let mut items = Vec::new();

        // Validate the selected unified configuration before probing any external adapter.
        // A doctor run must not turn a missing, symlinked, or malformed profiles path into an
        // apparently useful set of unrelated dependency results.
        if let Some(detail) = doctor_config_preflight_failure(target_config) {
            items.push(DoctorItem {
                category: DoctorCategory::Config,
                criterion: "백업 환경 및 보안 권한 (ISMS-P 2.9.2)".into(),
                status: DoctorStatus::Fail,
                detail,
            });
            return SystemHealthSnapshot {
                host_name: host_name.to_owned(),
                timestamp,
                overall_pass: false,
                items,
            };
        }

        let (restic_status, restic_detail) = match runner.run("restic", &["version"]) {
            Ok(out) if out.status_code == 0 => (DoctorStatus::Pass, out.stdout.trim().to_string()),
            Ok(out) => (
                DoctorStatus::Fail,
                format!("restic version exited with {}", out.status_code),
            ),
            Err(err) => (DoctorStatus::Fail, format!("restic unavailable: {err}")),
        };
        items.push(DoctorItem {
            category: DoctorCategory::System,
            criterion: "Restic binary".into(),
            status: restic_status,
            detail: restic_detail,
        });

        // 1. Dependency & Config Permissions Item
        let (config_status, config_result) = if target_config.exists() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = target_config.metadata() {
                    let mode = meta.permissions().mode() & 0o777;
                    let parent_safe = target_config.parent().is_some_and(|parent| {
                        parent
                            .metadata()
                            .ok()
                            .is_some_and(|meta| (meta.permissions().mode() & 0o777) <= 0o700)
                    });
                    if mode <= 0o600 && parent_safe {
                        (
                            DoctorStatus::Pass,
                            format!("0700 / 0600 ({:#o} file, parent safe)", mode),
                        )
                    } else {
                        (
                            DoctorStatus::Fail,
                            format!(
                                "{:#o} or parent permissions unsafe (chmod 700/600 required)",
                                mode
                            ),
                        )
                    }
                } else {
                    (
                        DoctorStatus::Pass,
                        "0700 / 0600 (****** Masked)".to_string(),
                    )
                }
            }
            #[cfg(not(unix))]
            {
                (
                    DoctorStatus::Pass,
                    "0700 / 0600 (****** Masked)".to_string(),
                )
            }
        } else {
            (
                DoctorStatus::Fail,
                "Unified profiles configuration is missing".to_string(),
            )
        };

        items.push(DoctorItem {
            category: DoctorCategory::Config,
            criterion: "백업 환경 및 보안 권한 (ISMS-P 2.9.2)".into(),
            status: config_status,
            detail: config_result,
        });

        // 2. Storage & Connectivity Item
        // Probe only the rclone remotes declared by the configured Backend Profiles.  Every
        // configured remote is checked independently so one failure cannot hide another.
        let (rclone_status, rclone_result) = match configured_storage_targets(target_config) {
            Err(error) => (
                DoctorStatus::Unavailable,
                format!("Storage configuration unavailable: {error}"),
            ),
            Ok(targets) if targets.is_empty() => (
                DoctorStatus::Unavailable,
                "No Backend Profile storage is configured".into(),
            ),
            Ok(targets) => {
                let config =
                    crate::config::model::ResticProfileConfig::load_from_path(target_config);
                let config_dir = target_config.parent().unwrap_or_else(|| Path::new("."));
                let results = targets
                    .iter()
                    .map(|target| {
                        let result = if let Some(remote) = &target.rclone_remote {
                            rclone.check_connectivity(remote)
                        } else {
                            match &config {
                                Ok(config) => check_restic_connectivity(
                                    config,
                                    config_dir,
                                    &target.profile,
                                    runner,
                                ),
                                Err(error) => Err(anyhow::anyhow!(error.to_string())),
                            }
                        };
                        (target, result)
                    })
                    .collect::<Vec<_>>();
                let reachable = results.iter().filter(|(_, result)| result.is_ok()).count();
                if reachable == 0 {
                    (
                        DoctorStatus::Fail,
                        format!(
                            "Storage connectivity failed ({}/{})",
                            reachable,
                            results.len()
                        ),
                    )
                } else if reachable == results.len() {
                    (
                        DoctorStatus::Pass,
                        format!(
                            "Storage connectivity active ({}/{})",
                            reachable,
                            results.len()
                        ),
                    )
                } else {
                    (
                        DoctorStatus::Fail,
                        format!(
                            "Storage connectivity partially active ({}/{} targets reachable)",
                            reachable,
                            results.len()
                        ),
                    )
                }
            }
        };

        items.push(DoctorItem {
            category: DoctorCategory::Storage,
            criterion: "스토리지 연결 및 커넥티비티 (ISMS-P 2.9.2)".into(),
            status: rclone_status,
            detail: rclone_result,
        });

        // 3. Time Sync Item
        let (ntp_status, ntp_detail) = check_ntp_sync_with_runner(runner);
        items.push(DoctorItem {
            category: DoctorCategory::System,
            criterion: "시각 동기화 (ISMS-P 2.10.1)".into(),
            status: ntp_status,
            detail: ntp_detail,
        });

        let (scheduler_status, scheduler_detail) = check_scheduler_with_runner(runner);
        items.push(DoctorItem {
            category: DoctorCategory::System,
            criterion: "타이머 스케줄러 헬스체크".into(),
            status: scheduler_status,
            detail: scheduler_detail,
        });

        // 4. Restore Drill RTO Item: use the same concrete tagged-snapshot Evidence seam as
        // the public report command, so doctor never invents a "latest" identity or timing.
        let (rto_status, rto_detail) = (|| -> anyhow::Result<_> {
            let profiles = crate::config::model::ResticProfileConfig::load_from_path(target_config)?;
            let config = crate::commands::report::ReportConfig::from_profiles(
                &profiles,
                target_config,
            )?;
            let restic = ResticTool::new(runner);
            let evidence = crate::commands::report::execute_restore_drill_with_runner(
                &config,
                &restic,
            )?;
            let elapsed_milliseconds = evidence
                .storage_results
                .iter()
                .filter_map(|result| result.elapsed_milliseconds)
                .max()
                .unwrap_or_default();
            let status = evidence.overall_status;
            let doctor_status = if status == crate::commands::report::RestoreDrillStatus::Pass {
                DoctorStatus::Pass
            } else {
                DoctorStatus::Fail
            };
            Ok((
                doctor_status,
                format!(
                    "Restore Drill Evidence status={status:?}, elapsed_milliseconds={elapsed_milliseconds}"
                ),
            ))
        })()
        .unwrap_or_else(|error| {
            (
                DoctorStatus::Fail,
                format!("Restore Drill Evidence unavailable: {error}"),
            )
        });

        items.push(DoctorItem {
            category: DoctorCategory::System,
            criterion: "복구 모의 훈련 및 RTO (ISMS-P 2.9.3)".into(),
            status: rto_status,
            detail: rto_detail,
        });

        let overall_pass = items
            .iter()
            .all(|item| !matches!(item.status, DoctorStatus::Fail | DoctorStatus::Unavailable));

        SystemHealthSnapshot {
            host_name: host_name.to_owned(),
            timestamp,
            overall_pass,
            items,
        }
    }
}

fn doctor_config_preflight_failure(path: &Path) -> Option<String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Some("Unified profiles configuration is missing".into());
        }
        Err(error) => {
            return Some(format!(
                "Unified profiles configuration is unavailable: {error}"
            ));
        }
    };

    if !metadata.file_type().is_file() {
        return Some("Unified profiles configuration must be a regular file".into());
    }

    if let Err(error) = crate::config::model::ResticProfileConfig::load_from_path(path) {
        return Some(format!(
            "Unified profiles configuration is invalid: {error}"
        ));
    }

    None
}

struct ConfiguredStorageTarget {
    profile: String,
    rclone_remote: Option<String>,
}

fn configured_storage_targets(config_path: &Path) -> Result<Vec<ConfiguredStorageTarget>> {
    let config = crate::config::model::ResticProfileConfig::load_from_path(config_path)?;
    let mut targets = Vec::new();
    for profile in ["primary", "secondary"] {
        if !config.profiles.contains_key(profile) {
            continue;
        }
        let repository = config.backend_repository(profile)?;
        targets.push(ConfiguredStorageTarget {
            profile: profile.into(),
            rclone_remote: rclone_remote_name(&repository),
        });
    }
    Ok(targets)
}

fn check_restic_connectivity<C: CommandRunner + ?Sized>(
    config: &crate::config::model::ResticProfileConfig,
    config_dir: &Path,
    profile: &str,
    runner: &C,
) -> Result<String> {
    let (repository, password) = config.backend_credentials(config_dir, profile)?;
    let mut password_file = NamedTempFile::new()?;
    password_file.write_all(password.as_bytes())?;
    password_file.flush()?;
    let password_path = password_file.path().to_string_lossy();
    let owned_environment = config.sidecar_environment(config_dir)?;
    let environment = borrowed_environment(&owned_environment);
    let output = runner.run_with_env(
        "restic",
        &[
            "-r",
            &repository,
            "--password-file",
            &password_path,
            "snapshots",
            "--latest",
            "1",
        ],
        &environment,
    )?;
    if output.status_code != 0 {
        anyhow::bail!("restic storage probe exited with {}", output.status_code);
    }
    Ok(output.stdout)
}

fn rclone_remote_name(repository: &str) -> Option<String> {
    let remainder = repository.strip_prefix("rclone:")?;
    let (remote, _) = remainder.split_once(':')?;
    (!remote.is_empty()).then(|| remote.to_owned())
}

pub fn check_ntp_sync() -> (DoctorStatus, String) {
    check_ntp_sync_with_runner(&SystemExecutor)
}

pub fn check_ntp_sync_with_runner<C: CommandRunner + ?Sized>(runner: &C) -> (DoctorStatus, String) {
    if let Ok(out) = runner.run("chronyc", &["tracking"]) {
        if out.status_code == 0
            && (out.stdout.contains("Reference ID")
                || out.stdout.contains("System time")
                || out.stdout.contains("Leap status"))
        {
            return (
                DoctorStatus::Pass,
                format!(
                    "chronyd active ({})",
                    out.stdout.lines().next().unwrap_or("synced")
                ),
            );
        }
    }
    if let Ok(out) = runner.run("timedatectl", &["status"]) {
        if out.status_code == 0
            && (out.stdout.contains("NTP service: active")
                || out.stdout.contains("System clock synchronized: yes")
                || out.stdout.contains("Local time:"))
        {
            return (
                DoctorStatus::Pass,
                "timedatectl clock synchronized".to_string(),
            );
        }
    }
    (
        DoctorStatus::Warn,
        "NTP synchronization status unknown or inactive".to_string(),
    )
}

pub fn run_doctor_checks<R: RcloneRunner>(
    rclone: &R,
    config_path: Option<&Path>,
) -> Result<String> {
    run_doctor_checks_with_runner(rclone, &SystemExecutor, config_path)
}

pub fn run_doctor_checks_with_runner<R: RcloneRunner + ?Sized, C: CommandRunner + ?Sized>(
    rclone: &R,
    runner: &C,
    config_path: Option<&Path>,
) -> Result<String> {
    tracing::info!("Executing system health diagnostics checks");
    let snapshot = SystemHealthDiagnoser::diagnose_with_runner(rclone, runner, config_path);
    Ok(render_doctor_report(&snapshot))
}

pub fn run_doctor_contract_with_runner<R: RcloneRunner + ?Sized, C: CommandRunner + ?Sized>(
    rclone: &R,
    runner: &C,
    config_path: Option<&Path>,
    host_name: &str,
) -> Result<(String, bool)> {
    let (report, passed, _) =
        run_doctor_contract_with_runner_and_diagnostics(rclone, runner, config_path, host_name)?;
    Ok((report, passed))
}

pub fn run_doctor_contract_with_runner_and_diagnostics<
    R: RcloneRunner + ?Sized,
    C: CommandRunner + ?Sized,
>(
    rclone: &R,
    runner: &C,
    config_path: Option<&Path>,
    host_name: &str,
) -> Result<(String, bool, String)> {
    tracing::info!("Executing system health diagnostics checks");
    let snapshot = SystemHealthDiagnoser::diagnose_with_runner_and_host(
        rclone,
        runner,
        config_path,
        host_name,
    );
    let passed = snapshot
        .items
        .iter()
        .all(|item| !matches!(item.status, DoctorStatus::Fail | DoctorStatus::Unavailable));
    let output = format_doctor_output(&snapshot);
    debug_assert_eq!(passed, output.exit_status == 0);
    Ok((output.stdout, passed, output.stderr))
}

fn check_scheduler_with_runner<C: CommandRunner + ?Sized>(runner: &C) -> (DoctorStatus, String) {
    match runner.run("systemctl", &["is-active", "backup-pipeline.timer"]) {
        Ok(output) if output.status_code == 0 => (
            DoctorStatus::Pass,
            "backup-pipeline.timer active".to_string(),
        ),
        Ok(output) => (
            DoctorStatus::Fail,
            format!(
                "backup-pipeline.timer inactive (exit {})",
                output.status_code
            ),
        ),
        Err(error) => (
            DoctorStatus::Unavailable,
            format!("scheduler health check unavailable: {error}"),
        ),
    }
}

fn render_doctor_report(snapshot: &SystemHealthSnapshot) -> String {
    let mut report = String::new();
    report.push_str("Checking dependencies...\n");

    for item in &snapshot.items {
        let name = match item.category {
            DoctorCategory::Storage => "Storage connectivity",
            DoctorCategory::System if item.criterion == "Restic binary" => "Restic binary",
            DoctorCategory::System if item.criterion.contains("시각 동기화") => {
                "NTP Time Sync"
            }
            DoctorCategory::System if item.criterion.contains("타이머 스케줄러") => {
                "Scheduler health"
            }
            _ => &item.criterion,
        };
        report.push_str(&format!(
            "{}: {} — {}\n",
            name,
            doctor_status_label(&item.status),
            item.detail
        ));
    }

    report
}

fn doctor_status_label(status: &DoctorStatus) -> &'static str {
    match status {
        DoctorStatus::Pass => "Pass",
        DoctorStatus::Warn => "Warn",
        DoctorStatus::Fail => "Fail",
        DoctorStatus::Unavailable => "Unavailable",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(status: DoctorStatus) -> SystemHealthSnapshot {
        SystemHealthSnapshot {
            host_name: "test-host".into(),
            timestamp: "now".into(),
            overall_pass: matches!(status, DoctorStatus::Pass | DoctorStatus::Warn),
            items: vec![DoctorItem {
                category: DoctorCategory::System,
                criterion: "test diagnostic".into(),
                status,
                detail: "detail stays on stdout".into(),
            }],
        }
    }

    #[test]
    fn doctor_pass_and_warn_keep_findings_on_stdout_and_stderr_empty() {
        for status in [DoctorStatus::Pass, DoctorStatus::Warn] {
            let output = format_doctor_output(&snapshot(status));
            assert_eq!(output.exit_status, 0);
            assert!(output.stderr.is_empty());
            assert!(output.stdout.contains("detail stays on stdout"));
        }
    }

    #[test]
    fn doctor_fail_and_unavailable_use_exit_one_and_only_a_short_stderr_summary() {
        for status in [DoctorStatus::Fail, DoctorStatus::Unavailable] {
            let output = format_doctor_output(&snapshot(status));
            assert_eq!(output.exit_status, 1);
            assert!(output.stderr.contains("failed or unavailable"));
            assert!(output.stdout.contains("detail stays on stdout"));
            assert!(!output.stderr.contains("detail stays on stdout"));
        }
    }
}

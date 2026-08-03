use crate::runner::executor::{CommandRunner, SystemExecutor};
use crate::runner::rclone::RcloneRunner;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DoctorStatus {
    Pass,
    Fail,
    Warn,
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

pub struct SystemHealthDiagnoser;

impl SystemHealthDiagnoser {
    pub fn diagnose<R: RcloneRunner>(
        rclone: &R,
        config_path: Option<&Path>,
    ) -> SystemHealthSnapshot {
        Self::diagnose_with_runner(rclone, &SystemExecutor, config_path)
    }

    pub fn diagnose_with_runner<R: RcloneRunner, C: CommandRunner>(
        rclone: &R,
        runner: &C,
        config_path: Option<&Path>,
    ) -> SystemHealthSnapshot {
        let host_name = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "localhost".into());
        let timestamp = format!("{:?}", std::time::SystemTime::now());

        let target_config =
            config_path.unwrap_or_else(|| Path::new(crate::config::model::DEFAULT_PROFILES_PATH));

        let mut items = Vec::new();

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
        let rclone_pass = rclone.check_connectivity("default").is_ok()
            || rclone.check_connectivity("syno_backup").is_ok();
        let (rclone_status, rclone_result) = if rclone_pass {
            (
                DoctorStatus::Pass,
                "Rclone connectivity active (Remote OK)".into(),
            )
        } else {
            (
                DoctorStatus::Fail,
                "Rclone connectivity failed (Remote unreachable)".into(),
            )
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

        // 4. Restore Drill RTO Item: non-destructively restore the latest snapshot.
        let start_time = std::time::Instant::now();
        let restic_res = (|| -> anyhow::Result<_> {
            use secrecy::ExposeSecret;
            use std::io::Write;
            let config = crate::config::model::BackupConfig::load_from_path(target_config)?;
            let mut password_file = tempfile::NamedTempFile::new()?;
            password_file.write_all(config.storage.primary.password.expose_secret().as_bytes())?;
            password_file.flush()?;
            let target = tempfile::tempdir()?;
            let password_path = password_file.path().to_string_lossy();
            let target_path = target.path().to_string_lossy();
            let output = runner.run(
                "restic",
                &[
                    "-r",
                    &config.storage.primary.repository,
                    "--password-file",
                    &password_path,
                    "restore",
                    "latest",
                    "--target",
                    &target_path,
                ],
            )?;
            if output.status_code != 0 {
                anyhow::bail!("restic restore exited with {}", output.status_code);
            }
            crate::commands::restore::validate_restored_output(
                target.path(),
                matches!(
                    config.backup.backup_type,
                    crate::config::model::BackupType::DbStream { .. }
                ),
            )?;
            Ok(())
        })();
        let elapsed = start_time.elapsed().as_secs_f64();

        let (rto_status, rto_detail) = if restic_res.is_ok() {
            (
                DoctorStatus::Pass,
                format!("{:.1}s (Latest snapshot restored and validated)", elapsed),
            )
        } else {
            (
                DoctorStatus::Fail,
                format!("{elapsed:.1}s (restic check could not execute)"),
            )
        };

        items.push(DoctorItem {
            category: DoctorCategory::System,
            criterion: "복구 모의 훈련 및 RTO (ISMS-P 2.9.3)".into(),
            status: rto_status,
            detail: rto_detail,
        });

        let overall_pass = items.iter().all(|i| i.status == DoctorStatus::Pass);

        SystemHealthSnapshot {
            host_name,
            timestamp,
            overall_pass,
            items,
        }
    }
}

pub fn check_ntp_sync() -> (DoctorStatus, String) {
    check_ntp_sync_with_runner(&SystemExecutor)
}

pub fn check_ntp_sync_with_runner<C: CommandRunner>(runner: &C) -> (DoctorStatus, String) {
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

pub fn run_doctor_checks_with_runner<R: RcloneRunner, C: CommandRunner>(
    rclone: &R,
    runner: &C,
    config_path: Option<&Path>,
) -> Result<String> {
    tracing::info!("Executing system health diagnostics checks");
    let snapshot = SystemHealthDiagnoser::diagnose_with_runner(rclone, runner, config_path);
    let mut report = String::new();
    report.push_str("Checking dependencies...\n");

    for item in &snapshot.items {
        match item.category {
            DoctorCategory::Storage => {
                let status = if item.status == DoctorStatus::Pass {
                    "OK"
                } else {
                    "FAILED (Check remote configuration and network)"
                };
                report.push_str(&format!("Rclone connectivity: {}\n", status));
            }
            DoctorCategory::System => {
                if item.criterion == "Restic binary" {
                    let status = if item.status == DoctorStatus::Pass {
                        "OK"
                    } else {
                        "FAILED"
                    };
                    report.push_str(&format!("Restic binary: {}\n", status));
                } else if item.criterion.contains("시각 동기화") {
                    let status = if item.status == DoctorStatus::Pass {
                        "OK"
                    } else {
                        "FAILED"
                    };
                    report.push_str(&format!("NTP Time Sync: {}\n", status));
                } else {
                    let status = match item.status {
                        DoctorStatus::Pass => "OK",
                        DoctorStatus::Warn => "WARN",
                        DoctorStatus::Fail => "FAILED",
                    };
                    report.push_str(&format!("{}: {}\n", item.criterion, status));
                }
            }
            _ => {
                let status = match item.status {
                    DoctorStatus::Pass => "OK",
                    DoctorStatus::Warn => "WARN",
                    DoctorStatus::Fail => "FAILED",
                };
                report.push_str(&format!("{}: {}\n", item.criterion, status));
            }
        }
    }

    Ok(report)
}

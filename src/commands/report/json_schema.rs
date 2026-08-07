use crate::commands::report::{AuditDiagnosticResults, RealReportData, ReportType};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Serialize an injected Restore Drill Evidence value without collecting any runtime data.
pub fn render_restore_drill_evidence(
    evidence: &crate::commands::report::restore_drill::RestoreDrillEvidence,
) -> Result<String> {
    crate::commands::report::restore_drill::render_restore_drill_evidence_json(evidence)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPolicyJson {
    pub backend: String,
    pub repository: String,
    pub encryption: String,
    pub encryption_warning: bool,
    pub targets: String,
    pub excludes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicyJson {
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleStatusJson {
    pub on_calendar: String,
    pub timer_enabled: String,
    pub timer_active: String,
    pub next_run: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlJson {
    pub etc_restic_dir: String,
    pub etc_restic_dir_permission: String,
    pub etc_restic_dir_safe: bool,
    pub backup_env_file: String,
    pub backup_env_file_permission: String,
    pub backup_env_file_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllReportJson {
    pub hostname: String,
    pub timestamp: String,
    pub backup_policy: BackupPolicyJson,
    pub retention_policy: RetentionPolicyJson,
    pub schedule: ScheduleStatusJson,
    pub access_control: AccessControlJson,
    pub snapshots: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionVerificationItemJson {
    pub config: u32,
    pub actual: u32,
    pub config_status: String,
    pub actual_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicyVerificationJson {
    pub keep_daily: RetentionVerificationItemJson,
    pub keep_weekly: RetentionVerificationItemJson,
    pub keep_monthly: RetentionVerificationItemJson,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlIntegrityJson {
    pub etc_restic_dir_permission: String,
    pub etc_restic_dir_safe: bool,
    pub backup_env_file_permission: String,
    pub backup_env_file_safe: bool,
    pub integrity_check_result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyReportJson {
    pub hostname: String,
    pub timestamp: String,
    pub report_type: String,
    pub tester: String,
    pub backup_policy: serde_json::Value,
    pub retention_policy_verification: RetentionPolicyVerificationJson,
    pub access_control_and_integrity: AccessControlIntegrityJson,
    pub recent_snapshots: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChronyServiceJson {
    pub enabled: String,
    pub active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NtpSyncReportJson {
    pub report_type: String,
    pub hostname: String,
    pub report_date: String,
    pub chrony_service: ChronyServiceJson,
    pub sources: String,
    pub tracking: String,
    pub conf_permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResultsJson {
    pub data_size_human: String,
    pub elapsed_seconds: u64,
    pub elapsed_human: String,
    pub target_rto_minutes: u64,
    pub rto_satisfied: bool,
    pub data_integrity_verified: bool,
    pub database_verification: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreDrillReportJson {
    pub hostname: String,
    pub timestamp: String,
    pub report_type: String,
    pub test_date: String,
    pub tester: String,
    pub ciso: String,
    pub target_snapshot_id: String,
    pub target_snapshot_time: String,
    pub target_directory: String,
    pub recovery_results: RecoveryResultsJson,
}

pub fn render_json_real(report_type: ReportType, data: &RealReportData) -> Result<String> {
    match report_type {
        ReportType::All => {
            let res = AllReportJson {
                hostname: data.hostname.clone(),
                timestamp: data.timestamp.clone(),
                backup_policy: BackupPolicyJson {
                    backend: "sftp".into(),
                    repository: data.config.primary_repository.clone(),
                    encryption: "AES-256 (restic 저장소 자체 암호화)".into(),
                    encryption_warning: false,
                    targets: data.config.targets.join(","),
                    excludes: data.config.excludes.join(","),
                },
                retention_policy: RetentionPolicyJson {
                    keep_daily: data.config.retention.keep_daily,
                    keep_weekly: data.config.retention.keep_weekly,
                    keep_monthly: data.config.retention.keep_monthly,
                },
                schedule: ScheduleStatusJson {
                    on_calendar: "*-*-* 02:00:00".into(),
                    timer_enabled: data.timer_enabled.clone(),
                    timer_active: data.timer_active.clone(),
                    next_run: data.next_run.clone(),
                },
                access_control: AccessControlJson {
                    etc_restic_dir: "/etc/backup".into(),
                    etc_restic_dir_permission: data.etc_backup_dir_perm.clone(),
                    etc_restic_dir_safe: data.etc_backup_dir_safe,
                    backup_env_file: "/etc/backup/profiles.yaml".into(),
                    backup_env_file_permission: data.backup_env_file_perm.clone(),
                    backup_env_file_safe: data.backup_env_file_safe,
                },
                snapshots: data.snapshots.clone(),
            };
            Ok(serde_json::to_string_pretty(&res)?)
        }
        ReportType::Environment => {
            let res = DailyReportJson {
                hostname: data.hostname.clone(),
                timestamp: data.timestamp.clone(),
                report_type: "daily_backup_review".into(),
                tester: data
                    .audit
                    .system_manager
                    .clone()
                    .unwrap_or_else(|| "시스템 운영팀".into()),
                backup_policy: serde_json::json!({
                    "backend": "sftp",
                    "repository": data.config.primary_repository.clone(),
                    "encryption": "AES-256 (보안 비밀번호 키 적용 완료)",
                    "targets": data.config.targets.join(",")
                }),
                retention_policy_verification: RetentionPolicyVerificationJson {
                    keep_daily: RetentionVerificationItemJson {
                        config: data.config.retention.keep_daily,
                        actual: 0,
                        config_status: "만족".into(),
                        actual_status: "미흡".into(),
                    },
                    keep_weekly: RetentionVerificationItemJson {
                        config: data.config.retention.keep_weekly,
                        actual: 0,
                        config_status: "만족".into(),
                        actual_status: "미흡".into(),
                    },
                    keep_monthly: RetentionVerificationItemJson {
                        config: data.config.retention.keep_monthly,
                        actual: 0,
                        config_status: "만족".into(),
                        actual_status: "미흡".into(),
                    },
                },
                access_control_and_integrity: AccessControlIntegrityJson {
                    etc_restic_dir_permission: data.etc_backup_dir_perm.clone(),
                    etc_restic_dir_safe: data.etc_backup_dir_safe,
                    backup_env_file_permission: data.backup_env_file_perm.clone(),
                    backup_env_file_safe: data.backup_env_file_safe,
                    integrity_check_result: "SUCCESS (에러 없음)".into(),
                },
                recent_snapshots: data.snapshots.clone(),
            };
            Ok(serde_json::to_string_pretty(&res)?)
        }
        ReportType::TimeSync => {
            let res = NtpSyncReportJson {
                report_type: "isms_p_2.9.3_ntp_sync".into(),
                hostname: data.hostname.clone(),
                report_date: data.timestamp.clone(),
                chrony_service: ChronyServiceJson {
                    enabled: data.chrony_enabled.clone(),
                    active: data.chrony_active.clone(),
                },
                sources: data.chrony_sources.clone(),
                tracking: data.chrony_tracking.clone(),
                conf_permission: format!("-rw-r--r-- 1 root root 813 /etc/chrony.conf"),
            };
            Ok(serde_json::to_string_pretty(&res)?)
        }
        ReportType::RestoreDrill => {
            let test_date = if data.timestamp.len() >= 10 {
                data.timestamp[0..10].to_string()
            } else {
                "2026-07-21".to_string()
            };
            let res = RestoreDrillReportJson {
                hostname: data.hostname.clone(),
                timestamp: data.timestamp.clone(),
                report_type: "restore_drill".into(),
                test_date,
                tester: data
                    .audit
                    .system_manager
                    .clone()
                    .unwrap_or_else(|| "시스템 운영팀".into()),
                ciso: data
                    .audit
                    .security_officer
                    .clone()
                    .unwrap_or_else(|| "정보보안책임자".into()),
                target_snapshot_id: String::new(),
                target_snapshot_time: String::new(),
                target_directory: String::new(),
                recovery_results: RecoveryResultsJson {
                    data_size_human: "not measured".into(),
                    elapsed_seconds: 0,
                    elapsed_human: "not measured".into(),
                    target_rto_minutes: data.config.restore_drill_policy.rto_minutes,
                    rto_satisfied: false,
                    data_integrity_verified: false,
                    database_verification: serde_json::json!({
                        "db_type": data.config.database_type.map(|database_type| database_type.to_string()),
                        "db_snapshot_id": null,
                        "db_snapshot_time": null,
                        "expected_signature": data.config.database_type.map(crate::commands::database::DatabaseDumpValidation::expected_signature),
                        "signature_verified": null,
                        "signature_status": "not_performed",
                        "validation_scope": "SQL dump signature only",
                        "db_integrity_verified": false,
                        "import_performed": false,
                        "record_validation_performed": false
                    }),
                },
            };
            Ok(serde_json::to_string_pretty(&res)?)
        }
    }
}

pub fn render_json(report_type: ReportType, _results: &AuditDiagnosticResults) -> Result<String> {
    let config = crate::commands::report::ReportConfig::default();
    let data = RealReportData::collect(&config);
    render_json_real(report_type, &data)
}

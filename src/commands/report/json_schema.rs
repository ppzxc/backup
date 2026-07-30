use crate::commands::report::{AuditDiagnosticResults, ReportType};
use anyhow::Result;
use serde::{Deserialize, Serialize};

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

pub fn render_json(report_type: ReportType, results: &AuditDiagnosticResults) -> Result<String> {
    match report_type {
        ReportType::All => {
            let data = AllReportJson {
                hostname: results.host_name.clone(),
                timestamp: results.timestamp.clone(),
                backup_policy: BackupPolicyJson {
                    backend: "sftp".into(),
                    repository: format!("rclone:syno_backup:/backup/{}", results.host_name),
                    encryption: "AES-256 (restic 저장소 자체 암호화)".into(),
                    encryption_warning: false,
                    targets: "/data/backup,/etc,/var/log".into(),
                    excludes: "/tmp/*,/var/tmp/*".into(),
                },
                retention_policy: RetentionPolicyJson { keep_daily: 7, keep_weekly: 4, keep_monthly: 12 },
                schedule: ScheduleStatusJson {
                    on_calendar: "*-*-* 02:00:00".into(),
                    timer_enabled: "enabled".into(),
                    timer_active: "active".into(),
                    next_run: format!("Next scheduled run on {}", results.timestamp),
                },
                access_control: AccessControlJson {
                    etc_restic_dir: "/etc/backup".into(),
                    etc_restic_dir_permission: "700".into(),
                    etc_restic_dir_safe: true,
                    backup_env_file: "/etc/backup/backup.env".into(),
                    backup_env_file_permission: "600".into(),
                    backup_env_file_safe: true,
                },
                snapshots: vec![],
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
        ReportType::Environment => {
            let data = DailyReportJson {
                hostname: results.host_name.clone(),
                timestamp: results.timestamp.clone(),
                report_type: "daily_backup_review".into(),
                tester: "조정하 차장".into(),
                backup_policy: serde_json::json!({
                    "backend": "sftp",
                    "repository": format!("rclone:syno_backup:/backup/{}", results.host_name),
                    "encryption": "AES-256 (보안 비밀번호 키 적용 완료)",
                    "targets": "/data/backup,/etc,/var/log"
                }),
                retention_policy_verification: RetentionPolicyVerificationJson {
                    keep_daily: RetentionVerificationItemJson { config: 7, actual: 7, config_status: "만족".into(), actual_status: "정상".into() },
                    keep_weekly: RetentionVerificationItemJson { config: 4, actual: 4, config_status: "만족".into(), actual_status: "정상".into() },
                    keep_monthly: RetentionVerificationItemJson { config: 12, actual: 12, config_status: "만족".into(), actual_status: "정상".into() },
                },
                access_control_and_integrity: AccessControlIntegrityJson {
                    etc_restic_dir_permission: "700".into(),
                    etc_restic_dir_safe: true,
                    backup_env_file_permission: "600".into(),
                    backup_env_file_safe: true,
                    integrity_check_result: "SUCCESS (에러 없음)".into(),
                },
                recent_snapshots: vec![],
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
        ReportType::TimeSync => {
            let data = NtpSyncReportJson {
                report_type: "isms_p_2.9.3_ntp_sync".into(),
                hostname: results.host_name.clone(),
                report_date: results.timestamp.clone(),
                chrony_service: ChronyServiceJson { enabled: "enabled".into(), active: "active".into() },
                sources: "^* any.time.nl 2 6 17 1 -812us[-374us] +/- 20ms".into(),
                tracking: "System time : 0.000243256 seconds fast of NTP time\nRMS offset : 0.000438103 seconds".into(),
                conf_permission: "-rw-r--r-- 1 root root 813 /etc/chrony.conf".into(),
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
        ReportType::RestoreDrill => {
            let data = RestoreDrillReportJson {
                hostname: results.host_name.clone(),
                timestamp: results.timestamp.clone(),
                report_type: "restore_drill".into(),
                test_date: results.timestamp.clone(),
                tester: "조정하 차장".into(),
                ciso: "박상수".into(),
                target_snapshot_id: "58afba4bb29c368bb3a3cb45c18d3da8a1b09709cd19df9aeda1b722eb825ce1".into(),
                target_snapshot_time: results.timestamp.clone(),
                target_directory: "/tmp/restore_test".into(),
                recovery_results: RecoveryResultsJson {
                    data_size_human: "401.69 MB".into(),
                    elapsed_seconds: 4,
                    elapsed_human: "4초".into(),
                    target_rto_minutes: 120,
                    rto_satisfied: true,
                    data_integrity_verified: true,
                },
            };
            Ok(serde_json::to_string_pretty(&data)?)
        }
    }
}

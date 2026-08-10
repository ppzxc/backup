use backup::cli::{AdapterSelection, AdapterSet, Cli, CliRuntimeContext, SchedulerMode, dispatch};
use backup::commands::report::{
    AuditReportMeta, ReportAction, ReportCommand, ReportFormat, ReportType,
    execute_report_file_export, render_html_isms_report, render_html_isms_report_with_type,
};
use backup::i18n::Language;
use clap::Parser;
mod support;
use std::fs;
use support::{
    MockExecutor, MockRcloneRunner, MockResticProfileRunner, MockResticRunner, MockScheduler,
};
use tempfile::tempdir;

#[test]
fn test_report_html_rendering() {
    let html = render_html_isms_report("test-host", "2026-07-23");
    assert!(html.contains("일일 백업 결과 및 보안 설정 검토 보고서"));
    assert!(html.contains("test-host"));
}

#[test]
fn test_report_types_rendering() {
    let html_env = render_html_isms_report_with_type(
        ReportType::Environment,
        &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"),
    );
    assert!(html_env.contains("일일 백업 결과 및 보안 설정 검토 보고서"));

    let html_ts = render_html_isms_report_with_type(
        ReportType::TimeSync,
        &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"),
    );
    assert!(html_ts.contains("ISMS-P 2.9.3 시각 동기화"));

    let html_rd = render_html_isms_report_with_type(
        ReportType::RestoreDrill,
        &backup::commands::report::AuditReportMeta::new("host-1", "2026-07-23"),
    );
    assert!(html_rd.contains("백업 데이터 복구 및 정합성 테스트 결과 보고서"));
}

#[test]
fn test_report_file_export_with_path() {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("sub").join("report.html");

    let msg = execute_report_file_export(ReportType::Environment, Some(&file_path)).unwrap();
    assert!(msg.contains("ISMS report saved to"));
    assert!(file_path.exists());

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("일일 백업 결과 및 보안 설정 검토 보고서"));
}

#[test]
fn test_report_type_all_rendering() {
    let html_all = render_html_isms_report_with_type(
        ReportType::All,
        &backup::commands::report::AuditReportMeta::new("host-all", "2026-07-30"),
    );
    assert!(html_all.contains("종합 백업 보안 설정 검토 보고서"));
    assert!(html_all.contains("백업 정책 및 대상 경로 정보"));
    assert!(html_all.contains("백업 보존 주기 정책"));
    assert!(html_all.contains("시스템 스케줄러 & 접근 통제"));
}

#[test]
fn test_report_file_export_dual_format_by_default() {
    use backup::commands::report::{ReportExportOptions, execute_report_export};
    let dir = tempdir().unwrap();
    let base_file = dir.path().join("audit_report");
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-07-30");
    let default_config = backup::commands::report::ReportConfig::default();

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::All,
        file: Some(&base_file),
        format: None,
        output_dir: dir.path(),
        meta: &meta,
        config: &default_config,
    })
    .unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let html_file = dir.path().join("audit_report.html");
    let json_file = dir.path().join("audit_report.json");

    assert!(html_file.exists(), "HTML report should be generated");
    assert!(json_file.exists(), "JSON report should be generated");

    let json_str = fs::read_to_string(&json_file).unwrap();
    assert!(json_str.contains("backup_policy"));
    assert!(json_str.contains("retention_policy"));
}

#[test]
fn test_report_file_export_single_format_when_specified() {
    use backup::commands::report::{ReportExportOptions, ReportFormat, execute_report_export};
    let dir = tempdir().unwrap();
    let base_file = dir.path().join("audit_report.json");
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-07-30");
    let default_config = backup::commands::report::ReportConfig::default();

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::Environment,
        file: Some(&base_file),
        format: Some(ReportFormat::Json),
        output_dir: dir.path(),
        meta: &meta,
        config: &default_config,
    })
    .unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let json_file = dir.path().join("audit_report.json");
    let html_file = dir.path().join("audit_report.html");

    assert!(json_file.exists(), "JSON report should be generated");
    assert!(
        !html_file.exists(),
        "HTML report should NOT be generated when format=json"
    );
}

#[test]
fn test_report_file_export_default_directory_when_file_none() {
    use backup::commands::report::{ReportExportOptions, execute_report_export};
    let dir = tempdir().unwrap();
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-07-30");
    let default_config = backup::commands::report::ReportConfig::default();

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::TimeSync,
        file: None,
        format: None,
        output_dir: dir.path(),
        meta: &meta,
        config: &default_config,
    })
    .unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "Expected 2 files (html & json) in output_dir"
    );
}
#[test]
fn test_domain_json_schemas_per_report_type() {
    use backup::commands::report::{AuditReport, AuditReportMeta, ReportType};

    let meta = AuditReportMeta::new("funa1.nanoit.kr", "2026-07-30 12:00:00 KST");

    let report_all = AuditReport::generate(ReportType::All, &meta.host_name, &meta.timestamp);
    let json_all = report_all.render_json().unwrap();
    assert!(
        json_all.contains("backup_policy"),
        "All report JSON must contain backup_policy"
    );
    assert!(
        json_all.contains("retention_policy"),
        "All report JSON must contain retention_policy"
    );
    assert!(
        json_all.contains("snapshots"),
        "All report JSON must contain snapshots"
    );

    let report_env =
        AuditReport::generate(ReportType::Environment, &meta.host_name, &meta.timestamp);
    let json_env = report_env.render_json().unwrap();
    assert!(
        json_env.contains("daily_backup_review"),
        "Environment JSON must contain report_type daily_backup_review"
    );
    assert!(
        json_env.contains("retention_policy_verification"),
        "Environment JSON must contain retention_policy_verification"
    );

    let report_ts = AuditReport::generate(ReportType::TimeSync, &meta.host_name, &meta.timestamp);
    let json_ts = report_ts.render_json().unwrap();
    assert!(
        json_ts.contains("isms_p_2.9.3_ntp_sync"),
        "TimeSync JSON must contain report_type isms_p_2.9.3_ntp_sync"
    );
    assert!(
        json_ts.contains("chrony_service"),
        "TimeSync JSON must contain chrony_service"
    );

    let report_rd =
        AuditReport::generate(ReportType::RestoreDrill, &meta.host_name, &meta.timestamp);
    let json_rd = report_rd.render_json().unwrap();
    assert!(
        json_rd.contains("restore_drill"),
        "RestoreDrill JSON must contain report_type restore_drill"
    );
    assert!(
        json_rd.contains("recovery_results"),
        "RestoreDrill JSON must contain recovery_results"
    );
}

#[test]
fn test_html_a4_print_css_and_signature_block() {
    use backup::commands::report::{AuditReport, AuditReportMeta, ReportType};

    let meta = AuditReportMeta::new("funa1.nanoit.kr", "2026-07-30 12:00:00 KST");
    let report = AuditReport::generate(ReportType::All, &meta.host_name, &meta.timestamp);
    let html = report.render_html();

    assert!(
        html.contains("종합 백업 보안 설정 검토 보고서"),
        "HTML title should match"
    );
    assert!(
        html.contains("report-card"),
        "HTML must contain report-card container"
    );
    assert!(
        html.contains("signature-area"),
        "HTML must contain signature approval area"
    );
    assert!(
        html.contains("검토자"),
        "HTML signature box must include reviewer title"
    );
    assert!(
        html.contains("승인자"),
        "HTML signature box must include approver title"
    );
}

#[test]
fn test_default_export_filename_format_date_prefix() {
    use backup::commands::report::{
        AuditReportMeta, ReportExportOptions, ReportType, execute_report_export,
    };

    let dir = tempfile::tempdir().unwrap();
    let meta = AuditReportMeta::new("funa1.nanoit.kr", "2026-07-30");
    let default_config = backup::commands::report::ReportConfig::default();

    let msg = execute_report_export(ReportExportOptions {
        report_type: ReportType::All,
        file: None,
        format: None,
        output_dir: dir.path(),
        meta: &meta,
        config: &default_config,
    })
    .unwrap();

    assert!(msg.contains("ISMS report saved to"));
    let entries: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(entries.len(), 2, "Expected 2 files in output_dir");
}

#[test]
fn test_real_report_data_collects_os_and_audit() {
    let config = backup::commands::report::ReportConfig::default();
    let data = backup::commands::report::RealReportData::collect(&config);
    assert!(!data.os_info.is_empty(), "os_info should be populated");
}

#[test]
fn test_html_report_contains_custom_audit_names_and_os() {
    let config = backup::commands::report::ReportConfig::default();
    let mut data = backup::commands::report::RealReportData::collect(&config);
    data.audit.system_manager = Some("홍길동 차장".into());
    data.audit.security_officer = Some("김보안 이사".into());
    data.os_info = "Ubuntu 22.04 LTS".into();

    let html = backup::commands::report::html_template::render_html_real(
        backup::commands::report::ReportType::RestoreDrill,
        &data,
    );

    assert!(
        html.contains("홍길동 차장"),
        "HTML must contain custom system manager name"
    );
    assert!(
        html.contains("김보안 이사"),
        "HTML must contain custom security officer name"
    );
    assert!(
        html.contains("Ubuntu 22.04 LTS"),
        "HTML must contain dynamic OS info"
    );
}

#[test]
fn test_report_command_fails_on_missing_config() {
    let non_existent_path = std::path::Path::new("/tmp/non_existent_config_12345.yaml");
    let res = backup::commands::report::run_report(non_existent_path, None, None, None);
    assert!(res.is_err());
}

#[test]
fn environment_report_does_not_require_restore_drill_credentials() {
    let temp = tempdir().unwrap();
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: true\n    enableAnnualDrDrillReport: true\nprofiles:\n  primary:\n    repository: s3:bucket\n    env:\n      AWS_ACCESS_KEY_ID: '{{{{ .Env.BACKUP_PRIMARY_AWS_ACCESS_KEY_ID }}}}'\n      AWS_SECRET_ACCESS_KEY: '{{{{ .Env.BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY }}}}'\n  daily:\n    inherit: primary\n    backup:\n      source: ['/data']\n",
            temp.path().display()
        ),
    )
    .unwrap();
    let profiles =
        backup::config::model::ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let output = temp.path().join("environment.json");

    let result = ReportCommand::run_with_profile_adapters(
        Some(ReportAction::Environment {
            file: Some(output.clone()),
            format: Some(ReportFormat::Json),
        }),
        None,
        None,
        &profiles,
        &profiles_path,
        &MockExecutor::new(),
        &MockResticRunner::new(0, "unused"),
        &AuditReportMeta::new("host", "2026-08-10"),
    );

    assert!(result.is_ok(), "{result:?}");
    assert!(output.exists());
}

#[test]
fn restore_drill_missing_primary_credentials_writes_not_performed_evidence() {
    let temp = tempdir().unwrap();
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: true\n    enableAnnualDrDrillReport: true\n  audit:\n    restore-drill-work-dir: {}/restore-drill\nprofiles:\n  primary:\n    repository: /primary\n    password-file: missing-password\n  daily:\n    inherit: primary\n    backup:\n      source: ['/data']\n",
            temp.path().display(),
            temp.path().display()
        ),
    )
    .unwrap();
    let profiles =
        backup::config::model::ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let output = temp.path().join("restore-drill.json");

    let error = ReportCommand::run_with_profile_adapters(
        Some(ReportAction::RestoreDrill {
            file: Some(output.clone()),
            format: Some(ReportFormat::Json),
        }),
        None,
        None,
        &profiles,
        &profiles_path,
        &MockExecutor::new(),
        &MockResticRunner::new(0, "unused"),
        &AuditReportMeta::new("host", "2026-08-10"),
    )
    .unwrap_err();

    assert!(error.to_string().contains("failure report"));
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(output).unwrap()).unwrap();
    assert_eq!(report["overall_status"], "not_performed");
    assert!(
        report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .any(|diagnostic| diagnostic
                .as_str()
                .is_some_and(|diagnostic| diagnostic.contains("primary Backend Profile"))),
        "diagnostics: {}",
        report["diagnostics"]
    );
}

#[test]
fn restore_drill_failure_is_recorded_in_the_written_report() {
    let temp = tempdir().unwrap();
    let password = temp.path().join("primary-password");
    fs::write(&password, "report-password").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&password, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: true\n    enableAnnualDrDrillReport: true\n  audit:\n    restore-drill-work-dir: {}/restore-drill\nprofiles:\n  primary:\n    repository: /tmp/repo\n    password-file: {}\n  files:\n    backup:\n      source: ['/work/source']\n",
            temp.path().display(),
            temp.path().display(),
            password.display()
        ),
    )
    .unwrap();
    let profiles =
        backup::config::model::ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let output_file = temp.path().join("restore-drill.json");
    let command_runner = MockExecutor::new();
    let restic_runner = MockResticRunner::new(1, "restore failed for report-password at /tmp/repo");
    let meta = backup::commands::report::AuditReportMeta::new("contract-host", "2026-08-04")
        .with_profiles_path(&profiles_path);

    let error = ReportCommand::run_with_profile_adapters(
        Some(ReportAction::RestoreDrill {
            file: Some(output_file.clone()),
            format: Some(ReportFormat::Json),
        }),
        None,
        None,
        &profiles,
        &profiles_path,
        &command_runner,
        &restic_runner,
        &meta,
    )
    .unwrap_err();

    assert!(error.to_string().contains("failure report"));
    assert!(!error.to_string().contains("report-password"));
    assert!(!error.to_string().contains("/tmp/repo"));
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_file).unwrap()).unwrap();
    assert_eq!(report["report_status"], "Fail");
    assert!(
        report["failure_diagnostic"]
            .as_str()
            .unwrap()
            .contains("snapshot")
    );
    assert!(!report.to_string().contains("report-password"));
    assert!(!report.to_string().contains("/tmp/repo"));

    let dispatch_command_runner = MockExecutor::new();
    let dispatch_restic_runner = MockResticRunner::new(1, "snapshot query failed");
    let dispatch_rclone_runner = MockRcloneRunner::new(0, "");
    let dispatch_profile_runner = MockResticProfileRunner::new(0, "");
    let dispatch_scheduler = MockScheduler::new(0, "");
    let adapters = AdapterSet {
        command: &dispatch_command_runner,
        rclone: &dispatch_rclone_runner,
        restic: &dispatch_restic_runner,
        resticprofile: &dispatch_profile_runner,
        scheduler: &dispatch_scheduler,
        selection: AdapterSelection::StrictTest,
    };
    let cli = Cli::try_parse_from([
        "backup",
        "--profiles",
        profiles_path.to_string_lossy().as_ref(),
        "report",
        "restore-drill",
        "--file",
        output_file.to_string_lossy().as_ref(),
        "--format",
        "json",
    ])
    .unwrap();
    let context = CliRuntimeContext::from_cli(
        &cli,
        Language::En,
        None,
        SchedulerMode::Auto,
        AdapterSelection::StrictTest,
    )
    .unwrap();
    let outcome = dispatch(&context, cli.command, &adapters);
    assert_eq!(outcome.exit_status, 1);
    assert!(outcome.stderr.contains("restore drill failed"));
    assert!(
        outcome
            .artifacts
            .iter()
            .any(|artifact| artifact.path == output_file)
    );
}

#[test]
fn restore_drill_reports_drill_and_partial_export_failures_together() {
    let temp = tempdir().unwrap();
    let password = temp.path().join("primary-password");
    fs::write(&password, "report-password").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&password, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let profiles_path = temp.path().join("profiles.yaml");
    fs::write(
        &profiles_path,
        format!(
            "version: '2'\napplication:\n  reports:\n    outputDir: {}\n    enableDailyReports: true\n    enableAnnualDrDrillReport: true\n  audit:\n    restore-drill-work-dir: {}/restore-drill\nprofiles:\n  primary:\n    repository: /tmp/repo\n    password-file: {}\n  files:\n    backup:\n      source: ['/work/source']\n",
            temp.path().display(),
            temp.path().display(),
            password.display()
        ),
    )
    .unwrap();
    let profiles =
        backup::config::model::ResticProfileConfig::load_from_path(&profiles_path).unwrap();
    let output_file = temp.path().join("restore-drill-report");
    fs::create_dir(output_file.with_extension("json")).unwrap();
    let meta = backup::commands::report::AuditReportMeta::new("contract-host", "2026-08-04")
        .with_profiles_path(&profiles_path);

    let error = ReportCommand::run_with_profile_adapters(
        Some(ReportAction::RestoreDrill {
            file: Some(output_file.clone()),
            format: None,
        }),
        None,
        None,
        &profiles,
        &profiles_path,
        &MockExecutor::new(),
        &MockResticRunner::new(1, "snapshot query failed"),
        &meta,
    )
    .unwrap_err();

    let message = error.to_string();
    assert!(message.contains("restore drill failed"));
    assert!(message.contains("snapshot"));
    assert!(message.contains("failure report also failed"));
    assert!(output_file.with_extension("html").exists());
}

#[test]
fn report_without_action_separates_each_report_artifact() {
    let dir = tempdir().unwrap();
    let base_file = dir.path().join("audit.json");
    let mut config = backup::commands::report::ReportConfig::default();
    config.primary_repository = "/tmp/report-repository".into();
    config.primary_password = secrecy::SecretString::new("report-password".into());
    config.restore_drill_work_dir = dir.path().join("restore-drill");
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-08-04");

    struct RestoringRunner;
    impl backup::runner::restic::ResticRunner for RestoringRunner {
        fn init_repo(&self, _: &str, _: &str) -> anyhow::Result<String> {
            Ok(String::new())
        }
        fn backup_paths(
            &self,
            _: &str,
            _: &str,
            _: &[String],
            _: &[String],
        ) -> anyhow::Result<String> {
            Ok(String::new())
        }
        fn list_snapshots(&self, _: &str, _: &str) -> anyhow::Result<String> {
            Ok(String::new())
        }
        fn list_snapshot_infos(
            &self,
            _: &str,
            _: &str,
        ) -> anyhow::Result<Vec<backup::runner::snapshot::SnapshotInfo>> {
            Ok(vec![backup::runner::snapshot::SnapshotInfo {
                id: "report-snapshot-001".into(),
                timestamp: "2026-08-04T09:00:00Z".into(),
                tags: vec!["backup-profile:default".into()],
            }])
        }
        fn restore(&self, _: &str, _: &str, _: &str, target: &str) -> anyhow::Result<String> {
            std::fs::write(
                std::path::Path::new(target).join("restored.txt"),
                "restored",
            )?;
            Ok(String::new())
        }
        fn backup_command(
            &self,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: &[String],
        ) -> anyhow::Result<String> {
            Ok(String::new())
        }
        fn backup_command_with_env(
            &self,
            _: &str,
            _: &str,
            _: &str,
            _: &str,
            _: &[String],
            _: &[(&str, &str)],
        ) -> anyhow::Result<String> {
            Ok(String::new())
        }
    }

    let output = ReportCommand::run_with_adapters_and_meta(
        None,
        Some(base_file.clone()),
        None,
        &config,
        &MockExecutor::new(),
        &RestoringRunner,
        &meta,
    )
    .unwrap();

    assert!(output.contains("audit-environment.json"));
    assert!(output.contains("audit-time-sync.json"));
    assert!(output.contains("audit-restore-drill.json"));
    assert!(dir.path().join("audit-environment.json").exists());
    assert!(dir.path().join("audit-time-sync.json").exists());
    assert!(dir.path().join("audit-restore-drill.json").exists());
    assert!(!base_file.exists());

    let html = fs::read_to_string(dir.path().join("audit-restore-drill.html")).unwrap();
    let json = fs::read_to_string(dir.path().join("audit-restore-drill.json")).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let execution_id = value["execution_id"].as_str().unwrap();
    assert!(!execution_id.is_empty());
    assert!(html.contains(execution_id));
    assert_eq!(
        value["storage_results"][0]["snapshot_id"],
        "report-snapshot-001"
    );
    assert_eq!(value["storage_results"][0]["profile"], "default");
    assert_eq!(value["storage_results"][0]["backend"], "primary");
    assert_eq!(value["storage_results"][0]["file_count"], 1);
    assert_eq!(value["storage_results"][0]["total_bytes"], 8);
    assert!(value["storage_results"][0]["elapsed_milliseconds"].is_number());
    assert_eq!(value["storage_results"][0]["validation_status"], "pass");
    assert!(html.contains("report-snapshot-001"));
}

#[test]
fn explicit_report_file_is_atomically_overwritten() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("audit.json");
    fs::write(&file, "old report").unwrap();
    let config = backup::commands::report::ReportConfig::default();
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-08-04");

    backup::commands::report::execute_report_export(
        backup::commands::report::ReportExportOptions {
            report_type: ReportType::Environment,
            file: Some(&file),
            format: Some(ReportFormat::Json),
            output_dir: dir.path(),
            meta: &meta,
            config: &config,
        },
    )
    .unwrap();

    let content = fs::read_to_string(file).unwrap();
    assert!(content.contains("daily_backup_review"));
    assert!(!content.contains("old report"));
}

#[test]
fn generic_restore_drill_export_requires_the_evidence_collection_path() {
    let dir = tempdir().unwrap();
    let config = backup::commands::report::ReportConfig::default();
    let meta = backup::commands::report::AuditReportMeta::new("host-1", "2026-08-04");

    let error = backup::commands::report::execute_report_export(
        backup::commands::report::ReportExportOptions {
            report_type: ReportType::RestoreDrill,
            file: Some(&dir.path().join("restore-drill")),
            format: Some(ReportFormat::Json),
            output_dir: dir.path(),
            meta: &meta,
            config: &config,
        },
    )
    .expect_err("generic exports must not claim a Restore Drill was collected");

    assert!(error.to_string().contains("collected Evidence"));
    assert!(dir.path().read_dir().unwrap().next().is_none());
}

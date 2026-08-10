use backup::commands::report::json_schema::{RecoveryResultsJson, RestoreDrillReportJson};
use backup::commands::report::restore_drill::{
    DatabaseVerificationEvidence, RestoreDrillEvidence, RestoreDrillPolicy, RestoreDrillStatus,
    RestoreDrillStorageResult, render_restore_drill_evidence_html,
    render_restore_drill_evidence_json,
};
use backup::commands::report::{AuditReportMeta, RealReportData, ReportConfig, ReportType};
use backup::commands::report::{html_template, json_schema};
use backup::config::model::DatabaseType;

mod support;
use support::MockExecutor;

fn passing_evidence() -> RestoreDrillEvidence {
    let policy = RestoreDrillPolicy::new(120, 240).unwrap();
    RestoreDrillEvidence::new(
        "drill-2026-08-07-001",
        "2026-08-07T10:00:00+09:00",
        "2026-08-07T10:00:04+09:00",
        policy.clone(),
        vec![
            RestoreDrillStorageResult::measured(
                "daily-files",
                "primary",
                "snapshot-full-001",
                "2026-08-07T09:59:00Z",
                "2026-08-07T10:00:01+09:00",
                "2026-08-07T10:00:04+09:00",
                3_421,
                2,
                4096,
                "regular file count and total bytes",
                &policy,
            ),
            RestoreDrillStorageResult::measured(
                "daily-files",
                "secondary",
                "snapshot-secondary-001",
                "2026-08-07T09:58:00Z",
                "2026-08-07T10:00:01+09:00",
                "2026-08-07T10:00:03+09:00",
                2_100,
                2,
                4096,
                "regular file count and total bytes",
                &policy,
            ),
            RestoreDrillStorageResult::measured(
                "weekly-files",
                "primary",
                "snapshot-weekly-001",
                "2026-08-07T09:57:00Z",
                "2026-08-07T10:00:01+09:00",
                "2026-08-07T10:00:02+09:00",
                1_500,
                1,
                2048,
                "regular file count and total bytes",
                &policy,
            ),
        ],
    )
    .with_metadata("backup-host", "운영팀", "보안팀", None)
}

#[test]
fn html_and_json_render_the_same_injected_restore_drill_evidence() {
    let evidence = passing_evidence();
    let html = render_restore_drill_evidence_html(&evidence);
    let json = render_restore_drill_evidence_json(&evidence).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["execution_id"], "drill-2026-08-07-001");
    assert_eq!(value["overall_status"], "pass");
    assert_eq!(value["storage_results"][0]["profile"], "daily-files");
    assert_eq!(value["storage_results"][0]["backend"], "primary");
    assert_eq!(
        value["storage_results"][0]["snapshot_id"],
        "snapshot-full-001"
    );
    assert_eq!(value["storage_results"][0]["elapsed_milliseconds"], 3_421);
    assert_eq!(value["storage_results"][0]["file_count"], 2);
    assert_eq!(value["storage_results"][0]["total_bytes"], 4096);
    assert_eq!(value["storage_results"][0]["status"], "pass");
    assert_eq!(value["storage_results"].as_array().unwrap().len(), 3);

    assert!(html.contains("drill-2026-08-07-001"));
    assert!(html.contains("snapshot-full-001"));
    assert!(html.contains("snapshot-secondary-001"));
    assert!(html.contains("weekly-files"));
    assert!(html.contains("3421 ms"));
    assert!(html.contains("regular file count and total bytes"));
    assert!(html.contains("overall-status-pass"));

    // The compatibility fields are derived from this same evidence rather than placeholders.
    assert_eq!(value["report_type"], "restore_drill");
    assert_eq!(value["target_snapshot_id"], serde_json::Value::Null);
    assert_eq!(value["target_snapshot_time"], serde_json::Value::Null);
    assert_eq!(value["recovery_results"]["elapsed_seconds"], 3);
    assert_eq!(value["recovery_results"]["rto_satisfied"], true);
    assert_eq!(value["recovery_results"]["data_integrity_verified"], true);
    assert_eq!(
        value["recovery_results"]["data_size_human"],
        "6.0 KiB (6,144)"
    );
    assert_eq!(value["schema_version"], "1");
}

#[test]
fn public_report_renderers_use_the_injected_restore_drill_evidence() {
    let evidence = passing_evidence();
    let meta = AuditReportMeta::new("test-host", "2026-08-07T10:00:00+09:00");
    let data = RealReportData::collect_with_meta_with_runner(
        &ReportConfig::default(),
        &meta,
        &MockExecutor::new(),
    )
    .with_restore_drill_evidence(evidence.clone());

    let html = html_template::render_html_real(ReportType::RestoreDrill, &data);
    let json = json_schema::render_json_real(ReportType::RestoreDrill, &data).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(html.contains("drill-2026-08-07-001"));
    assert!(html.contains("snapshot-full-001"));
    assert!(!html.contains("측정 로그 확인 필요"));
    assert_eq!(value["execution_id"], evidence.execution_id);
    assert_eq!(value["target_snapshot_id"], serde_json::Value::Null);
    assert_eq!(value["overall_status"], "pass");
}

#[test]
fn public_restore_drill_renderers_never_claim_success_without_evidence() {
    let meta = AuditReportMeta::new("test-host", "2026-08-07T10:00:00+09:00");
    let data = RealReportData::collect_with_meta_with_runner(
        &ReportConfig::default(),
        &meta,
        &MockExecutor::new(),
    );

    let html = html_template::render_html_real(ReportType::RestoreDrill, &data);
    let json = json_schema::render_json_real(ReportType::RestoreDrill, &data).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(html.contains("overall-status-not-performed"));
    assert!(!html.contains("측정 로그 확인 필요"));
    assert_eq!(value["overall_status"], "not_performed");
    assert_eq!(value["report_status"], "NotPerformed");
    assert_eq!(value["target_snapshot_id"], serde_json::Value::Null);
    assert_eq!(
        value["recovery_results"]["elapsed_milliseconds"],
        serde_json::Value::Null
    );
    assert_eq!(
        value["recovery_results"]["elapsed_seconds"],
        serde_json::Value::Null
    );
    assert_eq!(
        value["recovery_results"]["rto_satisfied"],
        serde_json::Value::Null
    );
    assert_eq!(
        value["recovery_results"]["data_integrity_verified"],
        serde_json::Value::Null
    );
}

#[test]
fn legacy_restore_drill_types_round_trip_nullable_evidence_fields() {
    let passing: RestoreDrillReportJson =
        serde_json::from_str(&render_restore_drill_evidence_json(&passing_evidence()).unwrap())
            .unwrap();
    assert!(passing.target_snapshot_id.is_none());
    assert_eq!(passing.recovery_results.elapsed_seconds, Some(3));
    assert_eq!(passing.recovery_results.rto_satisfied, Some(true));

    let not_performed = RestoreDrillEvidence::not_collected(
        "2026-08-07T10:00:00+09:00",
        RestoreDrillPolicy::default(),
    );
    let report: RestoreDrillReportJson =
        serde_json::from_str(&render_restore_drill_evidence_json(&not_performed).unwrap()).unwrap();
    let recovery: RecoveryResultsJson = report.recovery_results;
    assert!(report.target_snapshot_id.is_none());
    assert!(recovery.elapsed_seconds.is_none());
    assert!(recovery.rto_satisfied.is_none());
    assert!(recovery.data_integrity_verified.is_none());
}

#[test]
fn renderers_recompute_overall_status_from_storage_results() {
    let mut evidence = passing_evidence();
    evidence.overall_status = RestoreDrillStatus::Fail;

    let html = render_restore_drill_evidence_html(&evidence);
    let json = render_restore_drill_evidence_json(&evidence).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(html.contains("overall-status-pass"));
    assert_eq!(value["overall_status"], "pass");
    assert_eq!(value["report_status"], "Pass");
}

#[test]
fn compatibility_fields_prefer_file_primary_aggregate_over_database_result() {
    let policy = RestoreDrillPolicy::default();
    let database = RestoreDrillStorageResult::measured(
        "database",
        "primary",
        "database-snapshot-001",
        "2026-08-07T09:58:00Z",
        "2026-08-07T10:00:01+09:00",
        "2026-08-07T10:00:03+09:00",
        2_000,
        1,
        2048,
        "regular file count and total bytes",
        &policy,
    )
    .with_database_verification(DatabaseVerificationEvidence {
        db_type: DatabaseType::Postgres,
        expected_signature: "PostgreSQL pg_dump SQL signature".into(),
        signature_verified: true,
        signature_status: RestoreDrillStatus::Pass,
        validation_scope: "SQL dump signature only".into(),
        db_integrity_verified: false,
        import_performed: false,
        record_validation_performed: false,
    });
    let evidence = RestoreDrillEvidence::new(
        "drill-primary-aggregate-001",
        "2026-08-07T10:00:00+09:00",
        "2026-08-07T10:00:04+09:00",
        policy,
        vec![passing_evidence().storage_results[0].clone(), database],
    );

    let json = render_restore_drill_evidence_json(&evidence).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!(value["target_snapshot_id"], "snapshot-full-001");
    assert_eq!(value["target_snapshot_time"], "2026-08-07T09:59:00Z");
    assert_eq!(value["recovery_results"]["elapsed_milliseconds"], 3_421);
    assert_eq!(
        value["recovery_results"]["data_size_human"],
        "4.0 KiB (4,096)"
    );
}

#[test]
fn database_evidence_reports_signature_scope_without_claiming_import_or_record_validation() {
    let policy = RestoreDrillPolicy::default();
    let result = RestoreDrillStorageResult::measured(
        "database",
        "primary",
        "database-snapshot-001",
        "2026-08-07T09:59:00Z",
        "2026-08-07T10:00:01+09:00",
        "2026-08-07T10:00:04+09:00",
        3_421,
        1,
        4096,
        "regular file count, total bytes, and PostgreSQL pg_dump SQL signature",
        &policy,
    )
    .with_database_verification(DatabaseVerificationEvidence {
        db_type: DatabaseType::Postgres,
        expected_signature: "PostgreSQL pg_dump SQL signature".into(),
        signature_verified: true,
        signature_status: RestoreDrillStatus::Pass,
        validation_scope: "SQL dump signature only".into(),
        db_integrity_verified: false,
        import_performed: false,
        record_validation_performed: false,
    });
    let evidence = RestoreDrillEvidence::new(
        "drill-database-001",
        "2026-08-07T10:00:00+09:00",
        "2026-08-07T10:00:04+09:00",
        policy,
        vec![result],
    );

    let json = render_restore_drill_evidence_json(&evidence).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let database = &value["recovery_results"]["database_verification"];
    assert_eq!(database["db_type"], "postgres");
    assert_eq!(database["db_snapshot_id"], "database-snapshot-001");
    assert_eq!(database["signature_verified"], true);
    assert_eq!(database["validation_scope"], "SQL dump signature only");
    assert_eq!(database["db_integrity_verified"], false);
    assert_eq!(database["import_performed"], false);
    assert_eq!(database["record_validation_performed"], false);
    assert_eq!(value["recovery_results"]["data_integrity_verified"], true);

    let html = render_restore_drill_evidence_html(&evidence);
    assert!(html.contains("PostgreSQL pg_dump SQL signature"));
    assert!(html.contains("SQL dump signature only"));
    assert!(html.contains("not performed"));
}

#[test]
fn failed_evidence_is_rendered_with_escaped_and_masked_diagnostics() {
    let evidence = RestoreDrillEvidence::new(
        "drill-failed-001",
        "2026-08-07T11:00:00+09:00",
        "2026-08-07T11:00:02+09:00",
        RestoreDrillPolicy::new(120, 240).unwrap(),
        vec![RestoreDrillStorageResult::failed(
            "daily-files",
            "primary",
            "2026-08-07T11:00:01+09:00",
            "2026-08-07T11:00:02+09:00",
            "restore failed: <secret> at /tmp/private-drill",
        )],
    )
    .with_diagnostics(vec![
        "adapter said: \"<secret>\"".into(),
        "<script>alert(1)</script>".into(),
    ])
    .with_sensitive_values(["<secret>", "/tmp/private-drill"]);

    assert_eq!(evidence.overall_status, RestoreDrillStatus::Fail);

    let html = render_restore_drill_evidence_html(&evidence);
    let json = render_restore_drill_evidence_json(&evidence).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert!(html.contains("overall-status-fail"));
    assert!(html.contains("&lt;secret&gt;") || html.contains("***MASKED***"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
    assert!(!html.contains("/tmp/private-drill"));
    assert!(!json.contains("/tmp/private-drill"));
    assert!(!json.contains("<secret>"));
    assert_eq!(value["report_status"], "Fail");
    assert_eq!(value["storage_results"][0]["status"], "fail");
    assert!(
        value["storage_results"][0]["diagnostic"]
            .as_str()
            .unwrap()
            .contains("***MASKED***")
    );
    assert!(
        value["diagnostics"][0]
            .as_str()
            .unwrap()
            .contains("***MASKED***")
    );
}

#[test]
fn status_aggregation_keeps_not_applicable_secondary_from_failing_primary() {
    let policy = RestoreDrillPolicy::new(120, 240).unwrap();
    let evidence = RestoreDrillEvidence::new(
        "drill-status-001",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        policy.clone(),
        vec![
            RestoreDrillStorageResult::measured(
                "daily-files",
                "primary",
                "snapshot-1",
                "2026-08-07T11:59:00Z",
                "2026-08-07T12:00:00Z",
                "2026-08-07T12:00:01Z",
                100,
                1,
                1,
                "regular file count and total bytes",
                &policy,
            ),
            RestoreDrillStorageResult::not_applicable(
                "daily-files",
                "secondary",
                "secondary Backend Profile is not configured",
            ),
        ],
    );

    assert_eq!(evidence.overall_status, RestoreDrillStatus::Pass);

    let not_applicable = RestoreDrillEvidence::new(
        "drill-status-002",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        RestoreDrillPolicy::default(),
        vec![RestoreDrillStorageResult::not_applicable(
            "daily-files",
            "secondary",
            "secondary Backend Profile is not configured",
        )],
    );
    assert_eq!(
        not_applicable.overall_status,
        RestoreDrillStatus::NotApplicable
    );
}

#[test]
fn status_aggregation_prioritizes_fail_then_not_performed_then_pass() {
    let policy = RestoreDrillPolicy::default();
    let pass = RestoreDrillStorageResult::measured(
        "passing-profile",
        "primary",
        "snapshot-pass",
        "2026-08-07T11:59:00Z",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        100,
        1,
        1,
        "regular file count and total bytes",
        &policy,
    );
    let not_performed = RestoreDrillStorageResult::not_performed(
        "unavailable-profile",
        "primary",
        "snapshot tag missing",
    );
    let fail = RestoreDrillStorageResult::failed(
        "failed-profile",
        "secondary",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        "restore failed",
    );

    let with_failure = RestoreDrillEvidence::new(
        "drill-priority-fail",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        policy.clone(),
        vec![pass.clone(), not_performed.clone(), fail],
    );
    assert_eq!(with_failure.overall_status, RestoreDrillStatus::Fail);

    let without_failure = RestoreDrillEvidence::new(
        "drill-priority-not-performed",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        policy,
        vec![pass, not_performed],
    );
    assert_eq!(
        without_failure.overall_status,
        RestoreDrillStatus::NotPerformed
    );
}

#[test]
fn policy_derives_rto_judgment_from_measured_elapsed_time() {
    let policy = RestoreDrillPolicy::new(1, 2).unwrap();
    assert!(policy.is_within_rto(60_000));
    assert!(!policy.is_within_rto(60_001));

    let result = RestoreDrillStorageResult::measured(
        "daily-files",
        "primary",
        "snapshot-1",
        "2026-08-07T11:59:00Z",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:01:01Z",
        60_001,
        1,
        1,
        "regular file count and total bytes",
        &policy,
    );
    assert_eq!(result.status, RestoreDrillStatus::Fail);
    assert_eq!(result.rto_satisfied, Some(false));

    let empty_output = RestoreDrillStorageResult::measured(
        "daily-files",
        "primary",
        "snapshot-2",
        "2026-08-07T11:59:00Z",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        1_000,
        0,
        0,
        "regular file count and total bytes",
        &policy,
    );
    assert_eq!(empty_output.status, RestoreDrillStatus::Fail);
    assert_eq!(empty_output.validation_status, RestoreDrillStatus::Fail);
}

#[test]
fn diagnostics_mask_paths_and_urls_without_explicit_secret_registration() {
    let evidence = RestoreDrillEvidence::new(
        "drill-mask-001",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        RestoreDrillPolicy::default(),
        vec![RestoreDrillStorageResult::failed(
            "daily-files",
            "primary",
            "2026-08-07T12:00:00Z",
            "2026-08-07T12:00:01Z",
            "repository=/srv/private and endpoint=https://storage.example/backup",
        )],
    );

    let json = render_restore_drill_evidence_json(&evidence).unwrap();
    assert!(!json.contains("/srv/private"));
    assert!(!json.contains("https://storage.example/backup"));
    assert!(json.contains("***MASKED***"));

    let relative_and_windows = RestoreDrillEvidence::new(
        "drill-mask-002",
        "2026-08-07T12:00:00Z",
        "2026-08-07T12:00:01Z",
        RestoreDrillPolicy::default(),
        vec![RestoreDrillStorageResult::failed(
            "daily-files",
            "primary",
            "2026-08-07T12:00:00Z",
            "2026-08-07T12:00:01Z",
            "target=./tmp/drill target=C:\\tmp\\drill",
        )],
    );
    let relative_json = render_restore_drill_evidence_json(&relative_and_windows).unwrap();
    assert!(!relative_json.contains("./tmp/drill"));
    assert!(!relative_json.contains("C:\\tmp\\drill"));
}

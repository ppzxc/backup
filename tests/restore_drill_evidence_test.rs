use backup::commands::report::restore_drill::{
    RestoreDrillEvidence, RestoreDrillPolicy, RestoreDrillStatus, RestoreDrillStorageResult,
    render_restore_drill_evidence_html, render_restore_drill_evidence_json,
};

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
    assert_eq!(value["target_snapshot_id"], "snapshot-full-001");
    assert_eq!(value["target_snapshot_time"], "2026-08-07T09:59:00Z");
    assert_eq!(value["recovery_results"]["elapsed_seconds"], 3);
    assert_eq!(value["recovery_results"]["rto_satisfied"], true);
    assert_eq!(value["recovery_results"]["data_integrity_verified"], true);
    assert_eq!(value["schema_version"], "1");
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

use backup::commands::snapshots::{
    SnapshotInfo, SnapshotSelectionReason, SnapshotSelectionStatus, select_latest_tagged_snapshot,
    select_latest_tagged_snapshot_from_infos, select_latest_tagged_snapshot_from_json,
};
use secrecy::SecretString;

mod support;

#[test]
fn snapshot_selection_returns_the_latest_exactly_tagged_full_snapshot() {
    let selection = select_latest_tagged_snapshot_from_json(
        r#"[
          {"id":"older-full-id","time":"2026-08-07T08:00:00Z","tags":["backup-profile:daily"]},
          {"id":"wrong-profile","time":"2026-08-07T12:00:00Z","tags":["backup-profile:weekly"]},
          {"id":"newer-full-id","time":"2026-08-07T09:00:00Z","tags":["user-tag","backup-profile:daily"]}
        ]"#,
        "daily",
    );

    assert_eq!(selection.status, SnapshotSelectionStatus::Selected);
    assert_eq!(selection.snapshot_id.as_deref(), Some("newer-full-id"));
    assert_eq!(
        selection.snapshot_time.as_deref(),
        Some("2026-08-07T09:00:00Z")
    );
}

#[test]
fn timestamp_ties_choose_the_highest_full_id_deterministically() {
    let selection = select_latest_tagged_snapshot_from_json(
        r#"[
          {"id":"aaa-full-id","time":"2026-08-07T09:00:00Z","tags":["backup-profile:daily"]},
          {"id":"zzz-full-id","time":"2026-08-07T09:00:00Z","tags":["backup-profile:daily"]}
        ]"#,
        "daily",
    );

    assert_eq!(selection.status, SnapshotSelectionStatus::Selected);
    assert_eq!(selection.snapshot_id.as_deref(), Some("zzz-full-id"));
}

#[test]
fn snapshot_selection_never_guesses_from_untagged_or_malformed_data() {
    for (json, reason) in [
        (
            r#"[{"id":"untagged","time":"2026-08-07T09:00:00Z","paths":["/data"]}]"#,
            SnapshotSelectionReason::NoExactTagMatch,
        ),
        (
            r#"[{"id":"other","time":"2026-08-07T09:00:00Z","tags":["backup-profile:weekly"]}]"#,
            SnapshotSelectionReason::NoExactTagMatch,
        ),
        (r#"not-json"#, SnapshotSelectionReason::MalformedJson),
        (
            r#"[{"time":"2026-08-07T09:00:00Z","tags":["backup-profile:daily"]}]"#,
            SnapshotSelectionReason::MissingMetadata,
        ),
    ] {
        let selection = select_latest_tagged_snapshot_from_json(json, "daily");
        assert_eq!(selection.status, SnapshotSelectionStatus::NotPerformed);
        assert!(selection.snapshot_id.is_none());
        assert!(selection.snapshot_time.is_none());
        assert!(selection.diagnostic.is_some());
        assert_eq!(selection.reason, Some(reason));
    }
}

#[test]
fn snapshot_selection_requires_an_exact_profile_key() {
    let selection = select_latest_tagged_snapshot_from_json(
        r#"[{"id":"full-id","time":"2026-08-07T09:00:00Z","tags":["backup-profile:daily"]}]"#,
        " daily ",
    );

    assert_eq!(selection.status, SnapshotSelectionStatus::NotPerformed);
    assert_eq!(selection.snapshot_id, None);
}

#[test]
fn adapter_selection_converts_a_json_listing_to_a_concrete_restore_identity() {
    let runner = support::MockResticRunner::new(
        0,
        r#"[{"id":"full-id","time":"2026-08-07T09:00:00Z","tags":["backup-profile:daily"]}]"#,
    );

    let password = SecretString::new("secret".into());
    let selection = select_latest_tagged_snapshot(&runner, "/repo", &password, "daily");

    assert_eq!(selection.status, SnapshotSelectionStatus::Selected);
    assert_eq!(selection.snapshot_id.as_deref(), Some("full-id"));
    assert_eq!(
        selection.snapshot_time.as_deref(),
        Some("2026-08-07T09:00:00Z")
    );
}

#[test]
fn adapter_query_failure_is_a_structured_not_performed_result() {
    let runner = support::MockResticRunner::new(1, "repository unavailable");

    let password = SecretString::new("secret".into());
    let selection = select_latest_tagged_snapshot(&runner, "/repo", &password, "daily");

    assert_eq!(selection.status, SnapshotSelectionStatus::NotPerformed);
    assert!(selection.snapshot_id.is_none());
    assert!(selection.diagnostic.is_some());
}

#[test]
fn adapter_parse_failure_preserves_a_structured_reason() {
    let runner = support::MockResticRunner::new(0, "not-json");
    let password = SecretString::new("secret".into());
    let selection = select_latest_tagged_snapshot(&runner, "/repo", &password, "daily");

    assert_eq!(selection.status, SnapshotSelectionStatus::NotPerformed);
    assert_eq!(
        selection.reason,
        Some(SnapshotSelectionReason::MalformedJson)
    );
}

#[test]
fn direct_snapshot_infos_reject_missing_required_metadata() {
    let selection = select_latest_tagged_snapshot_from_infos(
        &[SnapshotInfo {
            id: "".into(),
            timestamp: "2026-08-07T09:00:00Z".into(),
            tags: vec!["backup-profile:daily".into()],
        }],
        "daily",
    );

    assert_eq!(selection.status, SnapshotSelectionStatus::NotPerformed);
    assert_eq!(
        selection.reason,
        Some(SnapshotSelectionReason::MissingMetadata)
    );
}

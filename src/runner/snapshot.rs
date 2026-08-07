use crate::config::model::backup_profile_snapshot_tag;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSelectionStatus {
    Selected,
    NotPerformed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSelectionReason {
    InvalidProfile,
    QueryFailed,
    MalformedJson,
    MissingMetadata,
    NoExactTagMatch,
}

impl SnapshotSelectionReason {
    fn diagnostic(self) -> &'static str {
        match self {
            Self::InvalidProfile => "snapshot selection requires an exact, non-empty profile key",
            Self::QueryFailed => "snapshot JSON listing was not available",
            Self::MalformedJson => "snapshot JSON was malformed",
            Self::MissingMetadata => "snapshot JSON omitted required ID, timestamp, or metadata",
            Self::NoExactTagMatch => "no snapshot has the exact reserved Backup Profile tag",
        }
    }
}

/// A concrete full snapshot identity selected from a structured restic snapshot listing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: String,
    pub timestamp: String,
    pub tags: Vec<String>,
}

/// The safe result of trying to identify one Backup Profile snapshot.
///
/// `NotPerformed` is deliberately represented as data so callers can preserve an audit result
/// without guessing from paths, hostnames, source lists, or an untagged `latest` snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotSelection {
    pub status: SnapshotSelectionStatus,
    pub snapshot_id: Option<String>,
    pub snapshot_time: Option<String>,
    pub diagnostic: Option<String>,
    pub reason: Option<SnapshotSelectionReason>,
}

impl SnapshotSelection {
    fn selected(snapshot: &SnapshotInfo) -> Self {
        Self {
            status: SnapshotSelectionStatus::Selected,
            snapshot_id: Some(snapshot.id.clone()),
            snapshot_time: Some(snapshot.timestamp.clone()),
            diagnostic: None,
            reason: None,
        }
    }

    fn not_performed(reason: SnapshotSelectionReason) -> Self {
        Self {
            status: SnapshotSelectionStatus::NotPerformed,
            snapshot_id: None,
            snapshot_time: None,
            diagnostic: Some(reason.diagnostic().into()),
            reason: Some(reason),
        }
    }

    pub fn is_selected(&self) -> bool {
        self.status == SnapshotSelectionStatus::Selected
    }
}

#[derive(Debug, Deserialize)]
struct RawSnapshot {
    id: Option<String>,
    time: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ParsedTimestamp {
    seconds: i64,
    nanoseconds: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotJsonError {
    Malformed,
    MissingMetadata,
}

impl fmt::Display for SnapshotJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Malformed => "snapshot JSON was malformed",
            Self::MissingMetadata => "snapshot JSON omitted required ID or timestamp metadata",
        })
    }
}

impl std::error::Error for SnapshotJsonError {}

/// Converts restic's machine-readable snapshot output into BackupEngine domain data.
pub fn parse_snapshot_json(
    json: &str,
) -> std::result::Result<Vec<SnapshotInfo>, SnapshotJsonError> {
    let snapshots: Vec<RawSnapshot> =
        serde_json::from_str(json).map_err(|_| SnapshotJsonError::Malformed)?;
    snapshots
        .into_iter()
        .map(|snapshot| {
            let id = snapshot
                .id
                .filter(|id| !id.is_empty() && id == id.trim())
                .ok_or(SnapshotJsonError::MissingMetadata)?;
            let timestamp = snapshot
                .time
                .filter(|timestamp| !timestamp.is_empty() && timestamp == timestamp.trim())
                .ok_or(SnapshotJsonError::MissingMetadata)?;
            if parse_rfc3339_timestamp(&timestamp).is_none() {
                return Err(SnapshotJsonError::MissingMetadata);
            }
            Ok(SnapshotInfo {
                id,
                timestamp,
                tags: snapshot.tags.unwrap_or_default(),
            })
        })
        .collect()
}

/// Selects the newest snapshot carrying exactly `backup-profile:<profile>`.
///
/// The timestamp is compared as an instant, including RFC3339 offsets. Equal instants use the
/// lexicographically greatest full snapshot ID, then timestamp text, as deterministic tie-breakers.
pub fn select_latest_tagged_snapshot_from_json(json: &str, profile: &str) -> SnapshotSelection {
    if !valid_profile_key(profile) {
        return SnapshotSelection::not_performed(SnapshotSelectionReason::InvalidProfile);
    }
    let snapshots = match parse_snapshot_json(json) {
        Ok(snapshots) => snapshots,
        Err(SnapshotJsonError::Malformed) => {
            return SnapshotSelection::not_performed(SnapshotSelectionReason::MalformedJson);
        }
        Err(SnapshotJsonError::MissingMetadata) => {
            return SnapshotSelection::not_performed(SnapshotSelectionReason::MissingMetadata);
        }
    };
    select_latest_tagged_snapshot_from_infos(&snapshots, profile)
}

pub fn select_latest_tagged_snapshot_from_infos(
    snapshots: &[SnapshotInfo],
    profile: &str,
) -> SnapshotSelection {
    if !valid_profile_key(profile) {
        return SnapshotSelection::not_performed(SnapshotSelectionReason::InvalidProfile);
    }
    if snapshots.iter().any(|snapshot| {
        snapshot.id.is_empty()
            || snapshot.id != snapshot.id.trim()
            || parse_rfc3339_timestamp(&snapshot.timestamp).is_none()
    }) {
        return SnapshotSelection::not_performed(SnapshotSelectionReason::MissingMetadata);
    }
    let reserved_tag = backup_profile_snapshot_tag(profile);
    let Some(snapshot) = snapshots
        .iter()
        .filter(|snapshot| snapshot.tags.iter().any(|tag| tag == &reserved_tag))
        .max_by(|left, right| {
            let left_time = parse_rfc3339_timestamp(&left.timestamp);
            let right_time = parse_rfc3339_timestamp(&right.timestamp);
            left_time
                .cmp(&right_time)
                .then_with(|| left.id.cmp(&right.id))
                .then_with(|| left.timestamp.cmp(&right.timestamp))
        })
    else {
        return SnapshotSelection::not_performed(SnapshotSelectionReason::NoExactTagMatch);
    };
    SnapshotSelection::selected(snapshot)
}

/// Queries structured restic snapshot data and converts adapter/parser failures into an explicit
/// `NotPerformed` result suitable for Restore Drill Evidence.
pub fn select_latest_tagged_snapshot<R: crate::runner::restic::ResticRunner + ?Sized>(
    runner: &R,
    repository: &str,
    password: &SecretString,
    profile: &str,
) -> SnapshotSelection {
    selection_from_listing(
        runner.list_snapshot_infos(repository, password.expose_secret()),
        profile,
    )
}

/// Environment-aware variant for repositories whose credentials are supplied only to the child
/// process (for example S3 backends). The environment values are never copied into the result.
pub fn select_latest_tagged_snapshot_with_env<R: crate::runner::restic::ResticRunner + ?Sized>(
    runner: &R,
    repository: &str,
    password: &SecretString,
    environment: &[(&str, &str)],
    profile: &str,
) -> SnapshotSelection {
    let listing = if environment.is_empty() {
        runner.list_snapshot_infos(repository, password.expose_secret())
    } else {
        runner.list_snapshot_infos_with_env(repository, password.expose_secret(), environment)
    };
    selection_from_listing(listing, profile)
}

fn selection_from_listing(
    listing: anyhow::Result<Vec<SnapshotInfo>>,
    profile: &str,
) -> SnapshotSelection {
    match listing {
        Ok(snapshots) => select_latest_tagged_snapshot_from_infos(&snapshots, profile),
        Err(error) => match error.downcast_ref::<SnapshotJsonError>() {
            Some(SnapshotJsonError::Malformed) => {
                SnapshotSelection::not_performed(SnapshotSelectionReason::MalformedJson)
            }
            Some(SnapshotJsonError::MissingMetadata) => {
                SnapshotSelection::not_performed(SnapshotSelectionReason::MissingMetadata)
            }
            None => SnapshotSelection::not_performed(SnapshotSelectionReason::QueryFailed),
        },
    }
}

fn valid_profile_key(profile: &str) -> bool {
    !profile.trim().is_empty() && profile == profile.trim()
}

fn parse_rfc3339_timestamp(value: &str) -> Option<ParsedTimestamp> {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || (bytes[10] != b'T' && bytes[10] != b't')
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }

    let year = parse_digits(bytes, 0, 4)? as i32;
    let month = parse_digits(bytes, 5, 7)? as u32;
    let day = parse_digits(bytes, 8, 10)? as u32;
    let hour = parse_digits(bytes, 11, 13)? as u32;
    let minute = parse_digits(bytes, 14, 16)? as u32;
    let second = parse_digits(bytes, 17, 19)? as u32;
    if !(1..=12).contains(&month)
        || !(1..=days_in_month(year, month)).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }

    let mut cursor = 19;
    let mut nanoseconds = 0;
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        let start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        let fraction = &bytes[start..cursor];
        if fraction.is_empty() || fraction.len() > 9 {
            return None;
        }
        nanoseconds =
            parse_digits(fraction, 0, fraction.len())? * 10u64.pow(9 - fraction.len() as u32);
    }

    let offset_minutes = match bytes.get(cursor) {
        Some(b'Z') | Some(b'z') if cursor + 1 == bytes.len() => 0i32,
        Some(sign @ (b'+' | b'-')) if cursor + 6 == bytes.len() && bytes[cursor + 3] == b':' => {
            let hours = parse_digits(bytes, cursor + 1, cursor + 3)? as i32;
            let minutes = parse_digits(bytes, cursor + 4, cursor + 6)? as i32;
            if hours > 23 || minutes > 59 {
                return None;
            }
            let total = hours * 60 + minutes;
            if *sign == b'+' { total } else { -total }
        }
        _ => return None,
    };

    let seconds = days_from_civil(year, month, day)
        .checked_mul(86_400)?
        .checked_add(hour as i64 * 3_600 + minute as i64 * 60 + second as i64)?
        .checked_sub(offset_minutes as i64 * 60)?;
    Some(ParsedTimestamp {
        seconds,
        nanoseconds: nanoseconds as u32,
    })
}

fn parse_digits(bytes: &[u8], start: usize, end: usize) -> Option<u64> {
    (start < end && end <= bytes.len()).then_some(())?;
    bytes[start..end].iter().try_fold(0u64, |value, digit| {
        digit
            .is_ascii_digit()
            .then_some(value * 10 + u64::from(digit - b'0'))
    })
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// Days since 1970-01-01, using the proleptic Gregorian calendar.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = (if year >= 0 { year } else { year - 399 }) / 400;
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::{SnapshotSelectionStatus, select_latest_tagged_snapshot_from_json};

    #[test]
    fn selects_newest_exact_tag_and_deterministic_tie() {
        let result = select_latest_tagged_snapshot_from_json(
            r#"[
              {"id":"aaa","time":"2026-08-07T09:00:00Z","tags":["backup-profile:daily"]},
              {"id":"zzz","time":"2026-08-07T09:00:00Z","tags":["backup-profile:daily"]},
              {"id":"wrong","time":"2026-08-07T10:00:00Z","tags":["backup-profile:weekly"]}
            ]"#,
            "daily",
        );
        assert_eq!(result.status, SnapshotSelectionStatus::Selected);
        assert_eq!(result.snapshot_id.as_deref(), Some("zzz"));
    }

    #[test]
    fn legacy_or_malformed_data_is_not_performed() {
        for json in [
            r#"[{"id":"old","time":"2026-08-07T09:00:00Z"}]"#,
            r#"not-json"#,
        ] {
            let result = select_latest_tagged_snapshot_from_json(json, "daily");
            assert_eq!(result.status, SnapshotSelectionStatus::NotPerformed);
        }
    }
}

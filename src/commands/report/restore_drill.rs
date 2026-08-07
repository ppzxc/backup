use anyhow::Result;
use secrecy::{ExposeSecret, SecretString};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Instant;

const SCHEMA_VERSION: &str = "1";

/// The outcome of one Restore Drill requirement or storage/profile combination.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestoreDrillStatus {
    Pass,
    Fail,
    NotPerformed,
    NotApplicable,
}

impl RestoreDrillStatus {
    fn report_label(self) -> &'static str {
        match self {
            Self::Pass => "Pass",
            Self::Fail => "Fail",
            Self::NotPerformed => "NotPerformed",
            Self::NotApplicable => "NotApplicable",
        }
    }

    fn css_class(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Fail => "fail",
            Self::NotPerformed => "not-performed",
            Self::NotApplicable => "not-applicable",
        }
    }
}

/// One wall-clock label and monotonic reading captured at a Restore Drill boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreDrillTimestamp {
    pub wall_clock: String,
    pub monotonic_milliseconds: u64,
}

/// Clock seam used by Evidence collection. Renderers never read a clock.
pub trait RestoreDrillClock {
    fn now(&self) -> RestoreDrillTimestamp;
}

#[derive(Debug, Clone)]
pub struct SystemRestoreDrillClock {
    origin: Instant,
}

impl SystemRestoreDrillClock {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Default for SystemRestoreDrillClock {
    fn default() -> Self {
        Self::new()
    }
}

impl RestoreDrillClock for SystemRestoreDrillClock {
    fn now(&self) -> RestoreDrillTimestamp {
        RestoreDrillTimestamp {
            wall_clock: crate::commands::report::get_formatted_time().0,
            monotonic_milliseconds: self.origin.elapsed().as_millis() as u64,
        }
    }
}

/// The RTO and hard timeout applied to a Restore Drill execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreDrillPolicy {
    pub rto_minutes: u64,
    pub timeout_minutes: u64,
}

impl RestoreDrillPolicy {
    pub fn new(rto_minutes: u64, timeout_minutes: u64) -> Result<Self> {
        if rto_minutes == 0 {
            anyhow::bail!("Restore Drill RTO must be at least one minute");
        }
        if timeout_minutes < rto_minutes {
            anyhow::bail!("Restore Drill timeout must be at least the RTO");
        }
        Ok(Self {
            rto_minutes,
            timeout_minutes,
        })
    }

    pub fn rto_milliseconds(&self) -> u64 {
        self.rto_minutes.saturating_mul(60_000)
    }

    pub fn timeout_milliseconds(&self) -> u64 {
        self.timeout_minutes.saturating_mul(60_000)
    }

    pub fn is_within_rto(&self, elapsed_milliseconds: u64) -> bool {
        elapsed_milliseconds <= self.rto_milliseconds()
    }

    pub fn validate(&self) -> Result<()> {
        Self::new(self.rto_minutes, self.timeout_minutes).map(|_| ())
    }
}

impl Default for RestoreDrillPolicy {
    fn default() -> Self {
        Self {
            rto_minutes: 120,
            timeout_minutes: 240,
        }
    }
}

/// Measurements and judgment for one Backend Profile × Backup Profile pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreDrillStorageResult {
    pub profile: String,
    pub backend: String,
    pub status: RestoreDrillStatus,
    pub snapshot_id: Option<String>,
    pub snapshot_time: Option<String>,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_milliseconds: Option<u64>,
    pub elapsed_seconds: Option<u64>,
    pub file_count: Option<u64>,
    pub total_bytes: Option<u64>,
    pub validation_method: Option<String>,
    pub validation_status: RestoreDrillStatus,
    pub rto_satisfied: Option<bool>,
    pub diagnostic: Option<String>,
}

impl RestoreDrillStorageResult {
    #[allow(clippy::too_many_arguments)]
    pub fn measured(
        profile: impl Into<String>,
        backend: impl Into<String>,
        snapshot_id: impl Into<String>,
        snapshot_time: impl Into<String>,
        started_at: impl Into<String>,
        finished_at: impl Into<String>,
        elapsed_milliseconds: u64,
        file_count: u64,
        total_bytes: u64,
        validation_method: impl Into<String>,
        rto_policy: &RestoreDrillPolicy,
    ) -> Self {
        let rto_satisfied = rto_policy.is_within_rto(elapsed_milliseconds);
        let output_valid = file_count > 0 && total_bytes > 0;
        let validation_status = if output_valid {
            RestoreDrillStatus::Pass
        } else {
            RestoreDrillStatus::Fail
        };
        let status = if rto_satisfied && output_valid {
            RestoreDrillStatus::Pass
        } else {
            RestoreDrillStatus::Fail
        };
        Self {
            profile: profile.into(),
            backend: backend.into(),
            status,
            snapshot_id: Some(snapshot_id.into()),
            snapshot_time: Some(snapshot_time.into()),
            started_at: started_at.into(),
            finished_at: finished_at.into(),
            elapsed_seconds: Some(elapsed_milliseconds / 1_000),
            elapsed_milliseconds: Some(elapsed_milliseconds),
            file_count: Some(file_count),
            total_bytes: Some(total_bytes),
            validation_method: Some(validation_method.into()),
            validation_status,
            rto_satisfied: Some(rto_satisfied),
            diagnostic: if !rto_satisfied {
                Some("Restore Drill RTO exceeded".into())
            } else if !output_valid {
                Some("Restore Output Validation produced no non-empty output".into())
            } else {
                None
            },
        }
    }

    pub fn failed(
        profile: impl Into<String>,
        backend: impl Into<String>,
        started_at: impl Into<String>,
        finished_at: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            profile: profile.into(),
            backend: backend.into(),
            status: RestoreDrillStatus::Fail,
            snapshot_id: None,
            snapshot_time: None,
            started_at: started_at.into(),
            finished_at: finished_at.into(),
            elapsed_milliseconds: None,
            elapsed_seconds: None,
            file_count: None,
            total_bytes: None,
            validation_method: None,
            validation_status: RestoreDrillStatus::NotPerformed,
            rto_satisfied: None,
            diagnostic: Some(diagnostic.into()),
        }
    }

    pub fn failed_after_snapshot(
        profile: impl Into<String>,
        backend: impl Into<String>,
        snapshot_id: impl Into<String>,
        snapshot_time: impl Into<String>,
        started_at: impl Into<String>,
        finished_at: impl Into<String>,
        elapsed_milliseconds: u64,
        diagnostic: impl Into<String>,
    ) -> Self {
        let mut result = Self::failed(profile, backend, started_at, finished_at, diagnostic);
        result.snapshot_id = Some(snapshot_id.into());
        result.snapshot_time = Some(snapshot_time.into());
        result.elapsed_milliseconds = Some(elapsed_milliseconds);
        result.elapsed_seconds = Some(elapsed_milliseconds / 1_000);
        result
    }

    pub fn with_timing(
        mut self,
        started_at: impl Into<String>,
        finished_at: impl Into<String>,
        elapsed_milliseconds: u64,
    ) -> Self {
        self.started_at = started_at.into();
        self.finished_at = finished_at.into();
        self.elapsed_milliseconds = Some(elapsed_milliseconds);
        self.elapsed_seconds = Some(elapsed_milliseconds / 1_000);
        self
    }

    pub fn not_performed(
        profile: impl Into<String>,
        backend: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            profile: profile.into(),
            backend: backend.into(),
            status: RestoreDrillStatus::NotPerformed,
            snapshot_id: None,
            snapshot_time: None,
            started_at: String::new(),
            finished_at: String::new(),
            elapsed_milliseconds: None,
            elapsed_seconds: None,
            file_count: None,
            total_bytes: None,
            validation_method: None,
            validation_status: RestoreDrillStatus::NotPerformed,
            rto_satisfied: None,
            diagnostic: Some(diagnostic.into()),
        }
    }

    pub fn not_applicable(
        profile: impl Into<String>,
        backend: impl Into<String>,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            profile: profile.into(),
            backend: backend.into(),
            status: RestoreDrillStatus::NotApplicable,
            snapshot_id: None,
            snapshot_time: None,
            started_at: String::new(),
            finished_at: String::new(),
            elapsed_milliseconds: None,
            elapsed_seconds: None,
            file_count: None,
            total_bytes: None,
            validation_method: None,
            validation_status: RestoreDrillStatus::NotApplicable,
            rto_satisfied: None,
            diagnostic: Some(diagnostic.into()),
        }
    }
}

/// Immutable, serializable evidence for one Restore Drill execution.
///
/// The collection side of the future Restore Drill implementation will create this value once.
/// The render functions below only consume it; they do not query time, filesystems, or adapters.
#[derive(Debug, Clone, Deserialize)]
pub struct RestoreDrillEvidence {
    pub schema_version: String,
    pub execution_id: String,
    pub hostname: String,
    pub started_at: String,
    pub finished_at: String,
    pub overall_status: RestoreDrillStatus,
    pub rto_policy: RestoreDrillPolicy,
    pub storage_results: Vec<RestoreDrillStorageResult>,
    pub diagnostics: Vec<String>,
    pub tester: String,
    pub ciso: String,
    pub target_directory: Option<String>,
    #[serde(skip)]
    sensitive_values: Vec<SecretString>,
}

#[derive(Serialize)]
struct RestoreDrillEvidenceView<'a> {
    schema_version: &'a str,
    execution_id: &'a str,
    hostname: &'a str,
    started_at: &'a str,
    finished_at: &'a str,
    overall_status: RestoreDrillStatus,
    rto_policy: &'a RestoreDrillPolicy,
    storage_results: &'a [RestoreDrillStorageResult],
    diagnostics: &'a [String],
    tester: &'a str,
    ciso: &'a str,
    target_directory: &'a Option<String>,
}

impl Serialize for RestoreDrillEvidence {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let redacted = self.redacted();
        RestoreDrillEvidenceView {
            schema_version: &redacted.schema_version,
            execution_id: &redacted.execution_id,
            hostname: &redacted.hostname,
            started_at: &redacted.started_at,
            finished_at: &redacted.finished_at,
            overall_status: redacted.overall_status,
            rto_policy: &redacted.rto_policy,
            storage_results: &redacted.storage_results,
            diagnostics: &redacted.diagnostics,
            tester: &redacted.tester,
            ciso: &redacted.ciso,
            target_directory: &redacted.target_directory,
        }
        .serialize(serializer)
    }
}

impl PartialEq for RestoreDrillEvidence {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.execution_id == other.execution_id
            && self.hostname == other.hostname
            && self.started_at == other.started_at
            && self.finished_at == other.finished_at
            && self.overall_status == other.overall_status
            && self.rto_policy == other.rto_policy
            && self.storage_results == other.storage_results
            && self.diagnostics == other.diagnostics
            && self.tester == other.tester
            && self.ciso == other.ciso
            && self.target_directory == other.target_directory
    }
}

impl Eq for RestoreDrillEvidence {}

impl RestoreDrillEvidence {
    pub fn new(
        execution_id: impl Into<String>,
        started_at: impl Into<String>,
        finished_at: impl Into<String>,
        rto_policy: RestoreDrillPolicy,
        storage_results: Vec<RestoreDrillStorageResult>,
    ) -> Self {
        let overall_status = aggregate_status(&storage_results);
        Self {
            schema_version: SCHEMA_VERSION.into(),
            execution_id: execution_id.into(),
            hostname: String::new(),
            started_at: started_at.into(),
            finished_at: finished_at.into(),
            overall_status,
            rto_policy,
            storage_results,
            diagnostics: Vec::new(),
            tester: String::new(),
            ciso: String::new(),
            target_directory: None,
            sensitive_values: Vec::new(),
        }
    }

    pub fn with_metadata(
        mut self,
        hostname: impl Into<String>,
        tester: impl Into<String>,
        ciso: impl Into<String>,
        target_directory: Option<String>,
    ) -> Self {
        self.hostname = hostname.into();
        self.tester = tester.into();
        self.ciso = ciso.into();
        self.target_directory = target_directory;
        self
    }

    pub fn with_diagnostics(mut self, diagnostics: Vec<String>) -> Self {
        let sensitive = self
            .sensitive_values
            .iter()
            .map(|value| value.expose_secret().as_str())
            .collect::<Vec<_>>();
        self.diagnostics = diagnostics
            .into_iter()
            .map(|diagnostic| mask_diagnostic(&diagnostic, &sensitive))
            .collect();
        self
    }

    /// Registers values that must never appear in a rendered evidence artifact.
    ///
    /// Values are retained only in memory and are skipped by serialization. The pure renderers
    /// apply them to both top-level and per-result diagnostics before HTML escaping or JSON
    /// encoding.
    pub fn with_sensitive_values<I, S>(mut self, values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.sensitive_values = values
            .into_iter()
            .map(Into::into)
            .filter(|value| !value.is_empty())
            .map(SecretString::new)
            .collect();
        let sensitive = self
            .sensitive_values
            .iter()
            .map(|value| value.expose_secret().as_str())
            .collect::<Vec<_>>();
        self.diagnostics = self
            .diagnostics
            .iter()
            .map(|diagnostic| mask_diagnostic(diagnostic, &sensitive))
            .collect();
        for result in &mut self.storage_results {
            result.diagnostic = result
                .diagnostic
                .as_deref()
                .map(|diagnostic| mask_diagnostic(diagnostic, &sensitive));
        }
        self
    }

    fn redacted(&self) -> Self {
        let mut redacted = self.clone();
        redacted.overall_status = aggregate_status(&self.storage_results);
        let sensitive = self
            .sensitive_values
            .iter()
            .map(|value| value.expose_secret().as_str())
            .collect::<Vec<_>>();
        redacted.diagnostics = self
            .diagnostics
            .iter()
            .map(|diagnostic| mask_diagnostic(diagnostic, &sensitive))
            .collect();
        redacted.storage_results = self
            .storage_results
            .iter()
            .map(|result| {
                let mut result = result.clone();
                result.diagnostic = result
                    .diagnostic
                    .as_deref()
                    .map(|diagnostic| mask_diagnostic(diagnostic, &sensitive));
                result
            })
            .collect();
        redacted.target_directory = self
            .target_directory
            .as_deref()
            .filter(|value| !looks_like_path(value))
            .map(str::to_owned);
        redacted.sensitive_values.clear();
        redacted
    }
}

pub fn aggregate_status(results: &[RestoreDrillStorageResult]) -> RestoreDrillStatus {
    if results
        .iter()
        .any(|result| result.status == RestoreDrillStatus::Fail)
    {
        return RestoreDrillStatus::Fail;
    }
    if results
        .iter()
        .any(|result| result.status == RestoreDrillStatus::NotPerformed)
    {
        return RestoreDrillStatus::NotPerformed;
    }
    if results
        .iter()
        .any(|result| result.status == RestoreDrillStatus::Pass)
    {
        return RestoreDrillStatus::Pass;
    }
    if results
        .iter()
        .any(|result| result.status == RestoreDrillStatus::NotApplicable)
    {
        return RestoreDrillStatus::NotApplicable;
    }
    RestoreDrillStatus::NotPerformed
}

fn mask_diagnostic(value: &str, sensitive_values: &[&str]) -> String {
    crate::commands::redact_diagnostic(value, sensitive_values)
        .split_whitespace()
        .map(|token| {
            let value = token
                .split_once('=')
                .map(|(_, value)| value)
                .unwrap_or(token);
            if token.starts_with('/')
                || token.starts_with('\\')
                || value.starts_with('/')
                || value.starts_with('\\')
                || value.starts_with("./")
                || value.starts_with("../")
                || token.contains("://")
                || value.contains("://")
                || is_windows_absolute_path(value)
            {
                "***MASKED***"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .replace("<redacted>", "***MASKED***")
        .replace("******", "***MASKED***")
}

fn looks_like_path(value: &str) -> bool {
    value.starts_with('/')
        || value.starts_with('\\')
        || value.starts_with("./")
        || value.starts_with("../")
        || value.contains('/')
        || value.contains('\\')
        || is_windows_absolute_path(value)
}

fn is_windows_absolute_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'/' || bytes[2] == b'\\')
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn display(value: Option<&str>) -> String {
    value.map(escape_html).unwrap_or_else(|| "—".into())
}

fn display_number(value: Option<u64>) -> String {
    value
        .map(|number| number.to_string())
        .unwrap_or_else(|| "—".into())
}

fn display_elapsed(value: Option<u64>) -> String {
    value
        .map(|number| format!("{number} ms"))
        .unwrap_or_else(|| "—".into())
}

fn format_number(value: u64) -> String {
    let digits = value.to_string();
    let mut formatted = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            formatted.push(',');
        }
        formatted.push(digit);
    }
    formatted
}

fn format_bytes(value: Option<u64>) -> String {
    let Some(value) = value else {
        return "—".into();
    };
    if value < 1024 {
        return format_number(value).to_string() + " B";
    }
    let units = ["KiB", "MiB", "GiB", "TiB"];
    let mut amount = value as f64;
    let mut unit = "B";
    for candidate in units {
        amount /= 1024.0;
        unit = candidate;
        if amount < 1024.0 {
            break;
        }
    }
    format!("{amount:.1} {unit} ({})", format_number(value))
}

fn status_badge(status: RestoreDrillStatus) -> String {
    format!(
        "<span class=\"status-badge overall-status-{}\">{}</span>",
        status.css_class(),
        status.report_label()
    )
}

/// Pure HTML renderer for a previously collected Restore Drill Evidence value.
pub fn render_restore_drill_evidence_html(evidence: &RestoreDrillEvidence) -> String {
    let evidence = evidence.redacted();
    let mut rows = String::new();
    for result in &evidence.storage_results {
        let diagnostic = result
            .diagnostic
            .as_deref()
            .map(escape_html)
            .unwrap_or_else(|| "—".into());
        rows.push_str(&format!(
            r#"<tr>
  <td>{}</td><td>{}</td><td>{}</td><td>{}</td>
  <td>{}</td><td>{}</td><td>{}</td><td>{}</td>
  <td>{}</td><td>{}</td><td>{}</td><td>{}</td>
</tr>"#,
            escape_html(&result.profile),
            escape_html(&result.backend),
            display(result.snapshot_id.as_deref()),
            display(result.snapshot_time.as_deref()),
            display_elapsed(result.elapsed_milliseconds),
            display_number(result.file_count),
            display_number(result.total_bytes),
            escape_html(
                &result
                    .validation_method
                    .clone()
                    .unwrap_or_else(|| "—".into())
            ),
            status_badge(result.validation_status),
            status_badge(result.status),
            result
                .rto_satisfied
                .map(|satisfied| if satisfied { "yes" } else { "no" })
                .unwrap_or("—"),
            diagnostic,
        ));
    }

    let top_level_diagnostics = if evidence.diagnostics.is_empty() {
        "<li>—</li>".into()
    } else {
        evidence
            .diagnostics
            .iter()
            .map(|diagnostic| format!("<li>{}</li>", escape_html(diagnostic)))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Restore Drill Evidence</title>
  <style>
    body {{ font-family: sans-serif; color: #1e293b; background: #f8fafc; margin: 0; padding: 24px; }}
    .evidence-card {{ max-width: 1500px; margin: 0 auto; background: white; padding: 28px; border: 1px solid #cbd5e1; border-radius: 8px; }}
    h1 {{ margin-top: 0; }}
    table {{ width: 100%; border-collapse: collapse; margin: 16px 0 24px; }}
    th, td {{ border: 1px solid #cbd5e1; padding: 7px 9px; text-align: left; vertical-align: top; font-size: 13px; }}
    th {{ background: #f1f5f9; }}
    .status-badge {{ display: inline-block; padding: 2px 7px; border-radius: 4px; font-weight: 600; }}
    .overall-status-pass {{ background: #dcfce7; color: #166534; }}
    .overall-status-fail {{ background: #fee2e2; color: #991b1b; }}
    .overall-status-not-performed, .overall-status-not-applicable {{ background: #fef3c7; color: #92400e; }}
    .diagnostics {{ white-space: pre-wrap; word-break: break-word; }}
  </style>
</head>
<body>
<main class="evidence-card">
  <h1>Restore Drill Evidence</h1>
  <table>
    <tr><th>Execution ID</th><td>{}</td><th>Overall status</th><td>{}</td></tr>
    <tr><th>Host</th><td>{}</td><th>Schema version</th><td>{}</td></tr>
    <tr><th>Started</th><td>{}</td><th>Finished</th><td>{}</td></tr>
    <tr><th>RTO policy</th><td>{} minutes</td><th>Timeout policy</th><td>{} minutes</td></tr>
  </table>

  <h2>Storage and profile results</h2>
  <table>
    <thead><tr><th>Profile</th><th>Backend</th><th>Snapshot ID</th><th>Snapshot time</th><th>Elapsed ms</th><th>Files</th><th>Bytes</th><th>Validation method</th><th>Validation</th><th>Result</th><th>RTO</th><th>Diagnostic</th></tr></thead>
    <tbody>{}</tbody>
  </table>

  <h2>Diagnostics</h2>
  <ul class="diagnostics">{}</ul>

  <p>Tester: {} | CISO: {}</p>
</main>
</body>
</html>"#,
        escape_html(&evidence.execution_id),
        status_badge(evidence.overall_status),
        escape_html(&evidence.hostname),
        escape_html(&evidence.schema_version),
        escape_html(&evidence.started_at),
        escape_html(&evidence.finished_at),
        evidence.rto_policy.rto_minutes,
        evidence.rto_policy.timeout_minutes,
        rows,
        top_level_diagnostics,
        escape_html(&evidence.tester),
        escape_html(&evidence.ciso),
    )
}

fn primary_result(evidence: &RestoreDrillEvidence) -> Option<&RestoreDrillStorageResult> {
    evidence
        .storage_results
        .iter()
        .find(|result| result.backend == "primary")
}

fn human_elapsed(milliseconds: Option<u64>) -> String {
    milliseconds
        .map(|value| format!("{value} ms"))
        .unwrap_or_else(|| "not measured".into())
}

/// Pure JSON renderer for a previously collected Restore Drill Evidence value.
///
/// The compatibility fields are derived from the primary result in this same value. They remain
/// present for existing consumers while `storage_results` is the lossless multi-profile contract.
pub fn render_restore_drill_evidence_json(evidence: &RestoreDrillEvidence) -> Result<String> {
    let evidence = evidence.redacted();
    let primary = primary_result(&evidence);
    let mut value = serde_json::to_value(&evidence)?;
    let object = value
        .as_object_mut()
        .expect("RestoreDrillEvidence serializes as an object");
    let status = evidence.overall_status;
    let primary_snapshot_id = primary.and_then(|result| result.snapshot_id.clone());
    let primary_snapshot_time = primary.and_then(|result| result.snapshot_time.clone());
    let primary_elapsed_ms = primary.and_then(|result| result.elapsed_milliseconds);
    let primary_elapsed_seconds = primary
        .and_then(|result| result.elapsed_seconds)
        .or_else(|| primary_elapsed_ms.map(|value| value / 1_000));
    let primary_rto_satisfied = primary.and_then(|result| result.rto_satisfied);
    let primary_integrity = primary.map(|result| {
        result.status == RestoreDrillStatus::Pass
            && result.validation_status == RestoreDrillStatus::Pass
    });

    object.insert("report_type".into(), json!("restore_drill"));
    object.insert("timestamp".into(), json!(evidence.started_at));
    object.insert(
        "test_date".into(),
        json!(evidence.started_at.get(..10).unwrap_or_default()),
    );
    object.insert("report_status".into(), json!(status.report_label()));
    if status != RestoreDrillStatus::Pass {
        if let Some(diagnostic) = evidence
            .diagnostics
            .first()
            .cloned()
            .or_else(|| primary.and_then(|result| result.diagnostic.clone()))
        {
            object.insert("failure_diagnostic".into(), json!(diagnostic));
        }
    }
    object.insert(
        "target_snapshot_id".into(),
        json!(primary_snapshot_id.unwrap_or_default()),
    );
    object.insert(
        "target_snapshot_time".into(),
        json!(primary_snapshot_time.unwrap_or_default()),
    );
    object.insert(
        "recovery_results".into(),
        json!({
            "data_size_human": human_bytes(primary.and_then(|result| result.total_bytes)),
            "elapsed_seconds": primary_elapsed_seconds.unwrap_or(0),
            "elapsed_milliseconds": primary_elapsed_ms,
            "elapsed_human": human_elapsed(primary_elapsed_ms),
            "target_rto_minutes": evidence.rto_policy.rto_minutes,
            "timeout_minutes": evidence.rto_policy.timeout_minutes,
            "rto_satisfied": primary_rto_satisfied.unwrap_or(false),
            "data_integrity_verified": primary_integrity.unwrap_or(false),
            "database_verification": Value::Null,
        }),
    );
    object.insert("hostname".into(), json!(evidence.hostname));
    object.insert("tester".into(), json!(evidence.tester));
    object.insert("ciso".into(), json!(evidence.ciso));
    object.insert(
        "target_directory".into(),
        json!(evidence.target_directory.unwrap_or_default()),
    );

    Ok(serde_json::to_string_pretty(&value)?)
}

fn human_bytes(value: Option<u64>) -> String {
    value
        .map(|value| format_bytes(Some(value)))
        .unwrap_or_else(|| "not measured".into())
}

#[cfg(test)]
mod tests {
    use super::{
        RestoreDrillPolicy, RestoreDrillStatus, RestoreDrillStorageResult, aggregate_status,
        mask_diagnostic,
    };

    #[test]
    fn policy_uses_inclusive_rto_boundary() {
        let policy = RestoreDrillPolicy::new(1, 2).unwrap();
        assert!(policy.is_within_rto(60_000));
        assert!(!policy.is_within_rto(60_001));
        assert!(RestoreDrillPolicy::new(0, 1).is_err());
        assert!(RestoreDrillPolicy::new(2, 1).is_err());
    }

    #[test]
    fn status_aggregation_follows_failure_precedence() {
        let not_performed = RestoreDrillStorageResult::not_performed("files", "primary", "no tag");
        let not_applicable =
            RestoreDrillStorageResult::not_applicable("files", "secondary", "not configured");
        assert_eq!(
            aggregate_status(&[not_performed.clone(), not_applicable.clone()]),
            RestoreDrillStatus::NotPerformed
        );
        assert_eq!(
            aggregate_status(&[not_applicable]),
            RestoreDrillStatus::NotApplicable
        );
    }

    #[test]
    fn masking_hides_registered_values_and_infrastructure_locations() {
        let masked = mask_diagnostic(
            "failed password at /tmp/drill using https://storage.example/repo",
            &["password"],
        );
        assert!(!masked.contains("password"));
        assert!(!masked.contains("/tmp/drill"));
        assert!(!masked.contains("https://storage.example/repo"));
        assert!(masked.contains("***MASKED***"));
    }
}

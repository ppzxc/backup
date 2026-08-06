pub mod copy;
pub mod doctor;
pub mod report;
pub mod restore;
pub mod run;
pub mod schedule;
pub mod setup;
pub mod snapshots;
pub mod status;
pub mod uninstall;
pub mod update;

pub mod database;

pub(crate) fn redact_diagnostic(value: &str, exact_secrets: &[&str]) -> String {
    let mut redacted = value.to_owned();
    for secret in exact_secrets
        .iter()
        .copied()
        .filter(|secret| !secret.is_empty())
    {
        redacted = redacted.replace(secret, "<redacted>");
    }
    redacted
        .split_whitespace()
        .map(|token| {
            let lower = token.to_ascii_lowercase();
            if lower.contains("password")
                || lower.contains("secret")
                || lower.contains("token")
                || lower.contains("credential")
            {
                return "<redacted>".to_string();
            }
            let Some(scheme_end) = token.find("://") else {
                return token.to_string();
            };
            let authority_start = scheme_end + 3;
            let Some(at_offset) = token[authority_start..].find('@') else {
                return token.to_string();
            };
            let at = authority_start + at_offset;
            if token[authority_start..at].contains(':') {
                format!("{}://<redacted>@{}", &token[..scheme_end], &token[at + 1..])
            } else {
                token.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

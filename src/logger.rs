//! Logger module for structured tracing and secret masking.

use std::fmt;
use tracing::field::{Field, Visit};
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::FormatFields;

pub const MASKED_VALUE: &str = "***MASKED***";

/// Returns `***MASKED***` if `field_name` is sensitive or if `value` indicates a secret,
/// otherwise returns `value.to_string()`.
pub fn mask_value(field_name: &str, value: &str) -> String {
    if is_sensitive_field(field_name)
        || value.contains("REDACTED")
        || value.contains("Secret(")
        || value.contains("[REDACTED]")
    {
        MASKED_VALUE.to_string()
    } else {
        value.to_string()
    }
}

/// Checks whether a field name is sensitive and should be masked.
pub fn is_sensitive_field(field_name: &str) -> bool {
    let lower = field_name.to_lowercase();
    lower == "password"
        || lower.ends_with("_password")
        || lower.ends_with("password")
        || lower == "access_key"
        || lower.ends_with("_access_key")
        || lower.ends_with("access_key")
        || lower == "secret_key"
        || lower.ends_with("_secret_key")
        || lower.ends_with("secret_key")
        || lower == "token"
        || lower.ends_with("_token")
        || lower.ends_with("token")
        || lower == "secret"
        || lower.ends_with("_secret")
        || lower.ends_with("secret")
        || lower == "credential"
        || lower.ends_with("_credential")
        || lower.ends_with("credential")
        || lower == "credentials"
        || lower.ends_with("_credentials")
        || lower.ends_with("credentials")
        || ((lower == "key" || lower.ends_with("_key") || lower.ends_with("key"))
            && !lower.ends_with("public_key")
            && !lower.ends_with("profile_key"))
}

/// Visitor that formats tracing fields while masking sensitive values.
pub struct SecretMaskingVisitor<'a, W> {
    writer: &'a mut W,
    is_first: bool,
    result: fmt::Result,
}

impl<'a, W: fmt::Write> SecretMaskingVisitor<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self {
            writer,
            is_first: true,
            result: Ok(()),
        }
    }

    pub fn result(&self) -> fmt::Result {
        self.result
    }

    pub fn write_field(&mut self, name: &str, val: &str) {
        if self.result.is_err() {
            return;
        }
        let prefix = if self.is_first {
            self.is_first = false;
            ""
        } else {
            " "
        };
        let masked = mask_value(name, val);
        self.result = write!(self.writer, "{}{}={}", prefix, name, masked);
    }
}

impl<'a, W: fmt::Write> Visit for SecretMaskingVisitor<'a, W> {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.write_field(field.name(), value);
    }

    fn record_debug(&mut self, field: &Field, value: &dyn fmt::Debug) {
        let dbg_val = format!("{:?}", value);
        self.write_field(field.name(), &dbg_val);
    }
}

/// Custom field formatter for `tracing-subscriber` that masks sensitive fields.
#[derive(Default, Debug, Clone)]
pub struct SecretMaskingFormatter;

impl SecretMaskingFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl<'writer> FormatFields<'writer> for SecretMaskingFormatter {
    fn format_fields<R: RecordFields>(
        &self,
        mut writer: Writer<'writer>,
        fields: R,
    ) -> fmt::Result {
        let mut visitor = SecretMaskingVisitor::new(&mut writer);
        fields.record(&mut visitor);
        visitor.result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::SecretString;

    #[test]
    fn test_secret_masking() {
        assert_eq!(mask_value("password", "supersecret"), "***MASKED***");
        assert_eq!(mask_value("access_key", "AKIA12345"), "***MASKED***");
        assert_eq!(mask_value("secret_key", "secret123"), "***MASKED***");
        assert_eq!(mask_value("token", "bearer_token"), "***MASKED***");
        assert_eq!(mask_value("secret", "my_secret"), "***MASKED***");
        assert_eq!(mask_value("credential", "my_cred"), "***MASKED***");
        assert_eq!(mask_value("profile_name", "default"), "default");
    }

    #[test]
    fn test_secrecy_redacted_masking() {
        let secret = SecretString::new("hidden".to_string());
        let dbg_str = format!("{:?}", secret);
        assert_eq!(mask_value("custom_field", &dbg_str), "***MASKED***");
    }

    #[test]
    fn test_visitor_masking() {
        let mut buf = String::new();
        {
            let mut visitor = SecretMaskingVisitor::new(&mut buf);
            visitor.write_field("user", "alice");
            visitor.write_field("password", "secret123");
        }
        assert_eq!(buf, "user=alice password=***MASKED***");
    }
}

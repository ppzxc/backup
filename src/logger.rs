//! Logger module for structured tracing and secret masking.

use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tracing::field::{Field, Visit};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use std::sync::atomic::{AtomicBool, Ordering};
use tracing_subscriber::Layer;
use tracing_subscriber::filter::filter_fn;

pub const MASKED_VALUE: &str = "***MASKED***";

static WORKER_GUARD: Mutex<Option<WorkerGuard>> = Mutex::new(None);
static TUI_MODE: AtomicBool = AtomicBool::new(false);

/// Sets interactive TUI mode state for stderr console log suppression.
pub fn set_tui_mode(enabled: bool) {
    TUI_MODE.store(enabled, Ordering::SeqCst);
}

/// Returns true if interactive TUI mode is currently active.
pub fn is_tui_mode() -> bool {
    TUI_MODE.load(Ordering::SeqCst)
}

/// Emits a user-facing setup notice while preserving the TUI console filter.
pub fn interactive_notice(message: impl AsRef<str>) {
    if is_tui_mode() {
        tracing::warn!("{}", message.as_ref());
    } else {
        tracing::info!("{}", message.as_ref());
    }
}

/// Represents the active system log target in the 3-tier fallback pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemLogTarget {
    Journald(PathBuf),
    Syslog(PathBuf),
    File(PathBuf),
}

/// Configuration options for initializing the logging subsystem.
#[derive(Debug, Clone, Default)]
pub struct LogConfig {
    pub level_filter: String,
    pub log_file: Option<PathBuf>,
}

impl LogConfig {
    pub fn new(level_filter: impl Into<String>, log_file: Option<PathBuf>) -> Self {
        Self {
            level_filter: level_filter.into(),
            log_file,
        }
    }
}

/// Resolves the level filter string from verbosity flags, quiet flag, and optional env override.
pub fn determine_level_filter(verbose: u8, quiet: bool, env_override: Option<&str>) -> String {
    if quiet {
        return "warn".to_string();
    }
    if verbose > 0 {
        return match verbose {
            1 => "debug".to_string(),
            _ => "trace".to_string(),
        };
    }
    if let Some(env_val) = env_override {
        if !env_val.is_empty() {
            return env_val.to_string();
        }
    }
    "info".to_string()
}

/// Resolves the active system log target according to the 3-tier fallback policy:
/// 1. Primary: Systemd Journald socket (`/run/systemd/journal/socket`).
/// 2. Fallback 1: Syslog socket (`/dev/log`).
/// 3. Fallback 2: Log file at `/var/log/backup/backup.log` (or `~/.local/state/backup/backup.log` if unprivileged, or `custom_file`).
pub fn resolve_system_log_target(custom_file: Option<&Path>) -> SystemLogTarget {
    if let Some(file) = custom_file {
        return SystemLogTarget::File(file.to_path_buf());
    }

    let journald_path = Path::new("/run/systemd/journal/socket");
    if journald_path.exists() {
        return SystemLogTarget::Journald(journald_path.to_path_buf());
    }

    let syslog_path = Path::new("/dev/log");
    if syslog_path.exists() {
        return SystemLogTarget::Syslog(syslog_path.to_path_buf());
    }

    let var_log_dir = Path::new("/var/log/backup");
    if can_write_to_dir(var_log_dir) {
        return SystemLogTarget::File(var_log_dir.join("backup.log"));
    }

    SystemLogTarget::File(get_user_state_log_path())
}

fn can_write_to_dir(dir: &Path) -> bool {
    if !dir.exists() {
        if std::fs::create_dir_all(dir).is_err() {
            return false;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
        }
    }
    let test_file = dir.join(".write_test");
    if std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&test_file)
        .is_ok()
    {
        let _ = std::fs::remove_file(test_file);
        true
    } else {
        false
    }
}

fn get_user_state_log_path() -> PathBuf {
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        PathBuf::from(state_home).join("backup/backup.log")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".local/state/backup/backup.log")
    } else {
        PathBuf::from("/tmp/backup/backup.log")
    }
}

/// Enforces POSIX 700 permissions on the log directory and 600 permissions on the log file.
pub fn ensure_secure_log_file(log_file: &Path) -> Result<(), anyhow::Error> {
    if let Some(parent) = log_file.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))?;
        }
    }

    if !log_file.exists() {
        std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_file)?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(log_file, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

/// Writer for datagram socket system log targets (journald, syslog).
pub struct SocketWriter {
    path: PathBuf,
}

impl SocketWriter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl std::io::Write for SocketWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        #[cfg(unix)]
        {
            use std::os::unix::net::UnixDatagram;
            if let Ok(socket) = UnixDatagram::unbound() {
                let _ = socket.send_to(buf, &self.path);
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Initializes global tracing logging subscriber with stderr and 3-tier system target layers.
pub fn init_logging(config: LogConfig) -> Result<(), anyhow::Error> {
    let filter_str = if config.level_filter.is_empty() {
        "info"
    } else {
        &config.level_filter
    };

    let env_filter = EnvFilter::try_new(filter_str).unwrap_or_else(|_| EnvFilter::new("info"));

    let target = resolve_system_log_target(config.log_file.as_deref());

    let sys_writer: BoxMakeWriter = match target {
        SystemLogTarget::Journald(path) | SystemLogTarget::Syslog(path) => {
            BoxMakeWriter::new(move || SocketWriter::new(path.clone()))
        }
        SystemLogTarget::File(file_path) => {
            ensure_secure_log_file(&file_path)?;
            let parent = file_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            let file_name = PathBuf::from(
                file_path
                    .file_name()
                    .unwrap_or_else(|| std::ffi::OsStr::new("backup.log")),
            );

            let file_appender = tracing_appender::rolling::never(parent, file_name);
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            if let Ok(mut g) = WORKER_GUARD.lock() {
                *g = Some(guard);
            }
            BoxMakeWriter::new(non_blocking)
        }
    };

    let stderr_layer = tracing_subscriber::fmt::layer()
        .with_writer(std::io::stderr)
        .fmt_fields(SecretMaskingFormatter::new())
        .with_filter(filter_fn(|meta| {
            if is_tui_mode() {
                *meta.level() <= tracing::Level::WARN
            } else {
                true
            }
        }));

    let sys_layer = tracing_subscriber::fmt::layer()
        .with_writer(sys_writer)
        .fmt_fields(SecretMaskingFormatter::new());

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(stderr_layer)
        .with(sys_layer);

    let _ = subscriber.try_init();

    Ok(())
}

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

    #[test]
    fn test_log_level_filter_resolution() {
        assert_eq!(determine_level_filter(0, false, None), "info");
        assert_eq!(determine_level_filter(1, false, None), "debug");
        assert_eq!(determine_level_filter(2, false, None), "trace");
        assert_eq!(determine_level_filter(0, true, None), "warn");
        assert_eq!(determine_level_filter(0, false, Some("debug")), "debug");
    }

    #[test]
    fn test_tui_mode_toggle() {
        assert!(!is_tui_mode());
        set_tui_mode(true);
        assert!(is_tui_mode());
        set_tui_mode(false);
        assert!(!is_tui_mode());
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_custom_log_file_target_creation_and_permissions() {
        let dir = tempdir().unwrap();
        let log_file = dir.path().join("sub/test.log");
        let target = resolve_system_log_target(Some(&log_file));
        assert!(matches!(target, SystemLogTarget::File(_)));
        ensure_secure_log_file(&log_file).expect("Permissions set successfully");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let parent = log_file.parent().unwrap();
            let parent_meta = std::fs::metadata(parent).unwrap();
            assert_eq!(parent_meta.permissions().mode() & 0o777, 0o700);
            let file_meta = std::fs::metadata(&log_file).unwrap();
            assert_eq!(file_meta.permissions().mode() & 0o777, 0o600);
        }
    }

    #[test]
    fn test_resolve_system_log_target_default() {
        let target = resolve_system_log_target(None);
        match target {
            SystemLogTarget::Journald(p) => {
                assert_eq!(p, PathBuf::from("/run/systemd/journal/socket"))
            }
            SystemLogTarget::Syslog(p) => assert_eq!(p, PathBuf::from("/dev/log")),
            SystemLogTarget::File(p) => assert!(p.to_string_lossy().contains("backup.log")),
        }
    }

    #[test]
    fn test_init_logging() {
        let dir = tempdir().unwrap();
        let log_file = dir.path().join("init_test/test.log");
        let config = LogConfig::new("info", Some(log_file.clone()));
        assert!(init_logging(config).is_ok());
        assert!(log_file.exists());
    }
}

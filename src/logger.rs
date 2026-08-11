//! Logger module for structured tracing and secret masking.

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::field::{Field, Visit};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::field::RecordFields;
use tracing_subscriber::fmt::FormatFields;
use tracing_subscriber::fmt::format::Writer;
use tracing_subscriber::fmt::writer::BoxMakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub const MASKED_VALUE: &str = "***MASKED***";

static WORKER_GUARD: Mutex<Option<WorkerGuard>> = Mutex::new(None);

/// Represents the active system log target in the 3-tier fallback pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemLogTarget {
    Journald(PathBuf),
    Syslog(PathBuf),
    File(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DeliveryResult {
    Delivered,
    Lost,
}

/// The system-log delivery seam. Each event is offered to the sinks in order. A sink must
/// return an error when it did not accept the event; returning `Ok` means delivery succeeded and
/// stops fallback for that event. Losing one event is intentionally not an application error.
pub(crate) trait SystemLogSink: Send + Sync {
    fn deliver(&self, event: &[u8]) -> io::Result<()>;
}

pub(crate) struct SystemLogDelivery {
    sinks: Vec<Arc<dyn SystemLogSink>>,
}

impl SystemLogDelivery {
    pub(crate) fn from_sinks(sinks: Vec<Arc<dyn SystemLogSink>>) -> Self {
        Self { sinks }
    }

    pub(crate) fn deliver_event(&self, event: &[u8]) -> DeliveryResult {
        for sink in &self.sinks {
            if sink.deliver(event).is_ok() {
                return DeliveryResult::Delivered;
            }
        }
        DeliveryResult::Lost
    }

    fn default_sinks() -> Vec<Arc<dyn SystemLogSink>> {
        let mut sinks: Vec<Arc<dyn SystemLogSink>> = vec![
            Arc::new(UnixDatagramSink::new(PathBuf::from(
                "/run/systemd/journal/socket",
            ))),
            Arc::new(UnixDatagramSink::new(PathBuf::from("/dev/log"))),
            Arc::new(DailyFileSink::new(
                PathBuf::from("/var/log/backup"),
                "backup.log",
            )),
        ];
        if let Some(directory) = user_state_log_directory() {
            sinks.push(Arc::new(DailyFileSink::new(directory, "backup.log")));
        }
        sinks
    }
}

impl Default for SystemLogDelivery {
    fn default() -> Self {
        Self::from_sinks(Self::default_sinks())
    }
}

impl io::Write for SystemLogDelivery {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let _ = self.deliver_event(buf);
        // A failed system sink is deliberately not propagated to the command. The contract is
        // best-effort system diagnostics, while stdout/stderr and command status remain intact.
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

struct UnixDatagramSink {
    path: PathBuf,
}

impl UnixDatagramSink {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl SystemLogSink for UnixDatagramSink {
    fn deliver(&self, event: &[u8]) -> io::Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::net::UnixDatagram;

            let socket = UnixDatagram::unbound()?;
            let written = socket.send_to(event, &self.path)?;
            if written == event.len() {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "system log datagram was only partially written",
                ))
            }
        }
        #[cfg(not(unix))]
        {
            let _ = event;
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "unix datagram system logging is unavailable",
            ))
        }
    }
}

struct DailyFileSink {
    directory: PathBuf,
    prefix: String,
}

impl DailyFileSink {
    fn new(directory: PathBuf, prefix: impl Into<String>) -> Self {
        Self {
            directory,
            prefix: prefix.into(),
        }
    }

    fn path_for_today(&self) -> PathBuf {
        self.directory
            .join(format!("{}.{}", self.prefix, utc_date_stamp()))
    }
}

impl SystemLogSink for DailyFileSink {
    fn deliver(&self, event: &[u8]) -> io::Result<()> {
        ensure_app_owned_directory(&self.directory)
            .map_err(|error| io::Error::new(io::ErrorKind::PermissionDenied, error.to_string()))?;
        let path = self.path_for_today();
        if let Ok(metadata) = std::fs::symlink_metadata(&path) {
            if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "rotating log path is not a regular file",
                ));
            }
        }
        let mut options = std::fs::OpenOptions::new();
        options.create(true).append(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&path)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        }
        io::Write::write_all(&mut file, event)
    }
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

/// Resolves the first observable default target. Actual default delivery uses all candidates in
/// order for every event; this helper remains useful to callers that need to display the policy.
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
    if var_log_dir.exists() || Path::new("/var/log").exists() {
        return SystemLogTarget::File(var_log_dir.join("backup.log"));
    }

    SystemLogTarget::File(
        user_state_log_directory()
            .unwrap_or_else(|| PathBuf::from("/var/log/backup"))
            .join("backup.log"),
    )
}

fn user_state_log_directory() -> Option<PathBuf> {
    if let Ok(state_home) = std::env::var("XDG_STATE_HOME") {
        if !state_home.trim().is_empty() {
            return Some(PathBuf::from(state_home).join("backup"));
        }
    }
    std::env::var("HOME")
        .ok()
        .filter(|home| !home.trim().is_empty())
        .map(|home| PathBuf::from(home).join(".local/state/backup"))
}

/// Creates missing app-owned log directories with mode 700 and enforces mode 600 on the file.
/// Existing parent directories are never chmod'ed.
pub fn ensure_secure_log_file(log_file: &Path) -> Result<(), anyhow::Error> {
    if let Some(parent) = log_file.parent() {
        ensure_app_owned_directory(parent)?;
    }

    if log_file.exists() {
        let metadata = std::fs::symlink_metadata(log_file)?;
        if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
            anyhow::bail!(
                "explicit log file path is not a regular file: {}",
                log_file.display()
            );
        }
    }

    // Open the selected target before the subscriber is installed so explicit logging failures
    // cannot be hidden by the fallback pipeline or by a later command dispatch.
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(log_file, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

fn ensure_app_owned_directory(path: &Path) -> Result<(), anyhow::Error> {
    if path.as_os_str().is_empty() || path == Path::new(".") {
        return Ok(());
    }
    let mut missing = Vec::new();
    let mut current = path.to_path_buf();
    loop {
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                anyhow::bail!(
                    "log directory must not be a symbolic link: {}",
                    current.display()
                )
            }
            Ok(metadata) if !metadata.file_type().is_dir() => {
                anyhow::bail!("log directory is not a directory: {}", current.display())
            }
            Ok(_) => break,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                missing.push(current.clone());
                let Some(parent) = current.parent() else {
                    anyhow::bail!("log directory has no existing parent: {}", path.display())
                };
                if parent == current {
                    anyhow::bail!("log directory has no existing parent: {}", path.display())
                }
                current = parent.to_path_buf();
            }
            Err(error) => return Err(error.into()),
        }
    }
    std::fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for directory in missing {
            std::fs::set_permissions(directory, std::fs::Permissions::from_mode(0o700))?;
        }
    }
    Ok(())
}

fn utc_date_stamp() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| (duration.as_secs() / 86_400) as i64)
        .unwrap_or_default();
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    format!("{year:04}-{month:02}-{day:02}")
}

/// Writer kept for compatibility with callers that need one socket target directly.
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
        UnixDatagramSink::new(self.path.clone()).deliver(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Initializes global tracing logging. Default events use the per-event Journald -> Syslog ->
/// daily file delivery boundary. An explicit file is a single fail-fast, non-rotating target.
pub fn init_logging(config: LogConfig) -> Result<(), anyhow::Error> {
    let filter_str = if config.level_filter.is_empty() {
        "info"
    } else {
        &config.level_filter
    };

    let env_filter = EnvFilter::try_new(filter_str).unwrap_or_else(|_| EnvFilter::new("info"));

    let sys_writer: BoxMakeWriter = if let Some(file_path) = config.log_file {
        ensure_secure_log_file(&file_path)?;
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&file_path)?;
        let (non_blocking, guard) = tracing_appender::non_blocking(file);
        if let Ok(mut g) = WORKER_GUARD.lock() {
            *g = Some(guard);
        }
        BoxMakeWriter::new(non_blocking)
    } else {
        let (non_blocking, guard) = tracing_appender::non_blocking(SystemLogDelivery::default());
        if let Ok(mut g) = WORKER_GUARD.lock() {
            *g = Some(guard);
        }
        BoxMakeWriter::new(non_blocking)
    };

    let sys_layer = tracing_subscriber::fmt::layer()
        .with_writer(sys_writer)
        .fmt_fields(SecretMaskingFormatter::new());

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(sys_layer);

    let _ = subscriber.try_init();

    Ok(())
}

/// Flushes asynchronous system-log writes before the CLI process exits.
pub fn shutdown_logging() {
    if let Ok(mut guard) = WORKER_GUARD.lock() {
        guard.take();
    }
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
        assert_eq!(determine_level_filter(3, false, Some("info")), "trace");
        assert_eq!(determine_level_filter(1, false, Some("trace")), "debug");
        assert_eq!(determine_level_filter(0, true, None), "warn");
        assert_eq!(determine_level_filter(0, true, Some("trace")), "warn");
        assert_eq!(determine_level_filter(0, false, Some("debug")), "debug");
    }
}

#[cfg(test)]
mod fallback_tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::tempdir;

    struct FailingSink;

    impl SystemLogSink for FailingSink {
        fn deliver(&self, _event: &[u8]) -> io::Result<()> {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "sink unavailable",
            ))
        }
    }

    struct RecordingSink {
        events: Mutex<Vec<Vec<u8>>>,
    }

    impl SystemLogSink for RecordingSink {
        fn deliver(&self, event: &[u8]) -> io::Result<()> {
            self.events.lock().unwrap().push(event.to_vec());
            Ok(())
        }
    }

    #[test]
    fn system_log_delivery_falls_back_per_event_and_stops_after_first_success() {
        let recording = Arc::new(RecordingSink {
            events: Mutex::new(Vec::new()),
        });
        let delivery = SystemLogDelivery::from_sinks(vec![
            Arc::new(FailingSink),
            recording.clone(),
            Arc::new(FailingSink),
        ]);

        assert_eq!(
            delivery.deliver_event(b"event-one\n"),
            DeliveryResult::Delivered
        );
        assert_eq!(
            recording.events.lock().unwrap().as_slice(),
            [b"event-one\n"]
        );
    }

    #[test]
    fn system_log_delivery_loses_only_the_event_when_all_sinks_fail() {
        let delivery =
            SystemLogDelivery::from_sinks(vec![Arc::new(FailingSink), Arc::new(FailingSink)]);

        assert_eq!(delivery.deliver_event(b"one-event\n"), DeliveryResult::Lost);
    }

    #[cfg(unix)]
    #[test]
    fn unix_socket_failure_falls_back_to_the_next_socket_for_the_same_event() {
        use std::os::unix::net::UnixDatagram;

        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("syslog.sock");
        let receiver = UnixDatagram::bind(&socket_path).unwrap();
        receiver
            .set_read_timeout(Some(std::time::Duration::from_secs(1)))
            .unwrap();
        let delivery = SystemLogDelivery::from_sinks(vec![
            Arc::new(UnixDatagramSink::new(dir.path().join("missing.sock"))),
            Arc::new(UnixDatagramSink::new(socket_path)),
        ]);

        assert_eq!(
            delivery.deliver_event(b"socket-event\n"),
            DeliveryResult::Delivered
        );
        let mut received = [0; 64];
        let size = receiver.recv(&mut received).unwrap();
        assert_eq!(&received[..size], b"socket-event\n");
    }

    #[cfg(unix)]
    #[test]
    fn daily_file_sink_creates_each_log_file_with_mode_600() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        let sink = DailyFileSink::new(dir.path().to_path_buf(), "backup.log");
        sink.deliver(b"structured event\n").unwrap();

        let files = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect::<Vec<_>>();
        assert_eq!(files.len(), 1);
        assert!(
            files[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("backup.log.")
        );
        assert_eq!(
            std::fs::metadata(&files[0]).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }

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

    #[cfg(unix)]
    #[test]
    fn explicit_log_file_does_not_change_an_existing_parent_directory() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempdir().unwrap();
        std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        let log_file = dir.path().join("backup.log");

        init_logging(LogConfig::new("info", Some(log_file.clone()))).unwrap();

        assert_eq!(
            std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777,
            0o755
        );
        assert_eq!(
            std::fs::metadata(log_file).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

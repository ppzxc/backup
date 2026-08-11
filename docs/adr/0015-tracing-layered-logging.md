# 0015. Adopt tracing Layered Architecture and 3-Tier Diagnostic Logging

## Status

Superseded by ADR-0025

## Context

Current CLI implementation relied on ad-hoc `eprintln!` and `println!` statements for log and diagnostic message output. This caused several operational and security issues:
1. **Output Contamination**: CLI structured data stdout output (JSON reports, snapshot tables) mixed with unstructured log messages.
2. **Lack of Structured System Logging**: Running under `systemd` timers or background crons offered no native integration with `journald`, syslog, or dedicated log files.
3. **Secret Leakage Risk**: Sensitive credentials (`password`, `access_key`, `secret_key`, `token`, `SecretString`) could potentially be printed to console or log files in plain text during debug or error logging.
4. **Terminal UX Redraw Glitches**: Uncoordinated output statements interrupted interactive TUI elements (e.g. `inquire` setup wizard) and `indicatif` progress bars.

## Decision

1. **Adopt `tracing` + `tracing-subscriber` Layered Architecture**:
   - Replace all ad-hoc diagnostic `eprintln!` calls with structured `tracing` macros (`info!`, `warn!`, `error!`, `debug!`, `trace!`).
   - Use `tracing_subscriber::registry()` with modular layers to cleanly route and format events across multiple sinks.
2. **Strict Output Separation & 3-Tier Fallback System Logging**:
   - `stdout` is strictly reserved for machine-readable or explicit user data output (JSON reports, snapshot lists, version string).
   - `stderr` receives human-readable terminal log events controlled via `-v/--verbose`, `-q/--quiet`, or `BACKUP_LOG`/`RUST_LOG` environment variables.
   - Implement a mandatory 3-Tier System Logging Fallback Pipeline:
     1. **Primary**: Systemd Journald socket (`/run/systemd/journal/socket`).
     2. **Fallback 1**: Syslog socket (`/dev/log`).
     3. **Fallback 2**: Secure rotating log file at `/var/log/backup/backup.log` (with POSIX `700` directory and `600` file permission enforcement).
3. **Terminal UX Protection**:
   - Integrate with `tracing-indicatif` for progress bar management during long-running operations.
   - Suppress console `stderr` log emissions (limiting to `WARN`/`ERROR`) during interactive `inquire` TUI setup wizard executions (`backup setup`) while preserving background system log events.
4. **Automatic Secret Masking & Audit Traceability**:
   - Implement custom event formatting / visitor to redact sensitive fields (`password`, `access_key`, `secret_key`, `token`, `SecretString`) as `***MASKED***`.
   - Decorate major pipeline stages with `tracing::info_span!` (`profile resolution`, `database`, `primary backup`, `secondary sync`, `retention`) to enrich audit execution reports with spans context.

## Consequences

- Diagnostic logs and system operational events are captured reliably by systemd/journald, syslog, or local log files regardless of terminal attachment.
- Zero risk of secret exposure in diagnostic logs due to automated masking at the subscriber layer.
- Terminal UX remains clean and visual elements (progress bars, setup forms) operate without distortion.
- Backward compatibility preserved while enabling granular environment and flag-based log filtering.

# Backup CLI Logging System Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate `backup` CLI application's logging infrastructure from `eprintln!` to a modular layered `tracing` architecture with 3-tier system logging fallback (journald -> syslog -> rotating file), CLI verbosity controls (`-v`, `-q`, `--log-file`), secret masking, and terminal UX protection.

**Architecture:** A modular `tracing-subscriber` setup with a multi-writer layer split between stderr terminal output and system logging (Journald datagram socket -> Syslog datagram socket -> rotating log file with strict 700/600 permissions), a custom secret masking visitor for sensitive attributes, and `tracing-indicatif` integration.

**Tech Stack:** Rust, `tracing`, `tracing-subscriber`, `tracing-appender`, `tracing-indicatif`, `std::os::unix::net::UnixDatagram`.

## Global Constraints

- All system log directories created must have `700` POSIX permissions; log files must have `600` permissions.
- CLI output stdout must be reserved strictly for command data (JSON reports, snapshot tables, version).
- All log events (INFO, WARN, ERROR, DEBUG) must be directed to stderr and/or background system log target.
- Sensitive values (`password`, `access_key`, `secret_key`, `token`, `secret`, `credential`) must be masked as `***MASKED***` in all log outputs.

---

### Task 1: Record ADR-0015 & Update CONTEXT.md Glossary

**Files:**
- Create: `docs/adr/0015-tracing-layered-logging.md`
- Modify: `CONTEXT.md`

**Interfaces:**
- Consumes: Architectural decisions from `/tmp/handoff-backup-logging-migration.md`
- Produces: ADR-0015 and updated domain glossary for `System Diagnostic Logging`

- [ ] **Step 1: Create ADR-0015**

Create `docs/adr/0015-tracing-layered-logging.md` documenting the decision to adopt `tracing` + `tracing-subscriber` layered architecture, 3-tier logging fallback (Journald -> Syslog -> Rotating File), secret masking, and terminal UX protection.

- [ ] **Step 2: Update CONTEXT.md Glossary**

Add `System Diagnostic Logging` term to `CONTEXT.md` under the domain glossary section detailing the 3-tier fallback strategy and secret masking rules.

- [ ] **Step 3: Commit Documentation Changes**

```bash
git add docs/adr/0015-tracing-layered-logging.md CONTEXT.md
git commit -m "docs: add ADR-0015 for tracing layered logging and update CONTEXT.md"
```

---

### Task 2: Add Logging Dependencies to Cargo.toml

**Files:**
- Modify: `Cargo.toml:7-22`

**Interfaces:**
- Consumes: Crate versions for `tracing-appender`, `tracing-indicatif`
- Produces: Updated `Cargo.toml` with logging dependencies available for `src/logger.rs`

- [ ] **Step 1: Update Cargo.toml dependencies**

Add `tracing-appender = "0.2"` and `tracing-indicatif = "0.3"` to `[dependencies]` in `Cargo.toml`.

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: PASS with new crates fetched and compiled.

- [ ] **Step 3: Commit Cargo.toml changes**

```bash
git add Cargo.toml Cargo.lock
git commit -m "build: add tracing-appender and tracing-indicatif dependencies"
```

---

### Task 3: Implement Secret Masking Visitor & Formatter in `src/logger.rs`

**Files:**
- Create: `src/logger.rs`
- Modify: `src/lib.rs:1-7`

**Interfaces:**
- Consumes: `tracing::field::Visit`, `tracing_subscriber::fmt::FormatFields`
- Produces: `pub struct SecretMaskingVisitor` and `pub fn mask_value(field_name: &str, value: &str) -> String`

- [ ] **Step 1: Write failing unit test for Secret Masking**

In `src/logger.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secret_masking() {
        assert_eq!(mask_value("password", "supersecret"), "***MASKED***");
        assert_eq!(mask_value("access_key", "AKIA12345"), "***MASKED***");
        assert_eq!(mask_value("secret_key", "secret123"), "***MASKED***");
        assert_eq!(mask_value("token", "bearer_token"), "***MASKED***");
        assert_eq!(mask_value("profile_name", "default"), "default");
    }
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --lib logger::tests::test_secret_masking`
Expected: FAIL (module or function not found)

- [ ] **Step 3: Implement Secret Masking logic**

Implement `mask_value` and `SecretMaskingVisitor` in `src/logger.rs`, and export `pub mod logger;` in `src/lib.rs`.

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test --lib logger::tests::test_secret_masking`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/logger.rs src/lib.rs
git commit -m "feat(logging): implement secret masking visitor for sensitive tracing fields"
```

---

### Task 4: Implement 3-Tier System Logger Fallback Pipeline and File Permissions

**Files:**
- Modify: `src/logger.rs`

**Interfaces:**
- Consumes: POSIX socket `/run/systemd/journal/socket`, `/dev/log`, file appender with `700`/`600` permissions
- Produces: `pub enum SystemLogTarget`, `pub fn resolve_system_log_target(custom_file: Option<&Path>) -> SystemLogTarget`, `pub fn init_logging(config: LogConfig) -> Result<(), anyhow::Error>`

- [ ] **Step 1: Write failing unit tests for Log Target Resolution and Directory/File Permissions**

In `src/logger.rs`:
```rust
#[cfg(test)]
mod fallback_tests {
    use super::*;
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
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test --lib logger::fallback_tests`
Expected: FAIL

- [ ] **Step 3: Implement 3-Tier Log Target Resolution and Security Enforcer**

Implement `resolve_system_log_target` checking `/run/systemd/journal/socket`, `/dev/log`, and falling back to `/var/log/backup/backup.log` (or `~/.local/state/backup/backup.log` if unprivileged) enforcing `700`/`600` permissions.

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test --lib logger::fallback_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/logger.rs
git commit -m "feat(logging): add 3-tier system log fallback pipeline and 700/600 permission enforcement"
```

---

### Task 5: Integrate CLI Flags, Environment Filter, and TUI Suppression

**Files:**
- Modify: `src/main.rs:6-15`
- Modify: `src/logger.rs`
- Modify: `src/commands/setup.rs`

**Interfaces:**
- Consumes: `Cli` struct with `-v`, `-q`, `--log-file` args
- Produces: Global logging initialization in `main()` with environment override (`BACKUP_LOG` / `RUST_LOG`) and interactive TUI stderr suppression helper `set_tui_mode(bool)`.

- [ ] **Step 1: Add CLI arguments to `Cli` struct in `src/main.rs`**

Add `--verbose` (`-v`), `--quiet` (`-q`), `--log-file` to `Cli` struct:
```rust
#[arg(long, short = 'v', global = true, action = clap::ArgAction::Count)]
verbose: u8,
#[arg(long, short = 'q', global = true)]
quiet: bool,
#[arg(long, global = true, value_name = "PATH")]
log_file: Option<PathBuf>,
```

- [ ] **Step 2: Write failing unit test for Log Level Filter Resolution**

In `src/logger.rs`:
```rust
#[test]
fn test_log_level_filter_resolution() {
    assert_eq!(determine_level_filter(0, false, None), "info");
    assert_eq!(determine_level_filter(1, false, None), "debug");
    assert_eq!(determine_level_filter(2, false, None), "trace");
    assert_eq!(determine_level_filter(0, true, None), "warn");
    assert_eq!(determine_level_filter(0, false, Some("debug")), "debug");
}
```

- [ ] **Step 3: Implement `determine_level_filter` and initialize logging in `main.rs`**

Call `backup::logger::init_logging(...)` at the very start of `main()`.

- [ ] **Step 4: Verify compilation and tests**

Run: `cargo test --lib logger`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/logger.rs src/commands/setup.rs
git commit -m "feat(logging): integrate CLI flags (-v, -q, --log-file), env filter, and TUI log mode"
```

---

### Task 6: Replace Raw `eprintln!` Calls with `tracing` Macros Across Commands & Add Pipeline Audit Spans

**Files:**
- Modify: `src/commands/run.rs`
- Modify: `src/commands/database.rs`
- Modify: `src/commands/copy.rs`
- Modify: `src/commands/doctor.rs`
- Modify: `src/commands/restore.rs`
- Modify: `src/commands/status.rs`
- Modify: `src/commands/update.rs`
- Modify: `src/commands/uninstall.rs`

**Interfaces:**
- Consumes: `tracing::{info, warn, error, debug, info_span}`
- Produces: Clean, structured logging events across all CLI commands and pipeline spans for audit reports.

- [ ] **Step 1: Search for all `eprintln!` instances**

Run: `grep -rn "eprintln!" src/`
Identify all occurrences requiring replacement with appropriate `tracing` level macros.

- [ ] **Step 2: Replace `eprintln!` calls and add pipeline stage spans**

Decorate pipeline stages in `src/commands/run.rs` with `tracing::info_span!`:
- `profile resolution`
- `database`
- `primary backup`
- `secondary sync`
- `retention`

Replace raw `eprintln!` with `tracing::info!`, `tracing::warn!`, `tracing::error!`, or `tracing::debug!`.

- [ ] **Step 3: Run full test suite**

Run: `cargo test`
Expected: PASS for all unit and integration tests.

- [ ] **Step 4: Commit**

```bash
git add src/commands/
git commit -m "refactor(logging): replace raw eprintln calls with structured tracing events and audit spans"
```

---

### Task 7: Final Verification & Code Review

**Files:**
- All touched files

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: PASS

- [ ] **Step 2: Run code coverage check**

Run: `./scripts/test_coverage.sh` or `cargo test`
Expected: All tests pass cleanly without regression.

- [ ] **Step 3: Perform code review**

Use `/code-review` to verify code quality, compliance with `AGENTS.md`, permissions, and secret masking rules.

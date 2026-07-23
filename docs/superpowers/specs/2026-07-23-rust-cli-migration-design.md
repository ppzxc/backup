# Design Specification: Migrating `backup.sh` to Rust CLI (`backup`)

- **Date**: 2026-07-23
- **Status**: Approved
- **Application Name**: `backup`

## 1. Context & Objective

Existing `backup.sh` (shell script) is being migrated to a modern, type-safe, high-performance Rust CLI application named `backup`. The legacy `backup.sh` will be converted into a lightweight downloading/installing/wrapper shell script that fetches the latest pre-compiled `backup` release binary from GitHub Releases and delegates execution to `/usr/local/sbin/backup`.

## 2. Architecture & Design Principles

### Functional Core / Imperative Shell
- **Core Domain Logic**: Pure functions handling YAML parsing, validation, default resolutions, and configuration state changes without side effects.
- **Imperative Shell**: Thin execution boundary handling file IO, process execution (`restic`, `rclone`, `systemctl`), user interaction (`inquire`), and progress feedback (`indicatif`).

### Primary Dependencies & Crates
- `clap`: Declarative CLI parsing with intuitive subcommand structure.
- `inquire`: Interactive CLI wizard dialogs.
- `indicatif`: Dynamic terminal spinners and progress meters.
- `config`: Multi-source configuration resolution (YAML + Environment Variables + CLI overrides).
- `serde` / `serde_yaml`: Config file parsing/serialization.
- `anyhow` / `thiserror`: Robust error handling and reporting.

---

## 3. Configuration & Security Specification

- **Configuration File Path**: `/etc/backup/config.yml` (Single Source of Truth)
- **Permissions**:
  - `/etc/backup/` directory: Mode `700` (`rwx------`)
  - `/etc/backup/config.yml`: Mode `600` (`rw-------`)
- **Precedence Hierarchy**:
  1. CLI Arguments / Flags
  2. Environment Variables (`BACKUP_*`)
  3. Settings in `/etc/backup/config.yml`
  4. Built-in defaults

---

## 4. Simplified Subcommand Structure

The previous scattered commands in `backup.sh` are refactored into a clear, hierarchical subcommand tree:

```
backup
├── run                 # Execute backup operation immediately
├── status              # View current status & connectivity checks
├── setup               # Interactive installation & wizard setup
├── schedule            # Manage backup timers/schedules
│   ├── enable
│   ├── disable
│   └── show
├── config              # Manage backup configuration
│   ├── show
│   ├── edit
│   ├── import
│   └── export
├── doctor              # System diagnostic checks (combines legacy audit, ntp, dependency checks)
├── update              # Update backup Rust binary to the latest version
└── uninstall           # Cleanly uninstall backup CLI and related configurations
```

---

## 5. Downloader Script (`backup.sh`) Specification

`backup.sh` will serve as the bootstrapper/downloader:
1. Detect architecture and OS environment.
2. Query/fetch the target `backup` Rust release binary.
3. Install binary to `/usr/local/sbin/backup` with `755` permissions.
4. Delegate execution arguments transparently to `/usr/local/sbin/backup "$@"`.

---

## 6. Testing Strategy & 1:1 Parity

- Unit & integration tests in Rust (`tests/*.rs`) replacing `bats` tests 1:1.
- Test helpers using `tempfile`, `assert_cmd`, and `predicates`.
- Command stubbing/mocking isolated to system boundary execution traits.

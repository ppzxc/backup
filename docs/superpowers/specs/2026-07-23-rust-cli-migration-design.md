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
- `serde` / `serde_yaml`: Config file parsing/serialization with camelCase convention.
- `secrecy`: Type-safe credential masking in memory and logs.
- `tracing` / `tracing-subscriber`: Structured logging and diagnostic tracing.
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

### Legacy Migration (`backup.env` -> `config.yml`)
- `backup config import --legacy-env` (or automatic migration during `setup` / `config import`) parses `/etc/restic/backup.env` and outputs a valid `/etc/backup/config.yml`.

### Credential Safety & Connectivity Verification
- All credential attributes (`password`, `accessKeyId`, `secretAccessKey`, etc.) MUST be wrapped in `SecretString` and masked (`******`) in any user-facing terminal logs, `status` command outputs, or generated `systemd` service environment blocks.
- SFTP connectivity checks MUST use `rclone_check_connectivity` (via `rclone` binary) to avoid SSH banner noise and credential leakages.

### YAML Configuration Schema (camelCase)

```yaml
version: "1.0"
profile: "host1"

backup:
  targets:
    - "/home/user/data"
  excludes:
    - "/home/user/data/temp"

retention:
  keepDaily: 7
  keepWeekly: 4
  keepMonthly: 12

storage:
  primary:
    backend: "sftp" # "sftp" | "s3"
    repository: "rclone:syno_backup:/backup/host1"
    password: "testpassword"
    sftp:
      host: "192.168.1.100"
      port: 2222
      user: "backupUser"
      keyFile: "/root/.ssh/id_rsa"
    s3:
      endpoint: "https://s3.amazonaws.com"
      accessKeyId: "xxx"
      secretAccessKey: "yyy"
  secondary:
    enabled: false
    backend: "s3"
    repository: "s3:https://s3.us-east-1.amazonaws.com/mybucket"
    password: "secpassword"
```

---

## 4. Simplified Subcommand Structure

The previous scattered commands in `backup.sh` are refactored into a clear, hierarchical subcommand tree:

```
backup
├── run                 # Execute backup operation immediately
├── restore             # Restore files from restic snapshot
├── snapshots           # List available backup snapshots
├── status              # View current status & connectivity checks (with masked credentials)
├── setup               # Interactive installation, init repository & wizard setup
├── schedule            # Manage backup timers/schedules
│   ├── enable
│   ├── disable
│   └── show
├── config              # Manage backup configuration
│   ├── show
│   ├── edit
│   ├── import (--legacy-env supported)
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

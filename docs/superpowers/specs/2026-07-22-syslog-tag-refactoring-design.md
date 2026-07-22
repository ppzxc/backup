# Design Spec: Refactor Syslog Tag to `backup.sh`

## Summary
Remove engine-specific naming (`restic-backup`) from syslog logger output in `/var/log/messages` and replace it with `backup.sh` to ensure abstract decoupling from underlying backup engines (restic, rclone, etc.).

## Target File Modifications

### 1. `backup.sh`
- Replace logger invocation tags in core logging functions:
  - `log_info`: `logger -t backup.sh`
  - `log_error`: `logger -t backup.sh`
  - `log_warn`: `logger -t backup.sh`
- Replace logger tags in `cmd_schedule` (cron block parsing & job creation):
  - `restic-backup-files` -> `backup.sh-files`
  - `restic-backup-db` -> `backup.sh-db`
  - `restic-backup-audit-daily` -> `backup.sh-audit-daily`
  - `restic-backup-audit-drill` -> `backup.sh-audit-drill`
  - `restic-backup-ntp-report` -> `backup.sh-ntp-report`
- Bump `BACKUP_SCRIPT_VERSION` in `backup.sh`.

### 2. `tests/scheduler.bats`
- Update all expected cron job strings and `grep` assertions to use `backup.sh-*` tags instead of `restic-backup-*`.

## Verification Criteria
- `shellcheck backup.sh` passes with 0 warnings.
- `bats tests/` (and `bats tests/scheduler.bats`) passes 100%.

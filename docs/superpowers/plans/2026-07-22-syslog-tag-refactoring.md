# Syslog Tag Refactoring (`backup.sh`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove engine-specific naming (`restic-backup`) from syslog logger output in `/var/log/messages` and replace it with `backup.sh` across core logging functions and cron scheduler configurations.

**Architecture:** Refactor `log_info`, `log_error`, `log_warn`, and cron logger pipe tags in `backup.sh`. Update associated tests in `tests/scheduler.bats` and bump `BACKUP_SCRIPT_VERSION`.

**Tech Stack:** Bash, ShellCheck, Bats

## Global Constraints
- `shellcheck backup.sh` must pass with 0 warnings.
- `BACKUP_SCRIPT_VERSION` must be bumped in `backup.sh`.
- All tests in `bats tests/` must pass.

---

### Task 1: Update Logger Tags in `backup.sh` & Tests

**Files:**
- Modify: `backup.sh`
- Modify: `tests/scheduler.bats`

- [ ] **Step 1: Write/Update the failing tests in `tests/scheduler.bats`**
Update all assertions referencing `restic-backup-` to `backup.sh-`.

- [ ] **Step 2: Run tests to verify failure**
Run: `bats tests/scheduler.bats`
Expected: FAIL due to tag mismatch.

- [ ] **Step 3: Update `backup.sh` logger tags and bump version**
Update `log_info`, `log_error`, `log_warn` logger tags from `restic-backup` to `backup.sh`.
Update `active_jobs` logger tags in `cmd_schedule` to `backup.sh-*`.
Bump `BACKUP_SCRIPT_VERSION` from `0.0.54` to `0.0.55`.

- [ ] **Step 4: Verify ShellCheck and Tests**
Run: `shellcheck backup.sh`
Run: `bats tests/`
Expected: PASS with 0 warnings/failures.

- [ ] **Step 5: Commit**
`git commit -am "refactor: update syslog logger tag to backup.sh"`

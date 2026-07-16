#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "cmd_status fails with guidance when backup.env is missing" {
  run cmd_status
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_status masks secrets and reports snapshot/timer state" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RESTIC_PASSWORD="super-secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_PROFILE_NAME="host1"
ENV
  chmod 600 "$BACKUP_ENV_FILE"
  stub_command "restic" 'case "$1" in snapshots) echo "[]" ;; esac'
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; echo "inactive"'

  run cmd_status
  [ "$status" -eq 0 ]
  [[ "$output" == *"rclone:syno_backup:/backup/host1"* ]]
  [[ "$output" != *"super-secret"* ]]
  [[ "$output" == *"700"* ]]
  [[ "$output" == *"600"* ]]
  [[ "$output" == *"일일 검토 타이머:"* ]]
  [[ "$output" == *"복구 테스트 타이머:"* ]]
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"is-active resticprofile-backup@profile-host1.timer"* ]]
  [[ "$output" == *"is-active backup-audit-daily.timer"* ]]
  [[ "$output" == *"is-active backup-audit-restore-drill.timer"* ]]
}

@test "cmd_status reports 'inactive' correctly when systemctl is-active exits nonzero (realistic stub)" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RESTIC_PASSWORD="super-secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_PROFILE_NAME="host1"
ENV
  chmod 600 "$BACKUP_ENV_FILE"
  stub_command "restic" 'case "$1" in snapshots) echo "[]" ;; esac'
  stub_command "systemctl" '
    if [[ "$1" == "is-active" ]]; then
      echo "inactive"
      exit 3
    fi
  '

  run cmd_status
  [ "$status" -eq 0 ]
  [[ "$output" == *"타이머 상태:"* ]]
  [[ "$output" == *"inactive"* ]]
  [[ "$output" != *$'\nunknown'* ]]
}

@test "cmd_status displays DB timer and isolates DB snapshots" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RESTIC_PASSWORD="super-secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_PROFILE_NAME="host1"
export BACKUP_DB_TYPE="mysql"
ENV
  chmod 600 "$BACKUP_ENV_FILE"

  stub_command "restic" '
    if [[ "$*" == *"snapshots"* ]]; then
      echo "[
        {\"id\":\"1a2b3c4d\",\"time\":\"2026-07-14T02:00:00Z\",\"hostname\":\"host1\",\"paths\":[\"/var/log\"],\"tags\":[]},
        {\"id\":\"5e6f7g8h\",\"time\":\"2026-07-14T03:00:00Z\",\"hostname\":\"host1\",\"paths\":[\"db-dump.sql\"],\"tags\":[\"db\"]}
      ]"
    fi
  '
  stub_command "systemctl" '
    if [[ "$*" == *"is-active"* ]]; then
      echo "active"
      exit 0
    fi
  '

  run cmd_status
  [ "$status" -eq 0 ]

  [[ "$output" == *"DB 백업 타이머:  active"* ]]

  local file_snapshots_part
  file_snapshots_part=$(echo "$output" | sed -n '/\[파일 백업 스냅샷\]/,/\[DB 백업 스냅샷\]/p')
  [[ "$file_snapshots_part" == *"1a2b3c4d"* ]]
  [[ "$file_snapshots_part" == *"/var/log"* ]]
  [[ "$file_snapshots_part" != *"5e6f7g8h"* ]]

  local db_snapshots_part
  db_snapshots_part=$(echo "$output" | sed -n '/\[DB 백업 스냅샷\]/,$p')
  [[ "$db_snapshots_part" == *"5e6f7g8h"* ]]
  [[ "$db_snapshots_part" == *"db-dump.sql"* ]]
  [[ "$db_snapshots_part" != *"1a2b3c4d"* ]]
}


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
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"is-active resticprofile-backup@profile-host1.timer"* ]]
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

#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/web01"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_EXCLUDES="/tmp/*"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="web01"
ENV
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"'
}

@test "cmd_schedule enable renders profiles.yaml and delegates to resticprofile schedule" {
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  [ -f "$RESTICPROFILE_CONFIG_FILE" ]
  perm=$(stat -c '%a' "$RESTICPROFILE_CONFIG_FILE")
  [ "$perm" = "600" ]
  grep -q 'schedule: "\*-\*-\* 02:00:00"' "$RESTICPROFILE_CONFIG_FILE"
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--config ${RESTICPROFILE_CONFIG_FILE} --name web01 schedule"* ]]
}

@test "cmd_schedule enable honors --on-calendar" {
  run cmd_schedule enable --on-calendar "*-*-* 03:15:00"
  [ "$status" -eq 0 ]
  grep -q 'schedule: "\*-\*-\* 03:15:00"' "$RESTICPROFILE_CONFIG_FILE"
}

@test "cmd_schedule disable delegates to resticprofile unschedule" {
  cmd_schedule enable
  run cmd_schedule disable
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--config ${RESTICPROFILE_CONFIG_FILE} --name web01 unschedule"* ]]
}

@test "cmd_schedule rejects unknown action" {
  run cmd_schedule bogus
  [ "$status" -eq 1 ]
}

@test "cmd_schedule fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_schedule enable
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

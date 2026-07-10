#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"'
}

@test "cmd_schedule enable writes unit files with default schedule and enables the timer" {
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  [ -f "$SYSTEMD_SERVICE_FILE" ]
  [ -f "$SYSTEMD_TIMER_FILE" ]
  grep -q 'OnCalendar=\*-\*-\* 02:00:00' "$SYSTEMD_TIMER_FILE"
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"daemon-reload"* ]]
  [[ "$output" == *"enable --now restic-backup.timer"* ]]
}

@test "cmd_schedule enable honors --on-calendar" {
  run cmd_schedule enable --on-calendar "*-*-* 03:15:00"
  [ "$status" -eq 0 ]
  grep -q 'OnCalendar=\*-\*-\* 03:15:00' "$SYSTEMD_TIMER_FILE"
}

@test "cmd_schedule disable disables and removes the timer" {
  cmd_schedule enable
  run cmd_schedule disable
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"disable --now restic-backup.timer"* ]]
}

@test "cmd_schedule rejects unknown action" {
  run cmd_schedule bogus
  [ "$status" -eq 1 ]
}

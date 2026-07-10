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

@test "cmd_schedule disable does not abort even if systemctl disable fails" {
  stub_command "systemctl" '
    if [[ "$1" == "disable" ]]; then
      exit 1
    fi
    echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"
  '
  # Deliberately NOT using `run` here: bats' `run` does `set +eET` before
  # invoking the command, which disables errexit for the duration of the
  # call - so a missing `|| true` guard in systemd_disable_timer would
  # never be observed via `run`'s captured $status (it would always be 0).
  # backup.sh is sourced with `set -euo pipefail` still active in this test
  # process (see the direct `cmd_schedule enable` call a few tests up), so
  # calling cmd_schedule directly is what actually exercises errexit: if
  # `systemctl disable` fails without the `|| true` guard, this call aborts
  # and bats reports the test as failed before reaching the assertion below.
  cmd_schedule disable
  status=$?
  [ "$status" -eq 0 ]
}

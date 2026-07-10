#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"'
  mkdir -p "$RESTIC_ETC_DIR"
  echo "export RESTIC_PASSWORD=secret" > "$BACKUP_ENV_FILE"
  echo "unit" > "$SYSTEMD_SERVICE_FILE"
  echo "unit" > "$SYSTEMD_TIMER_FILE"
}

@test "cmd_uninstall without --purge disables timer but keeps /etc/restic" {
  run cmd_uninstall
  [ "$status" -eq 0 ]
  [ ! -f "$SYSTEMD_SERVICE_FILE" ]
  [ ! -f "$SYSTEMD_TIMER_FILE" ]
  [ -f "$BACKUP_ENV_FILE" ]
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"disable --now restic-backup.timer"* ]]
}

@test "cmd_uninstall --purge also removes the restic config dir" {
  run cmd_uninstall --purge
  [ "$status" -eq 0 ]
  [ ! -d "$RESTIC_ETC_DIR" ]
}

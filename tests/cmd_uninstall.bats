#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"'
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; exit 0'
  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_PASSWORD='secret'
export BACKUP_PROFILE_NAME='web01'
ENV
}

@test "cmd_uninstall without --purge unschedules via resticprofile but keeps /etc/restic" {
  run cmd_uninstall
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--name web01 unschedule"* ]]
}

@test "cmd_uninstall --purge also removes the restic config dir, cache, installed binaries, and script" {
  mkdir -p "${TEST_ROOT}/root/.cache/restic"
  export HOME="${TEST_ROOT}/root"

  # Create mock files at installation paths to verify they get deleted
  mkdir -p "$(dirname "$RESTIC_INSTALL_PATH")" "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"
  touch "$RESTIC_INSTALL_PATH" "$RCLONE_INSTALL_PATH" "$RESTICPROFILE_INSTALL_PATH" "$BACKUP_SCRIPT_INSTALL_PATH"

  run cmd_uninstall --purge --yes
  [ "$status" -eq 0 ]
  [ ! -d "$RESTIC_ETC_DIR" ]
  [ ! -d "${HOME}/.cache/restic" ]
  [ ! -f "$RESTIC_INSTALL_PATH" ]
  [ ! -f "$RCLONE_INSTALL_PATH" ]
  [ ! -f "$RESTICPROFILE_INSTALL_PATH" ]
  [ ! -f "$BACKUP_SCRIPT_INSTALL_PATH" ]
}

@test "cmd_uninstall survives resticprofile unschedule failing (nothing was ever scheduled)" {
  stub_command "resticprofile" 'exit 1'
  run cmd_uninstall
  [ "$status" -eq 0 ]
}

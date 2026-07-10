#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"'
  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_PASSWORD=secret
export BACKUP_PROFILE_NAME=web01
ENV
}

@test "cmd_uninstall without --purge unschedules via resticprofile but keeps /etc/restic" {
  run cmd_uninstall
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--name web01 unschedule"* ]]
}

@test "cmd_uninstall --purge also removes the restic config dir and restic's cache" {
  mkdir -p "${TEST_ROOT}/root/.cache/restic"
  export HOME="${TEST_ROOT}/root"
  run cmd_uninstall --purge
  [ "$status" -eq 0 ]
  [ ! -d "$RESTIC_ETC_DIR" ]
  [ ! -d "${HOME}/.cache/restic" ]
}

@test "cmd_uninstall survives resticprofile unschedule failing (nothing was ever scheduled)" {
  stub_command "resticprofile" 'exit 1'
  run cmd_uninstall
  [ "$status" -eq 0 ]
}

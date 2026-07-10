#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "dnf" 'echo "dnf $*" >> "'"${STUB_BIN}"'/dnf.calls"'
  stub_command "install" 'echo "install $*" >> "'"${STUB_BIN}"'/install.calls"; cp "${@: -2:1}" "${@: -1}"'
}

@test "cmd_install installs packages, self-copies, and creates the restic dir" {
  run cmd_install
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/dnf.calls"
  [[ "$output" == *"install -y epel-release restic rclone"* ]]
  [ -d "$RESTIC_ETC_DIR" ]
  perm=$(stat -c '%a' "$RESTIC_ETC_DIR")
  [ "$perm" = "700" ]
  [ -f "$BACKUP_SCRIPT_INSTALL_PATH" ]
}

@test "cmd_install --dry-run makes no changes" {
  run cmd_install --dry-run
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/dnf.calls" ]
  [ ! -f "${STUB_BIN}/install.calls" ]
  [ ! -d "$RESTIC_ETC_DIR" ]
  [ ! -e "$BACKUP_SCRIPT_INSTALL_PATH" ]
  [[ "$output" == *"dry-run"* ]]
}

@test "cmd_install does not overwrite an existing install without --force" {
  mkdir -p "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"
  echo "old-content" > "$BACKUP_SCRIPT_INSTALL_PATH"
  run cmd_install
  [ "$status" -eq 0 ]
  run cat "$BACKUP_SCRIPT_INSTALL_PATH"
  [ "$output" = "old-content" ]
}

@test "cmd_install --force overwrites an existing install" {
  mkdir -p "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"
  echo "old-content" > "$BACKUP_SCRIPT_INSTALL_PATH"
  run cmd_install --force
  [ "$status" -eq 0 ]
  run cat "$BACKUP_SCRIPT_INSTALL_PATH"
  [ "$output" != "old-content" ]
}

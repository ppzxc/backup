#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
}

@test "cmd_init fails with guidance when backup.env is missing" {
  run cmd_init
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_init runs restic init when repository is not yet initialized" {
  echo 'export RESTIC_REPOSITORY="local:/tmp/fake-repo"' > "$BACKUP_ENV_FILE"
  echo 'export RESTIC_PASSWORD="secret"' >> "$BACKUP_ENV_FILE"
  stub_command "restic" '
    case "$1" in
      snapshots) exit 1 ;;
      init) echo "restic init $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0 ;;
    esac
  '
  run cmd_init
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" == *"init"* ]]
}

@test "cmd_init skips restic init when already initialized" {
  echo 'export RESTIC_REPOSITORY="local:/tmp/fake-repo"' > "$BACKUP_ENV_FILE"
  echo 'export RESTIC_PASSWORD="secret"' >> "$BACKUP_ENV_FILE"
  stub_command "restic" '
    case "$1" in
      snapshots) exit 0 ;;
      init) echo "restic init $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0 ;;
    esac
  '
  run cmd_init
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/restic.calls" ]
  [[ "$output" == *"이미"* ]]
}

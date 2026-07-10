#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="local:/tmp/fake-repo"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_EXCLUDES="/tmp/*,/var/tmp/*"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
ENV
}

@test "cmd_run fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_run
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_run backs up then forgets/prunes on success" {
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$1" in
      unlock) exit 0 ;;
      backup) exit 0 ;;
      forget) exit 0 ;;
    esac
  '
  run cmd_run
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" == *"unlock --stale"* ]]
  [[ "$output" == *"backup /var/log --exclude=/tmp/* --exclude=/var/tmp/*"* ]]
  [[ "$output" == *"forget --keep-daily 7 --keep-weekly 4 --keep-monthly 12 --prune"* ]]
}

@test "cmd_run stops before forget/prune when backup fails" {
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$1" in
      unlock) exit 0 ;;
      backup) exit 1 ;;
      forget) exit 0 ;;
    esac
  '
  run cmd_run
  [ "$status" -eq 1 ]
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" != *"forget"* ]]
}

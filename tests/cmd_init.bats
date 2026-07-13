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

@test "cmd_init dies with a clear guidance message when the SFTP backend is unreachable" {
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="49381"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="secret"
EOF
  stub_command "ssh" 'exit 1'
  stub_command "restic" 'echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0'

  run cmd_init
  [ "$status" -eq 1 ]
  [ ! -f "${STUB_BIN}/restic.calls" ]
  [[ "$output" == *"backup_restic@1.2.3.4:49381"* ]]
  [[ "$output" == *"authorized_keys"* ]]
}

@test "cmd_init proceeds past the SFTP connectivity check when the backend is reachable" {
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="49381"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="secret"
EOF
  stub_command "ssh" 'exit 0'
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

@test "cmd_init does not run the SFTP connectivity check for non-sftp backends" {
  echo 'export RESTIC_REPOSITORY="local:/tmp/fake-repo"' > "$BACKUP_ENV_FILE"
  echo 'export RESTIC_PASSWORD="secret"' >> "$BACKUP_ENV_FILE"
  stub_command "ssh" 'echo "ssh should not be called" >> "'"${STUB_BIN}"'/ssh.calls"; exit 1'
  stub_command "restic" '
    case "$1" in
      snapshots) exit 1 ;;
      init) echo "restic init $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0 ;;
    esac
  '

  run cmd_init
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/ssh.calls" ]
}

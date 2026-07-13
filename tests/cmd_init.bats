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
  stub_command "rclone" 'echo "NewFs: couldn'"'"'t connect SSH: ssh: handshake failed: EOF" >&2; exit 1'
  stub_command "restic" 'echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0'

  run cmd_init
  [ "$status" -eq 1 ]
  [ ! -f "${STUB_BIN}/restic.calls" ]
  [[ "$output" != *"handshake failed"* ]]
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
  stub_command "rclone" 'exit 0'
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
  stub_command "rclone" 'echo "rclone should not be called" >> "'"${STUB_BIN}"'/rclone.calls"; exit 1'
  stub_command "restic" '
    case "$1" in
      snapshots) exit 1 ;;
      init) echo "restic init $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0 ;;
    esac
  '

  run cmd_init
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/rclone.calls" ]
}

@test "cmd_init checks connectivity via 'rclone lsd syno_backup:' (remote root, not the backup subpath)" {
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="49381"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="secret"
EOF
  # 대상 서브경로("/backup/host")는 첫 init 시점엔 아직 없는 게 정상이므로,
  # 점검은 그 경로가 아니라 remote 루트("syno_backup:")를 봐야 한다.
  stub_command "rclone" 'echo "rclone $*" >> "'"${STUB_BIN}"'/rclone.calls"; exit 0'
  stub_command "restic" '
    case "$1" in
      snapshots) exit 1 ;;
      init) exit 0 ;;
    esac
  '

  run cmd_init
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/rclone.calls"
  [[ "$output" == *"lsd syno_backup:"* ]]
  [[ "$output" != *"/backup/host"* ]]
}

@test "cmd_init shows the underlying rclone error when BACKUP_VERBOSE=1" {
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="49381"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="secret"
EOF
  stub_command "rclone" 'echo "NewFs: couldn'"'"'t connect SSH: ssh: handshake failed: EOF" >&2; exit 1'
  stub_command "restic" 'echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0'

  BACKUP_VERBOSE=1 run cmd_init
  [ "$status" -eq 1 ]
  [[ "$output" == *"handshake failed"* ]]
}

@test "cmd_init passes --verbose to restic init when BACKUP_VERBOSE=1" {
  echo 'export RESTIC_REPOSITORY="local:/tmp/fake-repo"' > "$BACKUP_ENV_FILE"
  echo 'export RESTIC_PASSWORD="secret"' >> "$BACKUP_ENV_FILE"
  stub_command "restic" '
    case "$1" in
      snapshots) exit 1 ;;
      init) echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0 ;;
    esac
  '

  BACKUP_VERBOSE=1 run cmd_init
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" == *"init --verbose"* ]]
}

@test "main --verbose init propagates verbose down to the SFTP connectivity check" {
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="49381"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="secret"
EOF
  stub_command "rclone" 'echo "NewFs: couldn'"'"'t connect SSH: ssh: handshake failed: EOF" >&2; exit 1'
  stub_command "restic" 'exit 0'

  run main --verbose init
  [ "$status" -eq 1 ]
  [[ "$output" == *"handshake failed"* ]]
}

@test "main init -v propagates verbose regardless of flag position" {
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="49381"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="secret"
EOF
  stub_command "rclone" 'echo "NewFs: couldn'"'"'t connect SSH: ssh: handshake failed: EOF" >&2; exit 1'
  stub_command "restic" 'exit 0'

  run main init -v
  [ "$status" -eq 1 ]
  [[ "$output" == *"handshake failed"* ]]
}

@test "cmd_init's connectivity check does not depend on a system ssh/sftp client" {
  # NAS 계정이 SFTP 전용(쉘 로그인 권한 없음)으로 제한된 경우, 공개키 인증에는
  # 성공해도 일반 exec/쉘 세션(ssh ... true)은 거부된다. rclone 자체(restic이
  # 실제로 spawn하는 것과 동일한 바이너리)로 점검하면 이 문제가 사라지고,
  # 별도 openssh-clients 의존성도 없어진다.
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="49381"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="secret"
EOF
  stub_command "ssh" 'echo "This service allows sftp connections only." >&2; exit 1'
  stub_command "sftp" 'exit 1'
  stub_command "rclone" 'exit 0'
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

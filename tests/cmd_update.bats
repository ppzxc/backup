#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"

  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/web01"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_PROFILE_NAME="web01"
ENV

  # Stub installer downloads
  stub_command "curl" "exit 0"
  stub_command "sha256sum" "echo 'ok'"
  stub_command "bunzip2" "exit 0"
  stub_command "unzip" "exit 0"
  stub_command "chmod" "exit 0"
  stub_command "install" "exit 0"
  stub_command "systemctl" "exit 0"
  stub_command "resticprofile" 'echo "resticprofile $*"'
}

@test "cmd_update calls installer and scheduler enable" {
  # Mock require_root
  require_root() { true; }

  run main update
  [ "$status" -eq 0 ]
  [[ "$output" == *"최신 스크립트 버전"* ]]
  [[ "$output" == *"설치본을 갱신합니다"* ]]
  [[ "$output" == *"스케줄러 및 설정 프로필 구성을 새 버전 규격으로 업데이트합니다"* ]]
}

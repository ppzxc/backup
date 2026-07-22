#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
  mkdir -p "$BACKUP_ETC_DIR"

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

@test "cmd_update automatically migrates legacy key BACKUP_CHRONY_REPORT to BACKUP_NTP_REPORT in backup.env" {
  require_root() { true; }

  cat > "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_CHRONY_REPORT="1"
export RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-bucket/host"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export AWS_ACCESS_KEY_ID="somekey"
export AWS_SECRET_ACCESS_KEY="somesecret"
export BACKUP_PROFILE_NAME="testprofile"
ENV

  run main update
  [ "$status" -eq 0 ]

  # Check that backup.env was rewritten with BACKUP_NTP_REPORT='1'
  run config_get "BACKUP_NTP_REPORT" "$BACKUP_ENV_FILE"
  [ "$output" = "1" ]

  # The old BACKUP_CHRONY_REPORT should not exist in the rewritten file
  run grep "BACKUP_CHRONY_REPORT" "$BACKUP_ENV_FILE" || true
  [[ "$output" != *"BACKUP_CHRONY_REPORT"* ]]
}

@test "cmd_update cleans up legacy backup-chrony-report timer and service if present" {
  require_root() { true; }

  # Create mock legacy systemd timer/service files
  touch "$SYSTEMD_UNIT_DIR/backup-chrony-report.timer"
  touch "$SYSTEMD_UNIT_DIR/backup-chrony-report.service"

  # Stub systemctl to track calls
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; exit 0'

  run main update
  [ "$status" -eq 0 ]

  # Check systemctl disable was called on backup-chrony-report.timer
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"disable --now backup-chrony-report.timer"* ]]

  # Check the files were deleted
  [ ! -f "$SYSTEMD_UNIT_DIR/backup-chrony-report.timer" ]
  [ ! -f "$SYSTEMD_UNIT_DIR/backup-chrony-report.service" ]
}

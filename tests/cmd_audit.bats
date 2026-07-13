#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="web01"
ENV
  chmod 600 "$BACKUP_ENV_FILE"
  stub_command "systemctl" '
    case "$1" in
      is-enabled) echo "enabled"; exit 0 ;;
      is-active) echo "active"; exit 0 ;;
      list-timers) echo "Thu 2026-07-16 02:00:00 KST 2days left n/a n/a resticprofile-backup@profile-web01.timer" ;;
      cat) echo "OnCalendar=*-*-* 02:00:00" ;;
    esac
  '
  stub_command "restic" '
    case "$1" in
      snapshots) echo "ID   Time  Host  Tags  Paths"; echo "abc123 2026-07-15 host  -    /var/log"; exit 0 ;;
    esac
  '
}

@test "cmd_audit fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_audit
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_audit reports the sftp backend and repository location" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"백엔드: sftp"* ]]
  [[ "$output" == *"rclone:syno_backup:/backup/host"* ]]
}

@test "cmd_audit reports the s3 backend when no sftp config is present" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="s3:https://example.com/bucket"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
ENV
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"백엔드: s3"* ]]
}

@test "cmd_audit shows the retention policy numbers" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"일간 보관: 7개"* ]]
  [[ "$output" == *"주간 보관: 4개"* ]]
  [[ "$output" == *"월간 보관: 12개"* ]]
}

@test "cmd_audit warns when the repository password is not set" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export BACKUP_TARGETS="/var/log"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
ENV
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"경고: 비밀번호 미설정"* ]]
}

@test "cmd_audit does not warn when the repository password is set" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" != *"경고: 비밀번호 미설정"* ]]
}

@test "cmd_audit reports the timer's schedule and enabled/active state" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"반복 주기(OnCalendar): *-*-* 02:00:00"* ]]
  [[ "$output" == *"타이머 등록 상태: enabled"* ]]
  [[ "$output" == *"타이머 실행 상태: active"* ]]
  [[ "$output" == *"2026-07-16"* ]]
}

@test "cmd_audit includes the restic snapshots history section" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"백업 이력(restic snapshots)"* ]]
  [[ "$output" == *"abc123"* ]]
}

@test "cmd_audit reports file permissions for /etc/restic and backup.env" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"${RESTIC_ETC_DIR} 권한: 700"* ]]
  [[ "$output" == *"${BACKUP_ENV_FILE} 권한: 600"* ]]
}

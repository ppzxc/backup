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
export BACKUP_PROFILE_NAME="web01"
ENV
}

@test "cmd_run fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_run
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_run renders profiles.yaml fresh and delegates the backup to resticprofile" {
  stub_command "resticprofile" '
    echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"
    exit 0
  '
  run cmd_run
  [ "$status" -eq 0 ]
  [ -f "$RESTICPROFILE_CONFIG_FILE" ]
  perm=$(stat -c '%a' "$RESTICPROFILE_CONFIG_FILE")
  [ "$perm" = "600" ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--config ${RESTICPROFILE_CONFIG_FILE} --name web01 backup"* ]]
  [[ "$output" != *"restic unlock"* ]]
}

@test "cmd_run re-renders profiles.yaml every run so a stale copy never gets reused" {
  stub_command "resticprofile" 'exit 0'
  echo "stale placeholder, must be overwritten" > "$RESTICPROFILE_CONFIG_FILE"
  chmod 600 "$RESTICPROFILE_CONFIG_FILE"
  run cmd_run
  [ "$status" -eq 0 ]
  grep -q 'repository: "local:/tmp/fake-repo"' "$RESTICPROFILE_CONFIG_FILE"
}

@test "cmd_run dies when resticprofile fails" {
  stub_command "resticprofile" 'exit 1'
  run cmd_run
  [ "$status" -eq 1 ]
}

@test "cmd_run passes -v to resticprofile when BACKUP_VERBOSE=1" {
  stub_command "resticprofile" '
    echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"
    exit 0
  '
  BACKUP_VERBOSE=1 run cmd_run
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--config ${RESTICPROFILE_CONFIG_FILE} --name web01 backup -v"* ]]
}

@test "cmd_run triggers secondary copy command using resticprofile" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export SECONDARY_BACKEND="s3"
export SECONDARY_RESTIC_REPOSITORY="s3:https://sec-s3.com/sec-bucket/host"
export SECONDARY_RESTIC_PASSWORD="sec-secret"
export SECONDARY_AWS_ACCESS_KEY_ID="SEC_AK"
export SECONDARY_AWS_SECRET_ACCESS_KEY="SEC_SK"
ENV

  stub_command "resticprofile" '
    echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"
    exit 0
  '
  
  run cmd_run
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/resticprofile.calls"
  # 1차 백업 호출 검증
  [[ "$output" == *"--name web01 backup"* ]]
  # 2차 copy 호출 검증
  [[ "$output" == *"--name web01-secondary copy"* ]]
}

@test "cmd_run triggers notification on success even without secondary backup" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NOTIFICATION_URL="https://hooks.slack.com/services/test"
export BACKUP_NOTIFICATION_TYPE="slack"
export BACKUP_NOTIFICATION_ON="both"
ENV

  stub_command "resticprofile" 'exit 0'
  stub_command "curl" 'echo "curl $*" >> "'"${STUB_BIN}"'/curl.calls"; exit 0'

  run cmd_run
  [ "$status" -eq 0 ]
  
  [ -f "${STUB_BIN}/curl.calls" ]
  run cat "${STUB_BIN}/curl.calls"
  [[ "$output" == *"hooks.slack.com"* ]]
}

@test "cmd_run triggers notification on failure even without secondary backup" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NOTIFICATION_URL="https://hooks.slack.com/services/test"
export BACKUP_NOTIFICATION_TYPE="slack"
export BACKUP_NOTIFICATION_ON="both"
ENV

  stub_command "resticprofile" 'exit 1'
  stub_command "curl" 'echo "curl $*" >> "'"${STUB_BIN}"'/curl.calls"; exit 0'

  run cmd_run
  [ "$status" -eq 1 ]

  [ -f "${STUB_BIN}/curl.calls" ]
  run cat "${STUB_BIN}/curl.calls"
  [[ "$output" == *"hooks.slack.com"* ]]
}

@test "cmd_run outputs initialization recommendation warning when resticprofile exits with status 10" {
  stub_command "resticprofile" 'exit 10'
  
  run cmd_run
  [ "$status" -eq 1 ]
  [[ "$output" == *"WARNING:"* ]]
  [[ "$output" == *"초기화(init)되지 않았을 수 있습니다"* ]]
}



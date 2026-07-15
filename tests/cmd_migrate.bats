#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  hostname() {
    echo "host1"
  }
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="old-nas"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="22"
export RCLONE_CONFIG_SYNO_BACKUP_USER="old-user"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="oldsecret"
export BACKUP_TARGETS="/var/log"
export BACKUP_EXCLUDES="/tmp/*"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="host1"
ENV
  chmod 600 "$BACKUP_ENV_FILE"

  # Stub default utilities
  stub_command "systemctl" '
    echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"
    case "$1" in
      is-enabled) echo "enabled"; exit 0 ;;
      is-active) echo "active"; exit 0 ;;
      list-timers) echo "Thu 2026-07-16 02:00:00 KST 2days left n/a n/a resticprofile-backup@profile-host1.timer" ;;
      cat) echo "OnCalendar=*-*-* 02:00:00" ;;
    esac
  '

  # We stub resticprofile only to track calls when schedule is re-enabled
  stub_command "resticprofile" '
    echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"
  '
}

@test "cmd_migrate fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_migrate
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_migrate fails when source repository check fails" {
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    exit 1
  '
  run cmd_migrate --backend s3 --endpoint http://new-s3 --bucket new-bucket --access-key key --secret-key sec
  [ "$status" -eq 1 ]
  [[ "$output" == *"기존 저장소"* ]]
}

@test "cmd_migrate S3 to S3 migration (success case) matches parameters, updates config, and runs check/schedule" {
  # Stub restic to simulate source snapshots success, dest snapshots not-initialized, dest init, copy, and check
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$*" in
      *"snapshots"*)
        if [[ "$*" == *"new-bucket"* ]]; then
          # Destination repo not initialized yet
          exit 1
        fi
        exit 0
        ;;
      *"init"*)
        exit 0
        ;;
      *"copy"*)
        exit 0
        ;;
      *"check"*)
        exit 0
        ;;
    esac
  '

  run cmd_migrate --backend s3 --endpoint http://new-s3 --bucket new-bucket --access-key key --secret-key sec --new-password newsecret
  echo "--- cmd_migrate OUTPUT: ---" >&2
  echo "$output" >&2
  echo "---------------------------" >&2
  [ "$status" -eq 0 ]

  # Verify config was updated
  [ -f "$BACKUP_ENV_FILE" ]
  run cat "$BACKUP_ENV_FILE"
  echo "--- ACTUAL backup.env CONTENT: ---" >&2
  echo "$output" >&2
  echo "----------------------------------" >&2
  [[ "$output" == *"RESTIC_REPOSITORY='s3:http://new-s3/new-bucket/host1'"* ]]
  [[ "$output" == *"RESTIC_PASSWORD='newsecret'"* ]]
  [[ "$output" == *"AWS_ACCESS_KEY_ID='key'"* ]]
  [[ "$output" == *"AWS_SECRET_ACCESS_KEY='sec'"* ]]
  [[ "$output" != *"RCLONE_CONFIG_SYNO_BACKUP_HOST"* ]] # Old SFTP config should be removed

  # Verify restic calls
  run cat "${STUB_BIN}/restic.calls"
  echo "--- ACTUAL restic.calls: ---" >&2
  echo "$output" >&2
  echo "----------------------------" >&2
  
  # 1. Source unlock
  [[ "${lines[0]}" == *"unlock"* ]]
  # 2. Source snapshots (pre-flight check)
  [[ "${lines[1]}" == *"snapshots"* ]]
  # 3. Destination snapshots (check if dest exists)
  [[ "${lines[2]}" == *"s3:http://new-s3/new-bucket/host1"* ]]
  [[ "${lines[2]}" == *"snapshots"* ]]
  # 4. Destination init from source
  [[ "${lines[3]}" == *"s3:http://new-s3/new-bucket/host1"* ]]
  [[ "${lines[3]}" == *"init"* ]]
  [[ "${lines[3]}" == *"--from-repo rclone:syno_backup:/backup/host1"* ]]
  [[ "${lines[3]}" == *"--copy-chunker-params"* ]]
  # 5. Copy snapshots
  [[ "${lines[4]}" == *"rclone:syno_backup:/backup/host1"* ]]
  [[ "${lines[4]}" == *"copy"* ]]
  [[ "${lines[4]}" == *"--repo2 s3:http://new-s3/new-bucket/host1"* ]]
  # 6. Check consistency
  [[ "${lines[5]}" == *"s3:http://new-s3/new-bucket/host1"* ]]
  [[ "${lines[5]}" == *"check"* ]]

  # Verify systemd was updated because schedule was active
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--name host1 schedule"* ]]
}

@test "cmd_migrate reuse source password when --new-password is not provided" {
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$*" in
      *"snapshots"*)
        if [[ "$*" == *"new-bucket"* ]]; then
          exit 1
        fi
        exit 0
        ;;
      *)
        exit 0
        ;;
    esac
  '

  run cmd_migrate --backend s3 --endpoint http://new-s3 --bucket new-bucket --access-key key --secret-key sec --skip-check
  [ "$status" -eq 0 ]

  run cat "$BACKUP_ENV_FILE"
  # Should retain old password "oldsecret"
  [[ "$output" == *"RESTIC_PASSWORD='oldsecret'"* ]]

  run cat "${STUB_BIN}/restic.calls"
  # Should not have check command because --skip-check was specified
  [[ "$output" != *"check"* ]]
}

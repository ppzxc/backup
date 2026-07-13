#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "ssh-keygen" '
    keyfile=""
    while [[ $# -gt 0 ]]; do
      if [[ "$1" == "-f" ]]; then keyfile="$2"; fi
      shift
    done
    echo "fake-private-key" > "$keyfile"
    echo "ssh-ed25519 AAAAFAKEKEY test@stub" > "${keyfile}.pub"
  '
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"'
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"'
}

@test "cmd_config fails when backup.env does not exist" {
  run cmd_config --targets "/var/log"
  [ "$status" -eq 1 ]
  [[ "$output" == *"설정 파일이 존재하지 않습니다"* ]]
}

@test "cmd_config updates targets and excludes for SFTP, keeping other settings" {
  # 1. Setup initial setting
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_user --password secret_pass --targets "/etc" --exclude "/tmp/*"
  [ "$status" -eq 0 ]

  # 2. Run config command to update targets and excludes
  run cmd_config --targets "/var/log,/home" --exclude "/var/tmp/*"
  [ "$status" -eq 0 ]

  # 3. Verify backup.env content and permissions
  [ -f "$BACKUP_ENV_FILE" ]
  perm=$(stat -c '%a' "$BACKUP_ENV_FILE")
  [ "$perm" = "600" ]

  # Verify updated values
  grep -q 'export BACKUP_TARGETS="/var/log,/home"' "$BACKUP_ENV_FILE"
  grep -q 'export BACKUP_EXCLUDES="/var/tmp/\*"' "$BACKUP_ENV_FILE"

  # Verify preserved values
  grep -q 'export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"' "$BACKUP_ENV_FILE"
  grep -q 'export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_user"' "$BACKUP_ENV_FILE"
  grep -q 'export RESTIC_PASSWORD="secret_pass"' "$BACKUP_ENV_FILE"
}

@test "cmd_config updates S3 credentials and endpoint preserving bucket" {
  # 1. Setup initial setting
  run cmd_setting --backend s3 --endpoint http://127.0.0.1:9000 --bucket restic-bucket --access-key old-key --secret-key old-secret --password secret --targets "/etc"
  [ "$status" -eq 0 ]

  # 2. Run config to update access-key and secret-key
  run cmd_config --access-key new-key --secret-key new-secret
  [ "$status" -eq 0 ]

  # 3. Verify updated keys
  grep -q 'export AWS_ACCESS_KEY_ID="new-key"' "$BACKUP_ENV_FILE"
  grep -q 'export AWS_SECRET_ACCESS_KEY="new-secret"' "$BACKUP_ENV_FILE"

  # Verify preserved bucket
  grep -q 'export RESTIC_REPOSITORY="s3:http://127.0.0.1:9000/restic-bucket/' "$BACKUP_ENV_FILE"
}

@test "cmd_config preserves existing schedule from profiles.yaml unless overridden" {
  # 1. Setup initial sftp setting and schedule it
  cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_user --password secret_pass --targets "/etc"
  cmd_schedule enable --on-calendar "*-*-* 04:00:00"

  # 2. Modify targets with config command
  run cmd_config --targets "/var/log"
  [ "$status" -eq 0 ]

  # 3. Verify target is updated but schedule is preserved in profiles.yaml
  grep -q 'schedule: "\*-\*-\* 04:00:00"' "$RESTICPROFILE_CONFIG_FILE"
  grep -q -- '- "/var/log"' "$RESTICPROFILE_CONFIG_FILE"

  # 4. Verify group-by: host is added to retention section
  grep -q 'group-by: host' "$RESTICPROFILE_CONFIG_FILE"
}

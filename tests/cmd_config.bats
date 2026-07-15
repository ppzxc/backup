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
  run config_get "targets" "$BACKUP_ENV_FILE"
  echo "DEBUG_TARGETS: status=$status output=$output" >&2
  echo "DEBUG_ENV_FILE: $(cat "$BACKUP_ENV_FILE")" >&2
  [[ "$output" == "/var/log,/home" ]]
  run config_get "excludes_csv" "$BACKUP_ENV_FILE"
  [[ "$output" == "/var/tmp/*" ]]

  # Verify preserved values
  run config_get "host" "$BACKUP_ENV_FILE"
  [[ "$output" == "1.2.3.4" ]]
  run config_get "user" "$BACKUP_ENV_FILE"
  [[ "$output" == "backup_user" ]]
  run config_get "password" "$BACKUP_ENV_FILE"
  [[ "$output" == "secret_pass" ]]
}

@test "cmd_config updates S3 credentials and endpoint preserving bucket" {
  # 1. Setup initial setting
  run cmd_setting --backend s3 --endpoint http://127.0.0.1:9000 --bucket restic-bucket --access-key old-key --secret-key old-secret --password secret --targets "/etc"
  [ "$status" -eq 0 ]

  # 2. Run config to update access-key and secret-key
  run cmd_config --access-key new-key --secret-key new-secret
  [ "$status" -eq 0 ]

  # Verify updated keys
  run config_get "access_key" "$BACKUP_ENV_FILE"
  [[ "$output" == "new-key" ]]
  run config_get "secret_key" "$BACKUP_ENV_FILE"
  [[ "$output" == "new-secret" ]]

  # Verify preserved bucket
  run config_get "RESTIC_REPOSITORY" "$BACKUP_ENV_FILE"
  [[ "$output" == *"s3:http://127.0.0.1:9000/restic-bucket/"* ]]
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

@test "cmd_config updates DB settings while preserving other credentials" {
  run cmd_setting --backend s3 --endpoint http://127.0.0.1:9000 --bucket restic-bucket --access-key my-key --secret-key my-secret --password secret --targets "/etc"
  [ "$status" -eq 0 ]

  run cmd_config --db-type mysql --db-schedule "*-*-* 05:00:00"
  [ "$status" -eq 0 ]

  run config_get "db_type" "$BACKUP_ENV_FILE"
  [[ "$output" == "mysql" ]]
  run config_get "db_schedule" "$BACKUP_ENV_FILE"
  [[ "$output" == *"05:00:00"* ]]
  run config_get "db_command" "$BACKUP_ENV_FILE"
  [[ "$output" == "mysqldump --all-databases --single-transaction --quick --order-by-primary" ]]

  run config_get "access_key" "$BACKUP_ENV_FILE"
  [[ "$output" == "my-key" ]]
  run config_get "secret_key" "$BACKUP_ENV_FILE"
  [[ "$output" == "my-secret" ]]
  run config_get "password" "$BACKUP_ENV_FILE"
  [[ "$output" == "secret" ]]
}


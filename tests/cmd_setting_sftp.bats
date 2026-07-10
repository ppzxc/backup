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
}

@test "render_backup_env_sftp produces expected export lines" {
  run render_backup_env_sftp "host1" "1.2.3.4" "22" "backup_restic" "/etc/restic/backup_key" "secret" "/var/log" "/tmp/*,/var/tmp/*" "7" "4" "12"
  [[ "$output" == *'export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_PORT="22"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"'* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="secret"'* ]]
  [[ "$output" == *'export BACKUP_TARGETS="/var/log"'* ]]
  [[ "$output" == *'export KEEP_DAILY="7"'* ]]
}

@test "render_sftp_registration_notice includes the pubkey and next command" {
  run render_sftp_registration_notice "ssh-ed25519 AAAAFAKEKEY test@stub"
  [[ "$output" == *"ssh-ed25519 AAAAFAKEKEY test@stub"* ]]
  [[ "$output" == *"backup.sh init"* ]]
}

@test "cmd_setting sftp writes backup.env with 600 perms and generates ssh key" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  perm=$(stat -c '%a' "$BACKUP_ENV_FILE")
  [ "$perm" = "600" ]
  [ -f "$BACKUP_SSH_KEY" ]
  [ -f "${BACKUP_SSH_KEY}.pub" ]
  key_perm=$(stat -c '%a' "$BACKUP_SSH_KEY")
  [ "$key_perm" = "600" ]
  [[ "$output" == *"ssh-ed25519"* ]]
}

@test "cmd_setting sftp fails with actionable hint when --host is missing" {
  run cmd_setting --backend sftp --port 22 --user backup_restic --password secret
  [ "$status" -eq 1 ]
  [[ "$output" == *"--host <NAS_IP>"* ]]
  [[ "$output" == *"--user backup_restic"* ]]
}

@test "cmd_setting refuses to overwrite existing backup.env without --force" {
  cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret
  run cmd_setting --backend sftp --host 9.9.9.9 --port 22 --user someone --password other
  [ "$status" -eq 1 ]
  [[ "$output" == *"--force"* ]]
}

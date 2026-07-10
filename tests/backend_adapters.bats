#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "ssh-keygen" '
    keyfile=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-f" ]]; then keyfile="$a"; fi
      prev="$a"
    done
    echo "fake-private-key" > "$keyfile"
    echo "ssh-ed25519 AAAAFAKEKEY test@stub" > "${keyfile}.pub"
  '
}

@test "backend_sftp_env_vars lists host/port/user env-shadow pairs" {
  run backend_sftp_env_vars
  [[ "$output" == *$'host\tBACKUP_HOST'* ]]
  [[ "$output" == *$'port\tBACKUP_PORT'* ]]
  [[ "$output" == *$'user\tBACKUP_USER'* ]]
}

@test "backend_s3_env_vars lists endpoint/bucket/access_key/secret_key env-shadow pairs" {
  run backend_s3_env_vars
  [[ "$output" == *$'endpoint\tBACKUP_ENDPOINT'* ]]
  [[ "$output" == *$'bucket\tBACKUP_BUCKET'* ]]
  [[ "$output" == *$'access_key\tBACKUP_ACCESS_KEY'* ]]
  [[ "$output" == *$'secret_key\tBACKUP_SECRET_KEY'* ]]
}

@test "backend_sftp_resolve prefers cli over env, falls back to default port" {
  local -A cli=([host]="1.2.3.4" [user]="alice") env=([host]="9.9.9.9" [port]="2222") file=() fields=()
  backend_sftp_resolve cli env file fields
  [ "${fields[host]}" = "1.2.3.4" ]
  [ "${fields[port]}" = "2222" ]
  [ "${fields[user]}" = "alice" ]
}

@test "backend_sftp_resolve defaults port when nothing given" {
  local -A cli=() env=() file=() fields=()
  backend_sftp_resolve cli env file fields
  [ "${fields[port]}" = "$DEFAULT_SFTP_PORT" ]
  [ "${fields[host]}" = "" ]
}

@test "backend_s3_resolve prefers cli over env for every field" {
  local -A cli=([endpoint]="https://cli.example.com" [bucket]="cli-bucket" [access_key]="AK1" [secret_key]="SK1") env=([endpoint]="https://env.example.com" [bucket]="env-bucket") file=() fields=()
  backend_s3_resolve cli env file fields
  [ "${fields[endpoint]}" = "https://cli.example.com" ]
  [ "${fields[bucket]}" = "cli-bucket" ]
  [ "${fields[access_key]}" = "AK1" ]
  [ "${fields[secret_key]}" = "SK1" ]
}

@test "backend_s3_resolve falls back to env when cli is empty" {
  local -A cli=() env=([endpoint]="https://env.example.com" [bucket]="env-bucket") file=() fields=()
  backend_s3_resolve cli env file fields
  [ "${fields[endpoint]}" = "https://env.example.com" ]
  [ "${fields[bucket]}" = "env-bucket" ]
}

@test "backend_sftp_validate fails with hint when host missing" {
  local -A fields=([host]="" [port]="22" [user]="alice")
  run backend_sftp_validate fields
  [ "$status" -eq 1 ]
  [[ "$output" == *"--user alice"* ]]
}

@test "backend_sftp_validate fails when port is invalid" {
  local -A fields=([host]="1.2.3.4" [port]="notanumber" [user]="alice")
  run backend_sftp_validate fields
  [ "$status" -eq 1 ]
  [[ "$output" == *"port"* ]]
}

@test "backend_sftp_validate succeeds with complete fields" {
  local -A fields=([host]="1.2.3.4" [port]="22" [user]="alice")
  run backend_sftp_validate fields
  [ "$status" -eq 0 ]
}

@test "backend_s3_validate fails with hint when bucket missing" {
  local -A fields=([endpoint]="https://s3.example.com" [bucket]="" [access_key]="AK" [secret_key]="SK")
  run backend_s3_validate fields
  [ "$status" -eq 1 ]
  [[ "$output" == *"--endpoint https://s3.example.com"* ]]
}

@test "backend_s3_validate fails when access key missing" {
  local -A fields=([endpoint]="https://s3.example.com" [bucket]="my-bucket" [access_key]="" [secret_key]="SK")
  run backend_s3_validate fields
  [ "$status" -eq 1 ]
}

@test "backend_s3_validate succeeds with complete fields" {
  local -A fields=([endpoint]="https://s3.example.com" [bucket]="my-bucket" [access_key]="AK" [secret_key]="SK")
  run backend_s3_validate fields
  [ "$status" -eq 0 ]
}

@test "backend_sftp_prepare generates an ssh key and records its pubkey in fields" {
  mkdir -p "$RESTIC_ETC_DIR"
  local -A fields=()
  backend_sftp_prepare fields
  [ -f "$BACKUP_SSH_KEY" ]
  [ -f "${BACKUP_SSH_KEY}.pub" ]
  [[ "${fields[pubkey]}" == *"ssh-ed25519 AAAAFAKEKEY test@stub"* ]]
}

@test "backend_s3_prepare is a no-op" {
  local -A fields=()
  run backend_s3_prepare fields
  [ "$status" -eq 0 ]
}

@test "backend_sftp_render_env produces expected export lines" {
  local -A fields=([host]="1.2.3.4" [port]="22" [user]="backup_restic")
  local -A policy=([password]="secret" [targets]="/var/log" [excludes_csv]="/tmp/*,/var/tmp/*" [keep_daily]="7" [keep_weekly]="4" [keep_monthly]="12" [profile_name]="web01")
  run backend_sftp_render_env "host1" fields policy
  [[ "$output" == *'export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_PORT="22"'* ]]
  [[ "$output" == *"export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE=\"${BACKUP_SSH_KEY}\""* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="secret"'* ]]
  [[ "$output" == *'export BACKUP_PROFILE_NAME="web01"'* ]]
}

@test "backend_s3_render_env produces expected export lines" {
  local -A fields=([endpoint]="https://s3.example.com" [bucket]="my-bucket" [access_key]="AKIA123" [secret_key]="secretkey")
  local -A policy=([password]="repopass" [targets]="/var/log" [excludes_csv]="/tmp/*,/var/tmp/*" [keep_daily]="7" [keep_weekly]="4" [keep_monthly]="12" [profile_name]="web01")
  run backend_s3_render_env "host1" fields policy
  [[ "$output" == *'export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"'* ]]
  [[ "$output" == *'export AWS_ACCESS_KEY_ID="AKIA123"'* ]]
  [[ "$output" == *'export AWS_SECRET_ACCESS_KEY="secretkey"'* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="repopass"'* ]]
  [[ "$output" == *'export BACKUP_PROFILE_NAME="web01"'* ]]
}

@test "backend_sftp_render_notice prints the generated pubkey for NAS registration" {
  mkdir -p "$RESTIC_ETC_DIR"
  local -A fields=()
  backend_sftp_prepare fields
  run backend_sftp_render_notice fields
  [[ "$output" == *"ssh-ed25519 AAAAFAKEKEY test@stub"* ]]
  [[ "$output" == *"backup.sh init"* ]]
}

@test "backend_s3_render_notice prints the least-privilege bucket policy" {
  local -A fields=([bucket]="my-bucket")
  run backend_s3_render_notice fields
  [[ "$output" == *'"arn:aws:s3:::my-bucket"'* ]]
  [[ "$output" == *"ListBucket"* ]]
}

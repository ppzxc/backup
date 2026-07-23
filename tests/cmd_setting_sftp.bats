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

@test "cmd_setting sftp missing --host prints the hint via real invocation (not masked by bats run)" {
  # bats test bodies themselves run under set -e, so a naive
  # `output=$(...)` here would abort this test the same way the original
  # bug aborted backup.sh -- capture the real exit code with `|| exit_code=$?`
  # instead of letting a nonzero status propagate.
  local exit_code=0
  output=$(REQUIRE_ROOT_CHECK=0 \
    RESTIC_ETC_DIR="$RESTIC_ETC_DIR" \
    BACKUP_ENV_FILE="$BACKUP_ENV_FILE" \
    BACKUP_SSH_KEY="$BACKUP_SSH_KEY" \
    BACKUP_SCRIPT_INSTALL_PATH="$BACKUP_SCRIPT_INSTALL_PATH" \
    SYSTEMD_UNIT_DIR="$SYSTEMD_UNIT_DIR" \
    PATH="${STUB_BIN}:${PATH}" \
    bash "${BATS_TEST_DIRNAME}/../backup.sh" setting --backend sftp --port 22 --user backup_restic --password secret 2>&1) || exit_code=$?
  [ "$exit_code" -eq 1 ]
  [[ "$output" == *"--host <NAS_IP>"* ]]
}

@test "cmd_setting sftp reuses existing --exclude from backup.env on --force re-run" {
  cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --exclude '/custom/exclude/*'
  run config_get "excludes_csv" "$BACKUP_ENV_FILE"
  [ "$output" = "/custom/exclude/*" ]

  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --force
  [ "$status" -eq 0 ]
  run config_get "excludes_csv" "$BACKUP_ENV_FILE"
  [ "$output" = "/custom/exclude/*" ]
}

@test "cmd_setting sftp defaults profile-name to backup and honors --profile-name override" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret
  [ "$status" -eq 0 ]
  run config_get "profile_name" "$BACKUP_ENV_FILE"
  [ "$output" = "backup" ]

  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --profile-name web01 --force
  [ "$status" -eq 0 ]
  run config_get "profile_name" "$BACKUP_ENV_FILE"
  [ "$output" = "web01" ]
}

@test "cmd_setting rejects an invalid --profile-name" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --profile-name "bad name"
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}

@test "cmd_setting defaults profile-name to backup when unset" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret
  [ "$status" -eq 0 ]
  run config_get "profile_name" "$BACKUP_ENV_FILE"
  [ "$output" = "backup" ]
}

@test "cmd_setting sftp accepts --db-type and writes BACKUP_DB_TYPE to backup.env" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --db-type mysql
  [ "$status" -eq 0 ]
  run config_get "db_type" "$BACKUP_ENV_FILE"
  [ "$output" = "mysql" ]
}

@test "cmd_setting sftp auto-fills default dump commands based on --db-type" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --db-type mysql --force
  [ "$status" -eq 0 ]
  run config_get "db_command" "$BACKUP_ENV_FILE"
  [ "$output" = "mysqldump --all-databases --single-transaction --quick --order-by-primary" ]

  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --db-type mariadb --force
  [ "$status" -eq 0 ]
  run config_get "db_command" "$BACKUP_ENV_FILE"
  [ "$output" = "mariadb-dump --all-databases --single-transaction --quick --order-by-primary" ]

  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --db-type postgres --force
  [ "$status" -eq 0 ]
  run config_get "db_command" "$BACKUP_ENV_FILE"
  [ "$output" = "pg_dumpall -U postgres" ]
}

@test "cmd_setting sftp configuration with double quotes in password and user" {
  run cmd_setting --backend sftp --host "1.2.3.4" --port 22 --user 'user"with"quotes' --password 'pass"with"quotes'
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]

  run bash -c "source $BACKUP_ENV_FILE && echo \"user=\$RCLONE_CONFIG_SYNO_BACKUP_USER\" && echo \"pass=\$RESTIC_PASSWORD\""
  [ "$status" -eq 0 ]
  [[ "$output" == *"user=user\"with\"quotes"* ]]
  [[ "$output" == *"pass=pass\"with\"quotes"* ]]
}



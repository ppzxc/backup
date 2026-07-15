#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  unset BACKUP_TARGETS
  unset BACKUP_PASSWORD
  unset KEEP_DAILY
  unset KEEP_WEEKLY
  unset KEEP_MONTHLY
}

@test "load_and_validate_config loads and validates a valid sftp configuration profile" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="192.168.1.100"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_user"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="2222"
export RESTIC_PASSWORD="testpassword"
export BACKUP_TARGETS="/home/user/data"
export BACKUP_EXCLUDES="/home/user/data/temp"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="host1"
EOF
  chmod 600 "$BACKUP_ENV_FILE"

  # We declare an associative array for output configuration
  # and a normal array for error messages.
  declare -A resolved=()
  declare -a errors=()

  # Invoke the registry load function directly to preserve modifications in parent shell
  load_and_validate_config "host1" resolved errors
  local status=$?
  
  [ "$status" -eq 0 ]
  [ "${#errors[@]}" -eq 0 ]
  
  # Check if config was resolved correctly
  [ "${resolved[backend]}" = "sftp" ]
  [ "${resolved[host]}" = "192.168.1.100" ]
  [ "${resolved[user]}" = "backup_user" ]
  [ "${resolved[port]}" = "2222" ]
  [ "${resolved[password]}" = "testpassword" ]
  [ "${resolved[targets]}" = "/home/user/data" ]
  [ "${resolved[excludes_csv]}" = "/home/user/data/temp" ]
  [ "${resolved[keep_daily]}" = "7" ]
  [ "${resolved[keep_weekly]}" = "4" ]
  [ "${resolved[keep_monthly]}" = "12" ]
  [ "${resolved[profile_name]}" = "host1" ]
}

@test "load_and_validate_config resolves configuration using priority fallback rules" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"

  # File values
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export BACKUP_TARGETS="/file/targets"
export KEEP_DAILY="14"
export KEEP_WEEKLY="8"
export KEEP_MONTHLY="24"
export RESTIC_PASSWORD="file-pwd"
export RESTIC_REPOSITORY="s3:https://file.com/bucket"
export AWS_ACCESS_KEY_ID="file-access"
export AWS_SECRET_ACCESS_KEY="file-secret"
export BACKUP_PROFILE_NAME="file-profile"
EOF
  chmod 600 "$BACKUP_ENV_FILE"

  # Env values
  export BACKUP_TARGETS="/env/targets"
  export KEEP_DAILY="30"
  export BACKUP_PASSWORD="env-pwd"
  
  # CLI values
  declare -A cli_opts=()
  cli_opts[targets]="/cli/targets"
  cli_opts[keep-weekly]="12" # Overrides env/file
  
  declare -A resolved=()
  declare -a errors=()

  # Invoke registry with CLI overrides
  load_and_validate_config "cli-profile" cli_opts resolved errors
  local status=$?

  # Clean up exported env vars to avoid polluting other tests
  unset BACKUP_TARGETS
  unset KEEP_DAILY
  unset BACKUP_PASSWORD

  [ "$status" -eq 0 ]
  [ "${#errors[@]}" -eq 0 ]

  # CLI wins (highest priority)
  [ "${resolved[targets]}" = "/cli/targets" ]
  [ "${resolved[keep_weekly]}" = "12" ]

  # Env wins over File
  [ "${resolved[keep_daily]}" = "30" ]
  [ "${resolved[password]}" = "env-pwd" ]

  # File wins over Default
  [ "${resolved[keep_monthly]}" = "24" ]
  [ "${resolved[backend]}" = "s3" ]
  
  # CLI profile_name wins
  [ "${resolved[profile_name]}" = "cli-profile" ]
}

@test "load_and_validate_config returns 1 and accumulates errors on invalid configuration" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"

  # Missing targets, invalid port, missing password
  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="192.168.1.100"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_user"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="99999" # Invalid port (> 65535)
export KEEP_DAILY="invalid_int"
export BACKUP_PROFILE_NAME="test-profile"
EOF
  chmod 600 "$BACKUP_ENV_FILE"

  DEFAULT_TARGETS=""

  declare -A resolved=()
  declare -a errors=()

  local status=0
  load_and_validate_config "test-profile" resolved errors || status=$?

  [ "$status" -eq 1 ]
  [ "${#errors[@]}" -gt 0 ]

  # Check if specific error messages are present
  local found_port_err=0
  local found_targets_err=0
  local found_keep_err=0
  local found_pwd_err=0

  local e
  for e in "${errors[@]}"; do
    if [[ "$e" == *"포트"* || "$e" == *"port"* ]]; then
      found_port_err=1
    fi
    if [[ "$e" == *"백업 대상"* || "$e" == *"BACKUP_TARGETS"* ]]; then
      found_targets_err=1
    fi
    if [[ "$e" == *"keep-daily"* ]]; then
      found_keep_err=1
    fi
    if [[ "$e" == *"비밀번호"* || "$e" == *"password"* ]]; then
      found_pwd_err=1
    fi
  done

  [ "$found_port_err" -eq 1 ]
  [ "$found_targets_err" -eq 1 ]
  [ "$found_keep_err" -eq 1 ]
  [ "$found_pwd_err" -eq 1 ]
}

@test "save_profile_config writes backup.env and syncs resticprofile configurations with secure permissions" {
  # Mock systemctl as save_profile_config will call systemctl is-enabled
  stub_command "systemctl" '
    case "$1" in
      is-enabled) exit 1 ;; # Timer is not enabled
      *) exit 0 ;;
    esac
  '

  # Create resolved config
  declare -A resolved=()
  resolved[backend]="sftp"
  resolved[host]="192.168.1.100"
  resolved[port]="22"
  resolved[user]="user1"
  resolved[password]="pwd1"
  resolved[targets]="/var/log"
  resolved[excludes_csv]="/var/log/temp"
  resolved[keep_daily]="7"
  resolved[keep_weekly]="4"
  resolved[keep_monthly]="12"
  resolved[profile_name]="myprofile"
  resolved[on_calendar]="*-*-* 03:00:00"

  # Clean up existing files
  rm -f "$BACKUP_ENV_FILE"
  rm -f "$RESTICPROFILE_CONFIG_FILE"

  # Invoke save_profile_config
  save_profile_config resolved
  local status=$?

  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  [ -f "$RESTICPROFILE_CONFIG_FILE" ]

  # Check folder permissions (700)
  local folder_perm; folder_perm=$(stat -c "%a" "$RESTIC_ETC_DIR")
  [ "$folder_perm" = "700" ]

  # Check env file permissions (600)
  local env_perm; env_perm=$(stat -c "%a" "$BACKUP_ENV_FILE")
  [ "$env_perm" = "600" ]

  # Check profiles.yaml file permissions (600)
  local prof_perm; prof_perm=$(stat -c "%a" "$RESTICPROFILE_CONFIG_FILE")
  [ "$prof_perm" = "600" ]

  # Check backup.env content
  run config_get "host"
  [ "$output" = "192.168.1.100" ]
  run config_get "RESTIC_REPOSITORY"
  [ "$output" = "rclone:syno_backup:/backup/myprofile" ]

  # Check profiles.yaml content
  run cat "$RESTICPROFILE_CONFIG_FILE"
  [ "$status" -eq 0 ]
  [[ "$output" == *"myprofile:"* ]]
  [[ "$output" == *"- \"/var/log\""* ]]
  [[ "$output" == *"schedule: \"*-*-* 03:00:00\""* ]]
}

@test "config_get retrieves values and load_backup_env_to_array parses file correctly" {
  local env_file="${BATS_TEST_TMPDIR}/test_query.env"
  cat <<EOF > "$env_file"
export BACKUP_PROFILE_NAME='web01'
export RESTIC_PASSWORD='mysecretpassword'
export RCLONE_CONFIG_SYNO_BACKUP_HOST='192.168.10.5'
export BACKUP_EXCLUDES='/tmp/*'
EOF
  run config_get "profile_name" "$env_file"
  [ "$status" -eq 0 ]
  [ "$output" = "web01" ]

  run config_get "password" "$env_file"
  [ "$status" -eq 0 ]
  [ "$output" = "mysecretpassword" ]

  run config_get "host" "$env_file"
  [ "$status" -eq 0 ]
  [ "$output" = "192.168.10.5" ]

  run config_get "excludes_csv" "$env_file"
  [ "$status" -eq 0 ]
  [ "$output" = "/tmp/*" ]
}

@test "parse_env_file handles static KEY='VALUE' format and de-escapes single quotes correctly" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"

  cat > "$BACKUP_ENV_FILE" <<'EOF'
RESTIC_REPOSITORY='rclone:syno_backup:/backup/host1'
RCLONE_CONFIG_SYNO_BACKUP_TYPE='sftp'
RCLONE_CONFIG_SYNO_BACKUP_HOST='192.168.1.100'
RCLONE_CONFIG_SYNO_BACKUP_USER='backup_user'
RCLONE_CONFIG_SYNO_BACKUP_PORT='2222'
RESTIC_PASSWORD='pwd'\''with'\''quotes'
BACKUP_TARGETS='/home/user/data'
KEEP_DAILY='7'
KEEP_WEEKLY='4'
KEEP_MONTHLY='12'
BACKUP_PROFILE_NAME='host1'
EOF
  chmod 600 "$BACKUP_ENV_FILE"

  declare -A resolved=()
  declare -a errors=()

  load_and_validate_config "host1" resolved errors
  local status=$?

  [ "$status" -eq 0 ]
  [ "${#errors[@]}" -eq 0 ]
  [ "${resolved[password]}" = "pwd'with'quotes" ]
  [ "${resolved[backend]}" = "sftp" ]
}

@test "load_and_validate_config automatically migrates old export format to static format atomically" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"

  cat > "$BACKUP_ENV_FILE" <<'EOF'
export RESTIC_REPOSITORY='rclone:syno_backup:/backup/host1'
export RCLONE_CONFIG_SYNO_BACKUP_TYPE='sftp'
export RCLONE_CONFIG_SYNO_BACKUP_HOST='192.168.1.100'
export RCLONE_CONFIG_SYNO_BACKUP_USER='backup_user'
export RCLONE_CONFIG_SYNO_BACKUP_PORT='2222'
export RESTIC_PASSWORD='testpassword'
export BACKUP_TARGETS='/home/user/data'
export KEEP_DAILY='7'
export KEEP_WEEKLY='4'
export KEEP_MONTHLY='12'
export BACKUP_PROFILE_NAME='host1'
EOF
  chmod 600 "$BACKUP_ENV_FILE"

  declare -A resolved=()
  declare -a errors=()

  load_and_validate_config "host1" resolved errors
  local status=$?

  [ "$status" -eq 0 ]
  [ "${#errors[@]}" -eq 0 ]

  run grep "export " "$BACKUP_ENV_FILE"
  [ "$status" -ne 0 ]

  local env_perm; env_perm=$(stat -c "%a" "$BACKUP_ENV_FILE")
  [ "$env_perm" = "600" ]

  [ "${resolved[password]}" = "testpassword" ]
}

@test "parse_env_file strictly fails on syntax errors" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"

  cat > "$BACKUP_ENV_FILE" <<'EOF'
RESTIC_REPOSITORY='rclone:syno_backup:/backup/host1'
INVALID_LINE_WITHOUT_EQUALS
EOF
  chmod 600 "$BACKUP_ENV_FILE"

  declare -A resolved=()
  declare -a errors=()

  local status=0
  load_and_validate_config "host1" resolved errors || status=$?

  [ "$status" -eq 1 ]
  [ "${#errors[@]}" -gt 0 ]
}

@test "adapter overrides command restic to inject local variables as env context" {
  stub_command "restic" '
    echo "RESTIC_PASSWORD=$RESTIC_PASSWORD"
    echo "RESTIC_REPOSITORY=$RESTIC_REPOSITORY"
  '

  # Local variables, not exported
  RESTIC_PASSWORD="local_test_pwd"
  RESTIC_REPOSITORY="local_test_repo"
  
  # Invoke the overridden function in backup.sh
  run restic snapshots

  [ "$status" -eq 0 ]
  [[ "$output" == *"RESTIC_PASSWORD=local_test_pwd"* ]]
  [[ "$output" == *"RESTIC_REPOSITORY=local_test_repo"* ]]
}




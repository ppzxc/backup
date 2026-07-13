#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "validate_backend accepts s3" {
  run validate_backend "s3"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "validate_backend accepts sftp" {
  run validate_backend "sftp"
  [ "$status" -eq 0 ]
}

@test "validate_backend rejects unknown value" {
  run validate_backend "ftp"
  [ "$status" -eq 1 ]
  [[ "$output" == *"s3"* ]]
  [[ "$output" == *"sftp"* ]]
  [[ "$output" == *"ftp"* ]]
}

@test "validate_port accepts 22" {
  run validate_port "22"
  [ "$status" -eq 0 ]
}

@test "validate_port accepts 65535" {
  run validate_port "65535"
  [ "$status" -eq 0 ]
}

@test "validate_port rejects 0" {
  run validate_port "0"
  [ "$status" -eq 1 ]
}

@test "validate_port rejects 65536" {
  run validate_port "65536"
  [ "$status" -eq 1 ]
}

@test "validate_port rejects non-numeric" {
  run validate_port "abc"
  [ "$status" -eq 1 ]
  [[ "$output" == *"abc"* ]]
}

@test "validate_port rejects an out-of-range value containing an invalid octal digit" {
  run validate_port "099999"
  [ "$status" -eq 1 ]
  [[ "$output" == *"099999"* ]]
}

@test "validate_port accepts a leading-zero value that is in range" {
  run validate_port "017"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "validate_positive_int accepts positive integer" {
  run validate_positive_int "7" "keep-daily"
  [ "$status" -eq 0 ]
}

@test "validate_positive_int rejects zero" {
  run validate_positive_int "0" "keep-daily"
  [ "$status" -eq 1 ]
  [[ "$output" == *"keep-daily"* ]]
}

@test "validate_positive_int rejects non-numeric" {
  run validate_positive_int "seven" "keep-weekly"
  [ "$status" -eq 1 ]
  [[ "$output" == *"keep-weekly"* ]]
}

@test "validate_positive_int treats a leading-zero value as decimal, not octal" {
  run validate_positive_int "0700" "test-field"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "validate_profile_name accepts letters, digits, underscore, hyphen" {
  run validate_profile_name "web01-backup_1"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "validate_profile_name rejects a value containing a slash" {
  run validate_profile_name "web01/backup"
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}

@test "validate_profile_name rejects a value containing a space" {
  run validate_profile_name "web01 backup"
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}

@test "validate_profile_name rejects an empty value" {
  run validate_profile_name ""
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}

@test "die does not double the ERROR prefix for a validate_* failure message" {
  local err
  err=$(validate_profile_name "bad name") || true
  run die "$err"
  [ "$status" -eq 1 ]
  [[ "$output" != *"ERROR: ERROR:"* ]]
}

@test "validate_profile_name accepts a dotted FQDN hostname" {
  run validate_profile_name "funa1.nanoit.kr"
  [ "$status" -eq 0 ]
}

@test "check_targets_size_warning warns when both /var/log and /etc are over 1GB" {
  stub_command "du" '
    if [[ "$*" == *"/var/log"* ]]; then
      echo "2000000 /var/log"
    elif [[ "$*" == *"/etc"* ]]; then
      echo "1500000 /etc"
    fi
  '
  run check_targets_size_warning "/var/log,/etc"
  [ "$status" -eq 0 ]
  [[ "$output" == *"WARNING:"* ]]
  [[ "$output" == *"1GB를 초과합니다"* ]]
}

@test "check_targets_size_warning does not warn when only one of /var/log and /etc is over 1GB" {
  stub_command "du" '
    if [[ "$*" == *"/var/log"* ]]; then
      echo "2000000 /var/log"
    elif [[ "$*" == *"/etc"* ]]; then
      echo "500000 /etc"
    fi
  '
  run check_targets_size_warning "/var/log,/etc"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "resolve_and_validate_config resolves global options and collects errors on failure" {
  local -A opts=()
  opts[targets]=""
  opts[password]=""
  local -A resolved=()
  local -a errors=()
  local res=0
  resolve_and_validate_config opts resolved errors || res=$?
  [ "$res" -eq 1 ]
  [ ${#errors[@]} -gt 0 ]
}

@test "resolve_and_validate_config validates profile-name and retention periods" {
  local -A opts=()
  opts[targets]="/etc"
  opts[password]="mypassword"
  opts[keep-daily]="notanumber"
  opts[profile-name]="invalid name"

  local -A resolved=()
  local -a errors=()
  local res=0
  resolve_and_validate_config opts resolved errors || res=$?
  [ "$res" -eq 1 ]
  # We expect at least two errors: one for keep-daily and one for profile-name
  [ ${#errors[@]} -ge 2 ]
}

@test "resolve_and_validate_config succeeds with valid global parameters" {
  local -A opts=()
  opts[targets]="/etc"
  opts[password]="mypassword"
  opts[keep-daily]="7"
  opts[keep-weekly]="4"
  opts[keep-monthly]="12"
  opts[profile-name]="valid-name"

  local -A resolved=()
  local -a errors=()
  local res=0
  resolve_and_validate_config opts resolved errors || res=$?
  [ "$res" -eq 0 ]
  [ ${#errors[@]} -eq 0 ]
  [ "${resolved[targets]}" = "/etc" ]
  [ "${resolved[password]}" = "mypassword" ]
  [ "${resolved[keep_daily]}" = "7" ]
  [ "${resolved[keep_weekly]}" = "4" ]
  [ "${resolved[keep_monthly]}" = "12" ]
  [ "${resolved[profile_name]}" = "valid-name" ]
}

@test "resolve_and_validate_config validates sftp backend parameters" {
  local -A opts=()
  opts[targets]="/etc"
  opts[password]="mypassword"
  opts[backend]="sftp"
  opts[host]=""
  opts[port]="invalidport"
  opts[user]="backup_user"

  local -A resolved=()
  local -a errors=()
  local res=0
  resolve_and_validate_config opts resolved errors || res=$?
  [ "$res" -eq 1 ]
  [ ${#errors[@]} -gt 0 ]
}

@test "resolve_and_validate_config validates s3 backend parameters" {
  local -A opts=()
  opts[targets]="/etc"
  opts[password]="mypassword"
  opts[backend]="s3"
  opts[endpoint]=""
  opts[bucket]="mybucket"
  opts[access-key]=""
  opts[secret-key]=""

  local -A resolved=()
  local -a errors=()
  local res=0
  resolve_and_validate_config opts resolved errors || res=$?
  [ "$res" -eq 1 ]
  [ ${#errors[@]} -gt 0 ]
}

@test "resolve_and_validate_config succeeds with valid sftp backend" {
  local -A opts=()
  opts[targets]="/etc"
  opts[password]="mypassword"
  opts[backend]="sftp"
  opts[host]="192.168.1.100"
  opts[port]="22"
  opts[user]="backup_user"

  local -A resolved=()
  local -a errors=()
  local res=0
  resolve_and_validate_config opts resolved errors || res=$?
  [ "$res" -eq 0 ]
  [ ${#errors[@]} -eq 0 ]
  [ "${resolved[host]}" = "192.168.1.100" ]
  [ "${resolved[port]}" = "22" ]
  [ "${resolved[user]}" = "backup_user" ]
}




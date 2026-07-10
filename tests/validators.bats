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

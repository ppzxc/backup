#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_placeholder_or_value returns value when present" {
  run render_placeholder_or_value "1.2.3.4" "NAS_IP"
  [ "$output" = "1.2.3.4" ]
}

@test "render_placeholder_or_value returns placeholder when empty" {
  run render_placeholder_or_value "" "NAS_IP"
  [ "$output" = "<NAS_IP>" ]
}

@test "render_setting_hint_sftp fills known values and placeholders unknown ones" {
  run render_setting_hint_sftp "1.2.3.4" "" "backup_restic"
  [[ "$output" == *"--backend sftp"* ]]
  [[ "$output" == *"--host 1.2.3.4"* ]]
  [[ "$output" == *"--port <PORT>"* ]]
  [[ "$output" == *"--user backup_restic"* ]]
  [[ "$output" == *"--password '<REPO_PASSWORD>'"* ]]
}

@test "render_setting_hint_s3 always placeholders credentials" {
  run render_setting_hint_s3 "https://s3.example.com" "my-bucket"
  [[ "$output" == *"--backend s3"* ]]
  [[ "$output" == *"--endpoint https://s3.example.com"* ]]
  [[ "$output" == *"--bucket my-bucket"* ]]
  [[ "$output" == *"--access-key <ACCESS_KEY>"* ]]
  [[ "$output" == *"--secret-key '<SECRET_KEY>'"* ]]
  [[ "$output" == *"--password '<REPO_PASSWORD>'"* ]]
}

@test "render_missing_settings_message mentions both backends and setting command" {
  run render_missing_settings_message
  [[ "$output" == *"setting"* ]]
  [[ "$output" == *"s3"* ]]
  [[ "$output" == *"sftp"* ]]
}

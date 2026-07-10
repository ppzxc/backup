#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_backup_env_s3 produces expected export lines" {
  run render_backup_env_s3 "host1" "https://s3.example.com" "my-bucket" "AKIA123" "secretkey" "repopass" "/var/log" "/tmp/*,/var/tmp/*" "7" "4" "12" "web01"
  [[ "$output" == *'export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"'* ]]
  [[ "$output" == *'export AWS_ACCESS_KEY_ID="AKIA123"'* ]]
  [[ "$output" == *'export AWS_SECRET_ACCESS_KEY="secretkey"'* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="repopass"'* ]]
  [[ "$output" == *'export BACKUP_TARGETS="/var/log"'* ]]
  [[ "$output" == *'export BACKUP_PROFILE_NAME="web01"'* ]]
}

@test "render_s3_bucket_policy scopes actions and resource to the given bucket" {
  run render_s3_bucket_policy "my-bucket"
  [[ "$output" == *'"arn:aws:s3:::my-bucket"'* ]]
  [[ "$output" == *'"arn:aws:s3:::my-bucket/*"'* ]]
  [[ "$output" == *"ListBucket"* ]]
  [[ "$output" == *"GetObject"* ]]
  [[ "$output" == *"PutObject"* ]]
  [[ "$output" == *"DeleteObject"* ]]
}

@test "cmd_setting s3 writes backup.env with 600 perms and prints bucket policy" {
  run cmd_setting --backend s3 --endpoint https://s3.example.com --bucket my-bucket --access-key AKIA123 --secret-key secretkey --password repopass
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  perm=$(stat -c '%a' "$BACKUP_ENV_FILE")
  [ "$perm" = "600" ]
  [[ "$output" == *"arn:aws:s3:::my-bucket"* ]]
}

@test "cmd_setting s3 fails with actionable hint when --bucket is missing" {
  run cmd_setting --backend s3 --endpoint https://s3.example.com --access-key AKIA123 --secret-key secretkey --password repopass
  [ "$status" -eq 1 ]
  [[ "$output" == *"--endpoint https://s3.example.com"* ]]
  [[ "$output" == *"--bucket <BUCKET_NAME>"* ]]
}

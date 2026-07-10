#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
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

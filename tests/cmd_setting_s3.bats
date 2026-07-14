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

@test "cmd_setting s3 secondary backup writes secondary variables in backup.env" {
  run cmd_setting --backend sftp --host 192.168.1.100 --user backup_user --password 'repopass' --targets '/etc' \
    --secondary-backend s3 --secondary-endpoint https://sec.s3.com --secondary-bucket sec-bucket \
    --secondary-access-key SEC_AK --secondary-secret-key SEC_SK --secondary-password 'secpass' \
    --secondary-keep-daily 30 --secondary-keep-weekly 12 --secondary-keep-monthly 12
  
  [ "$status" -eq 0 ]


  [ -f "$BACKUP_ENV_FILE" ]
  
  # backup.env 내용 파싱 및 검증
  source "$BACKUP_ENV_FILE"
  [ "$SECONDARY_BACKEND" = "s3" ]
  [ "$SECONDARY_RESTIC_REPOSITORY" = "s3:https://sec.s3.com/sec-bucket/$(hostname)" ]
  [ "$SECONDARY_RESTIC_PASSWORD" = "secpass" ]
  [ "$SECONDARY_AWS_ACCESS_KEY_ID" = "SEC_AK" ]
  [ "$SECONDARY_AWS_SECRET_ACCESS_KEY" = "SEC_SK" ]
  [ "$SECONDARY_KEEP_DAILY" = "30" ]
  [ "$SECONDARY_KEEP_WEEKLY" = "12" ]
  [ "$SECONDARY_KEEP_MONTHLY" = "12" ]
}


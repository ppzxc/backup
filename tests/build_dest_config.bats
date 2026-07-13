#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

# ---------------------------------------------------------------------------
# Behavior 1: Dest=SFTP
# ---------------------------------------------------------------------------

@test "build_dest_config sftp dest: repo path uses rclone syno_backup_dst prefix" {
  run build_dest_config sftp sftp host1 secret \
    host=nas.local port=22 user=admin key_file=/etc/restic/backup_key

  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "rclone:syno_backup_dst:/backup/host1" ]
}

@test "build_dest_config sftp dest: env includes RCLONE DST TYPE=sftp" {
  run build_dest_config sftp sftp host1 secret \
    host=nas.local port=22 user=admin key_file=/etc/restic/backup_key

  [ "$status" -eq 0 ]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_TYPE=sftp"* ]]
}

@test "build_dest_config sftp dest: env includes host, port, user, key_file" {
  run build_dest_config sftp sftp host1 secret \
    host=nas.local port=2222 user=admin key_file=/etc/restic/backup_key

  [ "$status" -eq 0 ]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_HOST=nas.local"* ]]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_PORT=2222"* ]]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_USER=admin"* ]]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_KEY_FILE=/etc/restic/backup_key"* ]]
}

@test "build_dest_config sftp dest: env includes RESTIC_PASSWORD" {
  run build_dest_config sftp sftp host1 mysecret \
    host=nas.local port=22 user=admin key_file=/etc/restic/backup_key

  [ "$status" -eq 0 ]
  [[ "$output" == *"RESTIC_PASSWORD=mysecret"* ]]
}

# ---------------------------------------------------------------------------
# Behavior 2: Dest=S3, Src=SFTP
# ---------------------------------------------------------------------------

@test "build_dest_config s3 dest (from sftp source): repo path is native s3 format" {
  run build_dest_config s3 sftp host1 secret \
    endpoint=https://s3.amazonaws.com bucket=my-bucket access_key=key1 secret_key=sec1

  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "s3:https://s3.amazonaws.com/my-bucket/host1" ]
}

@test "build_dest_config s3 dest (from sftp source): env includes AWS credentials" {
  run build_dest_config s3 sftp host1 secret \
    endpoint=https://s3.amazonaws.com bucket=my-bucket access_key=key1 secret_key=sec1

  [ "$status" -eq 0 ]
  [[ "$output" == *"AWS_ACCESS_KEY_ID=key1"* ]]
  [[ "$output" == *"AWS_SECRET_ACCESS_KEY=sec1"* ]]
  [[ "$output" != *"RCLONE_CONFIG_SYNO_BACKUP_DST_TYPE"* ]]
}

# ---------------------------------------------------------------------------
# Behavior 3: Dest=S3, Src=S3
# ---------------------------------------------------------------------------

@test "build_dest_config s3 dest (from s3 source): repo path uses rclone prefix" {
  run build_dest_config s3 s3 host1 secret \
    endpoint=https://s3.amazonaws.com bucket=my-bucket access_key=key1 secret_key=sec1

  [ "$status" -eq 0 ]
  [ "${lines[0]}" = "rclone:syno_backup_dst:my-bucket/host1" ]
}

@test "build_dest_config s3 dest (from s3 source): env includes rclone config for s3 dst" {
  run build_dest_config s3 s3 host1 secret \
    endpoint=https://s3.amazonaws.com bucket=my-bucket access_key=key1 secret_key=sec1

  [ "$status" -eq 0 ]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_TYPE=s3"* ]]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_PROVIDER=other"* ]]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_ENDPOINT=https://s3.amazonaws.com"* ]]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_ACCESS_KEY_ID=key1"* ]]
  [[ "$output" == *"RCLONE_CONFIG_SYNO_BACKUP_DST_SECRET_ACCESS_KEY=sec1"* ]]
}

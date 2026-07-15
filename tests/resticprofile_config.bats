#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_resticprofile_config embeds repository, secrets, retention, and schedule" {
  export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
  export RESTIC_PASSWORD="super-secret"
  export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
  export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
  export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
  export RCLONE_CONFIG_SYNO_BACKUP_PORT="22"
  export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
  export BACKUP_TARGETS="/var/log,/etc"
  export BACKUP_EXCLUDES="/tmp/*"
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [[ "$output" == *'repository: "rclone:syno_backup:/backup/host1"'* ]]
  [[ "$output" == *'force-inactive-lock: true'* ]]
  [[ "$output" == *'RESTIC_PASSWORD: "super-secret"'* ]]
  [[ "$output" == *'RCLONE_CONFIG_SYNO_BACKUP_HOST: "1.2.3.4"'* ]]
  [[ "$output" == *'after-backup: true'* ]]
  [[ "$output" == *'prune: true'* ]]
  [[ "$output" == *'keep-daily: 7'* ]]
  [[ "$output" == *'keep-weekly: 4'* ]]
  [[ "$output" == *'keep-monthly: 12'* ]]
  [[ "$output" == *'schedule: "*-*-* 02:00:00"'* ]]
  [[ "$output" == *'schedule-permission: system'* ]]
  [[ "$output" == *'- "/var/log"'* ]]
  [[ "$output" == *'- "/etc"'* ]]
  [[ "$output" == *'exclude:'* ]]
  [[ "$output" == *'- "/tmp/*"'* ]]
  [[ "$output" == *'systemd-unit-template:'* ]]
  [[ "$output" == *'systemd-timer-template:'* ]]
}

@test "render_resticprofile_config embeds s3 credentials when AWS_* is set" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export AWS_ACCESS_KEY_ID="AKIA123"
  export AWS_SECRET_ACCESS_KEY="secretkey"
  export BACKUP_TARGETS="/var/log"
  export BACKUP_EXCLUDES=""
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [[ "$output" == *'AWS_ACCESS_KEY_ID: "AKIA123"'* ]]
  [[ "$output" == *'AWS_SECRET_ACCESS_KEY: "secretkey"'* ]]
}

@test "render_resticprofile_unit_template keeps the ISMS description and hardens the service" {
  run render_resticprofile_unit_template
  [[ "$output" == *"ISMS Compliance"* ]]
  [[ "$output" == *"User=root"* ]]
  [[ "$output" == *'ExecStart={{ .CommandLine }}'* ]]
}

@test "render_resticprofile_unit_template never emits the .Environment block (would leak RESTIC_PASSWORD into a 644 unit file)" {
  run render_resticprofile_unit_template
  [[ "$output" != *".Environment"* ]]
  [[ "$output" != *"Environment="* ]]
}

@test "render_resticprofile_timer_template keeps the ISMS description" {
  run render_resticprofile_timer_template
  [[ "$output" == *"ISMS Compliance"* ]]
  [[ "$output" == *'{{ range .OnCalendar -}}'* ]]
}

# Obsolete notification tests removed to unify alert dispatching in backup.sh run hook

@test "render_resticprofile_config renders secondary profile section when configured" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export BACKUP_TARGETS="/var/log"
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"
  
  export SECONDARY_BACKEND="s3"
  export SECONDARY_RESTIC_REPOSITORY="s3:https://sec-s3.com/sec-bucket/host1"
  export SECONDARY_RESTIC_PASSWORD="sec-secret"
  export SECONDARY_KEEP_DAILY="30"
  export SECONDARY_KEEP_WEEKLY="12"
  export SECONDARY_KEEP_MONTHLY="12"
  export SECONDARY_AWS_ACCESS_KEY_ID="SEC_AK"
  export SECONDARY_AWS_SECRET_ACCESS_KEY="SEC_SK"

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  
  # 1차 프로필 확인
  [[ "$output" == *"web01:"* ]]
  [[ "$output" == *"repository: \"s3:https://s3.example.com/my-bucket/host1\""* ]]
  
  # 2차 프로필 확인
  [[ "$output" == *"web01-secondary:"* ]]
  [[ "$output" == *"repository: \"s3:https://sec-s3.com/sec-bucket/host1\""* ]]
  [[ "$output" == *"RESTIC_PASSWORD: \"sec-secret\""* ]]
  [[ "$output" == *"AWS_ACCESS_KEY_ID: \"SEC_AK\""* ]]
  [[ "$output" == *"AWS_SECRET_ACCESS_KEY: \"SEC_SK\""* ]]
  [[ "$output" == *"keep-daily: 30"* ]]
  [[ "$output" == *"keep-weekly: 12"* ]]
  [[ "$output" == *"keep-weekly: 12"* ]]
  [[ "$output" == *"keep-monthly: 12"* ]]
}

@test "render_resticprofile_config renders db profile section when configured" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export BACKUP_TARGETS="/var/log"
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"
  
  export BACKUP_DB_TYPE="mysql"
  export BACKUP_DB_COMMAND="mysqldump --all"
  export BACKUP_DB_FILENAME="test-db.sql"
  export BACKUP_DB_SCHEDULE="*-*-* 03:00:00"
  export KEEP_DB_DAILY="5"
  export KEEP_DB_WEEKLY="2"
  export KEEP_DB_MONTHLY="1"

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]

  # DB 프로필 명칭 확인 (inherit 없이 독립 프로필)
  [[ "$output" == *"web01-db:"* ]]
  [[ "$output" == *"repository: \"s3:https://s3.example.com/my-bucket/host1\""* ]]
  [[ "$output" == *"stdin: true"* ]]
  [[ "$output" == *"stdin-command: \"mysqldump --all\""* ]]
  [[ "$output" == *"stdin-filename: \"test-db.sql\""* ]]
  [[ "$output" == *"schedule: \"*-*-* 03:00:00\""* ]]
  [[ "$output" == *"keep-daily: 5"* ]]
  [[ "$output" == *"keep-weekly: 2"* ]]
  [[ "$output" == *"keep-monthly: 1"* ]]
  [[ "$output" == *"tag:"* ]]
  [[ "$output" == *"- db"* ]]
  # inherit이 없어야 한다 (stdin과 source 충돌 방지)
  [[ "$output" != *"inherit: web01"* ]]
}




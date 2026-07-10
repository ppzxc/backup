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

  run render_resticprofile_config "web01" "*-*-* 02:00:00" "/var/log,/etc" "/tmp/*" "7" "4" "12"
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

  run render_resticprofile_config "web01" "*-*-* 02:00:00" "/var/log" "" "7" "4" "12"
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

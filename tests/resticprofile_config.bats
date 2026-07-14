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

@test "render_resticprofile_config renders slack notifications correctly" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export BACKUP_TARGETS="/var/log"
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"
  export BACKUP_NOTIFICATION_URL="https://hooks.slack.com/services/test"
  export BACKUP_NOTIFICATION_TYPE="slack"
  export BACKUP_NOTIFICATION_ON="both"

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [[ "$output" == *"HOSTNAME: "* ]]
  [[ "$output" == *"send-after:"* ]]
  [[ "$output" == *"send-after-fail:"* ]]
  [[ "$output" == *"https://hooks.slack.com/services/test"* ]]
  [[ "$output" == *"Content-Type"* ]]
  [[ "$output" == *"application/json"* ]]
  [[ "$output" == *"restic 백업 성공"* ]]
  [[ "$output" == *"restic 백업 실패"* ]]
}

@test "render_resticprofile_config renders discord notifications correctly" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export BACKUP_TARGETS="/var/log"
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"
  export BACKUP_NOTIFICATION_URL="https://discord.com/api/webhooks/test"
  export BACKUP_NOTIFICATION_TYPE="discord"
  export BACKUP_NOTIFICATION_ON="failure"

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [[ "$output" == *"HOSTNAME: "* ]]
  [[ "$output" != *"send-after:"* ]]
  [[ "$output" == *"send-after-fail:"* ]]
  [[ "$output" == *"https://discord.com/api/webhooks/test"* ]]
  [[ "$output" == *'"content":'* ]]
}

@test "render_resticprofile_config renders custom notifications correctly" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export BACKUP_TARGETS="/var/log"
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"
  export BACKUP_NOTIFICATION_URL="https://my.webhook.internal/alerts"
  export BACKUP_NOTIFICATION_TYPE="custom"
  export BACKUP_NOTIFICATION_ON="both"
  export BACKUP_NOTIFICATION_METHOD="PUT"
  export BACKUP_NOTIFICATION_HEADERS="X-Alert-Key: secret123, Content-Type: application/json"
  export BACKUP_NOTIFICATION_BODY_SUCCESS='{"status":"ok","msg":"done"}'
  export BACKUP_NOTIFICATION_BODY_FAILURE='{"status":"error","msg":"${ERROR}"}'

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [[ "$output" == *"send-after:"* ]]
  [[ "$output" == *"method: \"PUT\""* ]]
  [[ "$output" == *"url: \"https://my.webhook.internal/alerts\""* ]]
  [[ "$output" == *"body: |"* ]]
  [[ "$output" == *"{\"status\":\"ok\",\"msg\":\"done\"}"* ]]
  [[ "$output" == *"X-Alert-Key"* ]]
  [[ "$output" == *"secret123"* ]]
  [[ "$output" == *"send-after-fail:"* ]]
  [[ "$output" == *"{\"status\":\"error\",\"msg\":\"\${ERROR}\"}"* ]]
}

@test "render_resticprofile_config expands \\n in custom plain text notifications" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export BACKUP_TARGETS="/var/log"
  export KEEP_DAILY="7"
  export KEEP_WEEKLY="4"
  export KEEP_MONTHLY="12"
  export BACKUP_NOTIFICATION_URL="https://my.webhook.internal/alerts"
  export BACKUP_NOTIFICATION_TYPE="custom"
  export BACKUP_NOTIFICATION_ON="both"
  export BACKUP_NOTIFICATION_METHOD="POST"
  export BACKUP_NOTIFICATION_HEADERS="Content-Type: text/plain"
  export BACKUP_NOTIFICATION_BODY_SUCCESS='🔔 SUCCESS\n- Host: ${HOSTNAME}'
  export BACKUP_NOTIFICATION_BODY_FAILURE='🚨 FAILURE\n- Host: ${HOSTNAME}'

  run render_resticprofile_config "web01" "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [[ "$output" == *"body: |"* ]]
  [[ "$output" == *"🔔 SUCCESS"* ]]
  [[ "$output" == *"- Host: \${HOSTNAME}"* ]]
  # Verify that literal \n was expanded to a physical newline and is not literal \n characters
  [[ "$output" != *"🔔 SUCCESS\n"* ]]
}

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
  [[ "$output" == *"keep-monthly: 12"* ]]
}



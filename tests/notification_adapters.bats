#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "notification_slack_validate returns 0" {
  run notification_slack_validate
  [ "$status" -eq 0 ]
}

@test "notification_slack_send calls curl with Slack payload" {
  stub_command "curl" 'echo "curl $*" >> "'"${STUB_BIN}"'/curl.calls"; exit 0'
  declare -A ctx=(
    [notify_url]="https://slack.webhook"
    [status]="success"
    [err_msg]=""
  )
  BACKUP_PROFILE_NAME="myprofile"
  run notification_slack_send ctx
  [ "$status" -eq 0 ]
  [ -f "${STUB_BIN}/curl.calls" ]
  grep -q "성공" "${STUB_BIN}/curl.calls"
  grep -q "myprofile" "${STUB_BIN}/curl.calls"
}

@test "notification_discord_validate returns 0" {
  run notification_discord_validate
  [ "$status" -eq 0 ]
}

@test "notification_discord_send calls curl with Discord payload" {
  stub_command "curl" 'echo "curl $*" >> "'"${STUB_BIN}"'/curl.calls"; exit 0'
  declare -A ctx=(
    [notify_url]="https://discord.webhook"
    [status]="failure"
    [err_msg]="some error"
  )
  BACKUP_PROFILE_NAME="myprofile"
  run notification_discord_send ctx
  [ "$status" -eq 0 ]
  [ -f "${STUB_BIN}/curl.calls" ]
  grep -q "실패" "${STUB_BIN}/curl.calls"
  grep -q "some error" "${STUB_BIN}/curl.calls"
}

@test "notification_custom_validate returns 0" {
  run notification_custom_validate
  [ "$status" -eq 0 ]
}

@test "notification_custom_send calls curl with custom payload and method" {
  stub_command "curl" 'echo "curl $*" >> "'"${STUB_BIN}"'/curl.calls"; exit 0'
  declare -A ctx=(
    [notify_url]="https://custom.webhook"
    [status]="success"
    [err_msg]=""
    [method]="POST"
    [headers]="Content-Type: text/plain"
    [body_success]="success payload"
    [body_failure]="failure payload"
    [profile_command]="run-cmd"
  )
  BACKUP_PROFILE_NAME="myprofile"
  run notification_custom_send ctx
  [ "$status" -eq 0 ]
  [ -f "${STUB_BIN}/curl.calls" ]
  grep -q "custom.webhook" "${STUB_BIN}/curl.calls"
  grep -q "success payload" "${STUB_BIN}/curl.calls"
}

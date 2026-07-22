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
  run notification_slack_send "https://slack.webhook" "success" "myhost" "myprofile" ""
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
  run notification_discord_send "https://discord.webhook" "failure" "myhost" "myprofile" "some error"
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
  run notification_custom_send "https://custom.webhook" "success" "myhost" "myprofile" "" "POST" "Content-Type: text/plain" "success payload" "failure payload" "run-cmd"
  [ "$status" -eq 0 ]
  [ -f "${STUB_BIN}/curl.calls" ]
  grep -q "custom.webhook" "${STUB_BIN}/curl.calls"
  grep -q "success payload" "${STUB_BIN}/curl.calls"
}

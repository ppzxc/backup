#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_service_unit references the installed script path and run subcommand" {
  run render_service_unit
  [[ "$output" == *"ExecStart=${BACKUP_SCRIPT_INSTALL_PATH} run"* ]]
  [[ "$output" == *"Type=oneshot"* ]]
}

@test "render_timer_unit embeds the given OnCalendar value" {
  run render_timer_unit "*-*-* 03:30:00"
  [[ "$output" == *"OnCalendar=*-*-* 03:30:00"* ]]
  [[ "$output" == *"Persistent=true"* ]]
  [[ "$output" == *"WantedBy=timers.target"* ]]
}

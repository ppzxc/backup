#!/usr/bin/env bats

load 'test_helper'

setup() {
  setup_backup_sh_env
}

@test "is_interactive returns 1 in non-TTY BATS environment" {
  run is_interactive
  [ "$status" -eq 1 ]
}

@test "is_interactive returns 1 when GUM_DISABLE=1 or NO_COLOR=1 is set" {
  NO_COLOR=1 run is_interactive
  [ "$status" -eq 1 ]
  
  GUM_DISABLE=1 run is_interactive
  [ "$status" -eq 1 ]
}

@test "safe_spin executes command directly in non-TTY environment without ANSI codes" {
  run safe_spin "Testing task" -- echo "completed successfully"
  [ "$status" -eq 0 ]
  [[ "$output" =~ "[RUN] Testing task" ]]
  [[ "$output" =~ "completed successfully" ]]
  [[ ! "$output" =~ $'\x1b' ]]
}

@test "safe_spin preserves exact exit code of wrapped command" {
  run safe_spin "Failing task" -- bash -c "exit 3"
  [ "$status" -eq 3 ]
}

@test "safe_confirm handles piped input cleanly in non-TTY mode" {
  run bash -c "source ./backup.sh && echo 'y' | safe_confirm 'Proceed?' 'n'"
  [ "$status" -eq 0 ]

  run bash -c "source ./backup.sh && echo 'n' | safe_confirm 'Proceed?' 'y'"
  [ "$status" -eq 1 ]
}

@test "safe_input handles EOF without breaking set -e" {
  run bash -c "set -e; source ./backup.sh; safe_input 'Enter value' 'default_val' < /dev/null"
  [ "$status" -eq 0 ]
  [[ "$output" =~ "default_val" ]]
}

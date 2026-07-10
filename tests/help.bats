#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_help prints usage mentioning all subcommands" {
  run render_help
  [ "$status" -eq 0 ]
  [[ "$output" == *"install"* ]]
  [[ "$output" == *"setting"* ]]
  [[ "$output" == *"init"* ]]
  [[ "$output" == *"schedule"* ]]
  [[ "$output" == *"run"* ]]
  [[ "$output" == *"status"* ]]
  [[ "$output" == *"uninstall"* ]]
  [[ "$output" == *"wizard"* ]]
}

@test "main with no args prints help and exits 0" {
  run main
  [ "$status" -eq 0 ]
  [[ "$output" == *"install"* ]]
}

@test "main with -h exits 0" {
  run main -h
  [ "$status" -eq 0 ]
}

@test "main with --help exits 0" {
  run main --help
  [ "$status" -eq 0 ]
}

@test "main with unknown subcommand prints help and exits 1" {
  run main bogus-command
  [ "$status" -eq 1 ]
  [[ "$output" == *"install"* ]]
}

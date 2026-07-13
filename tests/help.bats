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
  [[ "$output" == *"audit"* ]]
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

@test "main strips --verbose/-v from argv before dispatch, from any position" {
  run main --verbose -h
  [ "$status" -eq 0 ]

  run main -h -v
  [ "$status" -eq 0 ]
}

@test "main with -V or --version prints the script version and exits 0" {
  run main -V
  [ "$status" -eq 0 ]
  [[ "$output" == "1.0.0" ]]

  run main --version
  [ "$status" -eq 0 ]
  [[ "$output" == "1.0.0" ]]
}

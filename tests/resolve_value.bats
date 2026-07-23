#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "cli value wins over everything" {
  run resolve_value "cli-val" "env-val" "file-val" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "cli-val" ]
}

@test "env value wins when cli is empty" {
  run resolve_value "" "env-val" "file-val" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "env-val" ]
}

@test "file value wins when cli and env are empty" {
  run resolve_value "" "" "file-val" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "file-val" ]
}

@test "default is used when everything else is empty" {
  run resolve_value "" "" "" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "default-val" ]
}

@test "returns 1 and prints nothing when all are empty" {
  run resolve_value "" "" "" ""
  [ "$status" -eq 1 ]
  [ -z "$output" ]
}

@test "resolve_value called with fewer than 4 arguments does not throw unbound variable error" {
  run resolve_value "cli-val" "env-val"
  [ "$status" -eq 0 ]
  [ "$output" = "cli-val" ]

  run resolve_value "" "env-val"
  [ "$status" -eq 0 ]
  [ "$output" = "env-val" ]
}


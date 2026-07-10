#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "parses a single value flag with space form" {
  run parse_long_opts "host:" -- --host 1.2.3.4
  [ "$status" -eq 0 ]
  [ "$output" = $'host\t1.2.3.4' ]
}

@test "parses a single value flag with equals form" {
  run parse_long_opts "host:" -- --host=1.2.3.4
  [ "$status" -eq 0 ]
  [ "$output" = $'host\t1.2.3.4' ]
}

@test "parses a boolean flag" {
  run parse_long_opts "force" -- --force
  [ "$status" -eq 0 ]
  [ "$output" = $'force\t1' ]
}

@test "parses repeated value flags into multiple lines" {
  run parse_long_opts "exclude:" -- --exclude /tmp/* --exclude /var/tmp/*
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = $'exclude\t/tmp/*' ]
  [ "${lines[1]}" = $'exclude\t/var/tmp/*' ]
}

@test "parses mixed value and boolean flags" {
  run parse_long_opts "backend: force" -- --backend s3 --force
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = $'backend\ts3' ]
  [ "${lines[1]}" = $'force\t1' ]
}

@test "rejects unknown flag" {
  run parse_long_opts "host:" -- --bogus x
  [ "$status" -eq 1 ]
  [[ "$output" == *"bogus"* ]]
}

@test "rejects value flag missing its value" {
  run parse_long_opts "host:" -- --host
  [ "$status" -eq 1 ]
  [[ "$output" == *"host"* ]]
}

@test "rejects unexpected positional argument" {
  run parse_long_opts "host:" -- extra-arg
  [ "$status" -eq 1 ]
  [[ "$output" == *"extra-arg"* ]]
}

@test "empty args with empty spec succeeds with no output" {
  run parse_long_opts "" --
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

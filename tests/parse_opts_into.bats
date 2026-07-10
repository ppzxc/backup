#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "parse_opts_into fills an assoc array with value flags" {
  local -A opts=()
  parse_opts_into opts "host:" -- --host 1.2.3.4
  [ "${opts[host]}" = "1.2.3.4" ]
}

@test "parse_opts_into fills boolean flags with 1" {
  local -A opts=()
  parse_opts_into opts "force" -- --force
  [ "${opts[force]}" = "1" ]
}

@test "parse_opts_into handles hyphenated flag names as keys" {
  local -A opts=()
  parse_opts_into opts "on-calendar:" -- --on-calendar "*-*-* 03:00:00"
  [ "${opts[on-calendar]}" = "*-*-* 03:00:00" ]
}

@test "parse_opts_into joins repeated flags with commas" {
  local -A opts=()
  parse_opts_into opts "exclude:" -- --exclude '/tmp/*' --exclude '/var/tmp/*'
  [ "${opts[exclude]}" = '/tmp/*,/var/tmp/*' ]
}

@test "parse_opts_into leaves the array empty when no flags are given" {
  local -A opts=()
  parse_opts_into opts "" --
  [ ${#opts[@]} -eq 0 ]
}

@test "parse_opts_into dies with parse_long_opts's error message on an unknown flag" {
  local -A opts=()
  run parse_opts_into opts "host:" -- --bogus x
  [ "$status" -eq 1 ]
  [[ "$output" == *"bogus"* ]]
}

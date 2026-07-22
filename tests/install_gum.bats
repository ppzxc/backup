#!/usr/bin/env bats

load 'test_helper'

setup() {
  setup_backup_sh_env
}

@test "install_gum succeeds or degrades gracefully without error" {
  run install_gum
  [ "$status" -eq 0 ]
}

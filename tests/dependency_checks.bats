#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "check_system_dependencies passes if all system tools exist" {
  # has_dependency를 오버라이드하여 모두 존재한다고 모킹
  has_dependency() {
    return 0
  }

  run check_system_dependencies
  [ "$status" -eq 0 ]
}

@test "check_system_dependencies fails if curl is missing" {
  # curl만 누락된 상황 모킹
  has_dependency() {
    if [[ "$1" == "curl" ]]; then
      return 1
    fi
    return 0
  }

  run check_system_dependencies
  [ "$status" -eq 1 ]
  [[ "$output" == *"[!] 필수 시스템 도구가 누락되어"* ]]
  [[ "$output" == *"* 누락된 도구:"* ]]
  [[ "$output" == *"curl"* ]]
  [[ "$output" == *"sudo apt-get"* ]]
}

@test "check_system_dependencies fails if python3 and tar are missing" {
  # python3와 tar가 누락된 상황 모킹
  has_dependency() {
    if [[ "$1" == "python3" || "$1" == "tar" ]]; then
      return 1
    fi
    return 0
  }

  run check_system_dependencies
  [ "$status" -eq 1 ]
  [[ "$output" == *"[!] 필수 시스템 도구가 누락되어"* ]]
  [[ "$output" == *"python3"* ]]
  [[ "$output" == *"tar"* ]]
}

@test "check_core_dependencies passes if all core tools exist" {
  has_dependency() {
    return 0
  }

  run check_core_dependencies
  [ "$status" -eq 0 ]
}

@test "check_core_dependencies fails if restic is missing" {
  # restic 누락
  has_dependency() {
    if [[ "$1" == "restic" ]]; then
      return 1
    fi
    return 0
  }

  run check_core_dependencies
  [ "$status" -eq 1 ]
  [[ "$output" == *"[!] 백업 핵심 도구가 누락되어"* ]]
  [[ "$output" == *"restic"* ]]
  [[ "$output" == *"sudo ./backup.sh install"* ]]
}

@test "main function runs dependency checks correctly based on subcommands" {
  # 시스템 도구는 다 있고 핵심 도구는 없는 상황
  has_dependency() {
    if [[ "$1" == "restic" || "$1" == "rclone" || "$1" == "resticprofile" ]]; then
      return 1
    fi
    return 0
  }

  # 1. install 명령 시도 -> 핵심 도구가 없어도 통과해야 함
  # 단, cmd_install 자체가 mock되지 않아 오류가 날 수 있으므로 cmd_install을 stub함.
  cmd_install() {
    echo "cmd_install called"
  }
  run main install
  [ "$status" -eq 0 ]
  [[ "$output" == *"cmd_install called"* ]]
  [[ "$output" != *"[!] 백업 핵심 도구가 누락되어"* ]]

  # 2. help 명령 시도 -> 핵심 도구 없어도 통과
  run main --help
  [ "$status" -eq 0 ]
  [[ "$output" == *"restic 기반 백업 설치"* ]]

  # 3. version 명령 시도 -> 핵심 도구 없어도 통과
  run main -V
  [ "$status" -eq 0 ]
  [[ "$output" != *"[!] 백업 핵심 도구가 누락되어"* ]]

  # 4. status 명령 시도 -> 핵심 도구 없으므로 에러로 실패해야 함
  run main status
  [ "$status" -eq 1 ]
  [[ "$output" == *"[!] 백업 핵심 도구가 누락되어"* ]]
}

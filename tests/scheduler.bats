#!/usr/bin/env bats

setup() {
  load test_helper
  setup_backup_sh_env
}

@test "scheduler_register under mock adapter records backup and audit settings in mock state file" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="mock"

  declare -A test_config=(
    [on-calendar]="*-*-* 02:00:00"
    [on-calendar-daily]="*-*-* 03:00:00"
    [on-calendar-drill]="*-*-01 04:00:00"
    [daily]="0"
    [restore-drill]="0"
  )

  run scheduler_register "test-profile" test_config
  [ "$status" -eq 0 ]

  local state_file="${TEST_ROOT}/var/log/scheduler_mock.state"
  [ -f "$state_file" ]

  # 등록된 주기 정보 확인
  run grep -F "backup_schedule=*-*-* 02:00:00" "$state_file"
  [ "$status" -eq 0 ]
  run grep -F "daily_schedule=*-*-* 03:00:00" "$state_file"
  [ "$status" -eq 0 ]
  run grep -F "drill_schedule=*-*-01 04:00:00" "$state_file"
  [ "$status" -eq 0 ]
  run grep "backup_enabled=1" "$state_file"
  [ "$status" -eq 0 ]
}

@test "scheduler_unregister under mock adapter disables targets selectively or fully" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="mock"

  # 초기 등록 상태 가상 생성
  mkdir -p "${TEST_ROOT}/var/log"
  local state_file="${TEST_ROOT}/var/log/scheduler_mock.state"
  cat > "$state_file" <<EOF
backup_schedule=*-*-* 02:00:00
daily_schedule=*-*-* 03:00:00
drill_schedule=*-*-01 04:00:00
backup_enabled=1
daily_enabled=1
drill_enabled=1
EOF

  # daily 해제 실행
  run scheduler_unregister "test-profile" "daily"
  [ "$status" -eq 0 ]
  run grep "daily_enabled=0" "$state_file"
  [ "$status" -eq 0 ]
  run grep "backup_enabled=1" "$state_file"
  [ "$status" -eq 0 ]

  # 전체 해제 실행
  run scheduler_unregister "test-profile" "all"
  [ "$status" -eq 0 ]
  [ ! -f "$state_file" ]
}

@test "scheduler_status under mock adapter populates status associative array with active states" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="mock"

  # mock 상태 기입
  mkdir -p "${TEST_ROOT}/var/log"
  local state_file="${TEST_ROOT}/var/log/scheduler_mock.state"
  cat > "$state_file" <<EOF
backup_enabled=1
daily_enabled=0
drill_enabled=1
EOF

  declare -A test_status=()
  # status 조회
  # BATS 서브쉘 밖에서 배열 상태 변화를 테스트하기 위해 직접 실행
  scheduler_status "test-profile" test_status

  [ "${test_status[backup]}" = "active" ]
  [ "${test_status[daily]}" = "inactive" ]
  [ "${test_status[drill]}" = "active" ]
}

@test "scheduler_register under systemd adapter generates unit assets and calls systemctl" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="systemd"

  # systemctl 및 resticprofile 명령어 스텁
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; exit 0'
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"; exit 0'

  declare -A test_config=(
    [on-calendar]="*-*-* 02:00:00"
    [on-calendar-daily]="*-*-* 03:00:00"
    [on-calendar-drill]="*-*-01 04:00:00"
    [daily]="0"
    [restore-drill]="0"
  )

  run scheduler_register "test-profile" test_config
  [ "$status" -eq 0 ]

  # resticprofile schedule 호출 확인
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"test-profile schedule"* ]]

  # systemctl enable 호출 확인
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"enable --now backup-audit-daily.timer"* ]]
  [[ "$output" == *"enable --now backup-audit-restore-drill.timer"* ]]

  # systemd 에셋 생성 확인
  [ -f "${SYSTEMD_UNIT_DIR}/backup-audit-daily.service" ]
  [ -f "${SYSTEMD_UNIT_DIR}/backup-audit-restore-drill.timer" ]
}

@test "scheduler_unregister under systemd adapter disables timers and cleans systemd service files" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="systemd"

  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; exit 0'
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"; exit 0'

  # 가상 유닛 파일 생성
  touch "${SYSTEMD_UNIT_DIR}/backup-audit-daily.service"
  touch "${SYSTEMD_UNIT_DIR}/backup-audit-daily.timer"
  touch "${SYSTEMD_UNIT_DIR}/backup-audit-restore-drill.service"
  touch "${SYSTEMD_UNIT_DIR}/backup-audit-restore-drill.timer"

  run scheduler_unregister "test-profile" "all"
  [ "$status" -eq 0 ]

  # resticprofile unschedule 호출 확인
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"test-profile unschedule"* ]]

  # systemctl disable 호출 및 파일 삭제 확인
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"disable --now backup-audit-daily.timer"* ]]
  [[ "$output" == *"disable --now backup-audit-restore-drill.timer"* ]]

  [ ! -f "${SYSTEMD_UNIT_DIR}/backup-audit-daily.service" ]
  [ ! -f "${SYSTEMD_UNIT_DIR}/backup-audit-restore-drill.timer" ]
}

@test "scheduler_status under systemd adapter queries systemctl state correctly" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="systemd"

  # systemctl 모킹: 유닛 이름에 따라 다른 상태 반환
  stub_command "systemctl" '
    if [[ "$*" == *"resticprofile-backup@profile-test-profile.timer" ]]; then
      echo "active"
    elif [[ "$*" == *"backup-audit-daily.timer" ]]; then
      echo "inactive"
    elif [[ "$*" == *"backup-audit-restore-drill.timer" ]]; then
      echo "unknown"
    fi
    exit 0
  '

  declare -A test_status=()
  scheduler_status "test-profile" test_status

  [ "${test_status[backup]}" = "active" ]
  [ "${test_status[daily]}" = "inactive" ]
  [ "${test_status[drill]}" = "unknown" ]
}

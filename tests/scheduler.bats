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

@test "convert_calendar_to_cron translates systemd calendar formats to cron formats" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"

  run convert_calendar_to_cron "*-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [ "$output" = "0 2 * * *" ]

  run convert_calendar_to_cron "*-*-* 01:00"
  [ "$status" -eq 0 ]
  [ "$output" = "0 1 * * *" ]

  run convert_calendar_to_cron "*-*-01 01:30:00"
  [ "$status" -eq 0 ]
  [ "$output" = "30 1 1 * *" ]

  run convert_calendar_to_cron "Mon..Fri *-*-* 02:00:00"
  [ "$status" -eq 0 ]
  [ "$output" = "0 2 * * Mon-Fri" ]

  run convert_calendar_to_cron "30 0 * * *"
  [ "$status" -eq 0 ]
  [ "$output" = "30 0 * * *" ]

  # Fallback case
  run convert_calendar_to_cron "invalid_format_here"
  [ "$status" -eq 0 ]
  [ "$output" = "0 2 * * *" ]
}

@test "scheduler_register under cron adapter writes crontab with path, logger tags and schedule" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="cron"

  stub_command "crontab" '
    mock_cron="'"${TEST_ROOT}"'/var/spool/cron/root"
    mkdir -p "$(dirname "$mock_cron")"
    if [[ "$1" == "-l" ]]; then
      if [[ -f "$mock_cron" ]]; then
        cat "$mock_cron"
      else
        echo "no crontab for root" >&2
        exit 1
      fi
    elif [[ "$1" == "-" ]]; then
      cat > "$mock_cron"
    elif [[ "$1" == "-r" ]]; then
      rm -f "$mock_cron"
    else
      exit 1
    fi
  '

  declare -A test_config=(
    [on-calendar]="*-*-* 02:00:00"
    [on-calendar-daily]="*-*-* 03:00:00"
    [on-calendar-drill]="*-*-01 04:00:00"
    [daily]="0"
    [restore-drill]="0"
  )

  run scheduler_register "test-cron-profile" test_config
  [ "$status" -eq 0 ]

  local mock_cron="${TEST_ROOT}/var/spool/cron/root"
  [ -f "$mock_cron" ]

  # Check begin/end tags
  run grep -F "# RESTIC_BACKUP_BEGIN" "$mock_cron"
  [ "$status" -eq 0 ]
  run grep -F "# RESTIC_BACKUP_END" "$mock_cron"
  [ "$status" -eq 0 ]

  # Check command lines
  run grep -F "restic-backup-files" "$mock_cron"
  [ "$status" -eq 0 ]
  run grep -F "restic-backup-audit-daily" "$mock_cron"
  [ "$status" -eq 0 ]
  run grep -F "restic-backup-audit-drill" "$mock_cron"
  [ "$status" -eq 0 ]
}

@test "scheduler_unregister under cron adapter clears crontab section or targets" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="cron"

  stub_command "crontab" '
    mock_cron="'"${TEST_ROOT}"'/var/spool/cron/root"
    mkdir -p "$(dirname "$mock_cron")"
    if [[ "$1" == "-l" ]]; then
      if [[ -f "$mock_cron" ]]; then
        cat "$mock_cron"
      else
        echo "no crontab for root" >&2
        exit 1
      fi
    elif [[ "$1" == "-" ]]; then
      cat > "$mock_cron"
    elif [[ "$1" == "-r" ]]; then
      rm -f "$mock_cron"
    else
      exit 1
    fi
  '

  local mock_cron="${TEST_ROOT}/var/spool/cron/root"
  mkdir -p "$(dirname "$mock_cron")"
  cat > "$mock_cron" <<EOF
# RESTIC_BACKUP_BEGIN
0 2 * * * PATH=... resticprofile --config /etc/restic/profiles.yaml --name test-cron-profile backup 2>&1 | logger -t restic-backup-files
0 3 * * * PATH=... /usr/local/bin/backup.sh audit --daily --report 2>&1 | logger -t restic-backup-audit-daily
0 4 1 * * PATH=... /usr/local/bin/backup.sh audit --restore-drill --report 2>&1 | logger -t restic-backup-audit-drill
# RESTIC_BACKUP_END
EOF

  # Unregister daily audit only
  run scheduler_unregister "test-cron-profile" "daily"
  [ "$status" -eq 0 ]
  run grep -F "restic-backup-audit-daily" "$mock_cron"
  [ "$status" -ne 0 ]
  run grep -F "restic-backup-files" "$mock_cron"
  [ "$status" -eq 0 ]

  # Unregister all
  run scheduler_unregister "test-cron-profile" "all"
  [ "$status" -eq 0 ]
  run grep -F "# RESTIC_BACKUP_BEGIN" "$mock_cron"
  [ "$status" -ne 0 ]
}

@test "scheduler_status under cron adapter returns active/inactive status from crontab" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="cron"

  stub_command "crontab" '
    mock_cron="'"${TEST_ROOT}"'/var/spool/cron/root"
    mkdir -p "$(dirname "$mock_cron")"
    if [[ "$1" == "-l" ]]; then
      if [[ -f "$mock_cron" ]]; then
        cat "$mock_cron"
      else
        echo "no crontab for root" >&2
        exit 1
      fi
    elif [[ "$1" == "-" ]]; then
      cat > "$mock_cron"
    elif [[ "$1" == "-r" ]]; then
      rm -f "$mock_cron"
    else
      exit 1
    fi
  '

  local mock_cron="${TEST_ROOT}/var/spool/cron/root"
  mkdir -p "$(dirname "$mock_cron")"
  cat > "$mock_cron" <<EOF
# RESTIC_BACKUP_BEGIN
0 2 * * * PATH=... resticprofile --config /etc/restic/profiles.yaml --name test-cron-profile backup 2>&1 | logger -t restic-backup-files
0 4 1 * * PATH=... /usr/local/bin/backup.sh audit --restore-drill --report 2>&1 | logger -t restic-backup-audit-drill
# RESTIC_BACKUP_END
EOF

  declare -A test_status=()
  scheduler_status "test-cron-profile" test_status

  [ "${test_status[backup]}" = "active" ]
  [ "${test_status[daily]}" = "inactive" ]
  [ "${test_status[drill]}" = "active" ]
}

@test "determine_scheduler_adapter returns adapter based on env variable or auto-detection" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"

  # Case 1: Explicitly set in env to mock
  export BACKUP_SCHEDULER_ADAPTER="mock"
  run determine_scheduler_adapter
  [ "$status" -eq 0 ]
  [ "$output" = "mock" ]
  unset BACKUP_SCHEDULER_ADAPTER

  # Case 2: Explicitly set to invalid value
  export BACKUP_SCHEDULER_ADAPTER="invalid_adapter"
  run determine_scheduler_adapter
  [ "$status" -ne 0 ]
  unset BACKUP_SCHEDULER_ADAPTER

  # Case 3: Systemd is active
  is_systemd_active() { return 0; }
  run determine_scheduler_adapter
  [ "$status" -eq 0 ]
  [ "$output" = "systemd" ]

  # Case 4: Systemd not active, but crontab is available
  is_systemd_active() { return 1; }
  stub_command "crontab" "exit 0"
  run determine_scheduler_adapter
  [ "$status" -eq 0 ]
  [ "${lines[-1]}" = "cron" ]
  rm -f "${STUB_BIN}/crontab"

  # Case 5: Neither is available
  is_systemd_active() { return 1; }
  local old_path="$PATH"
  export PATH="${STUB_BIN}"
  run determine_scheduler_adapter
  [ "$status" -ne 0 ]
  export PATH="$old_path"
}

@test "scheduler_register under cron adapter with specific target preserves other schedules" {
  source "${BATS_TEST_DIRNAME}/../backup.sh"
  export BACKUP_SCHEDULER_ADAPTER="cron"

  stub_command "crontab" '
    mock_cron="'"${TEST_ROOT}"'/var/spool/cron/root"
    mkdir -p "$(dirname "$mock_cron")"
    if [[ "$1" == "-l" ]]; then
      if [[ -f "$mock_cron" ]]; then
        cat "$mock_cron"
      else
        echo "no crontab for root" >&2
        exit 1
      fi
    elif [[ "$1" == "-" ]]; then
      cat > "$mock_cron"
    elif [[ "$1" == "-r" ]]; then
      rm -f "$mock_cron"
    else
      exit 1
    fi
  '

  local mock_cron="${TEST_ROOT}/var/spool/cron/root"
  mkdir -p "$(dirname "$mock_cron")"
  cat > "$mock_cron" <<EOF
# RESTIC_BACKUP_BEGIN
0 2 * * * PATH=... resticprofile --config /etc/restic/profiles.yaml --name test-cron-profile backup 2>&1 | logger -t restic-backup-files
0 3 * * * PATH=... /usr/local/bin/backup.sh audit --daily --report 2>&1 | logger -t restic-backup-audit-daily
0 4 1 * * PATH=... /usr/local/bin/backup.sh audit --restore-drill --report 2>&1 | logger -t restic-backup-audit-drill
# RESTIC_BACKUP_END
EOF

  declare -A test_config=(
    [on-calendar-daily]="*-*-* 05:00:00"
    [daily]="1"
    [restore-drill]="0"
  )

  # Register daily only (updates daily to 5:00, preserves backup and drill)
  run scheduler_register "test-cron-profile" test_config
  [ "$status" -eq 0 ]

  # Check that files and drill are still present, and daily is updated
  run grep -F "restic-backup-files" "$mock_cron"
  [ "$status" -eq 0 ]
  run grep -F "restic-backup-audit-drill" "$mock_cron"
  [ "$status" -eq 0 ]
  run grep -q "0 5 \* \* \*" "$mock_cron"
  [ "$status" -eq 0 ]
}





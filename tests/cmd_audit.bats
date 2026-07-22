#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="web01"
ENV
  chmod 600 "$BACKUP_ENV_FILE"
  stub_command "systemctl" '
    case "$1" in
      is-enabled) echo "enabled"; exit 0 ;;
      is-active) echo "active"; exit 0 ;;
      list-timers) echo "Thu 2026-07-16 02:00:00 KST 2days left n/a n/a resticprofile-backup@profile-web01.timer" ;;
      cat) echo "OnCalendar=*-*-* 02:00:00" ;;
    esac
  '
  stub_command "restic" '
    case "$1" in
      snapshots)
        if [[ "$*" == *"--json"* ]]; then
          echo "[{\"id\":\"abc123456789\",\"short_id\":\"abc12345\",\"time\":\"2026-07-15T02:00:00Z\",\"hostname\":\"host\",\"paths\":[\"/var/log\"]}]"
        else
          echo "ID   Time  Host  Tags  Paths"
          echo "abc123 2026-07-15 host  -    /var/log"
        fi
        exit 0
        ;;
      stats)
        echo "{\"total_size\":4561234567,\"total_file_count\":1234}"
        exit 0
        ;;
      check)
        exit 0
        ;;
      restore)
        exit 0
        ;;
    esac
  '
}

@test "cmd_audit fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_audit
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_audit reports the sftp backend and repository location" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"백엔드: sftp"* ]]
  [[ "$output" == *"rclone:syno_backup:/backup/host"* ]]
}

@test "cmd_audit reports the s3 backend when no sftp config is present" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="s3:https://example.com/bucket"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
ENV
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"백엔드: s3"* ]]
}

@test "cmd_audit shows the retention policy numbers" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"일간 보관: 7개"* ]]
  [[ "$output" == *"주간 보관: 4개"* ]]
  [[ "$output" == *"월간 보관: 12개"* ]]
}

@test "cmd_audit warns when the repository password is not set" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export BACKUP_TARGETS="/var/log"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
ENV
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"경고: 비밀번호 미설정"* ]]
}

@test "cmd_audit does not warn when the repository password is set" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" != *"경고: 비밀번호 미설정"* ]]
}

@test "cmd_audit reports the timer's schedule and enabled/active state" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"반복 주기(OnCalendar): *-*-* 02:00:00"* ]]
  [[ "$output" == *"타이머 등록 상태: enabled"* ]]
  [[ "$output" == *"타이머 실행 상태: active"* ]]
  [[ "$output" == *"2026-07-16"* ]]
}

@test "cmd_audit includes the restic snapshots history section" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"백업 이력(restic snapshots)"* ]]
  [[ "$output" == *"abc123"* ]]
}

@test "cmd_audit reports file permissions for /etc/restic and backup.env" {
  run cmd_audit
  [ "$status" -eq 0 ]
  [[ "$output" == *"${RESTIC_ETC_DIR} 권한: 700"* ]]
  [[ "$output" == *"${BACKUP_ENV_FILE} 권한: 600"* ]]
}

@test "cmd_audit --report-file writes txt, json, and html files concurrently" {
  local r_file="${TEST_ROOT}/var/log/audit_report.txt"
  local j_file="${TEST_ROOT}/var/log/audit_report.json"
  local h_file="${TEST_ROOT}/var/log/audit_report.html"

  run cmd_audit --report-file "$r_file"
  [ "$status" -eq 0 ]
  [ -f "$r_file" ]
  [ -f "$j_file" ]
  [ -f "$h_file" ]

  # Check permissions (600)
  local h_perm; h_perm=$(stat -c "%a" "$h_file")
  [ "$h_perm" = "600" ]

  # Check text report content (Plain Text format, not TTY formatted)
  run cat "$r_file"
  [ "$status" -eq 0 ]
  [[ "$output" == *"백엔드: sftp"* ]]
  [[ "$output" == *"일간 보관: 7개"* ]]

  # Check JSON report content
  run cat "$j_file"
  [ "$status" -eq 0 ]
  [[ "$output" == *"\"backend\": \"sftp\""* ]]
  [[ "$output" == *"\"keep_daily\": 7"* ]]

  # Check HTML report content
  run cat "$h_file"
  [ "$status" -eq 0 ]
  [[ "$output" == *"<!DOCTYPE html>"* ]]
  [[ "$output" == *"종합 백업 보안 설정 검토 보고서"* ]]
}

@test "cmd_audit fails when both --daily and --restore-drill are passed" {
  run cmd_audit --daily --restore-drill
  [ "$status" -eq 1 ]
  [[ "$output" == *"--daily와 --restore-drill 옵션은 동시에 사용할 수 없습니다"* ]]
}

@test "cmd_audit --daily outputs a compliant daily review report" {
  run cmd_audit --daily
  [ "$status" -eq 0 ]
  [[ "$output" == *"[보안 감사 증적] 일일 백업 수행 결과 및 보안 설정 검토 보고서"* ]]
  [[ "$output" == *"1. 보존 정책 (Retention Rule) 검증"* ]]
  [[ "$output" == *"2. 접근 통제 및 무결성 검사"* ]]
  [[ "$output" == *"3. 최근 백업 성공 스냅샷 이력"* ]]
  [[ "$output" == *"4. ISMS-P 규정 준수 검증 체크리스트"* ]]
  [[ "$output" == *"설정 디렉터리"* ]]
}

@test "cmd_audit --restore-drill performs restore and outputs drill report" {
  run cmd_audit --restore-drill --tester "테스터" --ciso "보안책임자" --rto 60
  [ "$status" -eq 0 ]
  [[ "$output" == *"[보안 감사 증적] 백업 데이터 복구 및 정합성 테스트 결과 보고서"* ]]
  [[ "$output" == *"테스터: 테스터"* ]]
  [[ "$output" == *"승인자: 보안책임자"* ]]
  [[ "$output" == *"복구 소요 시간"* ]]
}

@test "cmd_schedule enable registers backup and audit reports timers" {
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"; exit 0'
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; exit 0'
  
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  [ -f "${SYSTEMD_UNIT_DIR}/backup-audit-daily.timer" ]
  [ -f "${SYSTEMD_UNIT_DIR}/backup-audit-restore-drill.timer" ]
  
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"enable --now backup-audit-daily.timer"* ]]
  [[ "$output" == *"enable --now backup-audit-restore-drill.timer"* ]]
}

@test "cmd_audit --daily --report writes date-stamped reports" {
  local date_suffix; date_suffix=$(date +%Y%m%d)
  local r_file="${TEST_ROOT}/data/backup/reports/daily_backup_audit_report_${date_suffix}.txt"
  local j_file="${TEST_ROOT}/data/backup/reports/daily_backup_audit_report_${date_suffix}.json"
  local h_file="${TEST_ROOT}/data/backup/reports/daily_backup_audit_report_${date_suffix}.html"
  
  run cmd_audit --daily --report --report-file "$r_file"
  [ "$status" -eq 0 ]
  [ -f "$r_file" ]
  [ -f "$j_file" ]
  [ -f "$h_file" ]
  
  # Check permissions (600)
  local h_perm; h_perm=$(stat -c "%a" "$h_file")
  [ "$h_perm" = "600" ]
  
  run cat "$r_file"
  [[ "$output" == *"[보안 감사 증적] 일일 백업 수행 결과"* ]]
  
  run cat "$j_file"
  [[ "$output" == *"daily_backup_review"* ]]
  
  run cat "$h_file"
  [[ "$output" == *"<!DOCTYPE html>"* ]]
  [[ "$output" == *"일일 백업 감사 결과"* ]]
}

@test "cmd_audit --restore-drill --report writes date-stamped reports" {
  local date_suffix; date_suffix=$(date +%Y%m%d)
  local r_file="${TEST_ROOT}/data/backup/reports/restore_drill_report_${date_suffix}.txt"
  local j_file="${TEST_ROOT}/data/backup/reports/restore_drill_report_${date_suffix}.json"
  local h_file="${TEST_ROOT}/data/backup/reports/restore_drill_report_${date_suffix}.html"
  
  run cmd_audit --restore-drill --report --report-file "$r_file"
  [ "$status" -eq 0 ]
  [ -f "$r_file" ]
  [ -f "$j_file" ]
  [ -f "$h_file" ]
  
  # Check permissions (600)
  local h_perm; h_perm=$(stat -c "%a" "$h_file")
  [ "$h_perm" = "600" ]
  
  run cat "$r_file"
  [[ "$output" == *"[보안 감사 증적] 백업 데이터 복구"* ]]
  
  run cat "$j_file"
  [[ "$output" == *"restore_drill"* ]]
  
  run cat "$h_file"
  [[ "$output" == *"<!DOCTYPE html>"* ]]
  [[ "$output" == *"백업 데이터 복구 및 정합성 테스트 결과 보고서"* ]]
}

@test "cmd_audit resolves tester, ciso, and rto from backup.env" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_AUDIT_TESTER="김철수 (시스템 담당자)"
export BACKUP_AUDIT_CISO="CISO 박영희"
export BACKUP_AUDIT_RTO="45"
ENV

  run cmd_audit --restore-drill
  [ "$status" -eq 0 ]
  [[ "$output" == *"테스터: 김철수 (시스템 담당자)"* ]]
  [[ "$output" == *"승인자: CISO 박영희"* ]]
  [[ "$output" == *"RTO 기준 45분"* ]]
}

@test "cmd_audit --restore-drill performs dual restore drill for primary and secondary if configured" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export SECONDARY_BACKEND="s3"
export SECONDARY_RESTIC_REPOSITORY="s3:https://sec-s3.com/sec-bucket/host"
export SECONDARY_RESTIC_PASSWORD="sec-secret"
export SECONDARY_AWS_ACCESS_KEY_ID="SEC_AK"
export SECONDARY_AWS_SECRET_ACCESS_KEY="SEC_SK"
ENV

  stub_command "restic" '
    echo "restic -r ${RESTIC_REPOSITORY} $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$1" in
      snapshots)
        echo "[{\"id\":\"abc123456789\",\"short_id\":\"abc12345\",\"time\":\"2026-07-15T02:00:00Z\",\"hostname\":\"host\",\"paths\":[\"/var/log\"]}]"
        exit 0
        ;;
      stats|check|restore)
        exit 0
        ;;
    esac
  '

  run cmd_audit --restore-drill
  [ "$status" -eq 0 ]
  
  run cat "${STUB_BIN}/restic.calls"
  # 1차 복구 훈련 기동 검증
  [[ "$output" == *"restic -r rclone:syno_backup:/backup/host restore"* ]]
  # 2차 복구 훈련 기동 검증
  [[ "$output" == *"restic -r s3:https://sec-s3.com/sec-bucket/host restore"* ]]
}

@test "render_report_markdown outputs valid markdown audit report" {
  # resolved/audit_report 연관 배열 구조체 모의
  declare -A test_report_data=(
    [test_date]="2026-07-14"
    [tester]="홍길동"
    [ciso]="이몽룡"
    [rto_minutes]=120
    [primary_snap]="a6bb678e"
    [primary_elapsed_seconds]=1
    [primary_rto_satisfied]="true"
    [secondary_snap]="b20f4685"
    [secondary_elapsed_seconds]=1
    [secondary_rto_satisfied]="true"
  )

  source "${BATS_TEST_DIRNAME}/../backup.sh"
  local output; output=$(render_report_markdown test_report_data)
  [ "$?" -eq 0 ]
  [[ "$output" == *"- 테스터: 홍길동"* ]]
  [[ "$output" == *"- 승인자: 이몽룡"* ]]
  [[ "$output" == *"2차 테스트 대상 스냅샷 ID: b20f4685"* ]]
}

@test "render_report_json outputs valid json audit report" {
  declare -A test_report_data=(
    [test_date]="2026-07-14"
    [tester]="홍길동"
    [ciso]="이몽룡"
    [rto_minutes]=120
    [primary_snap]="a6bb678e"
    [primary_elapsed_seconds]=1
    [primary_rto_satisfied]="true"
    [secondary_snap]="b20f4685"
    [secondary_elapsed_seconds]=1
    [secondary_rto_satisfied]="true"
  )

  source "${BATS_TEST_DIRNAME}/../backup.sh"
  local output; output=$(render_report_json test_report_data)
  [ "$?" -eq 0 ]
  [[ "$output" == *"\"tester\": \"홍길동\""* ]]
  [[ "$output" == *"\"ciso\": \"이몽룡\""* ]]
  [[ "$output" == *"\"secondary_recovery_results\""* ]]
}

@test "write_audit_reports writes md, json, and html files concurrently with correct permissions" {
  local base_report="${TEST_ROOT}/var/log/audit_test_out"
  rm -f "${base_report}"*
  
  declare -A test_report_data=(
    [test_date]="2026-07-14"
    [tester]="홍길동"
    [ciso]="이몽룡"
    [rto_minutes]=120
    [primary_snap]="a6bb678e"
    [primary_elapsed_seconds]=1
    [primary_rto_satisfied]="true"
    [secondary_snap]="b20f4685"
    [secondary_elapsed_seconds]=1
    [secondary_rto_satisfied]="true"
  )

  source "${BATS_TEST_DIRNAME}/../backup.sh"
  write_audit_reports test_report_data "${base_report}.md"
  [ "$?" -eq 0 ]
  
  [ -f "${base_report}.md" ]
  [ -f "${base_report}.json" ]
  [ -f "${base_report}.html" ]
  
  local h_perm; h_perm=$(stat -c "%a" "${base_report}.html")
  [ "$h_perm" = "600" ]
}

@test "cmd_audit --restore-drill supports DB restore and validation" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_DB_TYPE="mysql"
export BACKUP_DB_COMMAND="mysqldump --all"
export BACKUP_DB_FILENAME="db-dump.sql"
ENV

  stub_command "restic" '
    case "$1" in
      snapshots)
        if [[ "$*" == *"--tag db"* ]]; then
          echo "[{\"id\":\"db123456\",\"short_id\":\"db123456\",\"time\":\"2026-07-15T03:00:00Z\",\"hostname\":\"host\",\"paths\":[\"db-dump.sql\"],\"tags\":[\"db\"]}]"
        else
          echo "[{\"id\":\"abc12345\",\"short_id\":\"abc12345\",\"time\":\"2026-07-15T02:00:00Z\",\"hostname\":\"host\",\"paths\":[\"/var/log\"],\"tags\":[]}]"
        fi
        exit 0
        ;;
      restore)
        if [[ "$*" == *"/tmp/restore_test_db"* ]]; then
          mkdir -p /tmp/restore_test_db
          echo "-- MySQL dump 10.13  Distrib 8.0.28" > /tmp/restore_test_db/db-dump.sql
          echo "CREATE DATABASE test;" >> /tmp/restore_test_db/db-dump.sql
        else
          mkdir -p /tmp/restore_test
          echo "dummy file" > /tmp/restore_test/dummy.txt
        fi
        exit 0
        ;;
    esac
  '

  run cmd_audit --restore-drill
  [ "$status" -eq 0 ]
  [[ "$output" == *"데이터베이스(mysql) 복원 무결성 검증: 성공"* ]]
}

@test "render_audit_report_unified generates general JSON and daily text reports correctly" {
  declare -A general_data=(
    [backend]="sftp"
    [on_calendar]="*-*-* 02:00:00"
    [timer_enabled]="enabled"
    [timer_active]="active"
    [next_run]="Thu 2026-07-16 02:00:00 KST"
    [etc_perm]="700"
    [env_perm]="600"
  )

  run render_audit_report_unified "general" "json" general_data
  [ "$status" -eq 0 ]
  [[ "$output" == *"\"etc_restic_dir_permission\": \"700\""* ]]

  declare -A daily_data=(
    [cur_time]="2026-07-16 17:00:00 KST"
    [hostname]="test-host"
    [tester]="test-tester"
    [backend]="s3"
    [repo]="s3:https://s3.amazonaws.com/test-bucket"
    [targets]="/etc"
    [config_daily]="7"
    [actual_daily]="10"
    [config_daily_status]="만족"
    [actual_daily_status]="만족"
    [config_weekly]="4"
    [actual_weekly]="5"
    [config_weekly_status]="만족"
    [actual_weekly_status]="만족"
    [config_monthly]="12"
    [actual_monthly]="15"
    [config_monthly_status]="만족"
    [actual_monthly_status]="만족"
    [etc_dir]="/etc/backup"
    [etc_perm]="700"
    [etc_safe_str]="안전"
    [env_file]="/etc/backup/backup.env"
    [env_perm]="600"
    [env_safe_str]="안전"
    [check_status]="SUCCESS"
    [snapshot_table]="ID Time Host Paths"
  )

  run render_audit_report_unified "daily" "txt" daily_data
  [ "$status" -eq 0 ]
  [[ "$output" == *"[보안 감사 증적] 일일 백업 수행 결과"* ]]
  [[ "$output" == *"- 대상 서버 호스트: test-host"* ]]
}









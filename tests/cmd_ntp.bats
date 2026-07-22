#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  export NTP_CONF_PATH="${TEST_ROOT}/etc/chrony.conf"
  mkdir -p "$(dirname "$NTP_CONF_PATH")"

  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/web01"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_EXCLUDES=""
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="web01"
ENV

  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; exit 0'
  stub_command "journalctl" 'echo "journalctl $*"'
  stub_command "chronyc"    'echo "chronyc $*"'
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"'
}

# ─────────────────────────────────────────────────────────────────────────────
# ntp setup
# ─────────────────────────────────────────────────────────────────────────────

@test "cmd_ntp setup writes chrony.conf with KRNIC server" {
  run cmd_ntp setup
  [ "$status" -eq 0 ]
  [ -f "$NTP_CONF_PATH" ]
  grep -q "time.krnic.net" "$NTP_CONF_PATH"
}

@test "cmd_ntp setup backs up existing chrony.conf before overwriting" {
  echo "old-config" > "$NTP_CONF_PATH"
  run cmd_ntp setup
  [ "$status" -eq 0 ]
  local bak
  bak=$(ls "$(dirname "$NTP_CONF_PATH")"/chrony.conf.bak.* 2>/dev/null | head -1)
  [ -n "$bak" ]
  grep -q "old-config" "$bak"
}

@test "cmd_ntp setup sets BACKUP_NTP_REPORT=1 in backup.env" {
  run cmd_ntp setup
  [ "$status" -eq 0 ]
  grep -q "BACKUP_NTP_REPORT='1'" "$BACKUP_ENV_FILE"
}

@test "cmd_ntp setup calls systemctl restart chronyd" {
  run cmd_ntp setup
  [ "$status" -eq 0 ]
  grep -q "restart chronyd" "${STUB_BIN}/systemctl.calls"
}

@test "cmd_ntp setup calls journalctl for log check" {
  run cmd_ntp setup
  [ "$status" -eq 0 ]
  [[ "$output" == *"journalctl"* ]]
}

@test "cmd_ntp setup calls chronyc sources -v" {
  run cmd_ntp setup
  [ "$status" -eq 0 ]
  [[ "$output" == *"chronyc"* ]]
}

# ─────────────────────────────────────────────────────────────────────────────
# ntp --report
# ─────────────────────────────────────────────────────────────────────────────

@test "cmd_ntp --report creates txt evidence file" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NTP_REPORT='1'
ENV
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_ntp --report
  [ "$status" -eq 0 ]
  local date_suffix; date_suffix=$(date +%Y%m%d)
  [ -f "${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.txt" ]
}

@test "cmd_ntp --report creates json evidence file" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NTP_REPORT='1'
ENV
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_ntp --report
  [ "$status" -eq 0 ]
  local date_suffix; date_suffix=$(date +%Y%m%d)
  [ -f "${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.json" ]
}

@test "cmd_ntp --report creates html evidence file" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NTP_REPORT='1'
ENV
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_ntp --report
  [ "$status" -eq 0 ]
  local date_suffix; date_suffix=$(date +%Y%m%d)
  [ -f "${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.html" ]
}

@test "cmd_ntp --report accepts custom --report-file path" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NTP_REPORT='1'
ENV
  local out_file="${TEST_ROOT}/custom_ntp.txt"
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_ntp --report --report-file "$out_file"
  [ "$status" -eq 0 ]
  [ -f "$out_file" ]
}

# ─────────────────────────────────────────────────────────────────────────────
# schedule enable/disable ntp 연동
# ─────────────────────────────────────────────────────────────────────────────

@test "schedule enable registers ntp timer when BACKUP_NTP_REPORT=1" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NTP_REPORT='1'
ENV
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  grep -q "backup-ntp-report" "${STUB_BIN}/systemctl.calls"
}

@test "schedule enable does NOT register ntp timer when BACKUP_NTP_REPORT unset" {
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  run grep "backup-ntp-report" "${STUB_BIN}/systemctl.calls"
  [ "$status" -ne 0 ]
}

@test "schedule disable removes ntp timer when BACKUP_NTP_REPORT=1" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NTP_REPORT='1'
ENV
  cmd_schedule enable
  > "${STUB_BIN}/systemctl.calls"
  run cmd_schedule disable
  [ "$status" -eq 0 ]
  grep -q "backup-ntp-report" "${STUB_BIN}/systemctl.calls"
}

# ─────────────────────────────────────────────────────────────────────────────
# legacy chrony command logs warning and delegates to ntp
# ─────────────────────────────────────────────────────────────────────────────

@test "legacy chrony command setup logs warning and delegates to ntp" {
  run main chrony setup
  [ "$status" -eq 0 ]
  [[ "$output" == *"더 이상 사용되지 않습니다"* ]]
  [ -f "$NTP_CONF_PATH" ]
}

# ─────────────────────────────────────────────────────────────────────────────
# render_ntp_txt (pure function)
# ─────────────────────────────────────────────────────────────────────────────

@test "render_ntp_txt contains ISMS-P 2.9.3 header" {
  # shellcheck disable=SC2034
  local -A data=(
    [hostname]="test-host"
    [report_date]="2026-07-21 00:30:00 KST"
    [service_enabled]="enabled"
    [service_active]="active (running)"
    [sources_output]="^* time.krnic.net"
    [tracking_output]="System time : 0.000123 seconds"
    [conf_perm]="-rw-r--r-- 1 root root"
  )
  run render_ntp_txt data
  [ "$status" -eq 0 ]
  [[ "$output" == *"ISMS-P 2.9.3"* ]]
  [[ "$output" == *"test-host"* ]]
}

@test "cmd_ntp setup under ntpd environment writes ntp.conf and restarts ntpd" {
  export MOCK_NTP_SERVICE="ntpd"
  # Remove chronyc stub from STUB_BIN to avoid chrony auto-detection
  rm -f "${STUB_BIN}/chronyc"
  
  # Stub ntpq
  stub_command "ntpq" 'echo "ntpq $*"'
  
  # Stub systemctl to list ntpd.service and log restart calls
  stub_command "systemctl" '
    if [[ "$*" == *"list-unit-files"* ]]; then
      echo "ntpd.service enabled"
    elif [[ "$*" == *"restart ntpd"* ]]; then
      echo "systemctl restart ntpd" >> "'"${STUB_BIN}"'/systemctl.calls"
    elif [[ "$*" == *"is-enabled ntpd"* || "$*" == *"is-active ntpd"* ]]; then
      echo "active"
    fi
    exit 0
  '

  run cmd_ntp setup
  [ "$status" -eq 0 ]

  # NTP_CONF_PATH should be resolved to TEST_ROOT/etc/ntp.conf
  local expected_ntp_conf="${TEST_ROOT}/etc/ntp.conf"
  [ -f "$expected_ntp_conf" ]
  grep -q "time.krnic.net" "$expected_ntp_conf"

  # Check that restart ntpd was called
  grep -q "restart ntpd" "${STUB_BIN}/systemctl.calls"
}

@test "cmd_ntp --report under ntpd environment uses ntpq and renders ntpq titles" {
  export MOCK_NTP_SERVICE="ntpd"
  rm -f "${STUB_BIN}/chronyc"
  stub_command "ntpq" '
    if [[ "$*" == "-p" ]]; then
      echo "remote refid"
      echo "*time.krnic.net .GPS."
    elif [[ "$*" == "-c rv" ]]; then
      echo "associd=0 status=0615 source_ip=... stratum=2 leap_none"
    fi
  '
  stub_command "systemctl" '
    if [[ "$*" == *"is-enabled"* ]]; then
      echo "enabled"
    elif [[ "$*" == *"is-active"* ]]; then
      echo "active"
    fi
    exit 0
  '

  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_NTP_REPORT='1'
ENV
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_ntp --report
  [ "$status" -eq 0 ]

  local date_suffix; date_suffix=$(date +%Y%m%d)
  local report_file="${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.txt"
  [ -f "$report_file" ]

  run grep -F "ntpq -p" "$report_file"
  [ "$status" -eq 0 ]
  run grep -F "ntpq -c rv" "$report_file"
  [ "$status" -eq 0 ]
  run grep -F "*time.krnic.net" "$report_file"
  [ "$status" -eq 0 ]
}


#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  export CHRONY_CONF_PATH="${TEST_ROOT}/etc/chrony.conf"
  mkdir -p "$(dirname "$CHRONY_CONF_PATH")"

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
# chrony setup
# ─────────────────────────────────────────────────────────────────────────────

@test "cmd_chrony setup writes chrony.conf with KRNIC server" {
  run cmd_chrony setup
  [ "$status" -eq 0 ]
  [ -f "$CHRONY_CONF_PATH" ]
  grep -q "time.krnic.net" "$CHRONY_CONF_PATH"
}

@test "cmd_chrony setup backs up existing chrony.conf before overwriting" {
  echo "old-config" > "$CHRONY_CONF_PATH"
  run cmd_chrony setup
  [ "$status" -eq 0 ]
  local bak
  bak=$(ls "$(dirname "$CHRONY_CONF_PATH")"/chrony.conf.bak.* 2>/dev/null | head -1)
  [ -n "$bak" ]
  grep -q "old-config" "$bak"
}

@test "cmd_chrony setup sets BACKUP_CHRONY_REPORT=1 in backup.env" {
  run cmd_chrony setup
  [ "$status" -eq 0 ]
  grep -q "BACKUP_CHRONY_REPORT='1'" "$BACKUP_ENV_FILE"
}

@test "cmd_chrony setup calls systemctl restart chronyd" {
  run cmd_chrony setup
  [ "$status" -eq 0 ]
  grep -q "restart chronyd" "${STUB_BIN}/systemctl.calls"
}

@test "cmd_chrony setup calls journalctl for log check" {
  run cmd_chrony setup
  [ "$status" -eq 0 ]
  [[ "$output" == *"journalctl"* ]]
}

@test "cmd_chrony setup calls chronyc sources -v" {
  run cmd_chrony setup
  [ "$status" -eq 0 ]
  [[ "$output" == *"chronyc"* ]]
}

# ─────────────────────────────────────────────────────────────────────────────
# chrony --report
# ─────────────────────────────────────────────────────────────────────────────

@test "cmd_chrony --report creates txt evidence file" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_CHRONY_REPORT='1'
ENV
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_chrony --report
  [ "$status" -eq 0 ]
  local date_suffix; date_suffix=$(date +%Y%m%d)
  [ -f "${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.txt" ]
}

@test "cmd_chrony --report creates json evidence file" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_CHRONY_REPORT='1'
ENV
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_chrony --report
  [ "$status" -eq 0 ]
  local date_suffix; date_suffix=$(date +%Y%m%d)
  [ -f "${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.json" ]
}

@test "cmd_chrony --report creates html evidence file" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_CHRONY_REPORT='1'
ENV
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_chrony --report
  [ "$status" -eq 0 ]
  local date_suffix; date_suffix=$(date +%Y%m%d)
  [ -f "${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.html" ]
}

@test "cmd_chrony --report accepts custom --report-file path" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_CHRONY_REPORT='1'
ENV
  local out_file="${TEST_ROOT}/custom_ntp.txt"
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  run cmd_chrony --report --report-file "$out_file"
  [ "$status" -eq 0 ]
  [ -f "$out_file" ]
}

# ─────────────────────────────────────────────────────────────────────────────
# schedule enable/disable chrony 연동
# ─────────────────────────────────────────────────────────────────────────────

@test "schedule enable registers chrony timer when BACKUP_CHRONY_REPORT=1" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_CHRONY_REPORT='1'
ENV
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  grep -q "backup-chrony-report" "${STUB_BIN}/systemctl.calls"
}

@test "schedule enable does NOT register chrony timer when BACKUP_CHRONY_REPORT unset" {
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  run grep "backup-chrony-report" "${STUB_BIN}/systemctl.calls"
  [ "$status" -ne 0 ]
}

@test "schedule disable removes chrony timer when BACKUP_CHRONY_REPORT=1" {
  cat >> "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_CHRONY_REPORT='1'
ENV
  cmd_schedule enable
  > "${STUB_BIN}/systemctl.calls"
  run cmd_schedule disable
  [ "$status" -eq 0 ]
  grep -q "backup-chrony-report" "${STUB_BIN}/systemctl.calls"
}

# ─────────────────────────────────────────────────────────────────────────────
# render_chrony_txt (pure function)
# ─────────────────────────────────────────────────────────────────────────────

@test "render_chrony_txt contains ISMS-P 2.9.3 header" {
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
  run render_chrony_txt data
  [ "$status" -eq 0 ]
  [[ "$output" == *"ISMS-P 2.9.3"* ]]
  [[ "$output" == *"test-host"* ]]
}

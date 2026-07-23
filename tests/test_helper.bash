#!/usr/bin/env bash

setup_backup_sh_env() {
  export TEST_ROOT="${BATS_TEST_TMPDIR}/root"
  mkdir -p "$TEST_ROOT"

  export RESTIC_ETC_DIR="${TEST_ROOT}/etc/restic"
  export BACKUP_ETC_DIR="${TEST_ROOT}/etc/backup"
  export BACKUP_ENV_FILE="${BACKUP_ETC_DIR}/backup.env"
  export BACKUP_SSH_KEY="${BACKUP_ETC_DIR}/backup_key"
  export BACKUP_SCRIPT_INSTALL_PATH="${TEST_ROOT}/usr/local/sbin/backup.sh"
  export RESTIC_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/restic"
  export RCLONE_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/rclone"
  export RESTICPROFILE_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/resticprofile"
  export SYSTEMD_UNIT_DIR="${TEST_ROOT}/etc/systemd/system"
  export CHRONY_CONF_PATH="${TEST_ROOT}/etc/chrony.conf"
  export NTP_CONF_PATH="${TEST_ROOT}/etc/chrony.conf"
  export BACKUP_REPORTS_DIR="${TEST_ROOT}/data/backup/reports"
  mkdir -p "$SYSTEMD_UNIT_DIR" "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")" "$(dirname "$RESTIC_INSTALL_PATH")" "$(dirname "$NTP_CONF_PATH")"

  export STUB_BIN="${BATS_TEST_TMPDIR}/stub-bin"
  mkdir -p "$STUB_BIN"
  export PATH="${STUB_BIN}:${PATH}"

  export REQUIRE_ROOT_CHECK=0
  export GUM_DISABLE=1

  # shellcheck source=/dev/null
  source "${BATS_TEST_DIRNAME}/../backup.sh"
}

stub_command() {
  local name="$1" body="$2"
  cat > "${STUB_BIN}/${name}" <<STUB
#!/usr/bin/env bash
${body}
STUB
  chmod +x "${STUB_BIN}/${name}"
}

stub_call_log() {
  printf '%s\n' "${STUB_BIN}/${1}.calls"
}

#!/usr/bin/env bash
set -euo pipefail

RESTIC_ETC_DIR="${RESTIC_ETC_DIR:-/etc/restic}"
BACKUP_ENV_FILE="${BACKUP_ENV_FILE:-${RESTIC_ETC_DIR}/backup.env}"
BACKUP_SSH_KEY="${BACKUP_SSH_KEY:-${RESTIC_ETC_DIR}/backup_key}"
BACKUP_SCRIPT_INSTALL_PATH="${BACKUP_SCRIPT_INSTALL_PATH:-/usr/local/sbin/backup.sh}"
SYSTEMD_UNIT_DIR="${SYSTEMD_UNIT_DIR:-/etc/systemd/system}"
SYSTEMD_SERVICE_FILE="${SYSTEMD_UNIT_DIR}/restic-backup.service"
SYSTEMD_TIMER_FILE="${SYSTEMD_UNIT_DIR}/restic-backup.timer"

DEFAULT_TARGETS="/var/log"
DEFAULT_EXCLUDES="/tmp/*,/var/tmp/*"
DEFAULT_KEEP_DAILY=7
DEFAULT_KEEP_WEEKLY=4
DEFAULT_KEEP_MONTHLY=12
DEFAULT_ON_CALENDAR="*-*-* 02:00:00"
DEFAULT_SFTP_PORT=22

log_info() {
  printf '%s\n' "$1"
  command -v logger >/dev/null 2>&1 && logger -t restic-backup -- "$1" || true
}

log_error() {
  printf 'ERROR: %s\n' "$1" >&2
  command -v logger >/dev/null 2>&1 && logger -t restic-backup -- "ERROR: $1" || true
}

die() {
  log_error "$1"
  exit "${2:-1}"
}

require_root() {
  if [[ "${REQUIRE_ROOT_CHECK:-1}" == "1" && "${EUID}" -ne 0 ]]; then
    die "이 명령은 root 권한으로 실행해야 합니다. sudo로 다시 실행하세요." 1
  fi
}

resolve_value() {
  local cli="$1" env="$2" file="$3" default="$4"
  if [[ -n "$cli" ]]; then printf '%s' "$cli"; return 0; fi
  if [[ -n "$env" ]]; then printf '%s' "$env"; return 0; fi
  if [[ -n "$file" ]]; then printf '%s' "$file"; return 0; fi
  if [[ -n "$default" ]]; then printf '%s' "$default"; return 0; fi
  return 1
}

validate_backend() {
  local value="$1"
  case "$value" in
    s3|sftp) return 0 ;;
    *) printf 'ERROR: backend must be s3 or sftp, got: %s\n' "$value"; return 1 ;;
  esac
}

validate_port() {
  local value="$1"
  if ! [[ "$value" =~ ^[0-9]+$ ]]; then
    printf 'ERROR: port must be numeric, got: %s\n' "$value"
    return 1
  fi
  if (( 10#$value < 1 || 10#$value > 65535 )); then
    printf 'ERROR: port must be between 1 and 65535, got: %s\n' "$value"
    return 1
  fi
  return 0
}

validate_positive_int() {
  local value="$1" label="$2"
  if ! [[ "$value" =~ ^[0-9]+$ ]]; then
    printf 'ERROR: %s must be numeric, got: %s\n' "$label" "$value"
    return 1
  fi
  if (( 10#$value < 1 )); then
    printf 'ERROR: %s must be positive, got: %s\n' "$label" "$value"
    return 1
  fi
  return 0
}

render_help() {
  cat <<'EOF'
backup.sh - restic 기반 백업 설치/운영 스크립트

사용법:
  backup.sh install [--force] [--dry-run]
  backup.sh setting --backend <s3|sftp> [옵션...] [--force] [--dry-run]
  backup.sh init
  backup.sh schedule enable [--on-calendar "<OnCalendar식>"]
  backup.sh schedule disable
  backup.sh run
  backup.sh status
  backup.sh uninstall [--purge]
  backup.sh wizard
  backup.sh -h | --help
EOF
}

main() {
  if [[ $# -eq 0 ]]; then
    render_help
    return 0
  fi

  case "$1" in
    -h|--help)
      render_help
      return 0
      ;;
    install|setting|init|schedule|run|status|uninstall|wizard)
      : # 다음 태스크에서 각 cmd_* 로 분기
      ;;
    *)
      render_help
      return 1
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi

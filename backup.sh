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

parse_long_opts() {
  local spec="$1"
  shift
  if [[ "${1:-}" != "--" ]]; then
    printf 'parse_long_opts: 내부 오류, "--" 구분자가 필요합니다\n'
    return 1
  fi
  shift

  local -A takes_value=()
  local name
  for name in $spec; do
    if [[ "$name" == *: ]]; then
      takes_value["${name%:}"]=1
    else
      takes_value["$name"]=0
    fi
  done

  while [[ $# -gt 0 ]]; do
    local arg="$1"
    case "$arg" in
      --*=*)
        local key="${arg%%=*}"
        key="${key#--}"
        local val="${arg#*=}"
        if [[ -z "${takes_value[$key]+x}" ]]; then
          printf '알 수 없는 플래그: --%s\n' "$key"
          return 1
        fi
        printf '%s\t%s\n' "$key" "$val"
        shift
        ;;
      --*)
        local key="${arg#--}"
        if [[ -z "${takes_value[$key]+x}" ]]; then
          printf '알 수 없는 플래그: --%s\n' "$key"
          return 1
        fi
        if [[ "${takes_value[$key]}" == 1 ]]; then
          if [[ $# -lt 2 ]]; then
            printf -- '--%s 플래그는 값이 필요합니다\n' "$key"
            return 1
          fi
          printf '%s\t%s\n' "$key" "$2"
          shift 2
        else
          printf '%s\t1\n' "$key"
          shift
        fi
        ;;
      *)
        printf '예상치 못한 인자: %s\n' "$arg"
        return 1
        ;;
    esac
  done
  return 0
}

render_placeholder_or_value() {
  local value="$1" placeholder="$2"
  if [[ -n "$value" ]]; then
    printf '%s' "$value"
  else
    printf '<%s>' "$placeholder"
  fi
}

render_setting_hint_sftp() {
  local host="$1" port="$2" user="$3"
  printf "backup.sh setting --backend sftp --host %s --port %s --user %s --password '<REPO_PASSWORD>'\\n" \
    "$(render_placeholder_or_value "$host" "NAS_IP")" \
    "$(render_placeholder_or_value "$port" "PORT")" \
    "$(render_placeholder_or_value "$user" "NAS_USER")"
}

render_setting_hint_s3() {
  local endpoint="$1" bucket="$2"
  printf "backup.sh setting --backend s3 --endpoint %s --bucket %s --access-key <ACCESS_KEY> --secret-key '<SECRET_KEY>' --password '<REPO_PASSWORD>'\\n" \
    "$(render_placeholder_or_value "$endpoint" "S3_ENDPOINT")" \
    "$(render_placeholder_or_value "$bucket" "BUCKET_NAME")"
}

render_missing_settings_message() {
  cat <<'EOF'
[!] 설정이 없습니다. 먼저 아래 중 하나로 설정을 완료하세요:

    backup.sh setting --backend sftp --host <NAS_IP> --port <PORT> --user <NAS_USER> --password '<REPO_PASSWORD>'
    backup.sh setting --backend s3 --endpoint <S3_ENDPOINT> --bucket <BUCKET_NAME> --access-key <ACCESS_KEY> --secret-key '<SECRET_KEY>' --password '<REPO_PASSWORD>'

자세한 옵션은 'backup.sh --help'를 참고하세요.
EOF
}

render_service_unit() {
  cat <<EOF
[Unit]
Description=Restic System Backup Service (ISMS Compliance)
After=network-online.target

[Service]
Type=oneshot
ExecStart=${BACKUP_SCRIPT_INSTALL_PATH} run
User=root
Group=root
Restart=no
EOF
}

render_timer_unit() {
  local on_calendar="$1"
  cat <<EOF
[Unit]
Description=Run Restic Backup on schedule

[Timer]
OnCalendar=${on_calendar}
Persistent=true

[Install]
WantedBy=timers.target
EOF
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

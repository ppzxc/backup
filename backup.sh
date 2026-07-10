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

dnf_install_packages() {
  dnf install -y epel-release restic rclone
}

self_install_copy() {
  local source_path="$1" force="$2"
  if [[ -e "$BACKUP_SCRIPT_INSTALL_PATH" && "$force" != 1 ]]; then
    log_info "이미 설치되어 있습니다: ${BACKUP_SCRIPT_INSTALL_PATH} (덮어쓰려면 install --force)"
    return 0
  fi
  install -m 0755 "$source_path" "$BACKUP_SCRIPT_INSTALL_PATH"
}

ensure_restic_dir() {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
}

write_secure_file() {
  local path="$1" mode="$2" content="$3"
  mkdir -p "$(dirname "$path")"
  printf '%s\n' "$content" > "$path"
  chmod "$mode" "$path"
}

generate_ssh_key_if_missing() {
  if [[ ! -f "$BACKUP_SSH_KEY" ]]; then
    ssh-keygen -t ed25519 -f "$BACKUP_SSH_KEY" -N ""
  fi
  chmod 600 "$BACKUP_SSH_KEY"
  chmod 644 "${BACKUP_SSH_KEY}.pub"
}

cmd_install() {
  require_root
  local parsed
  parsed=$(parse_long_opts "force dry-run" -- "$@") || die "$parsed"

  local force=0 dry_run=0
  local key val
  while IFS=$'\t' read -r key val; do
    case "$key" in
      force) force=1 ;;
      dry-run) dry_run=1 ;;
    esac
  done <<< "$parsed"

  if (( dry_run )); then
    cat <<EOF
[dry-run] dnf install -y epel-release restic rclone
[dry-run] install -m 0755 "\$0" "${BACKUP_SCRIPT_INSTALL_PATH}"
[dry-run] mkdir -p "${RESTIC_ETC_DIR}" && chmod 700 "${RESTIC_ETC_DIR}"
EOF
    return 0
  fi

  dnf_install_packages
  self_install_copy "$0" "$force"
  ensure_restic_dir
  log_info "install 완료"
}

render_backup_env_sftp() {
  local hostname_tag="$1" host="$2" port="$3" user="$4" ssh_key_path="$5" \
        password="$6" targets="$7" excludes_csv="$8" \
        keep_daily="$9" keep_weekly="${10}" keep_monthly="${11}"
  cat <<EOF
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/${hostname_tag}"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="${host}"
export RCLONE_CONFIG_SYNO_BACKUP_USER="${user}"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="${port}"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${ssh_key_path}"
export RESTIC_PASSWORD="${password}"
export BACKUP_TARGETS="${targets}"
export BACKUP_EXCLUDES="${excludes_csv}"
export KEEP_DAILY="${keep_daily}"
export KEEP_WEEKLY="${keep_weekly}"
export KEEP_MONTHLY="${keep_monthly}"
EOF
}

render_sftp_registration_notice() {
  local pubkey_content="$1"
  cat <<EOF
아래 공개키를 NAS의 authorized_keys(또는 File Station)에 등록하세요:
----------------------------------------------------------
${pubkey_content}
----------------------------------------------------------
등록 후 'backup.sh init'을 실행하세요.
EOF
}

render_backup_env_s3() {
  local hostname_tag="$1" endpoint="$2" bucket="$3" access_key="$4" secret_key="$5" \
        password="$6" targets="$7" excludes_csv="$8" \
        keep_daily="$9" keep_weekly="${10}" keep_monthly="${11}"
  cat <<EOF
export RESTIC_REPOSITORY="s3:${endpoint}/${bucket}/${hostname_tag}"
export AWS_ACCESS_KEY_ID="${access_key}"
export AWS_SECRET_ACCESS_KEY="${secret_key}"
export RESTIC_PASSWORD="${password}"
export BACKUP_TARGETS="${targets}"
export BACKUP_EXCLUDES="${excludes_csv}"
export KEEP_DAILY="${keep_daily}"
export KEEP_WEEKLY="${keep_weekly}"
export KEEP_MONTHLY="${keep_monthly}"
EOF
}

render_s3_bucket_policy() {
  local bucket="$1"
  cat <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["s3:ListBucket", "s3:GetObject", "s3:PutObject", "s3:DeleteObject"],
      "Resource": ["arn:aws:s3:::${bucket}", "arn:aws:s3:::${bucket}/*"]
    }
  ]
}
EOF
}

restic_is_initialized() {
  restic snapshots >/dev/null 2>&1
}

cmd_init() {
  require_root
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi

  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"

  if restic_is_initialized; then
    log_info "이미 초기화된 저장소입니다. 스킵합니다."
    return 0
  fi

  restic init
  log_info "restic init 완료"
}

systemd_enable_timer() {
  systemctl daemon-reload
  systemctl enable --now restic-backup.timer
}

systemd_disable_timer() {
  systemctl disable --now restic-backup.timer 2>/dev/null || true
}

cmd_schedule() {
  require_root
  local action="${1:-}"
  shift || true

  case "$action" in
    enable)
      local parsed
      parsed=$(parse_long_opts "on-calendar:" -- "$@") || die "$parsed"
      local on_calendar="$DEFAULT_ON_CALENDAR"
      local key val
      while IFS=$'\t' read -r key val; do
        case "$key" in
          on-calendar) on_calendar="$val" ;;
        esac
      done <<< "$parsed"

      write_secure_file "$SYSTEMD_SERVICE_FILE" 644 "$(render_service_unit)"
      write_secure_file "$SYSTEMD_TIMER_FILE" 644 "$(render_timer_unit "$on_calendar")"
      systemd_enable_timer
      log_info "schedule enable 완료 (${on_calendar})"
      ;;
    disable)
      systemd_disable_timer
      log_info "schedule disable 완료"
      ;;
    *)
      die "schedule은 'enable' 또는 'disable'만 지원합니다 (입력값: '${action}')"
      ;;
  esac
}

cmd_run() {
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi

  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"

  restic unlock --stale >/dev/null 2>&1 || true

  # IFS=',' prefixed directly on the `read` command scopes the field separator
  # to that single command only (no global IFS mutation, no `local IFS`
  # gymnastics needed). Populating via `read -ra` into an array, then iterating
  # with a quoted "${arr[@]}", also avoids unquoted-expansion pathname
  # (glob) expansion.
  local -a exclude_flags=()
  local -a excludes_arr=()
  IFS=',' read -ra excludes_arr <<< "${BACKUP_EXCLUDES:-}"
  local ex
  for ex in "${excludes_arr[@]}"; do
    exclude_flags+=("--exclude=${ex}")
  done

  local -a targets=()
  IFS=',' read -ra targets <<< "${BACKUP_TARGETS:-}"

  if restic backup "${targets[@]}" "${exclude_flags[@]}"; then
    log_info "백업 성공"
  else
    die "restic backup 실패"
  fi

  restic forget --keep-daily "${KEEP_DAILY}" --keep-weekly "${KEEP_WEEKLY}" --keep-monthly "${KEEP_MONTHLY}" --prune
  log_info "만료 스냅샷 정리 완료"
}

cmd_status() {
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi

  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"

  printf '저장소 위치: %s\n' "${RESTIC_REPOSITORY:-알 수 없음}"
  printf '백업 대상: %s\n' "${BACKUP_TARGETS:-알 수 없음}"

  printf '최근 스냅샷:\n'
  restic snapshots --json 2>/dev/null || printf '(조회 실패 또는 미초기화)\n'

  local timer_state
  timer_state=$(systemctl is-active restic-backup.timer 2>/dev/null) || true
  printf '타이머 상태: %s\n' "${timer_state:-unknown}"

  printf '%s 권한: %s\n' "$RESTIC_ETC_DIR" "$(stat -c '%a' "$RESTIC_ETC_DIR" 2>/dev/null || echo '?')"
  printf '%s 권한: %s\n' "$BACKUP_ENV_FILE" "$(stat -c '%a' "$BACKUP_ENV_FILE" 2>/dev/null || echo '?')"
}

cmd_uninstall() {
  require_root
  local parsed
  parsed=$(parse_long_opts "purge" -- "$@") || die "$parsed"

  local purge=0
  local key val
  while IFS=$'\t' read -r key val; do
    case "$key" in
      purge) purge=1 ;;
    esac
  done <<< "$parsed"

  systemd_disable_timer
  rm -f "$SYSTEMD_SERVICE_FILE" "$SYSTEMD_TIMER_FILE"

  if (( purge )); then
    rm -rf "$RESTIC_ETC_DIR"
    log_info "uninstall --purge 완료 (${RESTIC_ETC_DIR} 삭제됨)"
  else
    log_info "uninstall 완료 (${RESTIC_ETC_DIR}는 유지됨)"
  fi
}

cmd_setting() {
  require_root
  local parsed
  parsed=$(parse_long_opts "backend: targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: force dry-run" -- "$@") || die "$parsed"

  local backend="" targets="" password="" keep_daily="" keep_weekly="" keep_monthly=""
  local endpoint="" bucket="" access_key="" secret_key="" host="" port="" user=""
  local force=0 dry_run=0
  local -a excludes=()

  local key val
  while IFS=$'\t' read -r key val; do
    case "$key" in
      backend) backend="$val" ;;
      targets) targets="$val" ;;
      exclude) excludes+=("$val") ;;
      password) password="$val" ;;
      keep-daily) keep_daily="$val" ;;
      keep-weekly) keep_weekly="$val" ;;
      keep-monthly) keep_monthly="$val" ;;
      endpoint) endpoint="$val" ;;
      bucket) bucket="$val" ;;
      access-key) access_key="$val" ;;
      secret-key) secret_key="$val" ;;
      host) host="$val" ;;
      port) port="$val" ;;
      user) user="$val" ;;
      force) force=1 ;;
      dry-run) dry_run=1 ;;
    esac
  done <<< "$parsed"

  # 실제 사용자가 export한 환경변수는 backup.env를 source하기 전에 미리 캡처해둔다.
  # (source 이후에는 같은 변수명이 파일 값으로 덮어써지므로, 미리 캡처하지 않으면
  #  "환경변수 값"과 "기존 backup.env 값"을 구분할 수 없다.)
  local env_targets="${BACKUP_TARGETS:-}"
  local env_keep_daily="${KEEP_DAILY:-}"
  local env_keep_weekly="${KEEP_WEEKLY:-}"
  local env_keep_monthly="${KEEP_MONTHLY:-}"
  local env_password="${BACKUP_PASSWORD:-}"
  local env_host="${BACKUP_HOST:-}"
  local env_port="${BACKUP_PORT:-}"
  local env_user="${BACKUP_USER:-}"
  local env_endpoint="${BACKUP_ENDPOINT:-}"
  local env_bucket="${BACKUP_BUCKET:-}"
  local env_access_key="${BACKUP_ACCESS_KEY:-}"
  local env_secret_key="${BACKUP_SECRET_KEY:-}"

  if [[ -z "$backend" ]]; then
    die "$(render_missing_settings_message)"
  fi
  local err
  if ! err=$(validate_backend "$backend"); then die "$err"; fi

  if [[ -f "$BACKUP_ENV_FILE" && "$force" != 1 ]]; then
    die "이미 설정이 있습니다: ${BACKUP_ENV_FILE} (덮어쓰려면 setting --force)"
  fi

  local file_targets="" file_keep_daily="" file_keep_weekly="" file_keep_monthly="" file_excludes=""
  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    # shellcheck source=/dev/null
    source "$BACKUP_ENV_FILE"
    file_targets="${BACKUP_TARGETS:-}"
    file_keep_daily="${KEEP_DAILY:-}"
    file_keep_weekly="${KEEP_WEEKLY:-}"
    file_keep_monthly="${KEEP_MONTHLY:-}"
    file_excludes="${BACKUP_EXCLUDES:-}"
  fi

  targets=$(resolve_value "$targets" "$env_targets" "$file_targets" "$DEFAULT_TARGETS")
  keep_daily=$(resolve_value "$keep_daily" "$env_keep_daily" "$file_keep_daily" "$DEFAULT_KEEP_DAILY")
  keep_weekly=$(resolve_value "$keep_weekly" "$env_keep_weekly" "$file_keep_weekly" "$DEFAULT_KEEP_WEEKLY")
  keep_monthly=$(resolve_value "$keep_monthly" "$env_keep_monthly" "$file_keep_monthly" "$DEFAULT_KEEP_MONTHLY")
  password=$(resolve_value "$password" "$env_password" "" "") || die "저장소 비밀번호(--password 또는 BACKUP_PASSWORD)가 필요합니다"

  if ! err=$(validate_positive_int "$keep_daily" "keep-daily"); then die "$err"; fi
  if ! err=$(validate_positive_int "$keep_weekly" "keep-weekly"); then die "$err"; fi
  if ! err=$(validate_positive_int "$keep_monthly" "keep-monthly"); then die "$err"; fi

  # excludes는 반복 가능한 --exclude 플래그로만 CLI에서 받으므로 환경변수 계층은 없다.
  # CLI에서 하나도 주지 않았으면 기존 backup.env 값을 재사용하고, 그것도 없으면 기본값.
  local excludes_csv
  if [[ ${#excludes[@]} -eq 0 ]]; then
    excludes_csv=$(resolve_value "" "" "$file_excludes" "$DEFAULT_EXCLUDES")
  else
    excludes_csv=$(IFS=,; printf '%s' "${excludes[*]}")
  fi

  if [[ "$backend" == "sftp" ]]; then
    host=$(resolve_value "$host" "$env_host" "" "") || true
    port=$(resolve_value "$port" "$env_port" "" "$DEFAULT_SFTP_PORT") || true
    user=$(resolve_value "$user" "$env_user" "" "") || true

    if [[ -z "$host" || -z "$user" ]]; then
      die "$(render_setting_hint_sftp "$host" "$port" "$user")"
    fi
    if ! err=$(validate_port "$port"); then die "$err"; fi

    if (( dry_run )); then
      log_info "[dry-run] backup.env(sftp) 생성 예정: ${BACKUP_ENV_FILE}"
      return 0
    fi

    ensure_restic_dir
    generate_ssh_key_if_missing

    local content
    content=$(render_backup_env_sftp "$(hostname)" "$host" "$port" "$user" "$BACKUP_SSH_KEY" "$password" "$targets" "$excludes_csv" "$keep_daily" "$keep_weekly" "$keep_monthly")
    write_secure_file "$BACKUP_ENV_FILE" 600 "$content"

    render_sftp_registration_notice "$(cat "${BACKUP_SSH_KEY}.pub")"
    log_info "setting(sftp) 완료"
    return 0
  fi

  if [[ "$backend" == "s3" ]]; then
    # Same `|| true` guard as the sftp branch: resolve_value returns 1 with no
    # output when every source is empty, and under set -euo pipefail an
    # unguarded assignment would abort here instead of reaching the emptiness
    # check below.
    endpoint=$(resolve_value "$endpoint" "$env_endpoint" "" "") || true
    bucket=$(resolve_value "$bucket" "$env_bucket" "" "") || true
    access_key=$(resolve_value "$access_key" "$env_access_key" "" "") || true
    secret_key=$(resolve_value "$secret_key" "$env_secret_key" "" "") || true

    if [[ -z "$endpoint" || -z "$bucket" ]]; then
      die "$(render_setting_hint_s3 "$endpoint" "$bucket")"
    fi
    if [[ -z "$access_key" || -z "$secret_key" ]]; then
      die "$(render_setting_hint_s3 "$endpoint" "$bucket")"
    fi

    if (( dry_run )); then
      log_info "[dry-run] backup.env(s3) 생성 예정: ${BACKUP_ENV_FILE}"
      return 0
    fi

    ensure_restic_dir

    local content
    content=$(render_backup_env_s3 "$(hostname)" "$endpoint" "$bucket" "$access_key" "$secret_key" "$password" "$targets" "$excludes_csv" "$keep_daily" "$keep_weekly" "$keep_monthly")
    write_secure_file "$BACKUP_ENV_FILE" 600 "$content"

    log_info "최소권한 버킷 정책을 아래와 같이 적용하세요:"
    render_s3_bucket_policy "$bucket"
    log_info "setting(s3) 완료"
    return 0
  fi

  die "지원하지 않는 backend입니다: ${backend}"
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
    install)
      shift
      cmd_install "$@"
      return $?
      ;;
    setting)
      shift
      cmd_setting "$@"
      return $?
      ;;
    init)
      shift
      cmd_init "$@"
      return $?
      ;;
    schedule)
      shift
      cmd_schedule "$@"
      return $?
      ;;
    run)
      shift
      cmd_run "$@"
      return $?
      ;;
    status)
      shift
      cmd_status "$@"
      return $?
      ;;
    uninstall)
      shift
      cmd_uninstall "$@"
      return $?
      ;;
    wizard)
      : # Task 15에서 연결
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

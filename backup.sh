#!/usr/bin/env bash
set -euo pipefail

RESTIC_ETC_DIR="${RESTIC_ETC_DIR:-/etc/restic}"
BACKUP_ENV_FILE="${BACKUP_ENV_FILE:-${RESTIC_ETC_DIR}/backup.env}"
BACKUP_SSH_KEY="${BACKUP_SSH_KEY:-${RESTIC_ETC_DIR}/backup_key}"
BACKUP_SCRIPT_INSTALL_PATH="${BACKUP_SCRIPT_INSTALL_PATH:-/usr/local/sbin/backup.sh}"
SYSTEMD_UNIT_DIR="${SYSTEMD_UNIT_DIR:-/etc/systemd/system}"
# resticprofile 마이그레이션(Task 20/22) 이후 아무 cmd_*도 참조하지 않지만, 죽은 코드로
# 유지 중인 render_service_unit/render_timer_unit의 회귀 테스트 자산과 짝을 맞추기 위해 남긴다.
# shellcheck disable=SC2034
SYSTEMD_SERVICE_FILE="${SYSTEMD_UNIT_DIR}/restic-backup.service"
# shellcheck disable=SC2034
SYSTEMD_TIMER_FILE="${SYSTEMD_UNIT_DIR}/restic-backup.timer"
RESTICPROFILE_INSTALL_PATH="${RESTICPROFILE_INSTALL_PATH:-/usr/local/bin/resticprofile}"
RESTICPROFILE_VERSION="0.33.1"
RESTICPROFILE_SHA256="${RESTICPROFILE_SHA256:-1d7027d15e3e2456e585a210f811d0f72ec40f6b3388f00425642ed579165d70}"
RESTICPROFILE_URL="${RESTICPROFILE_URL:-https://github.com/creativeprojects/resticprofile/releases/download/v${RESTICPROFILE_VERSION}/resticprofile_no_self_update_${RESTICPROFILE_VERSION}_linux_amd64.tar.gz}"
RESTICPROFILE_CONFIG_FILE="${RESTICPROFILE_CONFIG_FILE:-${RESTIC_ETC_DIR}/profiles.yaml}"
RESTICPROFILE_UNIT_TEMPLATE="${RESTICPROFILE_UNIT_TEMPLATE:-${RESTIC_ETC_DIR}/resticprofile-service.tmpl}"
RESTICPROFILE_TIMER_TEMPLATE="${RESTICPROFILE_TIMER_TEMPLATE:-${RESTIC_ETC_DIR}/resticprofile-timer.tmpl}"

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

require_backup_env() {
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi
  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"
}

resolve_profile_name() {
  printf '%s' "${BACKUP_PROFILE_NAME:-$(hostname)}"
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

validate_profile_name() {
  local value="$1"
  if [[ -z "$value" ]]; then
    printf 'ERROR: profile-name must not be empty\n'
    return 1
  fi
  if ! [[ "$value" =~ ^[a-zA-Z0-9_-]+$ ]]; then
    printf 'ERROR: profile-name must contain only letters, digits, _ or -, got: %s\n' "$value"
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

render_resticprofile_unit_template() {
  # 의도적으로 `{{ range .Environment }}` 블록을 넣지 않는다: 넣으면 resticprofile이
  # RESTIC_PASSWORD를 이 파일에 평문 `Environment=`로 주입한다(644, /etc/systemd/system/ -
  # 2026-07-10 docker 컨테이너에서 실측 확인, 기본 템플릿에서도 재현됨). 유닛의 ExecStart는
  # 실행 시점에 `--config <profiles.yaml>`을 그대로 다시 읽으므로(run-schedule), Environment
  # 블록이 없어도 비밀값은 profiles.yaml의 env: 블록에서 정상 공급된다.
  cat <<'EOF'
[Unit]
Description={{ .JobDescription }} (ISMS Compliance)
{{ if .AfterNetworkOnline }}After=network-online.target
{{ end }}
[Service]
Type=notify
User=root
Group=root
WorkingDirectory={{ .WorkingDirectory }}
ExecStart={{ .CommandLine }}
{{ if .Nice }}Nice={{ .Nice }}
{{ end -}}
EOF
}

render_resticprofile_timer_template() {
  cat <<'EOF'
[Unit]
Description={{ .TimerDescription }} (ISMS Compliance)

[Timer]
{{ range .OnCalendar -}}
OnCalendar={{ . }}
{{ end -}}
Unit={{ .SystemdProfile }}
Persistent=true

[Install]
WantedBy=timers.target
EOF
}

resticprofile_timer_unit_name() {
  printf 'resticprofile-backup@profile-%s.timer' "$1"
}

write_resticprofile_assets() {
  local profile_name="$1" on_calendar="$2"
  write_secure_file "$RESTICPROFILE_UNIT_TEMPLATE" 644 "$(render_resticprofile_unit_template)"
  write_secure_file "$RESTICPROFILE_TIMER_TEMPLATE" 644 "$(render_resticprofile_timer_template)"
  write_secure_file "$RESTICPROFILE_CONFIG_FILE" 600 "$(render_resticprofile_config "$profile_name" "$on_calendar")"
}

render_resticprofile_config() {
  local profile_name="$1" on_calendar="$2"
  local targets_csv="${BACKUP_TARGETS:-}" excludes_csv="${BACKUP_EXCLUDES:-}"
  # KEEP_DAILY/KEEP_WEEKLY/KEEP_MONTHLY are exported by the caller having
  # sourced backup.env, not assigned in this function - shellcheck can't see that.
  # shellcheck disable=SC2153
  local keep_daily="${KEEP_DAILY}" keep_weekly="${KEEP_WEEKLY}" keep_monthly="${KEEP_MONTHLY}"
  local -a sources=() excludes=()
  IFS=',' read -ra sources <<< "$targets_csv"
  if [[ -n "$excludes_csv" ]]; then
    IFS=',' read -ra excludes <<< "$excludes_csv"
  fi

  printf 'version: "1"\n\n'
  printf 'global:\n'
  printf '  restic-lock-retry-after: 1m\n'
  printf '  restic-stale-lock-age: 2h\n'
  printf '  systemd-unit-template: %s\n' "$RESTICPROFILE_UNIT_TEMPLATE"
  printf '  systemd-timer-template: %s\n\n' "$RESTICPROFILE_TIMER_TEMPLATE"

  printf '%s:\n' "$profile_name"
  printf '  repository: "%s"\n' "${RESTIC_REPOSITORY:-}"
  printf '  force-inactive-lock: true\n'
  printf '  env:\n'
  printf '    RESTIC_PASSWORD: "%s"\n' "${RESTIC_PASSWORD:-}"
  if [[ -n "${AWS_ACCESS_KEY_ID:-}" ]]; then
    printf '    AWS_ACCESS_KEY_ID: "%s"\n' "$AWS_ACCESS_KEY_ID"
    printf '    AWS_SECRET_ACCESS_KEY: "%s"\n' "${AWS_SECRET_ACCESS_KEY:-}"
  fi
  if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
    printf '    RCLONE_CONFIG_SYNO_BACKUP_TYPE: "%s"\n' "$RCLONE_CONFIG_SYNO_BACKUP_TYPE"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_HOST: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_USER: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_USER:-}"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_PORT: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-}"
  fi

  printf '  retention:\n'
  printf '    after-backup: true\n'
  printf '    prune: true\n'
  printf '    keep-daily: %s\n' "$keep_daily"
  printf '    keep-weekly: %s\n' "$keep_weekly"
  printf '    keep-monthly: %s\n' "$keep_monthly"

  printf '  backup:\n'
  printf '    schedule: "%s"\n' "$on_calendar"
  printf '    schedule-permission: system\n'
  printf '    source:\n'
  local s
  for s in "${sources[@]}"; do
    printf '      - "%s"\n' "$s"
  done
  if [[ ${#excludes[@]} -gt 0 ]]; then
    printf '    exclude:\n'
    local e
    for e in "${excludes[@]}"; do
      printf '      - "%s"\n' "$e"
    done
  fi
}

dnf_install_packages() {
  # epel-release must land in its own transaction first: dnf resolves a
  # single `install` command's package set before epel-release's post-install
  # repo registration takes effect, so restic/rclone (EPEL-only packages)
  # are unresolvable if requested in the same command.
  dnf install -y epel-release
  dnf install -y restic rclone
}

install_resticprofile() {
  if [[ -x "$RESTICPROFILE_INSTALL_PATH" ]]; then
    return 0
  fi

  local tmp_dir
  tmp_dir=$(mktemp -d)
  curl -fsSL -o "${tmp_dir}/resticprofile.tar.gz" "$RESTICPROFILE_URL"

  local actual_sha256
  actual_sha256=$(sha256sum "${tmp_dir}/resticprofile.tar.gz" | awk '{print $1}')
  if [[ "$actual_sha256" != "$RESTICPROFILE_SHA256" ]]; then
    rm -rf "$tmp_dir"
    die "resticprofile 체크섬 불일치 (예상: ${RESTICPROFILE_SHA256}, 실제: ${actual_sha256}) - 설치를 중단합니다"
  fi

  tar -xzf "${tmp_dir}/resticprofile.tar.gz" -C "$tmp_dir" resticprofile
  mkdir -p "$(dirname "$RESTICPROFILE_INSTALL_PATH")"
  install -m 0755 "${tmp_dir}/resticprofile" "$RESTICPROFILE_INSTALL_PATH"
  rm -rf "$tmp_dir"
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
[dry-run] dnf install -y epel-release
[dry-run] dnf install -y restic rclone
[dry-run] resticprofile ${RESTICPROFILE_VERSION} 다운로드+체크섬 검증 후 ${RESTICPROFILE_INSTALL_PATH}에 설치
[dry-run] install -m 0755 "\$0" "${BACKUP_SCRIPT_INSTALL_PATH}"
[dry-run] mkdir -p "${RESTIC_ETC_DIR}" && chmod 700 "${RESTIC_ETC_DIR}"
EOF
    return 0
  fi

  dnf_install_packages
  install_resticprofile
  self_install_copy "$0" "$force"
  ensure_restic_dir
  log_info "install 완료"
}

# --- sftp backend adapter ---
# 두 backend(sftp/s3)가 공유하는 계약: env_vars/resolve/validate/prepare/render_env/render_notice.
# cmd_setting은 이 6개 함수만 알면 되고, 백엔드별 필드 지식은 각 adapter 블록 안에 갇혀 있다.

backend_sftp_env_vars() {
  printf 'host\tBACKUP_HOST\nport\tBACKUP_PORT\nuser\tBACKUP_USER\n'
}

render_setting_hint_sftp() {
  local host="$1" port="$2" user="$3"
  printf "backup.sh setting --backend sftp --host %s --port %s --user %s --password '<REPO_PASSWORD>'\\n" \
    "$(render_placeholder_or_value "$host" "NAS_IP")" \
    "$(render_placeholder_or_value "$port" "PORT")" \
    "$(render_placeholder_or_value "$user" "NAS_USER")"
}

backend_sftp_resolve() {
  local -n cli_ref="$1" env_ref="$2" file_ref="$3" fields_ref="$4"
  fields_ref[host]=$(resolve_value "${cli_ref[host]:-}" "${env_ref[host]:-}" "${file_ref[host]:-}" "") || true
  fields_ref[port]=$(resolve_value "${cli_ref[port]:-}" "${env_ref[port]:-}" "${file_ref[port]:-}" "$DEFAULT_SFTP_PORT") || true
  fields_ref[user]=$(resolve_value "${cli_ref[user]:-}" "${env_ref[user]:-}" "${file_ref[user]:-}" "") || true
}

backend_sftp_validate() {
  # fields_ref는 nameref로 연관 배열을 가리키는데, 같은 변수명이 다른 함수에서도
  # nameref로 재사용되다 보니 shellcheck가 스칼라/배열 재할당으로 오인한다.
  # shellcheck disable=SC2178
  local -n fields_ref="$1"
  if [[ -z "${fields_ref[host]:-}" || -z "${fields_ref[user]:-}" ]]; then
    render_setting_hint_sftp "${fields_ref[host]:-}" "${fields_ref[port]:-}" "${fields_ref[user]:-}"
    return 1
  fi
  validate_port "${fields_ref[port]:-}"
}

backend_sftp_prepare() {
  generate_ssh_key_if_missing
}

backend_sftp_render_env() {
  local hostname_tag="$1"
  local -n fields_ref="$2" policy_ref="$3"
  cat <<EOF
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/${hostname_tag}"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="${fields_ref[host]}"
export RCLONE_CONFIG_SYNO_BACKUP_USER="${fields_ref[user]}"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="${fields_ref[port]}"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${BACKUP_SSH_KEY}"
export RESTIC_PASSWORD="${policy_ref[password]}"
export BACKUP_TARGETS="${policy_ref[targets]}"
export BACKUP_EXCLUDES="${policy_ref[excludes_csv]}"
export KEEP_DAILY="${policy_ref[keep_daily]}"
export KEEP_WEEKLY="${policy_ref[keep_weekly]}"
export KEEP_MONTHLY="${policy_ref[keep_monthly]}"
export BACKUP_PROFILE_NAME="${policy_ref[profile_name]}"
EOF
}

backend_sftp_render_notice() {
  # shellcheck disable=SC2034
  # fields_ref는 s3 adapter와 시그니처를 맞추기 위한 것으로, sftp 공개키 안내에는 필요 없다.
  local -n fields_ref="$1"
  cat <<EOF
아래 공개키를 NAS의 authorized_keys(또는 File Station)에 등록하세요:
----------------------------------------------------------
$(cat "${BACKUP_SSH_KEY}.pub")
----------------------------------------------------------
등록 후 'backup.sh init'을 실행하세요.
EOF
}

# --- s3 backend adapter ---

backend_s3_env_vars() {
  printf 'endpoint\tBACKUP_ENDPOINT\nbucket\tBACKUP_BUCKET\naccess_key\tBACKUP_ACCESS_KEY\nsecret_key\tBACKUP_SECRET_KEY\n'
}

render_setting_hint_s3() {
  local endpoint="$1" bucket="$2"
  printf "backup.sh setting --backend s3 --endpoint %s --bucket %s --access-key <ACCESS_KEY> --secret-key '<SECRET_KEY>' --password '<REPO_PASSWORD>'\\n" \
    "$(render_placeholder_or_value "$endpoint" "S3_ENDPOINT")" \
    "$(render_placeholder_or_value "$bucket" "BUCKET_NAME")"
}

backend_s3_resolve() {
  local -n cli_ref="$1" env_ref="$2" file_ref="$3" fields_ref="$4"
  fields_ref[endpoint]=$(resolve_value "${cli_ref[endpoint]:-}" "${env_ref[endpoint]:-}" "${file_ref[endpoint]:-}" "") || true
  fields_ref[bucket]=$(resolve_value "${cli_ref[bucket]:-}" "${env_ref[bucket]:-}" "${file_ref[bucket]:-}" "") || true
  fields_ref[access_key]=$(resolve_value "${cli_ref[access_key]:-}" "${env_ref[access_key]:-}" "${file_ref[access_key]:-}" "") || true
  fields_ref[secret_key]=$(resolve_value "${cli_ref[secret_key]:-}" "${env_ref[secret_key]:-}" "${file_ref[secret_key]:-}" "") || true
}

backend_s3_validate() {
  # 위 backend_sftp_validate와 같은 이유의 nameref 오탐.
  # shellcheck disable=SC2178
  local -n fields_ref="$1"
  if [[ -z "${fields_ref[endpoint]:-}" || -z "${fields_ref[bucket]:-}" ]]; then
    render_setting_hint_s3 "${fields_ref[endpoint]:-}" "${fields_ref[bucket]:-}"
    return 1
  fi
  if [[ -z "${fields_ref[access_key]:-}" || -z "${fields_ref[secret_key]:-}" ]]; then
    render_setting_hint_s3 "${fields_ref[endpoint]:-}" "${fields_ref[bucket]:-}"
    return 1
  fi
  return 0
}

backend_s3_prepare() {
  :
}

backend_s3_render_env() {
  local hostname_tag="$1"
  local -n fields_ref="$2" policy_ref="$3"
  cat <<EOF
export RESTIC_REPOSITORY="s3:${fields_ref[endpoint]}/${fields_ref[bucket]}/${hostname_tag}"
export AWS_ACCESS_KEY_ID="${fields_ref[access_key]}"
export AWS_SECRET_ACCESS_KEY="${fields_ref[secret_key]}"
export RESTIC_PASSWORD="${policy_ref[password]}"
export BACKUP_TARGETS="${policy_ref[targets]}"
export BACKUP_EXCLUDES="${policy_ref[excludes_csv]}"
export KEEP_DAILY="${policy_ref[keep_daily]}"
export KEEP_WEEKLY="${policy_ref[keep_weekly]}"
export KEEP_MONTHLY="${policy_ref[keep_monthly]}"
export BACKUP_PROFILE_NAME="${policy_ref[profile_name]}"
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

backend_s3_render_notice() {
  local -n fields_ref="$1"
  printf '최소권한 버킷 정책을 아래와 같이 적용하세요:\n'
  render_s3_bucket_policy "${fields_ref[bucket]}"
}

restic_is_initialized() {
  restic snapshots >/dev/null 2>&1
}

cmd_init() {
  require_root
  require_backup_env

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

  require_backup_env
  local profile_name; profile_name=$(resolve_profile_name)

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

      write_resticprofile_assets "$profile_name" "$on_calendar"
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" schedule
      log_info "schedule enable 완료 (${on_calendar})"
      ;;
    disable)
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" unschedule 2>/dev/null || true
      log_info "schedule disable 완료"
      ;;
    *)
      die "schedule은 'enable' 또는 'disable'만 지원합니다 (입력값: '${action}')"
      ;;
  esac
}

cmd_run() {
  require_backup_env
  local profile_name; profile_name=$(resolve_profile_name)

  write_resticprofile_assets "$profile_name" "$DEFAULT_ON_CALENDAR"

  if resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" backup; then
    log_info "백업 성공"
  else
    die "resticprofile backup 실패"
  fi
}

cmd_status() {
  require_backup_env

  printf '저장소 위치: %s\n' "${RESTIC_REPOSITORY:-알 수 없음}"
  printf '백업 대상: %s\n' "${BACKUP_TARGETS:-알 수 없음}"

  printf '최근 스냅샷:\n'
  restic snapshots --json 2>/dev/null || printf '(조회 실패 또는 미초기화)\n'

  local profile_name; profile_name=$(resolve_profile_name)
  local timer_state
  timer_state=$(systemctl is-active "$(resticprofile_timer_unit_name "$profile_name")" 2>/dev/null) || true
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

  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    # shellcheck source=/dev/null
    source "$BACKUP_ENV_FILE"
    local profile_name; profile_name=$(resolve_profile_name)
    resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" unschedule 2>/dev/null || true
  fi

  if (( purge )); then
    rm -rf "$RESTIC_ETC_DIR"
    rm -rf "${HOME:-/root}/.cache/restic"
    log_info "uninstall --purge 완료 (${RESTIC_ETC_DIR} 삭제됨)"
  else
    log_info "uninstall 완료 (${RESTIC_ETC_DIR}는 유지됨)"
  fi
}

cmd_wizard() {
  require_root

  if [[ ! -f "$BACKUP_SCRIPT_INSTALL_PATH" ]]; then
    log_info "패키지를 설치합니다..."
    cmd_install
  fi

  printf '백엔드를 선택하세요:\n'
  printf '  [1] S3 호환 스토리지 - HTTPS 기반 오브젝트 스토리지(AWS S3, MinIO 등)\n'
  printf '  [2] SFTP(NAS) - SSH로 접속하는 시놀로지 NAS 등\n'
  printf '선택 (1/2): '
  local choice
  read -r choice

  local backend
  case "$choice" in
    1) backend="s3" ;;
    2) backend="sftp" ;;
    *) die "1 또는 2를 입력하세요" ;;
  esac

  local -a setting_args=(--backend "$backend" --force)

  if [[ "$backend" == "sftp" ]]; then
    printf 'NAS_IP: 백업 데이터를 저장할 NAS의 IP 주소입니다.\nNAS IP 주소 입력: '
    local host; read -r host
    printf 'PORT: NAS의 SSH/SFTP 포트입니다. Enter로 기본값(%s) 사용.\n포트 입력: ' "$DEFAULT_SFTP_PORT"
    local port; read -r port
    port="${port:-$DEFAULT_SFTP_PORT}"
    printf 'USER: NAS에 접속할 SFTP 계정입니다.\n사용자 입력: '
    local user; read -r user
    setting_args+=(--host "$host" --port "$port" --user "$user")
  else
    printf 'S3_ENDPOINT: 접속할 S3 호환 엔드포인트 URL입니다.\n엔드포인트 입력: '
    local endpoint; read -r endpoint
    printf 'BUCKET: 백업을 저장할 버킷 이름입니다.\n버킷 입력: '
    local bucket; read -r bucket
    printf 'ACCESS_KEY: 버킷 접근용 access key입니다.\naccess key 입력: '
    local access_key; read -r access_key
    printf 'SECRET_KEY: 버킷 접근용 secret key입니다.\nsecret key 입력: '
    local secret_key; read -r secret_key
    setting_args+=(--endpoint "$endpoint" --bucket "$bucket" --access-key "$access_key" --secret-key "$secret_key")
  fi

  printf '저장소 비밀번호: 분실 시 백업 데이터를 복구할 수 없습니다. 안전한 곳에 별도 보관하세요.\n비밀번호 입력(화면에 표시되지 않습니다): '
  local password
  read -rs password
  printf '\n'
  setting_args+=(--password "$password")

  printf '\n다음 설정으로 진행합니다:\n'
  printf '  백엔드: %s\n' "$backend"
  if [[ "$backend" == "sftp" ]]; then
    printf '  NAS: %s:%s (사용자: %s)\n' "$host" "$port" "$user"
  else
    printf '  S3 엔드포인트: %s\n' "$endpoint"
    printf '  버킷: %s\n' "$bucket"
  fi
  printf '이대로 진행할까요? [Y/n]: '
  local confirm
  read -r confirm
  if [[ -n "$confirm" && ! "$confirm" =~ ^[Yy]$ ]]; then
    log_info "설정을 취소했습니다."
    return 0
  fi

  cmd_setting "${setting_args[@]}"

  printf '위 안내(공개키 등록 또는 버킷 정책 적용)를 완료하셨으면 Enter를 누르세요: '
  local _ack; read -r _ack

  cmd_init

  printf '지금 정기 백업 스케줄을 등록할까요? 기본값은 매일 새벽 2시입니다. [Y/n]: '
  local schedule_choice; read -r schedule_choice
  local schedule_enabled=0
  if [[ -z "$schedule_choice" || "$schedule_choice" =~ ^[Yy]$ ]]; then
    cmd_schedule enable
    schedule_enabled=1
  fi

  # 요약 출력을 위해 backup.env에서 실제 저장소 위치를 읽어온다(하드코딩된 형식
  # 문자열을 다시 조립하지 않고 render_backup_env_* 가 실제로 쓴 값을 그대로 사용).
  local repo_location=""
  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    # shellcheck source=/dev/null
    source "$BACKUP_ENV_FILE"
    repo_location="${RESTIC_REPOSITORY:-}"
  fi

  printf '\n=========================================\n'
  printf ' 설정이 완료되었습니다\n'
  printf '=========================================\n'
  printf ' 백엔드: %s\n' "$backend"
  printf ' 저장소 위치: %s\n' "${repo_location:-알 수 없음}"
  if (( schedule_enabled )); then
    printf ' 정기 백업: 등록됨 (%s)\n' "$DEFAULT_ON_CALENDAR"
  else
    printf ' 정기 백업: 등록하지 않음 (필요시 backup.sh schedule enable 실행)\n'
  fi
  printf ' 이후에는 backup.sh run / status / uninstall 을 사용하세요.\n'
  printf '=========================================\n'
  log_info "wizard 완료"
}

cmd_setting() {
  require_root
  local parsed
  parsed=$(parse_long_opts "backend: targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: profile-name: force dry-run" -- "$@") || die "$parsed"

  local backend="" targets_csv="" password="" keep_daily="" keep_weekly="" keep_monthly="" profile_name=""
  local force=0 dry_run=0
  local -a excludes=()
  local -A cli=()

  local key val
  while IFS=$'\t' read -r key val; do
    case "$key" in
      backend) backend="$val" ;;
      targets) targets_csv="$val" ;;
      exclude) excludes+=("$val") ;;
      password) password="$val" ;;
      keep-daily) keep_daily="$val" ;;
      keep-weekly) keep_weekly="$val" ;;
      keep-monthly) keep_monthly="$val" ;;
      profile-name) profile_name="$val" ;;
      force) force=1 ;;
      dry-run) dry_run=1 ;;
      endpoint) cli[endpoint]="$val" ;;
      bucket) cli[bucket]="$val" ;;
      access-key) cli[access_key]="$val" ;;
      secret-key) cli[secret_key]="$val" ;;
      host) cli[host]="$val" ;;
      port) cli[port]="$val" ;;
      user) cli[user]="$val" ;;
    esac
  done <<< "$parsed"

  if [[ -z "$backend" ]]; then
    die "$(render_missing_settings_message)"
  fi
  local err
  if ! err=$(validate_backend "$backend"); then die "$err"; fi

  if [[ -f "$BACKUP_ENV_FILE" && "$force" != 1 ]]; then
    die "이미 설정이 있습니다: ${BACKUP_ENV_FILE} (덮어쓰려면 setting --force)"
  fi

  # 실제 사용자가 export한 환경변수는 backup.env를 source하기 전에 미리 캡처해둔다.
  # (source 이후에는 같은 변수명이 파일 값으로 덮어써지므로, 미리 캡처하지 않으면
  #  "환경변수 값"과 "기존 backup.env 값"을 구분할 수 없다.)
  local env_targets="${BACKUP_TARGETS:-}"
  local env_keep_daily="${KEEP_DAILY:-}"
  local env_keep_weekly="${KEEP_WEEKLY:-}"
  local env_keep_monthly="${KEEP_MONTHLY:-}"
  local env_password="${BACKUP_PASSWORD:-}"
  local env_profile_name="${BACKUP_PROFILE_NAME:-}"

  # backend 전용 필드의 env-shadow는 adapter의 env_vars가 알려주는 이름만큼만 캡처한다.
  # cmd_setting은 "어떤 이름을 캡처할지"를 모르고, 그 지식은 adapter 쪽에 남는다.
  local -A env=()
  local field_key var_name
  while IFS=$'\t' read -r field_key var_name; do
    [[ -z "$field_key" ]] && continue
    env["$field_key"]="${!var_name:-}"
  done < <(case "$backend" in sftp) backend_sftp_env_vars ;; s3) backend_s3_env_vars ;; esac)

  local -A file=()

  local file_targets="" file_keep_daily="" file_keep_weekly="" file_keep_monthly="" file_excludes="" file_profile_name=""
  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    # shellcheck source=/dev/null
    source "$BACKUP_ENV_FILE"
    file_targets="${BACKUP_TARGETS:-}"
    file_keep_daily="${KEEP_DAILY:-}"
    file_keep_weekly="${KEEP_WEEKLY:-}"
    file_keep_monthly="${KEEP_MONTHLY:-}"
    file_excludes="${BACKUP_EXCLUDES:-}"
    file_profile_name="${BACKUP_PROFILE_NAME:-}"
  fi

  targets_csv=$(resolve_value "$targets_csv" "$env_targets" "$file_targets" "$DEFAULT_TARGETS")
  keep_daily=$(resolve_value "$keep_daily" "$env_keep_daily" "$file_keep_daily" "$DEFAULT_KEEP_DAILY")
  keep_weekly=$(resolve_value "$keep_weekly" "$env_keep_weekly" "$file_keep_weekly" "$DEFAULT_KEEP_WEEKLY")
  keep_monthly=$(resolve_value "$keep_monthly" "$env_keep_monthly" "$file_keep_monthly" "$DEFAULT_KEEP_MONTHLY")
  password=$(resolve_value "$password" "$env_password" "" "") || die "저장소 비밀번호(--password 또는 BACKUP_PASSWORD)가 필요합니다"

  if ! err=$(validate_positive_int "$keep_daily" "keep-daily"); then die "$err"; fi
  if ! err=$(validate_positive_int "$keep_weekly" "keep-weekly"); then die "$err"; fi
  if ! err=$(validate_positive_int "$keep_monthly" "keep-monthly"); then die "$err"; fi

  profile_name=$(resolve_value "$profile_name" "$env_profile_name" "$file_profile_name" "$(hostname)")
  if ! err=$(validate_profile_name "$profile_name"); then die "$err"; fi

  # excludes는 반복 가능한 --exclude 플래그로만 CLI에서 받으므로 환경변수 계층은 없다.
  # CLI에서 하나도 주지 않았으면 기존 backup.env 값을 재사용하고, 그것도 없으면 기본값.
  local excludes_csv
  if [[ ${#excludes[@]} -eq 0 ]]; then
    excludes_csv=$(resolve_value "" "" "$file_excludes" "$DEFAULT_EXCLUDES")
  else
    excludes_csv=$(IFS=,; printf '%s' "${excludes[*]}")
  fi

  # fields is populated via backend_*_resolve's nameref, not directly in this
  # scope - shellcheck can't see across that indirection.
  # shellcheck disable=SC2034
  local -A fields=()
  case "$backend" in
    sftp) backend_sftp_resolve cli env file fields ;;
    s3) backend_s3_resolve cli env file fields ;;
  esac

  if ! err=$(case "$backend" in sftp) backend_sftp_validate fields ;; s3) backend_s3_validate fields ;; esac); then
    die "$err"
  fi

  if (( dry_run )); then
    log_info "[dry-run] backup.env(${backend}) 생성 예정: ${BACKUP_ENV_FILE}"
    return 0
  fi

  ensure_restic_dir
  case "$backend" in
    sftp) backend_sftp_prepare ;;
    s3) backend_s3_prepare ;;
  esac

  # policy is read via backend_*_render_env's nameref, not directly in this
  # scope - shellcheck can't see across that indirection.
  # shellcheck disable=SC2034
  local -A policy=(
    [password]="$password" [targets]="$targets_csv" [excludes_csv]="$excludes_csv"
    [keep_daily]="$keep_daily" [keep_weekly]="$keep_weekly" [keep_monthly]="$keep_monthly"
    [profile_name]="$profile_name"
  )
  local content
  case "$backend" in
    sftp) content=$(backend_sftp_render_env "$(hostname)" fields policy) ;;
    s3) content=$(backend_s3_render_env "$(hostname)" fields policy) ;;
  esac
  write_secure_file "$BACKUP_ENV_FILE" 600 "$content"

  case "$backend" in
    sftp) backend_sftp_render_notice fields ;;
    s3) backend_s3_render_notice fields ;;
  esac
  log_info "setting(${backend}) 완료"
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
      shift
      cmd_wizard "$@"
      return $?
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

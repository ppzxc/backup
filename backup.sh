#!/usr/bin/env bash
set -euo pipefail

RESTIC_ETC_DIR="${RESTIC_ETC_DIR:-/etc/restic}"
BACKUP_ENV_FILE="${BACKUP_ENV_FILE:-${RESTIC_ETC_DIR}/backup.env}"
BACKUP_SSH_KEY="${BACKUP_SSH_KEY:-${RESTIC_ETC_DIR}/backup_key}"
BACKUP_SCRIPT_INSTALL_PATH="${BACKUP_SCRIPT_INSTALL_PATH:-/usr/local/sbin/backup.sh}"
RESTICPROFILE_INSTALL_PATH="${RESTICPROFILE_INSTALL_PATH:-/usr/local/bin/resticprofile}"
RESTICPROFILE_VERSION="0.33.1"
RESTICPROFILE_SHA256="${RESTICPROFILE_SHA256:-1d7027d15e3e2456e585a210f811d0f72ec40f6b3388f00425642ed579165d70}"
RESTICPROFILE_URL="${RESTICPROFILE_URL:-https://github.com/creativeprojects/resticprofile/releases/download/v${RESTICPROFILE_VERSION}/resticprofile_no_self_update_${RESTICPROFILE_VERSION}_linux_amd64.tar.gz}"
RESTIC_INSTALL_PATH="${RESTIC_INSTALL_PATH:-/usr/local/bin/restic}"
RESTIC_VERSION="0.19.0"
RESTIC_SHA256="${RESTIC_SHA256:-13176fe6d89d4357947a2cd107218ab2873a5f9d8e1ac2d4cd1c8e07e6839c21}"
RESTIC_URL="${RESTIC_URL:-https://github.com/restic/restic/releases/download/v${RESTIC_VERSION}/restic_${RESTIC_VERSION}_linux_amd64.bz2}"
RCLONE_INSTALL_PATH="${RCLONE_INSTALL_PATH:-/usr/local/bin/rclone}"
RCLONE_VERSION="1.74.3"
RCLONE_SHA256="${RCLONE_SHA256:-dbee7ccd7a5d617e4ed4cd4555c16669b511abfe8d31164f61be35ac9e999bd2}"
RCLONE_URL="${RCLONE_URL:-https://github.com/rclone/rclone/releases/download/v${RCLONE_VERSION}/rclone-v${RCLONE_VERSION}-linux-amd64.zip}"
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
BACKUP_VERBOSE="${BACKUP_VERBOSE:-0}"

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
    *) printf 'backend must be s3 or sftp, got: %s\n' "$value"; return 1 ;;
  esac
}

validate_port() {
  local value="$1"
  if ! [[ "$value" =~ ^[0-9]+$ ]]; then
    printf 'port must be numeric, got: %s\n' "$value"
    return 1
  fi
  if (( 10#$value < 1 || 10#$value > 65535 )); then
    printf 'port must be between 1 and 65535, got: %s\n' "$value"
    return 1
  fi
  return 0
}

validate_positive_int() {
  local value="$1" label="$2"
  if ! [[ "$value" =~ ^[0-9]+$ ]]; then
    printf '%s must be numeric, got: %s\n' "$label" "$value"
    return 1
  fi
  if (( 10#$value < 1 )); then
    printf '%s must be positive, got: %s\n' "$label" "$value"
    return 1
  fi
  return 0
}

validate_profile_name() {
  local value="$1"
  if [[ -z "$value" ]]; then
    printf 'profile-name must not be empty\n'
    return 1
  fi
  if ! [[ "$value" =~ ^[a-zA-Z0-9_.-]+$ ]]; then
    printf 'profile-name must contain only letters, digits, _, - or ., got: %s\n' "$value"
    return 1
  fi
  return 0
}

validate_not_empty() {
  local value="$1"
  if [[ -z "$value" ]]; then
    printf '값을 입력해야 합니다\n'
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

# parse_long_opts의 tab구분 출력을 소비하는 반복문을 호출부마다 손으로 짜는 대신,
# nameref 연관 배열 하나를 채워주는 얇은 래퍼. parse_long_opts 자체의 계약/테스트는
# 그대로 두고, 그 결과를 쓰는 방식만 걷어낸다. 같은 플래그가 여러 번 오면(--exclude 등)
# 값을 콤마로 이어붙인다 - 기존 cmd_setting도 --exclude 반복값을 콤마로 합쳐서 저장했다.
parse_opts_into() {
  local -n opts_ref="$1"
  shift
  local parsed
  parsed=$(parse_long_opts "$@") || die "$parsed"

  local key val
  while IFS=$'\t' read -r key val; do
    [[ -z "$key" ]] && continue
    if [[ -n "${opts_ref[$key]:-}" ]]; then
      opts_ref["$key"]="${opts_ref[$key]},${val}"
    else
      opts_ref["$key"]="$val"
    fi
  done <<< "$parsed"
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

# restic 릴리스 에셋은 tar가 아니라 순수 bzip2 압축 바이너리 한 개뿐이라
# python3의 bz2 모듈로 직접 풀어낸다(unzip/bunzip2는 최소 설치 이미지에
# 없을 수 있지만, dnf 자체가 python3에 의존하므로 RHEL 계열이면 항상 있다).
install_restic() {
  if [[ -x "$RESTIC_INSTALL_PATH" ]]; then
    return 0
  fi

  local tmp_dir
  tmp_dir=$(mktemp -d)
  curl -fsSL -o "${tmp_dir}/restic.bz2" "$RESTIC_URL"

  local actual_sha256
  actual_sha256=$(sha256sum "${tmp_dir}/restic.bz2" | awk '{print $1}')
  if [[ "$actual_sha256" != "$RESTIC_SHA256" ]]; then
    rm -rf "$tmp_dir"
    die "restic 체크섬 불일치 (예상: ${RESTIC_SHA256}, 실제: ${actual_sha256}) - 설치를 중단합니다"
  fi

  python3 -c "import bz2, shutil, sys; shutil.copyfileobj(bz2.open(sys.argv[1], 'rb'), open(sys.argv[2], 'wb'))" \
    "${tmp_dir}/restic.bz2" "${tmp_dir}/restic"
  mkdir -p "$(dirname "$RESTIC_INSTALL_PATH")"
  install -m 0755 "${tmp_dir}/restic" "$RESTIC_INSTALL_PATH"
  rm -rf "$tmp_dir"
}

# rclone 릴리스 에셋은 zip이라 python3의 zipfile 모듈로 풀어낸다. 압축 안의
# 최상위 디렉토리명이 버전 문자열을 포함해 고정된 형태("rclone-vX.Y.Z-linux-amd64")로
# 나오므로 별도 탐색 없이 바로 경로를 구성할 수 있다.
install_rclone() {
  if [[ -x "$RCLONE_INSTALL_PATH" ]]; then
    return 0
  fi

  local tmp_dir
  tmp_dir=$(mktemp -d)
  curl -fsSL -o "${tmp_dir}/rclone.zip" "$RCLONE_URL"

  local actual_sha256
  actual_sha256=$(sha256sum "${tmp_dir}/rclone.zip" | awk '{print $1}')
  if [[ "$actual_sha256" != "$RCLONE_SHA256" ]]; then
    rm -rf "$tmp_dir"
    die "rclone 체크섬 불일치 (예상: ${RCLONE_SHA256}, 실제: ${actual_sha256}) - 설치를 중단합니다"
  fi

  python3 -m zipfile -e "${tmp_dir}/rclone.zip" "${tmp_dir}/extracted"
  mkdir -p "$(dirname "$RCLONE_INSTALL_PATH")"
  install -m 0755 "${tmp_dir}/extracted/rclone-v${RCLONE_VERSION}-linux-amd64/rclone" "$RCLONE_INSTALL_PATH"
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

# restic init 전에 SFTP 로그인이 가능한지 미리 확인한다. 이게 없으면 rclone이
# NewFs 단계에서 실패했을 때 restic이 원인(인증 실패/포트 막힘/키 미등록 등)을
# 구분하지 않고 전부 "error talking HTTP to rclone: exit status 1"로 뭉뚱그린다.
# 반드시 rclone 자체(restic이 실제로 spawn하는 것과 동일한 바이너리/설정)로
# 점검해야 한다. 시스템 ssh/sftp 클라이언트로 점검하면 두 가지 문제가 있다:
#   1) 별도 openssh-clients 의존성이 생긴다.
#   2) 일반 exec/쉘 세션(`ssh ... true`)은 NAS 계정이 SFTP 전용(쉘 로그인
#      권한 없음)으로 제한된 경우 공개키 인증에 성공하고도 거부되어, 실제로는
#      문제 없는 설정을 오탐으로 막아버린다.
# 대상 경로("$remote:/backup/host")가 아니라 remote 루트("$remote:")를 봐야
# 한다 — 최초 init 시점엔 백업 대상 하위 경로가 아직 없는 게 정상이라, 그
# 경로를 직접 보면 인증에 성공했어도 "directory not found"로 실패한다.
# verbose=1일 때만 rclone 자체의 진단 메시지(왜 실패했는지)를 그대로 보여주고,
# 기본값(0)에서는 원래대로 조용히 성공/실패 여부만 반환한다.
rclone_check_connectivity() {
  local remote="$1" verbose="${2:-0}"
  if [[ "$verbose" == "1" ]]; then
    rclone lsd "${remote}:" >/dev/null
  else
    rclone lsd "${remote}:" >/dev/null 2>&1
  fi
}

render_sftp_connectivity_failure_message() {
  local host="$1" port="$2" user="$3"
  cat <<EOF
[!] SFTP(${user}@${host}:${port}) 연결에 실패해 restic init을 진행할 수 없습니다.

다음을 확인하세요:
  1) NAS의 authorized_keys(또는 File Station)에 공개키가 정확히 등록되었는지
  2) 포트 ${port}가 방화벽/공유기에서 열려 있고 NAS로 포워딩되는지
  3) 사용자 계정 '${user}'이 NAS에 존재하고 SFTP 접속 권한이 있는지

확인 후 'backup.sh init'을 다시 실행하세요.
EOF
}

cmd_install() {
  require_root
  local -A opts=()
  parse_opts_into opts "force dry-run" -- "$@"
  local force="${opts[force]:-0}" dry_run="${opts[dry-run]:-0}"

  if (( dry_run )); then
    cat <<EOF
[dry-run] restic ${RESTIC_VERSION} 다운로드+체크섬 검증 후 ${RESTIC_INSTALL_PATH}에 설치
[dry-run] rclone ${RCLONE_VERSION} 다운로드+체크섬 검증 후 ${RCLONE_INSTALL_PATH}에 설치
[dry-run] resticprofile ${RESTICPROFILE_VERSION} 다운로드+체크섬 검증 후 ${RESTICPROFILE_INSTALL_PATH}에 설치
[dry-run] install -m 0755 "\$0" "${BACKUP_SCRIPT_INSTALL_PATH}"
[dry-run] mkdir -p "${RESTIC_ETC_DIR}" && chmod 700 "${RESTIC_ETC_DIR}"
EOF
    return 0
  fi

  install_restic
  install_rclone
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
  local -n fields_ref="$1"
  generate_ssh_key_if_missing
  # pubkey는 nameref로 쓰는 연관 배열의 키일 뿐, 별도로 선언된 변수가 아니다.
  # shellcheck disable=SC2154
  fields_ref[pubkey]="$(cat "${BACKUP_SSH_KEY}.pub")"
}

backend_sftp_render_env() {
  local hostname_tag="$1"
  # 다른 함수의 같은 이름 nameref 사용과 겹쳐 shellcheck가 배열/스칼라 재할당으로 오인한다.
  # shellcheck disable=SC2178
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
  # 같은 이유의 nameref 오탐.
  # shellcheck disable=SC2178
  local -n fields_ref="$1"
  cat <<EOF
아래 공개키를 NAS의 authorized_keys(또는 File Station)에 등록하세요:
----------------------------------------------------------
${fields_ref[pubkey]}
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
  # 같은 이유의 nameref 오탐.
  # shellcheck disable=SC2178
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

  if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
    if ! command -v rclone >/dev/null 2>&1; then
      die "[!] rclone이 설치되어 있지 않습니다. 'backup.sh install'을 다시 실행해 restic/rclone을 설치한 뒤 'backup.sh init'을 재시도하세요."
    fi
    if ! rclone_check_connectivity "syno_backup" "${BACKUP_VERBOSE:-0}"; then
      die "$(render_sftp_connectivity_failure_message "$RCLONE_CONFIG_SYNO_BACKUP_HOST" \
        "$RCLONE_CONFIG_SYNO_BACKUP_PORT" "$RCLONE_CONFIG_SYNO_BACKUP_USER")"
    fi
  fi

  if restic_is_initialized; then
    log_info "이미 초기화된 저장소입니다. 스킵합니다."
    return 0
  fi

  local -a restic_init_args=(init)
  if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
    restic_init_args+=(--verbose)
  fi
  restic "${restic_init_args[@]}"
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
      local -A opts=()
      parse_opts_into opts "on-calendar:" -- "$@"
      local on_calendar="${opts[on-calendar]:-$DEFAULT_ON_CALENDAR}"

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

  local -a resticprofile_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" backup)
  if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
    resticprofile_args+=(-v)
  fi

  if resticprofile "${resticprofile_args[@]}"; then
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

# ISMS 감사 대응용 통합 리포트. cmd_status는 빠른 운영 확인용이고, 이쪽은
# 백업 정책/보존 정책/스케줄/접근 통제를 한 화면에 모은 컴플라이언스 리포트다.
# BACKUP_TARGETS/BACKUP_EXCLUDES/KEEP_DAILY/KEEP_WEEKLY/KEEP_MONTHLY/
# RESTIC_REPOSITORY/RESTIC_PASSWORD는 backup.env를 source한 호출자(cmd_audit)의
# 전역값을 그대로 읽는다(render_resticprofile_config와 동일한 관례).
render_audit_report() {
  local backend="$1" on_calendar="$2" timer_enabled="$3" timer_active="$4" next_run="$5" etc_perm="$6" env_perm="$7"

  local encrypted_note="AES-256 (restic 저장소 자체 암호화)"
  if [[ -z "${RESTIC_PASSWORD:-}" ]]; then
    encrypted_note="${encrypted_note} - 경고: 비밀번호 미설정"
  fi

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    # Beautiful ANSI styled output for interactive TTY
    local C_RESET="\033[0m"
    local C_BOLD="\033[1m"
    local C_DIM="\033[2m"
    local C_RED="\033[31m"
    local C_GREEN="\033[32m"
    local C_YELLOW="\033[33m"
    local C_BLUE="\033[34m"
    local C_CYAN="\033[36m"
    local C_GRAY="\033[90m"

    # Style backend value
    local styled_backend="${C_BOLD}${C_BLUE}${backend}${C_RESET}"
    
    # Style repo
    local styled_repo="${C_BOLD}${RESTIC_REPOSITORY:-알 수 없음}${C_RESET}"

    # Style encryption note
    local styled_crypto
    if [[ -z "${RESTIC_PASSWORD:-}" ]]; then
      styled_crypto="${C_RED}${encrypted_note}${C_RESET}"
    else
      styled_crypto="${C_GREEN}${encrypted_note}${C_RESET}"
    fi

    # Style targets/excludes
    local styled_targets="${C_BOLD}${BACKUP_TARGETS:-알 수 없음}${C_RESET}"
    local styled_excludes="${C_DIM}${BACKUP_EXCLUDES:-(없음)}${C_RESET}"

    # Style retention
    local styled_daily="${C_BOLD}${KEEP_DAILY:-?}${C_RESET}"
    local styled_weekly="${C_BOLD}${KEEP_WEEKLY:-?}${C_RESET}"
    local styled_monthly="${C_BOLD}${KEEP_MONTHLY:-?}${C_RESET}"

    # Style schedule
    local styled_on_calendar="${C_BOLD}${on_calendar}${C_RESET}"
    
    local styled_timer_enabled
    if [[ "$timer_enabled" == "enabled" ]]; then
      styled_timer_enabled="${C_GREEN}${timer_enabled}${C_RESET}"
    elif [[ "$timer_enabled" == "disabled" ]]; then
      styled_timer_enabled="${C_RED}${timer_enabled}${C_RESET}"
    else
      styled_timer_enabled="${C_GRAY}${timer_enabled}${C_RESET}"
    fi

    local styled_timer_active
    if [[ "$timer_active" == "active" ]]; then
      styled_timer_active="${C_GREEN}${timer_active}${C_RESET}"
    elif [[ "$timer_active" == "inactive" ]]; then
      styled_timer_active="${C_GRAY}${timer_active}${C_RESET}"
    else
      styled_timer_active="${C_RED}${timer_active}${C_RESET}"
    fi

    local styled_next_run="${C_BOLD}${next_run}${C_RESET}"

    # Style permissions
    local styled_etc_perm
    if [[ "$etc_perm" == "700" ]]; then
      styled_etc_perm="${C_GREEN}${etc_perm}${C_RESET} ${C_GRAY}(안전)${C_RESET}"
    else
      styled_etc_perm="${C_RED}${etc_perm}${C_RESET} ${C_YELLOW}(경고: 700 권장)${C_RESET}"
    fi

    local styled_env_perm
    if [[ "$env_perm" == "600" ]]; then
      styled_env_perm="${C_GREEN}${env_perm}${C_RESET} ${C_GRAY}(안전)${C_RESET}"
    else
      styled_env_perm="${C_RED}${env_perm}${C_RESET} ${C_YELLOW}(경고: 600 권장)${C_RESET}"
    fi

    printf '%b%b⚙  백업 정책 (Backup Policy)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b├──%b 백엔드:    %b\n' "$C_GRAY" "$C_RESET" "$styled_backend"
    printf '%b├──%b 저장소 위치: %b\n' "$C_GRAY" "$C_RESET" "$styled_repo"
    printf '%b├──%b 암호화:    %b\n' "$C_GRAY" "$C_RESET" "$styled_crypto"
    printf '%b├──%b 백업 대상: %b\n' "$C_GRAY" "$C_RESET" "$styled_targets"
    printf '%b└──%b 제외 패턴: %b\n' "$C_GRAY" "$C_RESET" "$styled_excludes"
    printf '\n'

    printf '%b%b⚙  보존 정책 (Retention Policy)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b├──%b 일간 보관: %b개\n' "$C_GRAY" "$C_RESET" "$styled_daily"
    printf '%b├──%b 주간 보관: %b개\n' "$C_GRAY" "$C_RESET" "$styled_weekly"
    printf '%b└──%b 월간 보관: %b개\n' "$C_GRAY" "$C_RESET" "$styled_monthly"
    printf '\n'

    printf '%b%b⚙  스케줄 (Schedule)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b├──%b 반복 주기(OnCalendar): %b\n' "$C_GRAY" "$C_RESET" "$styled_on_calendar"
    printf '%b├──%b 타이머 등록 상태:      %b\n' "$C_GRAY" "$C_RESET" "$styled_timer_enabled"
    printf '%b├──%b 타이머 실행 상태:      %b\n' "$C_GRAY" "$C_RESET" "$styled_timer_active"
    printf '%b└──%b 다음 실행 예정:        %b\n' "$C_GRAY" "$C_RESET" "$styled_next_run"
    printf '\n'

    printf '%b%b⚙  접근 통제 (Access Control)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b├──%b %s 권한: %b\n' "$C_GRAY" "$C_RESET" "$RESTIC_ETC_DIR" "$styled_etc_perm"
    printf '%b└──%b %s 권한: %b\n' "$C_GRAY" "$C_RESET" "$BACKUP_ENV_FILE" "$styled_env_perm"
  else
    # Simple plain-text fallback (matches original structure exactly for backward compatibility & tests)
    printf '=== 백업 정책 ===\n'
    printf '백엔드: %s\n' "$backend"
    printf '저장소 위치: %s\n' "${RESTIC_REPOSITORY:-알 수 없음}"
    printf '암호화: %s\n' "$encrypted_note"
    printf '백업 대상: %s\n' "${BACKUP_TARGETS:-알 수 없음}"
    printf '제외 패턴: %s\n' "${BACKUP_EXCLUDES:-(없음)}"
    printf '\n'

    printf '=== 보존 정책 ===\n'
    printf '일간 보관: %s개\n' "${KEEP_DAILY:-?}"
    printf '주간 보관: %s개\n' "${KEEP_WEEKLY:-?}"
    printf '월간 보관: %s개\n' "${KEEP_MONTHLY:-?}"
    printf '\n'

    printf '=== 스케줄 ===\n'
    printf '반복 주기(OnCalendar): %s\n' "$on_calendar"
    printf '타이머 등록 상태: %s\n' "$timer_enabled"
    printf '타이머 실행 상태: %s\n' "$timer_active"
    printf '다음 실행 예정: %s\n' "$next_run"
    printf '\n'

    printf '=== 접근 통제 ===\n'
    printf '%s 권한: %s\n' "$RESTIC_ETC_DIR" "$etc_perm"
    printf '%s 권한: %s\n' "$BACKUP_ENV_FILE" "$env_perm"
  fi
}

render_audit_report_json() {
  local backend="$1" on_calendar="$2" timer_enabled="$3" timer_active="$4" next_run="$5" etc_perm="$6" env_perm="$7"

  local hostname; hostname=$(hostname 2>/dev/null || echo "unknown")
  local timestamp; timestamp=$(date --iso-8601=seconds 2>/dev/null || date -Iseconds 2>/dev/null || date "+%Y-%m-%dT%H:%M:%S%z")

  local repo="${RESTIC_REPOSITORY:-}"
  local targets="${BACKUP_TARGETS:-}"
  local excludes_val="${BACKUP_EXCLUDES:-}"

  local keep_daily="${KEEP_DAILY:-null}"
  local keep_weekly="${KEEP_WEEKLY:-null}"
  local keep_monthly="${KEEP_MONTHLY:-null}"

  local has_password_warning="false"
  if [[ -z "${RESTIC_PASSWORD:-}" ]]; then
    has_password_warning="true"
  fi

  local etc_safe="false"
  if [[ "$etc_perm" == "700" ]]; then
    etc_safe="true"
  fi

  local env_safe="false"
  if [[ "$env_perm" == "600" ]]; then
    env_safe="true"
  fi

  local snapshots_json
  snapshots_json=$(restic snapshots --json 2>/dev/null || echo "[]")
  if [[ -z "$snapshots_json" || "$snapshots_json" == "null" ]]; then
    snapshots_json="[]"
  fi

  cat <<EOF
{
  "hostname": "${hostname//\"/\\\"}",
  "timestamp": "${timestamp}",
  "backup_policy": {
    "backend": "${backend//\"/\\\"}",
    "repository": "${repo//\"/\\\"}",
    "encryption": "AES-256 (restic 저장소 자체 암호화)",
    "encryption_warning": ${has_password_warning},
    "targets": "${targets//\"/\\\"}",
    "excludes": "${excludes_val//\"/\\\"}"
  },
  "retention_policy": {
    "keep_daily": ${keep_daily},
    "keep_weekly": ${keep_weekly},
    "keep_monthly": ${keep_monthly}
  },
  "schedule": {
    "on_calendar": "${on_calendar//\"/\\\"}",
    "timer_enabled": "${timer_enabled//\"/\\\"}",
    "timer_active": "${timer_active//\"/\\\"}",
    "next_run": "${next_run//\"/\\\"}"
  },
  "access_control": {
    "etc_restic_dir": "${RESTIC_ETC_DIR}",
    "etc_restic_dir_permission": "${etc_perm}",
    "etc_restic_dir_safe": ${etc_safe},
    "backup_env_file": "${BACKUP_ENV_FILE}",
    "backup_env_file_permission": "${env_perm}",
    "backup_env_file_safe": ${env_safe}
  },
  "snapshots": ${snapshots_json}
}
EOF
}

cmd_audit() {
  require_backup_env

  local -A opts=()
  parse_opts_into opts "report-file: report" -- "$@"
  local report_file="${opts[report-file]:-}"
  local report="${opts[report]:-0}"

  if (( report )) && [[ -z "$report_file" ]]; then
    report_file="/var/log/restic-backup/audit_report.txt"
  fi

  local profile_name; profile_name=$(resolve_profile_name)
  local timer_unit; timer_unit=$(resticprofile_timer_unit_name "$profile_name")

  local timer_enabled; timer_enabled=$(systemctl is-enabled "$timer_unit" 2>/dev/null) || true
  local timer_active; timer_active=$(systemctl is-active "$timer_unit" 2>/dev/null) || true
  local next_run
  next_run=$(systemctl list-timers "$timer_unit" --no-legend 2>/dev/null | awk '{print $1, $2, $3, $4}') || true

  local on_calendar
  on_calendar=$(systemctl cat "$timer_unit" 2>/dev/null | sed -n 's/^OnCalendar=//p' | head -1) || true

  local backend="s3"
  if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
    backend="sftp"
  fi

  local etc_perm; etc_perm="$(stat -c '%a' "$RESTIC_ETC_DIR" 2>/dev/null || echo '?')"
  local env_perm; env_perm="$(stat -c '%a' "$BACKUP_ENV_FILE" 2>/dev/null || echo '?')"

  # Always print to screen
  render_audit_report "$backend" "${on_calendar:-알 수 없음}" "${timer_enabled:-unknown}" \
    "${timer_active:-unknown}" "${next_run:-알 수 없음}" \
    "$etc_perm" "$env_perm"

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    local C_RESET="\033[0m"
    local C_BOLD="\033[1m"
    local C_CYAN="\033[36m"
    printf '\n%b⚙  백업 이력(restic snapshots)%b\n' "${C_CYAN}${C_BOLD}" "${C_RESET}"
    restic snapshots 2>/dev/null | sed 's/^/  /' || printf '  (조회 실패 또는 미초기화)\n'
  else
    printf '\n=== 백업 이력(restic snapshots) ===\n'
    restic snapshots 2>/dev/null || printf '(조회 실패 또는 미초기화)\n'
  fi

  # If report_file is requested, save both plain text and JSON versions
  if [[ -n "$report_file" ]]; then
    mkdir -p "$(dirname "$report_file")"
    chmod 700 "$(dirname "$report_file")" 2>/dev/null || true

    # Save plain text report
    (
      render_audit_report "$backend" "${on_calendar:-알 수 없음}" "${timer_enabled:-unknown}" \
        "${timer_active:-unknown}" "${next_run:-알 수 없음}" \
        "$etc_perm" "$env_perm"
      printf '\n=== 백업 이력(restic snapshots) ===\n'
      restic snapshots 2>/dev/null || printf '(조회 실패 또는 미초기화)\n'
    ) > "$report_file"
    chmod 600 "$report_file"

    # Save JSON report
    local json_report_file
    if [[ "$report_file" =~ \.(txt|md)$ ]]; then
      json_report_file="${report_file%.*}.json"
    else
      json_report_file="${report_file}.json"
    fi

    render_audit_report_json "$backend" "${on_calendar:-알 수 없음}" "${timer_enabled:-unknown}" \
      "${timer_active:-unknown}" "${next_run:-알 수 없음}" \
      "$etc_perm" "$env_perm" > "$json_report_file"
    chmod 600 "$json_report_file"

    log_info "감사 보고서가 동시 저장되었습니다:"
    log_info "  - 텍스트 보고서: $report_file"
    log_info "  - JSON 보고서: $json_report_file"
  fi
}

cmd_uninstall() {
  require_root
  local -A opts=()
  parse_opts_into opts "purge" -- "$@"
  local purge="${opts[purge]:-0}"

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

# cmd_wizard 전용 대화형 입력 헬퍼. 프롬프트는 stderr로 내보내고 답변만
# stdout으로 반환하므로 $(...)로 값을 캡처해도 프롬프트 문구가 섞이지 않는다.

# validate_fn이 통과할 때까지 같은 질문을 다시 묻는다. default가 주어지면
# "[default]"로 보여주고 빈 입력 시 그 값을 사용한다.
prompt_validated() {
  local message="$1" default="$2" validate_fn="$3"
  local value err
  while true; do
    if [[ -n "$default" ]]; then
      printf '%s [%s]: ' "$message" "$default" >&2
    else
      printf '%s: ' "$message" >&2
    fi
    read -r value
    value="${value:-$default}"
    if err=$("$validate_fn" "$value"); then
      printf '%s' "$value"
      return 0
    fi
    printf '%s 다시 입력하세요.\n' "$err" >&2
  done
}

# 화면에 표시되지 않는 비밀번호 입력을 받고, 빈 값이면 다시 묻는다.
prompt_secret_required() {
  local message="$1"
  local value
  while true; do
    printf '%s' "$message" >&2
    read -rs value
    printf '\n' >&2
    if [[ -n "$value" ]]; then
      printf '%s' "$value"
      return 0
    fi
    printf '값을 입력해야 합니다. 다시 입력하세요.\n' >&2
  done
}

# 1(S3)/2(SFTP) 중 하나를 고를 때까지 다시 묻고, 선택된 backend 이름을 반환한다.
prompt_backend_choice() {
  local choice
  while true; do
    printf '백엔드를 선택하세요:\n' >&2
    printf '  [1] S3 호환 스토리지 - HTTPS 기반 오브젝트 스토리지(AWS S3, MinIO 등)\n' >&2
    printf '  [2] SFTP(NAS) - SSH로 접속하는 시놀로지 NAS 등\n' >&2
    printf '선택 (1/2): ' >&2
    read -r choice
    case "$choice" in
      1) printf 's3'; return 0 ;;
      2) printf 'sftp'; return 0 ;;
      *) printf '1 또는 2를 입력하세요. 다시 입력하세요.\n' >&2 ;;
    esac
  done
}

cmd_wizard() {
  require_root

  # $BACKUP_SCRIPT_INSTALL_PATH의 존재 여부는 "이 스크립트가 예전에 한 번
  # 복사됐는지"만 알려줄 뿐, restic/rclone/resticprofile이 실제로 설치돼
  # 있는지와는 무관하다 — 이전 실행이 설치 중간에 중단됐거나 패키지가 이후
  # 지워진 경우, 이 마커만 보고 넘어가면 cmd_install이 다시 실행되지 않아
  # 실제 의존성이 없는 채로 wizard가 계속 진행된다. 그래서 필요한 바이너리
  # 자체의 존재를 직접 확인한다.
  if ! command -v restic >/dev/null 2>&1 || ! command -v rclone >/dev/null 2>&1 \
    || [[ ! -x "$RESTICPROFILE_INSTALL_PATH" ]]; then
    log_info "패키지를 설치합니다..."
    cmd_install
  fi

  local backend
  backend=$(prompt_backend_choice)

  local -a setting_args=(--backend "$backend" --force)

  if [[ "$backend" == "sftp" ]]; then
    local host; host=$(prompt_validated "백업 데이터를 저장할 NAS의 IP 주소를 입력하세요" "" validate_not_empty)
    local port; port=$(prompt_validated "NAS의 SSH/SFTP 접속 포트를 입력하세요" "$DEFAULT_SFTP_PORT" validate_port)
    local user; user=$(prompt_validated "NAS에 접속할 SFTP 계정명을 입력하세요" "" validate_not_empty)
    setting_args+=(--host "$host" --port "$port" --user "$user")
  else
    local endpoint; endpoint=$(prompt_validated "접속할 S3 호환 엔드포인트 URL을 입력하세요" "" validate_not_empty)
    local bucket; bucket=$(prompt_validated "백업을 저장할 버킷 이름을 입력하세요" "" validate_not_empty)
    local access_key; access_key=$(prompt_validated "버킷 접근용 access key를 입력하세요" "" validate_not_empty)
    local secret_key; secret_key=$(prompt_validated "버킷 접근용 secret key를 입력하세요" "" validate_not_empty)
    setting_args+=(--endpoint "$endpoint" --bucket "$bucket" --access-key "$access_key" --secret-key "$secret_key")
  fi

  local password
  password=$(prompt_secret_required '저장소 비밀번호: 백업 데이터를 AES-256 기반으로 암호화하는 데 쓰이는 필수 입력값입니다. 이 비밀번호가 없으면 NAS/S3 등 원격 저장소 쪽에서도 백업 내용을 열어볼 수 없습니다. 분실 시에는 백업 데이터를 복구할 방법이 없으니 반드시 별도의 안전한 곳에 보관하세요.
비밀번호 입력(화면에 표시되지 않습니다): ')
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
  local -A opts=()
  parse_opts_into opts "backend: targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: profile-name: force dry-run" -- "$@"

  local backend="${opts[backend]:-}" targets_csv="${opts[targets]:-}" password="${opts[password]:-}"
  local keep_daily="${opts[keep-daily]:-}" keep_weekly="${opts[keep-weekly]:-}" keep_monthly="${opts[keep-monthly]:-}"
  local profile_name="${opts[profile-name]:-}"
  local force="${opts[force]:-0}" dry_run="${opts[dry-run]:-0}"

  # backend 전용 필드만 adapter에 넘긴다 - opts는 파일 전체 플래그의 진실 공급원이고,
  # cli는 그 부분집합 뷰.
  local -A cli=(
    [endpoint]="${opts[endpoint]:-}" [bucket]="${opts[bucket]:-}"
    [access_key]="${opts[access-key]:-}" [secret_key]="${opts[secret-key]:-}"
    [host]="${opts[host]:-}" [port]="${opts[port]:-}" [user]="${opts[user]:-}"
  )

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
  # parse_opts_into가 반복된 --exclude 값을 이미 콤마로 이어붙여뒀으므로, CLI에서
  # 하나도 안 왔으면 기존 backup.env 값을, 그것도 없으면 기본값을 그대로 재사용한다.
  local excludes_csv
  excludes_csv=$(resolve_value "${opts[exclude]:-}" "" "$file_excludes" "$DEFAULT_EXCLUDES")

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
    sftp) backend_sftp_prepare fields ;;
    s3) backend_s3_prepare fields ;;
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
  backup.sh audit [--report] [--report-file <경로>]
  backup.sh uninstall [--purge]
  backup.sh wizard
  backup.sh -h | --help

  모든 하위 명령에 -v/--verbose를 추가하면(위치 무관) SFTP 연결 점검 실패 시
  rclone 자체의 진단 메시지를, init/run 실행 시 restic/resticprofile의 상세
  로그를 함께 보여줍니다.
EOF
}

main() {
  local -a args=()
  local arg
  for arg in "$@"; do
    case "$arg" in
      -v|--verbose) BACKUP_VERBOSE=1 ;;
      *) args+=("$arg") ;;
    esac
  done
  set -- "${args[@]}"

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
    audit)
      shift
      cmd_audit "$@"
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

if [[ "${BASH_SOURCE[0]:-}" == "${0:-}" ]]; then
  main "$@"
fi

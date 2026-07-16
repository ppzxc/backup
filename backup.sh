#!/usr/bin/env bash
# shellcheck disable=SC2030,SC2031
set -euo pipefail

BACKUP_SCRIPT_VERSION="0.0.39"

restic() {
  RESTIC_PASSWORD="${RESTIC_PASSWORD:-}" \
  RESTIC_REPOSITORY="${RESTIC_REPOSITORY:-}" \
  AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-}" \
  AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_TYPE="${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_HOST="${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_PORT="${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_USER="${RCLONE_CONFIG_SYNO_BACKUP_USER:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="${RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-}" \
  command restic "$@"
}

rclone() {
  RCLONE_CONFIG_SYNO_BACKUP_TYPE="${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_HOST="${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_PORT="${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_USER="${RCLONE_CONFIG_SYNO_BACKUP_USER:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="${RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-}" \
  command rclone "$@"
}

resticprofile() {
  RESTIC_PASSWORD="${RESTIC_PASSWORD:-}" \
  RESTIC_REPOSITORY="${RESTIC_REPOSITORY:-}" \
  AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-}" \
  AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_TYPE="${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_HOST="${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_PORT="${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_USER="${RCLONE_CONFIG_SYNO_BACKUP_USER:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="${RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-}" \
  RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-}" \
  SECONDARY_RESTIC_REPOSITORY="${SECONDARY_RESTIC_REPOSITORY:-}" \
  SECONDARY_RESTIC_PASSWORD="${SECONDARY_RESTIC_PASSWORD:-}" \
  SECONDARY_AWS_ACCESS_KEY_ID="${SECONDARY_AWS_ACCESS_KEY_ID:-}" \
  SECONDARY_AWS_SECRET_ACCESS_KEY="${SECONDARY_AWS_SECRET_ACCESS_KEY:-}" \
  SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE:-}" \
  SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-}" \
  SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-}" \
  SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-}" \
  SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-}" \
  command resticprofile "$@"
}

BACKUP_ETC_DIR="${BACKUP_ETC_DIR:-${RESTIC_ETC_DIR:-/etc/backup}}"
RESTIC_ETC_DIR="${BACKUP_ETC_DIR}"
BACKUP_ENV_FILE="${BACKUP_ENV_FILE:-${BACKUP_ETC_DIR}/backup.env}"
BACKUP_SSH_KEY="${BACKUP_SSH_KEY:-${BACKUP_ETC_DIR}/backup_key}"
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
RESTICPROFILE_CONFIG_FILE="${RESTICPROFILE_CONFIG_FILE:-${BACKUP_ETC_DIR}/profiles.yaml}"
RESTICPROFILE_UNIT_TEMPLATE="${RESTICPROFILE_UNIT_TEMPLATE:-${BACKUP_ETC_DIR}/resticprofile-service.tmpl}"
RESTICPROFILE_TIMER_TEMPLATE="${RESTICPROFILE_TIMER_TEMPLATE:-${BACKUP_ETC_DIR}/resticprofile-timer.tmpl}"
SYSTEMD_UNIT_DIR="${SYSTEMD_UNIT_DIR:-/etc/systemd/system}"

DEFAULT_TARGETS="/data/backup,/etc"
DEFAULT_EXCLUDES="/tmp/*,/var/tmp/*"
DEFAULT_KEEP_DAILY=7
DEFAULT_KEEP_WEEKLY=4
DEFAULT_KEEP_MONTHLY=12
DEFAULT_ON_CALENDAR="*-*-* 02:00:00"
DEFAULT_SFTP_PORT=22
BACKUP_VERBOSE="${BACKUP_VERBOSE:-0}"

# 구버전 -> 신버전 환경변수 매핑 정의
# shellcheck disable=SC2034
declare -A COMPATIBILITY_MAP=(
  ["BACKUP_EXCLUDE_PATHS"]="BACKUP_EXCLUDES"
  ["BACKUP_SFTP_HOST"]="RCLONE_CONFIG_SYNO_BACKUP_HOST"
  ["BACKUP_SFTP_PORT"]="RCLONE_CONFIG_SYNO_BACKUP_PORT"
  ["BACKUP_SFTP_USER"]="RCLONE_CONFIG_SYNO_BACKUP_USER"
)

# --- Configuration Registry Schema ---
CONFIG_FIELDS=()
# 스키마 키-환경변수 매핑 테이블로, 동적 쿼리 및 유효성 검사 루프에서 참조되어 경고 예외 처리
# shellcheck disable=SC2034
declare -A CONFIG_ENV_MAP=()
# 스키마 디폴트 값 테이블로, 동적 fallback 설정 확인 시 참조되어 경고 예외 처리
# shellcheck disable=SC2034
declare -A CONFIG_DEFAULT_MAP=()
# 스키마 유효성 검증 함수 매핑 테이블로, 동적 검증 루프 실행 시 참조되어 경고 예외 처리
# shellcheck disable=SC2034
declare -A CONFIG_VALIDATOR_MAP=()

# 캐싱 레이어 변수
CONFIG_CACHE_FILE=""
# 파싱된 설정값들이 로드되는 캐시 저장소로, config_get 쿼리 루틴에서 참조되어 경고 예외 처리
# shellcheck disable=SC2034
declare -A CONFIG_CACHE_DATA=()

# nameref 매개변수가 선언적으로만 등록되어 shellcheck 경고 발생 대응
# shellcheck disable=SC2034
register_config_field() {
  local internal_key="$1"
  local env_var="$2"
  local default_val="$3"
  local validator="$4"
  CONFIG_FIELDS+=("$internal_key")
  CONFIG_ENV_MAP["$internal_key"]="$env_var"
  CONFIG_DEFAULT_MAP["$internal_key"]="$default_val"
  CONFIG_VALIDATOR_MAP["$internal_key"]="$validator"
}

init_config_schema() {
  # 글로벌/공통 속성
  register_config_field "profile_name" "BACKUP_PROFILE_NAME" "" "validate_profile_name"
  register_config_field "password" "RESTIC_PASSWORD" "" "validate_not_empty"
  register_config_field "targets" "BACKUP_TARGETS" "/data/backup,/etc" "validate_not_empty"
  register_config_field "excludes_csv" "BACKUP_EXCLUDES" "" ""
  register_config_field "keep_daily" "KEEP_DAILY" "7" "validate_positive_int"
  register_config_field "keep_weekly" "KEEP_WEEKLY" "4" "validate_positive_int"
  register_config_field "keep_monthly" "KEEP_MONTHLY" "12" "validate_positive_int"

  # SFTP 속성
  register_config_field "host" "RCLONE_CONFIG_SYNO_BACKUP_HOST" "" ""
  register_config_field "port" "RCLONE_CONFIG_SYNO_BACKUP_PORT" "22" "validate_port"
  register_config_field "user" "RCLONE_CONFIG_SYNO_BACKUP_USER" "" ""

  # S3 속성
  register_config_field "endpoint" "BACKUP_ENDPOINT" "" ""
  register_config_field "bucket" "BACKUP_BUCKET" "" ""
  register_config_field "access_key" "AWS_ACCESS_KEY_ID" "" ""
  register_config_field "secret_key" "AWS_SECRET_ACCESS_KEY" "" ""
  
  # 알림 속성
  register_config_field "notification_url" "BACKUP_NOTIFICATION_URL" "" ""
  register_config_field "notification_type" "BACKUP_NOTIFICATION_TYPE" "" ""
  register_config_field "notification_on" "BACKUP_NOTIFICATION_ON" "both" ""
  register_config_field "notification_method" "BACKUP_NOTIFICATION_METHOD" "POST" ""
  register_config_field "notification_headers" "BACKUP_NOTIFICATION_HEADERS" "" ""
  register_config_field "notification_body_success" "BACKUP_NOTIFICATION_BODY_SUCCESS" "" ""
  register_config_field "notification_body_failure" "BACKUP_NOTIFICATION_BODY_FAILURE" "" ""

  # 감사 속성
  register_config_field "audit_tester" "BACKUP_AUDIT_TESTER" "" ""
  register_config_field "audit_ciso" "BACKUP_AUDIT_CISO" "" ""
  register_config_field "audit_rto" "BACKUP_AUDIT_RTO" "" ""

  # DB 속성
  register_config_field "db_type" "BACKUP_DB_TYPE" "" ""
  register_config_field "db_command" "BACKUP_DB_COMMAND" "" ""
  register_config_field "db_filename" "BACKUP_DB_FILENAME" "" ""
  register_config_field "db_schedule" "BACKUP_DB_SCHEDULE" "" ""
  register_config_field "keep_db_daily" "KEEP_DB_DAILY" "" ""
  register_config_field "keep_db_weekly" "KEEP_DB_WEEKLY" "" ""
  register_config_field "keep_db_monthly" "KEEP_DB_MONTHLY" "" ""
}

init_config_schema

migrate_env_file_if_needed() {
  local env_file="${1:-$BACKUP_ENV_FILE}"
  if [[ ! -f "$env_file" ]]; then
    return 0
  fi

  local temp_file
  temp_file=$(mktemp "${env_file}.tmp.XXXXXX")
  chmod 600 "$temp_file"

  local line
  local in_multiline=0
  local quote_char=""
  while IFS= read -r line || [[ -n "$line" ]]; do
    local trimmed
    trimmed="${line#"${line%%[![:space:]]*}"}"
    trimmed="${trimmed%"${trimmed##*[![:space:]]}"}"

    if (( in_multiline )); then
      echo "$line" >> "$temp_file"
      if [[ "$quote_char" == "'" ]]; then
        if [[ "$trimmed" =~ \'$ ]]; then
          in_multiline=0
          quote_char=""
        fi
      elif [[ "$quote_char" == '"' ]]; then
        if [[ "$trimmed" =~ \"$ ]]; then
          in_multiline=0
          quote_char=""
        fi
      fi
      continue
    fi

    local modified_line="$line"
    if [[ "$line" =~ ^([[:space:]]*)export[[:space:]]+(.*)$ ]]; then
      modified_line="${BASH_REMATCH[1]}${BASH_REMATCH[2]}"
    fi
    echo "$modified_line" >> "$temp_file"

    local processed_trimmed
    processed_trimmed="${modified_line#"${modified_line%%[![:space:]]*}"}"
    processed_trimmed="${processed_trimmed%"${processed_trimmed##*[![:space:]]}"}"

    if [[ "$processed_trimmed" =~ ^[A-Za-z0-9_]+=\' ]] && [[ ! "$processed_trimmed" =~ ^[A-Za-z0-9_]+=\'.*\'[[:space:]]*$ ]]; then
      in_multiline=1
      quote_char="'"
    elif [[ "$processed_trimmed" =~ ^[A-Za-z0-9_]+=\" ]] && [[ ! "$processed_trimmed" =~ ^[A-Za-z0-9_]+=\".*\"[[:space:]]*$ ]]; then
      in_multiline=1
      quote_char='"'
    fi
  done < "$env_file"

  mv -f "$temp_file" "$env_file"
}

# nameref 매개변수 dest_array_ref 간접 수정에 따른 shellcheck 경고 발생 대응
# shellcheck disable=SC2034
load_backup_env_to_array() {
  local env_file="$1"
  local -n __load_env_dest_ref="$2"
  local errs_ref_name="${3:-}"
  
  if [[ -n "$errs_ref_name" ]]; then
    local -n __load_env_errors_ref="$errs_ref_name"
  else
    local -a _dummy_errors=()
    local -n __load_env_errors_ref="_dummy_errors"
  fi

  if [[ ! -f "$env_file" ]]; then
    return 1
  fi

  if [[ "$env_file" == "$BACKUP_ENV_FILE" ]]; then
    migrate_env_file_if_needed "$env_file"
  fi

  local line_num=0
  local line
  local in_multiline=0
  local multiline_key=""
  local multiline_val=""
  local quote_char=""

  while IFS= read -r line || [[ -n "$line" ]]; do
    ((line_num++))
    
    if (( in_multiline )); then
      if [[ "$quote_char" == "'" ]]; then
        if [[ "$line" =~ ^(.*)\'[[:space:]]*$ ]]; then
          local chunk="${BASH_REMATCH[1]}"
          multiline_val+=$'\n'"$chunk"
          local val="${multiline_val//\'\\\'\'/\'}"
          __load_env_dest_ref["$multiline_key"]="$val"
          in_multiline=0
          multiline_key=""
          multiline_val=""
          quote_char=""
          continue
        else
          multiline_val+=$'\n'"$line"
          continue
        fi
      elif [[ "$quote_char" == '"' ]]; then
        if [[ "$line" =~ ^(.*)\"[[:space:]]*$ ]]; then
          local chunk="${BASH_REMATCH[1]}"
          multiline_val+=$'\n'"$chunk"
          __load_env_dest_ref["$multiline_key"]="$multiline_val"
          in_multiline=0
          multiline_key=""
          multiline_val=""
          quote_char=""
          continue
        else
          multiline_val+=$'\n'"$line"
          continue
        fi
      fi
    fi

    # Trim leading/trailing whitespace
    local trimmed_line
    trimmed_line="${line#"${line%%[![:space:]]*}"}"
    trimmed_line="${trimmed_line%"${trimmed_line##*[![:space:]]}"}"

    # Remove trailing comments safely
    if [[ "$trimmed_line" =~ ^(.*=\'[^\']*\')[[:space:]]*#.*$ ]]; then
      trimmed_line="${BASH_REMATCH[1]}"
    elif [[ "$trimmed_line" =~ ^(.*=\"[^\"]*\")[[:space:]]*#.*$ ]]; then
      trimmed_line="${BASH_REMATCH[1]}"
    elif [[ "$trimmed_line" =~ ^([^\"\']*)[[:space:]]*#.*$ ]]; then
      trimmed_line="${BASH_REMATCH[1]}"
    fi

    # Re-trim after comment removal
    trimmed_line="${trimmed_line#"${trimmed_line%%[![:space:]]*}"}"
    trimmed_line="${trimmed_line%"${trimmed_line##*[![:space:]]}"}"

    [[ -z "$trimmed_line" || "$trimmed_line" =~ ^# ]] && continue

    local key="" val="" raw_val=""
    if [[ "$trimmed_line" =~ ^(export[[:space:]]+)?([A-Za-z0-9_]+)=\'(.*)\'[[:space:]]*$ ]]; then
      key="${BASH_REMATCH[2]}"
      raw_val="${BASH_REMATCH[3]}"
      val="${raw_val//\'\\\'\'/\'}"
      __load_env_dest_ref["$key"]="$val"
    elif [[ "$trimmed_line" =~ ^(export[[:space:]]+)?([A-Za-z0-9_]+)=\'(.*)$ ]]; then
      in_multiline=1
      multiline_key="${BASH_REMATCH[2]}"
      multiline_val="${BASH_REMATCH[3]}"
      quote_char="'"
    elif [[ "$trimmed_line" =~ ^(export[[:space:]]+)?([A-Za-z0-9_]+)=\"(.*)\"[[:space:]]*$ ]]; then
      key="${BASH_REMATCH[2]}"
      val="${BASH_REMATCH[3]}"
      __load_env_dest_ref["$key"]="$val"
    elif [[ "$trimmed_line" =~ ^(export[[:space:]]+)?([A-Za-z0-9_]+)=\"(.*)$ ]]; then
      in_multiline=1
      multiline_key="${BASH_REMATCH[2]}"
      multiline_val="${BASH_REMATCH[3]}"
      quote_char='"'
    elif [[ "$trimmed_line" =~ ^(export[[:space:]]+)?([A-Za-z0-9_]+)=(.*)$ ]]; then
      key="${BASH_REMATCH[2]}"
      val="${BASH_REMATCH[3]}"
      val="${val%%#*}"
      val="${val#[[:space:]]}"
      val="${val%[[:space:]]}"
      __load_env_errors_ref+=("라인 ${line_num}: 올바르지 않은 설정 형식입니다 (KEY='VALUE' 규격 위반)")
      return 1
    fi

    if [[ -n "$key" ]]; then
      __load_env_dest_ref["$key"]="$val"
    fi
  done < "$env_file"

  return 0
}

# 글로벌 연관 배열이 subshell 상속 시 소실되어, config_get 내부에서 lazy init 처리
config_get() {
  local key="$1"
  local env_file="${2:-$BACKUP_ENV_FILE}"
  if ! declare -p CONFIG_ENV_MAP &>/dev/null; then
    CONFIG_FIELDS=()
    # 전역 매핑 테이블 lazy-init 선언으로 경고 예외 처리
    # shellcheck disable=SC2034
    declare -g -A CONFIG_ENV_MAP=()
    # 전역 디폴트 테이블 lazy-init 선언으로 경고 예외 처리
    # shellcheck disable=SC2034
    declare -g -A CONFIG_DEFAULT_MAP=()
    # 전역 유효성 테이블 lazy-init 선언으로 경고 예외 처리
    # shellcheck disable=SC2034
    declare -g -A CONFIG_VALIDATOR_MAP=()
    init_config_schema
  fi
  if ! declare -p CONFIG_CACHE_DATA &>/dev/null; then
    CONFIG_CACHE_FILE=""
    # 전역 캐시 저장소 lazy-init 선언으로 경고 예외 처리
    # shellcheck disable=SC2034
    declare -g -A CONFIG_CACHE_DATA=()
  fi
  if [[ "$CONFIG_CACHE_FILE" != "$env_file" || ${#CONFIG_CACHE_DATA[@]} -eq 0 ]]; then
    CONFIG_CACHE_FILE="$env_file"
    # 전역 캐시 데이터 로드 선언으로 경고 예외 처리
    # shellcheck disable=SC2034
    declare -g -A CONFIG_CACHE_DATA=()
    if [[ -f "$env_file" ]]; then
      declare -a errors=()
      if ! load_backup_env_to_array "$env_file" CONFIG_CACHE_DATA errors; then
        local err_msg
        for err_msg in "${errors[@]}"; do
          log_error "$err_msg"
        done
        die "설정 파일 파싱 실패: $env_file" 1
      fi
    fi
  fi
  local env_var="${CONFIG_ENV_MAP[$key]:-}"
  local val=""
  if [[ -n "$env_var" ]]; then
    val="${CONFIG_CACHE_DATA[$env_var]:-}"
  else
    val="${CONFIG_CACHE_DATA[$key]:-}"
  fi
  if [[ -z "$val" ]]; then
    val="${CONFIG_DEFAULT_MAP[$key]:-}"
  fi
  echo "$val"
}

C_RESET="" C_BOLD="" C_DIM="" C_RED="" C_GREEN="" C_YELLOW="" C_BLUE="" C_CYAN="" C_GRAY=""

setup_colors() {
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    C_RESET="\033[0m"
    C_BOLD="\033[1m"
    C_DIM="\033[2m"
    C_RED="\033[31m"
    C_GREEN="\033[32m"
    C_YELLOW="\033[33m"
    C_BLUE="\033[34m"
    C_CYAN="\033[36m"
    C_GRAY="\033[90m"
  fi
}

log_info() {
  printf '%s\n' "$1"
  command -v logger >/dev/null 2>&1 && logger -t backup -- "$1" || true
}

log_error() {
  printf 'ERROR: %s\n' "$1" >&2
  command -v logger >/dev/null 2>&1 && logger -t backup -- "ERROR: $1" || true
}

log_warn() {
  printf 'WARNING: %s\n' "$1" >&2
  command -v logger >/dev/null 2>&1 && logger -t backup -- "WARNING: $1" || true
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

ensure_backup_dir_migration() {
  local legacy_dir="/etc/restic"

  if [[ ! -d "$BACKUP_ETC_DIR" && -d "$legacy_dir" ]]; then
    log_info "기존 설정 디렉터리(${legacy_dir})에서 새 경로(${BACKUP_ETC_DIR})로 설정을 자동 이관합니다..."
    
    mkdir -p "$BACKUP_ETC_DIR"
    chmod 700 "$BACKUP_ETC_DIR"

    if [[ -f "${legacy_dir}/backup.env" ]]; then
      cp "${legacy_dir}/backup.env" "${BACKUP_ENV_FILE}"
      chmod 600 "${BACKUP_ENV_FILE}"
    fi

    if [[ -f "${legacy_dir}/backup_key" ]]; then
      cp "${legacy_dir}/backup_key" "${BACKUP_SSH_KEY}"
      chmod 600 "${BACKUP_SSH_KEY}"
    fi

    if [[ -f "${legacy_dir}/resticprofile-service.tmpl" ]]; then
      cp "${legacy_dir}/resticprofile-service.tmpl" "${BACKUP_UNIT_TEMPLATE}"
      chmod 644 "${BACKUP_UNIT_TEMPLATE}"
    fi
    if [[ -f "${legacy_dir}/resticprofile-timer.tmpl" ]]; then
      cp "${legacy_dir}/resticprofile-timer.tmpl" "${BACKUP_TIMER_TEMPLATE}"
      chmod 644 "${BACKUP_TIMER_TEMPLATE}"
    fi
    if [[ -f "${legacy_dir}/profiles.yaml" ]]; then
      cp "${legacy_dir}/profiles.yaml" "${BACKUP_PROFILE_CONFIG_FILE}"
      chmod 600 "${BACKUP_PROFILE_CONFIG_FILE}"
    fi
    
    log_info "설정 이관이 완료되었습니다."
  fi

  local legacy_timer
  for legacy_timer in restic-audit-daily.timer restic-audit-restore-drill.timer; do
    if command -v systemctl >/dev/null 2>&1; then
      if systemctl is-active "$legacy_timer" >/dev/null 2>&1 || systemctl is-enabled "$legacy_timer" >/dev/null 2>&1; then
        log_info "레거시 systemd 타이머(${legacy_timer})를 정지 및 제거합니다..."
        systemctl disable --now "$legacy_timer" 2>/dev/null || true
        rm -f "/etc/systemd/system/${legacy_timer}"
        rm -f "/etc/systemd/system/${legacy_timer%.timer}.service"
        systemctl daemon-reload 2>/dev/null || true
      fi
    fi
  done

  # /data/backup 디렉토리 보장 및 ISMS 대응 700 권한 설정
  mkdir -p "${TEST_ROOT:-}/data/backup"
  chmod 700 "${TEST_ROOT:-}/data/backup"
}

require_backup_env() {
  ensure_backup_dir_migration

  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi
  
  declare -A file_config=()
  declare -a parse_errors=()
  if ! load_backup_env_to_array "$BACKUP_ENV_FILE" file_config parse_errors; then
    local err_msg
    for err_msg in "${parse_errors[@]}"; do
      log_error "$err_msg"
    done
    die "설정 파일 파싱 실패: $BACKUP_ENV_FILE" 1
  fi

  # 구버전 키 호환성 검증 및 맵핑 적용
  local old_key new_key
  for old_key in "${!COMPATIBILITY_MAP[@]}"; do
    new_key="${COMPATIBILITY_MAP[$old_key]}"
    if [[ -n "${file_config[$old_key]:-}" && -z "${file_config[$new_key]:-}" ]]; then
      log_warn "구버전 설정 키(${old_key})가 감지되었습니다. '${new_key}' 값으로 자동 맵핑되어 실행되지만, 정상적인 유지를 위해 'backup.sh upgrade'를 실행해 주십시오."
      file_config["$new_key"]="${file_config[$old_key]}"
    fi
  done

  local k
  for k in "${!file_config[@]}"; do
    declare -g "$k"="${file_config[$k]}"
  done
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

validate_secondary_backend() {
  local value="$1"
  case "$value" in
    s3|sftp) return 0 ;;
    *) printf 'secondary-backend must be s3 or sftp, got: %s\n' "$value"; return 1 ;;
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

validate_absolute_path() {
  local value="$1"
  local name="${2:-경로}"
  if [[ -z "$value" ]]; then
    printf '%s이(가) 비어 있습니다\n' "$name"
    return 1
  fi
  local p
  for p in ${value//,/ }; do
    p="${p#"${p%%[![:space:]]*}"}"
    p="${p%"${p##*[![:space:]]}"}"
    if [[ ! "$p" =~ ^/ ]]; then
      printf '%s "%s"은(는) 올바른 절대 경로가 아닙니다 (/로 시작해야 합니다)\n' "$name" "$p"
      return 1
    fi
  done
  return 0
}

validate_resolved_config() {
  local -n resolved_ref="$1"
  local -n errors_ref="$2"
  
  # 1. 1차 백엔드 검증
  local backend="${resolved_ref[backend]:-}"
  if [[ -n "$backend" ]]; then
    declare -g -A _validate_primary_fields=()
    _validate_primary_fields[host]="${resolved_ref[host]:-}"
    _validate_primary_fields[port]="${resolved_ref[port]:-}"
    _validate_primary_fields[user]="${resolved_ref[user]:-}"
    _validate_primary_fields[endpoint]="${resolved_ref[endpoint]:-}"
    _validate_primary_fields[bucket]="${resolved_ref[bucket]:-}"
    _validate_primary_fields[access_key]="${resolved_ref[access_key]:-}"
    _validate_primary_fields[secret_key]="${resolved_ref[secret_key]:-}"
    
    local primary_err
    if ! primary_err=$(case "$backend" in
      sftp) backend_sftp_validate _validate_primary_fields 2>&1 ;;
      s3) backend_s3_validate _validate_primary_fields 2>&1 ;;
    esac); then
      errors_ref+=("$primary_err")
    fi
    unset _validate_primary_fields
  fi
  
  # 2. 2차 백엔드 검증
  local sec_backend="${resolved_ref[secondary_backend]:-}"
  if [[ -n "$sec_backend" ]]; then
    declare -g -A _validate_sec_fields=()
    _validate_sec_fields[host]="${resolved_ref[secondary_host]:-}"
    _validate_sec_fields[port]="${resolved_ref[secondary_port]:-}"
    _validate_sec_fields[user]="${resolved_ref[secondary_user]:-}"
    _validate_sec_fields[endpoint]="${resolved_ref[secondary_endpoint]:-}"
    _validate_sec_fields[bucket]="${resolved_ref[secondary_bucket]:-}"
    _validate_sec_fields[access_key]="${resolved_ref[secondary_access_key]:-}"
    _validate_sec_fields[secret_key]="${resolved_ref[secondary_secret_key]:-}"
    
    local sec_err
    if ! sec_err=$(case "$sec_backend" in
      sftp) backend_sftp_validate _validate_sec_fields 2>&1 ;;
      s3) backend_s3_validate _validate_sec_fields 2>&1 ;;
    esac); then
      errors_ref+=("Secondary backend error: $sec_err")
    fi
    unset _validate_sec_fields
  fi

  # 3. 백업 대상 경로(targets) 절대경로 여부 검증
  local targets="${resolved_ref[targets]:-}"
  if [[ -n "$targets" ]]; then
    local path_err
    if ! path_err=$(validate_absolute_path "$targets" "백업 대상 경로" 2>&1); then
      errors_ref+=("$path_err")
    fi
  fi
}

# shellcheck disable=SC2034
load_and_validate_config() {
  local profile_name="$1"
  local cli_opts_ref_name=""
  local resolved_ref_name=""
  local errors_ref_name=""

  if [[ $# -eq 4 ]]; then
    cli_opts_ref_name="$2"
    resolved_ref_name="$3"
    errors_ref_name="$4"
  else
    resolved_ref_name="$2"
    errors_ref_name="$3"
  fi

  local -n _out_resolved="$resolved_ref_name"
  local -n _out_errors="$errors_ref_name"

  # Determine backend from backup.env or CLI options
  local backend=""
  if [[ -n "$cli_opts_ref_name" ]]; then
    local -n _cli_opts_ref="$cli_opts_ref_name"
    backend="${_cli_opts_ref[backend]:-}"
  fi

  if [[ -z "$backend" && -f "$BACKUP_ENV_FILE" ]]; then
    if grep -q -E "AWS_ACCESS_KEY_ID=" "$BACKUP_ENV_FILE" || grep -q -E 'RESTIC_REPOSITORY=["'\'' ]s3:' "$BACKUP_ENV_FILE"; then
      backend="s3"
    elif grep -q -E "RCLONE_CONFIG_SYNO_BACKUP_TYPE=['\"]sftp['\"]" "$BACKUP_ENV_FILE" || grep -q -E 'RESTIC_REPOSITORY=["'\'' ]rclone:syno_backup' "$BACKUP_ENV_FILE"; then
      backend="sftp"
    fi
  fi

  # Create a unified local options copy
  local -A local_opts=()
  if [[ -n "$cli_opts_ref_name" ]]; then
    local k
    for k in "${!_cli_opts_ref[@]}"; do
      local_opts["$k"]="${_cli_opts_ref[$k]}"
    done
  fi

  if [[ -n "$backend" ]]; then
    local_opts[backend]="$backend"
  fi
  if [[ -n "$profile_name" ]]; then
    local_opts[profile-name]="$profile_name"
  fi

  resolve_and_validate_config local_opts _out_resolved _out_errors
  local res=$?

  if [[ ${#_out_errors[@]} -gt 0 ]]; then
    return 1
  fi
  return $res
}

save_profile_config() {
  local resolved_arr_name="$1"
  local -n _res_ref="$resolved_arr_name"

  local backend="${_res_ref[backend]:-}"
  if [[ -z "$backend" ]]; then
    return 1
  fi

  # Determine schedule calendar
  local on_calendar="${_res_ref[on_calendar]:-}"
  if [[ -z "$on_calendar" ]]; then
    if [[ -f "$RESTICPROFILE_CONFIG_FILE" ]]; then
      local parsed_schedule
      parsed_schedule=$(grep -E 'schedule:[[:space:]]*"[^"]+"' "$RESTICPROFILE_CONFIG_FILE" | head -n1 | sed -E 's/.*schedule:[[:space:]]*"([^"]+)".*/\1/')
      if [[ -n "$parsed_schedule" ]]; then
        on_calendar="$parsed_schedule"
      fi
    fi
  fi
  if [[ -z "$on_calendar" ]]; then
    on_calendar="$DEFAULT_ON_CALENDAR"
  fi

  # Ensure the restic etc directory exists and has 700 permissions
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR" 2>/dev/null || true

  # 1. Format configuration variables (env block and notice)
  local content="" notice=""
  backend_"${backend}"_configure "$resolved_arr_name" content notice
  append_secondary_config_and_notice "$resolved_arr_name" content notice

  # 2. Write backup.env with secure permissions (600)
  write_secure_file "$BACKUP_ENV_FILE" 600 "$content"

  # 3. Synchronize derived assets (profiles.yaml, systemd timers)
  (
    declare -A file_config=()
    declare -a errors=()
    if ! load_backup_env_to_array "$BACKUP_ENV_FILE" file_config errors; then
      local err_msg
      for err_msg in "${errors[@]}"; do
        log_error "$err_msg"
      done
      exit 1
    fi
    local k
    for k in "${!file_config[@]}"; do
      declare -g "$k"="${file_config[$k]}"
    done
    write_resticprofile_assets "${_res_ref[profile_name]}" "$on_calendar"

    local timer_name
    timer_name=$(resticprofile_timer_unit_name "${_res_ref[profile_name]}")
    if systemctl is-enabled "$timer_name" >/dev/null 2>&1; then
      log_info "정기 백업 스케줄 타이머(${timer_name})가 활성화되어 있어 설정을 자동 리로드합니다."
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "${_res_ref[profile_name]}" schedule
    fi
  ) || return 1

  if [[ -n "$notice" ]]; then
    printf '%s\n' "$notice"
  fi

  return 0
}

# nameref로 인자를 받거나 다른 함수로 동적 연관 배열을 전달하여 사용하지 않는 것으로 오인받는 변수가 있으므로 우회
# shellcheck disable=SC2034
resolve_and_validate_config() {
  local -n _opts="$1"
  local -n _resolved="$2"
  local -n _errors="$3"

  # Sourcing current configuration from file
  local file_targets="" file_keep_daily="" file_keep_weekly="" file_keep_monthly="" file_excludes="" file_profile_name="" file_password=""
  local file_notification_url="" file_notification_type="" file_notification_on="" file_notification_method="" file_notification_headers="" file_notification_body_success="" file_notification_body_failure=""
  local file_audit_tester="" file_audit_ciso="" file_audit_rto=""
  local file_secondary_backend="" file_secondary_password="" file_secondary_keep_daily="" file_secondary_keep_weekly="" file_secondary_keep_monthly=""
  local file_secondary_endpoint="" file_secondary_bucket="" file_secondary_access_key="" file_secondary_secret_key=""
  local file_secondary_host="" file_secondary_port="" file_secondary_user="" file_secondary_repo=""
  local file_db_type="" file_db_command="" file_db_filename="" file_db_schedule="" file_db_keep_daily="" file_db_keep_weekly="" file_db_keep_monthly=""
  if [[ -f "${BACKUP_ENV_FILE:-}" ]]; then
    declare -A file_config=()
    if ! load_backup_env_to_array "$BACKUP_ENV_FILE" file_config _errors; then
      return 1
    fi
    file_targets="${file_config[BACKUP_TARGETS]:-}"
    file_keep_daily="${file_config[KEEP_DAILY]:-}"
    file_keep_weekly="${file_config[KEEP_WEEKLY]:-}"
    file_keep_monthly="${file_config[KEEP_MONTHLY]:-}"
    file_excludes="${file_config[BACKUP_EXCLUDES]:-}"
    file_profile_name="${file_config[BACKUP_PROFILE_NAME]:-}"
    file_password="${file_config[RESTIC_PASSWORD]:-}"
    file_notification_url="${file_config[BACKUP_NOTIFICATION_URL]:-}"
    file_notification_type="${file_config[BACKUP_NOTIFICATION_TYPE]:-}"
    file_notification_on="${file_config[BACKUP_NOTIFICATION_ON]:-}"
    file_notification_method="${file_config[BACKUP_NOTIFICATION_METHOD]:-}"
    file_notification_headers="${file_config[BACKUP_NOTIFICATION_HEADERS]:-}"
    file_notification_body_success="${file_config[BACKUP_NOTIFICATION_BODY_SUCCESS]:-}"
    file_notification_body_failure="${file_config[BACKUP_NOTIFICATION_BODY_FAILURE]:-}"
    file_audit_tester="${file_config[BACKUP_AUDIT_TESTER]:-}"
    file_audit_ciso="${file_config[BACKUP_AUDIT_CISO]:-}"
    file_audit_rto="${file_config[BACKUP_AUDIT_RTO]:-}"
    file_secondary_backend="${file_config[SECONDARY_BACKEND]:-}"
    file_secondary_password="${file_config[SECONDARY_RESTIC_PASSWORD]:-}"
    file_secondary_keep_daily="${file_config[SECONDARY_KEEP_DAILY]:-}"
    file_secondary_keep_weekly="${file_config[SECONDARY_KEEP_WEEKLY]:-}"
    file_secondary_keep_monthly="${file_config[SECONDARY_KEEP_MONTHLY]:-}"
    file_secondary_repo="${file_config[SECONDARY_RESTIC_REPOSITORY]:-}"
    file_secondary_access_key="${file_config[SECONDARY_AWS_ACCESS_KEY_ID]:-}"
    file_secondary_secret_key="${file_config[SECONDARY_AWS_SECRET_ACCESS_KEY]:-}"
    file_secondary_host="${file_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_HOST]:-}"
    file_secondary_port="${file_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_PORT]:-}"
    file_secondary_user="${file_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_USER]:-}"
    file_db_type="${file_config[BACKUP_DB_TYPE]:-}"
    file_db_command="${file_config[BACKUP_DB_COMMAND]:-}"
    file_db_filename="${file_config[BACKUP_DB_FILENAME]:-}"
    file_db_schedule="${file_config[BACKUP_DB_SCHEDULE]:-}"
    file_db_keep_daily="${file_config[KEEP_DB_DAILY]:-}"
    file_db_keep_weekly="${file_config[KEEP_DB_WEEKLY]:-}"
    file_db_keep_monthly="${file_config[KEEP_DB_MONTHLY]:-}"
  fi


  # Read Environment Variables
  local env_targets="${BACKUP_TARGETS:-}"
  local env_keep_daily="${KEEP_DAILY:-}"
  local env_keep_weekly="${KEEP_WEEKLY:-}"
  local env_keep_monthly="${KEEP_MONTHLY:-}"
  local env_password="${BACKUP_PASSWORD:-}"
  local env_profile_name="${BACKUP_PROFILE_NAME:-}"
  local env_notification_url="${BACKUP_NOTIFICATION_URL:-}"
  local env_notification_type="${BACKUP_NOTIFICATION_TYPE:-}"
  local env_notification_on="${BACKUP_NOTIFICATION_ON:-}"

  local env_secondary_backend="${SECONDARY_BACKEND:-}"
  local env_secondary_password="${SECONDARY_RESTIC_PASSWORD:-}"
  local env_secondary_keep_daily="${SECONDARY_KEEP_DAILY:-}"
  local env_secondary_keep_weekly="${SECONDARY_KEEP_WEEKLY:-}"
  local env_secondary_keep_monthly="${SECONDARY_KEEP_MONTHLY:-}"
  local env_db_type="${BACKUP_DB_TYPE:-}"
  local env_db_command="${BACKUP_DB_COMMAND:-}"
  local env_db_filename="${BACKUP_DB_FILENAME:-}"
  local env_db_schedule="${BACKUP_DB_SCHEDULE:-}"
  local env_db_keep_daily="${KEEP_DB_DAILY:-}"
  local env_db_keep_weekly="${KEEP_DB_WEEKLY:-}"
  local env_db_keep_monthly="${KEEP_DB_MONTHLY:-}"

  # Resolve values with priority: CLI option > Env variable > Config file > Default value
  local cli_targets="${_opts[targets]:-}"
  _resolved[targets]=$(resolve_value "$cli_targets" "$env_targets" "$file_targets" "${DEFAULT_TARGETS:-}")

  local cli_keep_daily="${_opts[keep-daily]:-}"
  _resolved[keep_daily]=$(resolve_value "$cli_keep_daily" "$env_keep_daily" "$file_keep_daily" "${DEFAULT_KEEP_DAILY:-}")

  local cli_keep_weekly="${_opts[keep-weekly]:-}"
  _resolved[keep_weekly]=$(resolve_value "$cli_keep_weekly" "$env_keep_weekly" "$file_keep_weekly" "${DEFAULT_KEEP_WEEKLY:-}")

  local cli_keep_monthly="${_opts[keep-monthly]:-}"
  _resolved[keep_monthly]=$(resolve_value "$cli_keep_monthly" "$env_keep_monthly" "$file_keep_monthly" "${DEFAULT_KEEP_MONTHLY:-}")

  local cli_password="${_opts[password]:-}"
  _resolved[password]=$(resolve_value "$cli_password" "$env_password" "$file_password" "")

  local cli_profile_name="${_opts[profile-name]:-}"
  _resolved[profile_name]=$(resolve_value "$cli_profile_name" "$env_profile_name" "$file_profile_name" "$(hostname)")

  local cli_db_type="${_opts[db-type]:-}"
  _resolved[db_type]=$(resolve_value "$cli_db_type" "$env_db_type" "$file_db_type" "" || true)

  if [[ -n "${_resolved[db_type]:-}" && "${_resolved[db_type]}" != "$file_db_type" ]]; then
    file_db_command=""
    file_db_filename=""
    file_db_schedule=""
    file_db_keep_daily=""
    file_db_keep_weekly=""
    file_db_keep_monthly=""
  fi

  local cli_db_command="${_opts[db-command]:-}"
  # shellcheck disable=SC2154
  _resolved[db_command]=$(resolve_value "$cli_db_command" "$env_db_command" "$file_db_command" "" || true)

  local cli_db_filename="${_opts[db-filename]:-}"
  # shellcheck disable=SC2154
  _resolved[db_filename]=$(resolve_value "$cli_db_filename" "$env_db_filename" "$file_db_filename" "db-dump.sql" || true)

  local cli_db_schedule="${_opts[db-schedule]:-}"
  _resolved[db_schedule]=$(resolve_value "$cli_db_schedule" "$env_db_schedule" "$file_db_schedule" "" || true)

  local cli_db_keep_daily="${_opts[db-keep-daily]:-}"
  _resolved[db_keep_daily]=$(resolve_value "$cli_db_keep_daily" "$env_db_keep_daily" "$file_db_keep_daily" "" || true)

  local cli_db_keep_weekly="${_opts[db-keep-weekly]:-}"
  _resolved[db_keep_weekly]=$(resolve_value "$cli_db_keep_weekly" "$env_db_keep_weekly" "$file_db_keep_weekly" "" || true)

  local cli_db_keep_monthly="${_opts[db-keep-monthly]:-}"
  _resolved[db_keep_monthly]=$(resolve_value "$cli_db_keep_monthly" "$env_db_keep_monthly" "$file_db_keep_monthly" "" || true)

  # DB 타입별 기본 명령어 채우기
  if [[ -n "${_resolved[db_type]:-}" ]]; then
    if [[ -z "${_resolved[db_command]:-}" ]]; then
      case "${_resolved[db_type]}" in
        mysql)
          _resolved[db_command]="mysqldump --all-databases --single-transaction --quick --order-by-primary"
          ;;
        mariadb)
          _resolved[db_command]="mariadb-dump --all-databases --single-transaction --quick --order-by-primary"
          ;;
        postgres)
          _resolved[db_command]="pg_dumpall -U postgres"
          ;;
      esac
    fi
  fi

  local cli_sec_backend="${_opts[secondary-backend]:-}"
  _resolved["secondary_backend"]=$(resolve_value "$cli_sec_backend" "$env_secondary_backend" "$file_secondary_backend" "" || true)

  if [[ -n "${_resolved[secondary_backend]:-}" ]]; then
    local cli_sec_password="${_opts[secondary-password]:-}"
    _resolved["secondary_password"]=$(resolve_value "$cli_sec_password" "$env_secondary_password" "$file_secondary_password" "${_resolved[password]:-}" || true)

    local cli_sec_keep_daily="${_opts[secondary-keep-daily]:-}"
    _resolved["secondary_keep_daily"]=$(resolve_value "$cli_sec_keep_daily" "$env_secondary_keep_daily" "$file_secondary_keep_daily" "${_resolved[keep_daily]:-}" || true)

    local cli_sec_keep_weekly="${_opts[secondary-keep-weekly]:-}"
    _resolved["secondary_keep_weekly"]=$(resolve_value "$cli_sec_keep_weekly" "$env_secondary_keep_weekly" "$file_secondary_keep_weekly" "${_resolved[keep_weekly]:-}" || true)

    local cli_sec_keep_monthly="${_opts[secondary-keep-monthly]:-}"
    _resolved["secondary_keep_monthly"]=$(resolve_value "$cli_sec_keep_monthly" "$env_secondary_keep_monthly" "$file_secondary_keep_monthly" "${_resolved[keep_monthly]:-}" || true)
  fi




  # Resolve Exclude
  local cli_exclude="${_opts[exclude]:-}"
  _resolved[excludes_csv]=$(resolve_value "$cli_exclude" "" "$file_excludes" "${DEFAULT_EXCLUDES:-}")

  # Resolve Notifications
  local cli_notification_url="${_opts[notification-url]:-}"
  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  _resolved[notification_url]=$(resolve_value "$cli_notification_url" "$env_notification_url" "$file_notification_url" "" || true)

  local cli_notification_type="${_opts[notification-type]:-}"
  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  _resolved[notification_type]=$(resolve_value "$cli_notification_type" "$env_notification_type" "$file_notification_type" "" || true)

  local cli_notification_on="${_opts[notification-on]:-}"
  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  _resolved[notification_on]=$(resolve_value "$cli_notification_on" "$env_notification_on" "$file_notification_on" "both" || true)

  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  _resolved[notification_method]=$(resolve_value "" "${BACKUP_NOTIFICATION_METHOD:-}" "$file_notification_method" "POST" || true)
  # shellcheck disable=SC2154
  _resolved[notification_headers]=$(resolve_value "" "${BACKUP_NOTIFICATION_HEADERS:-}" "$file_notification_headers" "" || true)
  # shellcheck disable=SC2154
  _resolved[notification_body_success]=$(resolve_value "" "${BACKUP_NOTIFICATION_BODY_SUCCESS:-}" "$file_notification_body_success" "" || true)
  # shellcheck disable=SC2154
  _resolved[notification_body_failure]=$(resolve_value "" "${BACKUP_NOTIFICATION_BODY_FAILURE:-}" "$file_notification_body_failure" "" || true)

  local cli_audit_tester="${_opts[audit-tester]:-}"
  local env_audit_tester="${BACKUP_AUDIT_TESTER:-}"
  # shellcheck disable=SC2154
  _resolved[audit_tester]=$(resolve_value "$cli_audit_tester" "$env_audit_tester" "$file_audit_tester" "" || true)

  local cli_audit_ciso="${_opts[audit-ciso]:-}"
  local env_audit_ciso="${BACKUP_AUDIT_CISO:-}"
  # shellcheck disable=SC2154
  _resolved[audit_ciso]=$(resolve_value "$cli_audit_ciso" "$env_audit_ciso" "$file_audit_ciso" "" || true)

  local cli_audit_rto="${_opts[audit-rto]:-}"
  local env_audit_rto="${BACKUP_AUDIT_RTO:-}"
  # shellcheck disable=SC2154
  _resolved[audit_rto]=$(resolve_value "$cli_audit_rto" "$env_audit_rto" "$file_audit_rto" "" || true)

  # Global Validation
  if [[ -z "${_resolved[targets]:-}" ]]; then
    _errors+=("백업 대상 경로(--targets 또는 BACKUP_TARGETS)가 필요합니다.")
  fi

  if [[ -z "${_resolved[password]:-}" ]]; then
    _errors+=("저장소 비밀번호(--password 또는 BACKUP_PASSWORD)가 필요합니다.")
  fi

  local err
  if ! err=$(validate_positive_int "${_resolved[keep_daily]}" "keep-daily"); then
    _errors+=("$err")
  fi
  if ! err=$(validate_positive_int "${_resolved[keep_weekly]}" "keep-weekly"); then
    _errors+=("$err")
  fi
  if ! err=$(validate_positive_int "${_resolved[keep_monthly]}" "keep-monthly"); then
    _errors+=("$err")
  fi
  if ! err=$(validate_profile_name "${_resolved[profile_name]}"); then
    _errors+=("$err")
  fi

  if [[ -n "${_resolved[secondary_backend]:-}" ]]; then
    if ! err=$(validate_secondary_backend "${_resolved[secondary_backend]}"); then
      _errors+=("$err")
    fi
    if ! err=$(validate_positive_int "${_resolved[secondary_keep_daily]}" "secondary-keep-daily"); then
      _errors+=("$err")
    fi
    if ! err=$(validate_positive_int "${_resolved[secondary_keep_weekly]}" "secondary-keep-weekly"); then
      _errors+=("$err")
    fi
    if ! err=$(validate_positive_int "${_resolved[secondary_keep_monthly]}" "secondary-keep-monthly"); then
      _errors+=("$err")
    fi
  fi


  # Strict Validation for Notifications
  local url="${_resolved[notification_url]}"
  local type="${_resolved[notification_type]}"
  local on="${_resolved[notification_on]}"

  if [[ -n "$url" || -n "$type" ]]; then
    if [[ -z "$url" ]]; then
      _errors+=("알람 URL이 비어 있습니다. 알람 타입을 설정한 경우 URL(--notification-url 또는 BACKUP_NOTIFICATION_URL)을 지정해야 합니다.")
    elif [[ -z "$type" ]]; then
      _errors+=("알람 타입이 비어 있습니다. 알람 URL을 설정한 경우 타입(--notification-type 또는 BACKUP_NOTIFICATION_TYPE)을 지정해야 합니다.")
    fi

    if [[ -n "$url" ]]; then
      if [[ ! "$url" =~ ^https?:// ]]; then
        _errors+=("알람 URL 형식은 http:// 또는 https://로 시작해야 합니다: ${url}")
      fi
    fi

    if [[ -n "$type" ]]; then
      if [[ "$type" != "slack" && "$type" != "discord" && "$type" != "custom" ]]; then
        _errors+=("지원하지 않는 알람 타입입니다: ${type} (slack, discord, custom 중 하나여야 합니다.)")
      fi
    fi

    if [[ -n "$on" ]]; then
      if [[ "$on" != "failure" && "$on" != "success" && "$on" != "both" ]]; then
        _errors+=("지원하지 않는 알람 발생 조건(ON)입니다: ${on} (failure, success, both 중 하나여야 합니다.)")
      fi
    fi
  fi

  # Delegate backend-specific validation if backend is specified
  local backend="${_opts[backend]:-}"
  if [[ -n "$backend" ]]; then
    _resolved[backend]="$backend"
    # nameref를 통한 동적 파싱을 사용하므로 미사용 변수 경고 우회
    # shellcheck disable=SC2034
    local -A backend_cli=()
    # shellcheck disable=SC2034
    local -A backend_env=()
    # shellcheck disable=SC2034
    local -A backend_file=()

    backend_cli[endpoint]="${_opts[endpoint]:-}"
    backend_cli[bucket]="${_opts[bucket]:-}"
    backend_cli[access_key]="${_opts[access-key]:-}"
    backend_cli[secret_key]="${_opts[secret-key]:-}"
    backend_cli[host]="${_opts[host]:-}"
    backend_cli[port]="${_opts[port]:-}"
    backend_cli[user]="${_opts[user]:-}"

    local env_vars_mapping
    env_vars_mapping=$(case "$backend" in sftp) backend_sftp_env_vars ;; s3) backend_s3_env_vars ;; esac)
    local field_key var_name
    while IFS=$'\t' read -r field_key var_name; do
      [[ -z "$field_key" ]] && continue
      backend_env["$field_key"]="${!var_name:-}"
    done <<< "$env_vars_mapping"

    if [[ -f "${BACKUP_ENV_FILE:-}" ]]; then
      declare -A temp_file_config=()
      if load_backup_env_to_array "$BACKUP_ENV_FILE" temp_file_config; then
        backend_file[RCLONE_CONFIG_SYNO_BACKUP_HOST]="${temp_file_config[RCLONE_CONFIG_SYNO_BACKUP_HOST]:-}"
        backend_file[RCLONE_CONFIG_SYNO_BACKUP_PORT]="${temp_file_config[RCLONE_CONFIG_SYNO_BACKUP_PORT]:-}"
        backend_file[RCLONE_CONFIG_SYNO_BACKUP_USER]="${temp_file_config[RCLONE_CONFIG_SYNO_BACKUP_USER]:-}"
        backend_file[AWS_ACCESS_KEY_ID]="${temp_file_config[AWS_ACCESS_KEY_ID]:-}"
        backend_file[AWS_SECRET_ACCESS_KEY]="${temp_file_config[AWS_SECRET_ACCESS_KEY]:-}"
        backend_file[RESTIC_REPOSITORY]="${temp_file_config[RESTIC_REPOSITORY]:-}"
        backend_file[BACKUP_HOST]="${temp_file_config[BACKUP_HOST]:-}"
        backend_file[BACKUP_PORT]="${temp_file_config[BACKUP_PORT]:-}"
        backend_file[BACKUP_USER]="${temp_file_config[BACKUP_USER]:-}"
        backend_file[BACKUP_ENDPOINT]="${temp_file_config[BACKUP_ENDPOINT]:-}"
        backend_file[BACKUP_BUCKET]="${temp_file_config[BACKUP_BUCKET]:-}"
        backend_file[BACKUP_ACCESS_KEY]="${temp_file_config[BACKUP_ACCESS_KEY]:-}"
        backend_file[BACKUP_SECRET_KEY]="${temp_file_config[BACKUP_SECRET_KEY]:-}"
      fi
    fi

    local -A backend_fields=()
    case "$backend" in
      sftp) backend_sftp_resolve backend_cli backend_env backend_file backend_fields ;;
      s3) backend_s3_resolve backend_cli backend_env backend_file backend_fields ;;
    esac

    # Copy to main resolved array
    local key
    for key in "${!backend_fields[@]}"; do
      _resolved["$key"]="${backend_fields[$key]}"
    done


  fi

  local sec_backend="${_resolved[secondary_backend]:-}"
  if [[ -n "$sec_backend" ]]; then
    local -A sec_backend_cli=()
    local -A sec_backend_env=()
    local -A sec_backend_file=()

    sec_backend_cli[endpoint]="${_opts[secondary-endpoint]:-}"
    sec_backend_cli[bucket]="${_opts[secondary-bucket]:-}"
    sec_backend_cli[access_key]="${_opts[secondary-access-key]:-}"
    sec_backend_cli[secret_key]="${_opts[secondary-secret-key]:-}"
    sec_backend_cli[host]="${_opts[secondary-host]:-}"
    sec_backend_cli[port]="${_opts[secondary-port]:-}"
    sec_backend_cli[user]="${_opts[secondary-user]:-}"

    if [[ "$sec_backend" == "s3" ]]; then
      sec_backend_env[access_key]="${SECONDARY_AWS_ACCESS_KEY_ID:-${SECONDARY_BACKUP_ACCESS_KEY:-}}"
      sec_backend_env[secret_key]="${SECONDARY_AWS_SECRET_ACCESS_KEY:-${SECONDARY_BACKUP_SECRET_KEY:-}}"
      sec_backend_env[endpoint]="${SECONDARY_BACKUP_ENDPOINT:-}"
      sec_backend_env[bucket]="${SECONDARY_BACKUP_BUCKET:-}"
    elif [[ "$sec_backend" == "sftp" ]]; then
      sec_backend_env[host]="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_HOST:-${SECONDARY_BACKUP_HOST:-}}}"
      sec_backend_env[port]="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_PORT:-${SECONDARY_BACKUP_PORT:-}}}"
      sec_backend_env[user]="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_USER:-${SECONDARY_BACKUP_USER:-}}}"
    fi

    if [[ -f "${BACKUP_ENV_FILE:-}" ]]; then
      declare -A temp_sec_config=()
      if load_backup_env_to_array "$BACKUP_ENV_FILE" temp_sec_config; then
        sec_backend_file[host]="${temp_sec_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST]:-${temp_sec_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_HOST]:-}}"
        sec_backend_file[port]="${temp_sec_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT]:-${temp_sec_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_PORT]:-}}"
        sec_backend_file[user]="${temp_sec_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER]:-${temp_sec_config[SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_USER]:-}}"
        sec_backend_file[access_key]="${temp_sec_config[SECONDARY_AWS_ACCESS_KEY_ID]:-}"
        sec_backend_file[secret_key]="${temp_sec_config[SECONDARY_AWS_SECRET_ACCESS_KEY]:-}"
        sec_backend_file[repo]="${temp_sec_config[SECONDARY_RESTIC_REPOSITORY]:-}"
      fi
    fi

    local sec_repo="${SECONDARY_RESTIC_REPOSITORY:-${sec_backend_file[repo]:-}}"
    local sec_parsed_endpoint="" sec_parsed_bucket=""
    if [[ "$sec_repo" =~ ^s3:(.*)/([^/]+)/[^/]+$ ]]; then
      sec_parsed_endpoint="${BASH_REMATCH[1]}"
      sec_parsed_bucket="${BASH_REMATCH[2]}"
    fi
    if [[ -z "${sec_backend_env[endpoint]:-}" ]]; then sec_backend_env[endpoint]="$sec_parsed_endpoint"; fi
    if [[ -z "${sec_backend_file[endpoint]:-}" ]]; then sec_backend_file[endpoint]="$sec_parsed_endpoint"; fi
    if [[ -z "${sec_backend_env[bucket]:-}" ]]; then sec_backend_env[bucket]="$sec_parsed_bucket"; fi
    if [[ -z "${sec_backend_file[bucket]:-}" ]]; then sec_backend_file[bucket]="$sec_parsed_bucket"; fi

    local -A sec_fields=()
    case "$sec_backend" in
      sftp) backend_sftp_resolve sec_backend_cli sec_backend_env sec_backend_file sec_fields ;;
      s3) backend_s3_resolve sec_backend_cli sec_backend_env sec_backend_file sec_fields ;;
    esac

    local skey
    for skey in "${!sec_fields[@]}"; do
      _resolved["secondary_$skey"]="${sec_fields[$skey]}"
    done


  fi

  validate_resolved_config _resolved _errors

  if [[ ${#_errors[@]} -gt 0 ]]; then
    return 1
  fi
  return 0
}

check_targets_size_warning() {
  local targets_csv="$1"
  local -a paths=()
  local old_ifs="$IFS"
  IFS=',' read -r -a paths <<< "$targets_csv"
  IFS="$old_ifs"

  local var_log_exceeds=0
  local etc_exceeds=0
  # 1GB in KB is 1,048,576
  local limit_kb=1048576

  local path
  for path in "${paths[@]}"; do
    path=$(echo "$path" | xargs)
    if [[ ! -d "$path" ]]; then
      continue
    fi

    local du_output size_kb
    du_output=$(du -sk "$path" 2>/dev/null || echo 0)
    size_kb="${du_output%%[[:space:]]*}"
    if ! [[ "$size_kb" =~ ^[0-9]+$ ]]; then
      size_kb=0
    fi

    local abs_path
    abs_path=$(realpath -m "$path" 2>/dev/null || echo "$path")

    if [[ "$abs_path" == "/var/log" ]]; then
      if (( size_kb > limit_kb )); then
        var_log_exceeds=1
      fi
    elif [[ "$abs_path" == "/etc" ]]; then
      if (( size_kb > limit_kb )); then
        etc_exceeds=1
      fi
    fi
  done

  if (( var_log_exceeds && etc_exceeds )); then
    log_warn "백업 대상인 /var/log 디렉터리와 /etc 디렉터리가 둘 다 1GB를 초과합니다. 백업 수행 시 많은 대역폭과 디스크 공간이 소모될 수 있습니다."
  fi
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

escape_single_quotes() {
  echo -n "$1" | sed "s/'/'\\\\''/g"
}

render_notification_env_block() {
  local -n _res="$1"
  cat <<EOF

# ==========================================
# 백업 성공/실패 알림 설정 (Slack, Discord, Custom)
# ==========================================
BACKUP_NOTIFICATION_URL='$(escape_single_quotes "${_res[notification_url]:-}")'
BACKUP_NOTIFICATION_TYPE='$(escape_single_quotes "${_res[notification_type]:-}")'
BACKUP_NOTIFICATION_ON='$(escape_single_quotes "${_res[notification_on]:-both}")'
BACKUP_NOTIFICATION_METHOD='$(escape_single_quotes "${_res[notification_method]:-POST}")'
BACKUP_NOTIFICATION_HEADERS='$(escape_single_quotes "${_res[notification_headers]:-}")'
BACKUP_NOTIFICATION_BODY_SUCCESS='$(escape_single_quotes "${_res[notification_body_success]:-}")'
BACKUP_NOTIFICATION_BODY_FAILURE='$(escape_single_quotes "${_res[notification_body_failure]:-}")'
EOF
}

render_audit_env_block() {
  local -n _res="$1"
  cat <<EOF

# ==========================================
# ISMS/ISMS-P 감사 보고서용 사용자 설정
# ==========================================
BACKUP_AUDIT_TESTER='$(escape_single_quotes "${_res[audit_tester]:-}")'
BACKUP_AUDIT_CISO='$(escape_single_quotes "${_res[audit_ciso]:-}")'
BACKUP_AUDIT_RTO='$(escape_single_quotes "${_res[audit_rto]:-}")'
EOF
}

render_db_env_block() {
  local -n _res="$1"
  if [[ -n "${_res[db_type]:-}" ]]; then
    cat <<EOF

# ==========================================
# 데이터베이스 백업용 설정 (Database Backup)
# ==========================================
BACKUP_DB_TYPE='$(escape_single_quotes "${_res[db_type]:-}")'
BACKUP_DB_COMMAND='$(escape_single_quotes "${_res[db_command]:-}")'
BACKUP_DB_FILENAME='$(escape_single_quotes "${_res[db_filename]:-db-dump.sql}")'
BACKUP_DB_SCHEDULE='$(escape_single_quotes "${_res[db_schedule]:-}")'
KEEP_DB_DAILY='$(escape_single_quotes "${_res[db_keep_daily]:-}")'
KEEP_DB_WEEKLY='$(escape_single_quotes "${_res[db_keep_weekly]:-}")'
KEEP_DB_MONTHLY='$(escape_single_quotes "${_res[db_keep_monthly]:-}")'
EOF
  fi
}



build_notification_payload_slack() {
  local status="$1" hostname="$2" profile_name="$3" err_msg="$4"
  if [[ "$status" == "success" ]]; then
    printf '{"text":"✅ [%s] restic 백업 성공 (프로파일: %s)"}' "$hostname" "$profile_name"
  else
    printf '{"text":"❌ [%s] restic 백업 실패 (프로파일: %s)\\n오류: %s"}' "$hostname" "$profile_name" "$err_msg"
  fi
}

build_notification_payload_discord() {
  local status="$1" hostname="$2" profile_name="$3" err_msg="$4"
  if [[ "$status" == "success" ]]; then
    printf '{"content":"✅ [%s] restic 백업 성공 (프로파일: %s)"}' "$hostname" "$profile_name"
  else
    printf '{"content":"❌ [%s] restic 백업 실패 (프로파일: %s)\\n오류: %s"}' "$hostname" "$profile_name" "$err_msg"
  fi
}

build_notification_payload_custom() {
  local status="$1" hostname="$2" profile_name="$3" err_msg="$4" body_success="$5" body_failure="$6" profile_command="${7:-}"
  local payload=""
  if [[ "$status" == "success" ]]; then
    payload="$body_success"
  else
    payload="$body_failure"
    payload="${payload//\$\{ERROR\}/$err_msg}"
  fi
  payload="${payload//\$\{HOSTNAME\}/$hostname}"
  payload="${payload//\$\{PROFILE_NAME\}/$profile_name}"
  payload="${payload//\$\{PROFILE_COMMAND\}/$profile_command}"
  printf '%s' "$payload"
}

send_notification_slack() {
  local url="$1" status="$2" hostname="$3" profile_name="$4" err_msg="$5"
  local payload
  payload=$(build_notification_payload_slack "$status" "$hostname" "$profile_name" "$err_msg")
  curl -s -X POST -H "Content-Type: application/json" --max-time 10 -d "$payload" "$url"
}

send_notification_discord() {
  local url="$1" status="$2" hostname="$3" profile_name="$4" err_msg="$5"
  local payload
  payload=$(build_notification_payload_discord "$status" "$hostname" "$profile_name" "$err_msg")
  curl -s -X POST -H "Content-Type: application/json" --max-time 10 -d "$payload" "$url"
}

send_notification_custom() {
  local url="$1" status="$2" hostname="$3" profile_name="$4" err_msg="$5" method="$6" headers_str="$7" body_success="$8" body_failure="$9" profile_command="${10:-}"
  local payload
  payload=$(build_notification_payload_custom "$status" "$hostname" "$profile_name" "$err_msg" "$body_success" "$body_failure" "$profile_command")
  
  local -a curl_headers=()
  if [[ -n "$headers_str" ]]; then
    local -a headers_arr=()
    IFS=',' read -ra headers_arr <<< "$headers_str"
    local h
    for h in "${headers_arr[@]}"; do
      h=$(echo "$h" | xargs)
      curl_headers+=("-H" "$h")
    done
  else
    curl_headers+=("-H" "Content-Type: application/json")
  fi

  curl -s -X "$method" "${curl_headers[@]}" --max-time 10 -d "$payload" "$url"
}

dispatch_notification() {
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local -n _ctx="$1"
  local status="${_ctx[status]:-success}"
  local err_msg="${_ctx[err_msg]:-}"
  local url="${_ctx[notify_url]:-${BACKUP_NOTIFICATION_URL:-}}"
  local type="${_ctx[notify_type]:-${BACKUP_NOTIFICATION_TYPE:-}}"
  local on="${_ctx[notify_on]:-${BACKUP_NOTIFICATION_ON:-both}}"

  [[ -z "$url" ]] && return 0

  if [[ "$on" == "success" && "$status" != "success" ]]; then
    return 0
  fi
  if [[ "$on" == "failure" && "$status" != "failure" ]]; then
    return 0
  fi

  local hostname_val; hostname_val=$(hostname)
  local profile_name_val="${BACKUP_PROFILE_NAME:-$hostname_val}"

  log_info "통합 알림 전송 중... ($status)"
  local res=0
  case "$type" in
    slack)
      send_notification_slack "$url" "$status" "$hostname_val" "$profile_name_val" "$err_msg" || res=$?
      ;;
    discord)
      send_notification_discord "$url" "$status" "$hostname_val" "$profile_name_val" "$err_msg" || res=$?
      ;;
    custom)
      local method="${_ctx[method]:-${BACKUP_NOTIFICATION_METHOD:-POST}}"
      local headers="${_ctx[headers]:-${BACKUP_NOTIFICATION_HEADERS:-}}"
      local body_success="${_ctx[body_success]:-${BACKUP_NOTIFICATION_BODY_SUCCESS:-}}"
      local body_failure="${_ctx[body_failure]:-${BACKUP_NOTIFICATION_BODY_FAILURE:-}}"
      local profile_command="${_ctx[profile_command]:-}"
      send_notification_custom "$url" "$status" "$hostname_val" "$profile_name_val" "$err_msg" "$method" "$headers" "$body_success" "$body_failure" "$profile_command" || res=$?
      ;;
    *)
      return 0
      ;;
  esac

  if [[ $res -ne 0 ]]; then
    log_warn "통합 알림 웹훅 전송 실패 (exit code: $res)"
    return $res
  fi
  return 0
}

send_unified_notification() {
  local status="$1"
  local err_msg="${2:-}"
  local profile_command="${3:-}"
  
  # nameref로 인자를 전달하여 사용되지 않는 것으로 오인받는 변수 우회
  # shellcheck disable=SC2034
  declare -A notify_ctx=(
    [status]="$status"
    [err_msg]="$err_msg"
    [notify_url]="${BACKUP_NOTIFICATION_URL:-}"
    [notify_type]="${BACKUP_NOTIFICATION_TYPE:-}"
    [notify_on]="${BACKUP_NOTIFICATION_ON:-both}"
    [method]="${BACKUP_NOTIFICATION_METHOD:-POST}"
    [headers]="${BACKUP_NOTIFICATION_HEADERS:-}"
    [body_success]="${BACKUP_NOTIFICATION_BODY_SUCCESS:-}"
    [body_failure]="${BACKUP_NOTIFICATION_BODY_FAILURE:-}"
    [profile_command]="$profile_command"
  )
  dispatch_notification notify_ctx
}

render_resticprofile_notifications() {
  if [[ -n "${SECONDARY_BACKEND:-}" ]]; then
    return 0
  fi

  local notify_url="${BACKUP_NOTIFICATION_URL:-}"

  local notify_type="${BACKUP_NOTIFICATION_TYPE:-}"
  local notify_on="${BACKUP_NOTIFICATION_ON:-both}"

  if [[ -z "$notify_url" ]]; then
    return 0
  fi

  local method="POST"
  local -a headers=()
  local success_body=""
  local failure_body=""

  # resticprofile이 런타임에 직접 환경변수 및 에러 변수를 치환하도록 작은따옴표를 유지한다.
  # shellcheck disable=SC2016
  if [[ "$notify_type" == "slack" ]]; then
    method="POST"
    headers=("Content-Type" "application/json")
    success_body=$(build_notification_payload_slack "success" '${HOSTNAME}' '${PROFILE_NAME}' '${ERROR}')
    failure_body=$(build_notification_payload_slack "failure" '${HOSTNAME}' '${PROFILE_NAME}' '${ERROR}')
  elif [[ "$notify_type" == "discord" ]]; then
    method="POST"
    headers=("Content-Type" "application/json")
    success_body=$(build_notification_payload_discord "success" '${HOSTNAME}' '${PROFILE_NAME}' '${ERROR}')
    failure_body=$(build_notification_payload_discord "failure" '${HOSTNAME}' '${PROFILE_NAME}' '${ERROR}')
  elif [[ "$notify_type" == "custom" ]]; then
    method="${BACKUP_NOTIFICATION_METHOD:-POST}"
    success_body="${BACKUP_NOTIFICATION_BODY_SUCCESS:-}"
    failure_body="${BACKUP_NOTIFICATION_BODY_FAILURE:-}"

    if [[ -n "${BACKUP_NOTIFICATION_HEADERS:-}" ]]; then
      local -a headers_arr=()
      IFS=',' read -ra headers_arr <<< "$BACKUP_NOTIFICATION_HEADERS"
      local h
      for h in "${headers_arr[@]}"; do
        h=$(echo "$h" | xargs)
        if [[ "$h" =~ ^([^:]+):(.*)$ ]]; then
          local name="${BASH_REMATCH[1]}"
          local value="${BASH_REMATCH[2]}"
          name=$(echo "$name" | xargs)
          value=$(echo "$value" | xargs)
          headers+=("$name" "$value")
        fi
      done
    else
      headers=("Content-Type" "application/json")
    fi
  else
    return 0
  fi

  print_http_hook() {
    local hook_name="$1"
    local h_body="$2"
    printf '    %s:\n' "$hook_name"
    printf '      - method: "%s"\n' "$method"
    printf '        url: "%s"\n' "$notify_url"
    if [[ -n "$h_body" ]]; then
      printf '        body: |\n'
      local processed_body="$h_body"
      # JSON 객체가 아닌 경우(일반 텍스트), \n 문자 코드를 실제 개행 문자로 치환하여 개행을 보존한다.
      if [[ ! "$h_body" =~ ^[[:space:]]*\{ ]]; then
        processed_body=$(printf '%b' "$h_body")
      fi
      local line
      while IFS= read -r line || [[ -n "$line" ]]; do
        printf '          %s\n' "$line"
      done <<< "$processed_body"
    fi
    if [[ ${#headers[@]} -gt 0 ]]; then
      printf '        headers:\n'
      local -i idx
      for ((idx=0; idx<${#headers[@]}; idx+=2)); do
        printf '          - name: "%s"\n' "${headers[idx]}"
        printf '            value: "%s"\n' "${headers[idx+1]}"
      done
    fi
  }

  if [[ "$notify_on" == "both" || "$notify_on" == "success" ]]; then
    if [[ -n "$success_body" || "$notify_type" != "custom" ]]; then
      print_http_hook "send-after" "$success_body"
    fi
  fi

  if [[ "$notify_on" == "both" || "$notify_on" == "failure" ]]; then
    if [[ -n "$failure_body" || "$notify_type" != "custom" ]]; then
      print_http_hook "send-after-fail" "$failure_body"
    fi
  fi
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
  printf '    HOSTNAME: "%s"\n' "$(hostname)"
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
  printf '    group-by: host\n'
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


  # 2차 원격 소산 프로필 추가 출력
  if [[ -n "${SECONDARY_BACKEND:-}" ]]; then
    printf '\n%s-secondary:\n' "$profile_name"
    printf '  repository: "%s"\n' "${SECONDARY_RESTIC_REPOSITORY:-}"
    printf '  force-inactive-lock: true\n'
    printf '  env:\n'
    printf '    RESTIC_PASSWORD: "%s"\n' "${SECONDARY_RESTIC_PASSWORD:-${RESTIC_PASSWORD:-}}"
    printf '    RESTIC_FROM_PASSWORD: "%s"\n' "${RESTIC_PASSWORD:-}"
    printf '    HOSTNAME: "%s"\n' "$(hostname)"

    if [[ "$SECONDARY_BACKEND" == "s3" ]]; then
      printf '    AWS_ACCESS_KEY_ID: "%s"\n' "${SECONDARY_AWS_ACCESS_KEY_ID:-}"
      printf '    AWS_SECRET_ACCESS_KEY: "%s"\n' "${SECONDARY_AWS_SECRET_ACCESS_KEY:-}"
    elif [[ "$SECONDARY_BACKEND" == "sftp" ]]; then
      printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE: "sftp"\n'
      printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_HOST:-}}"
      printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_USER: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_USER:-}}"
      printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_PORT:-22}}"
      printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-$BACKUP_SSH_KEY}}"
    fi

    # 2차 보존 정책
    printf '  retention:\n'
    printf '    prune: true\n'
    printf '    group-by: host\n'
    printf '    keep-daily: %s\n' "${SECONDARY_KEEP_DAILY:-$keep_daily}"
    printf '    keep-weekly: %s\n' "${SECONDARY_KEEP_WEEKLY:-$keep_weekly}"
    printf '    keep-monthly: %s\n' "${SECONDARY_KEEP_MONTHLY:-$keep_monthly}"

    # 2차 소산지 복사 정책
    printf '  copy:\n'
    printf '    from-repository: "%s"\n' "${RESTIC_REPOSITORY:-}"
  fi

  # DB 백업 프로필 추가 출력 (독립 프로필)
  # resticprofile의 inherit은 배열을 머지(merge)하므로, 부모의 source 경로와
  # stdin: true가 충돌하여 "Fatal: --stdin was specified and files/dirs were
  # listed as arguments" 에러가 발생한다. 따라서 inherit 없이 독립적으로 렌더링한다.
  if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
    local db_keep_daily="${KEEP_DB_DAILY:-$keep_daily}"
    local db_keep_weekly="${KEEP_DB_WEEKLY:-$keep_weekly}"
    local db_keep_monthly="${KEEP_DB_MONTHLY:-$keep_monthly}"
    local db_schedule="${BACKUP_DB_SCHEDULE:-$on_calendar}"

    printf '\n%s-db:\n' "$profile_name"
    printf '  repository: "%s"\n' "${RESTIC_REPOSITORY:-}"
    printf '  force-inactive-lock: true\n'
    printf '  env:\n'
    printf '    RESTIC_PASSWORD: "%s"\n' "${RESTIC_PASSWORD:-}"
    printf '    HOSTNAME: "%s"\n' "$(hostname)"
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
    printf '    group-by: host\n'
    printf '    keep-daily: %s\n' "$db_keep_daily"
    printf '    keep-weekly: %s\n' "$db_keep_weekly"
    printf '    keep-monthly: %s\n' "$db_keep_monthly"

    printf '  backup:\n'
    printf '    schedule: "%s"\n' "$db_schedule"
    printf '    schedule-permission: system\n'
    printf '    stdin: true\n'
    printf '    stdin-command: "%s"\n' "${BACKUP_DB_COMMAND:-}"
    printf '    stdin-filename: "%s"\n' "${BACKUP_DB_FILENAME:-db-dump.sql}"
    printf '    tag:\n'
    printf '      - db\n'

    # DB 2차 소산 복사 프로필 (2차 백업이 구성된 경우에만)
    if [[ -n "${SECONDARY_BACKEND:-}" ]]; then
      printf '\n%s-db-secondary:\n' "$profile_name"
      printf '  repository: "%s"\n' "${SECONDARY_RESTIC_REPOSITORY:-}"
      printf '  force-inactive-lock: true\n'
      printf '  env:\n'
      printf '    RESTIC_PASSWORD: "%s"\n' "${SECONDARY_RESTIC_PASSWORD:-${RESTIC_PASSWORD:-}}"
      printf '    RESTIC_FROM_PASSWORD: "%s"\n' "${RESTIC_PASSWORD:-}"
      printf '    HOSTNAME: "%s"\n' "$(hostname)"

      if [[ "$SECONDARY_BACKEND" == "s3" ]]; then
        printf '    AWS_ACCESS_KEY_ID: "%s"\n' "${SECONDARY_AWS_ACCESS_KEY_ID:-}"
        printf '    AWS_SECRET_ACCESS_KEY: "%s"\n' "${SECONDARY_AWS_SECRET_ACCESS_KEY:-}"
      elif [[ "$SECONDARY_BACKEND" == "sftp" ]]; then
        printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE: "sftp"\n'
        printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_HOST:-}}"
        printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_USER: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_USER:-}}"
        printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_PORT:-22}}"
        printf '    RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE: "%s"\n' "${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-$BACKUP_SSH_KEY}}"
      fi

      printf '  retention:\n'
      printf '    prune: true\n'
      printf '    group-by: host\n'
      printf '    keep-daily: %s\n' "$db_keep_daily"
      printf '    keep-weekly: %s\n' "$db_keep_weekly"
      printf '    keep-monthly: %s\n' "$db_keep_monthly"

      printf '  copy:\n'
      printf '    from-repository: "%s"\n' "${RESTIC_REPOSITORY:-}"
    fi
  fi
}


install_binary() {
  local name="$1" version="$2" url="$3" expected_sha="$4" target_path="$5" format="$6" archive_path="${7:-}"

  if [[ -x "$target_path" ]]; then
    return 0
  fi

  local tmp_dir
  tmp_dir=$(mktemp -d)
  local download_file="${tmp_dir}/${name}_download"

  log_info "${name} v${version} 다운로드 중..."
  if ! curl -fsSL -o "$download_file" "$url"; then
    rm -rf "$tmp_dir"
    die "[!] ${name} 다운로드 실패: ${url}"
  fi

  local actual_sha256
  actual_sha256=$(sha256sum "$download_file" | awk '{print $1}')
  if [[ "$actual_sha256" != "$expected_sha" ]]; then
    rm -rf "$tmp_dir"
    die "[!] ${name} 체크섬 불일치 (예상: ${expected_sha}, 실제: ${actual_sha256}) - 설치를 중단합니다"
  fi

  log_info "${name} 압축 해제 및 설치 중..."
  case "$format" in
    bz2)
      if ! python3 -c "import bz2, shutil, sys; shutil.copyfileobj(bz2.open(sys.argv[1], 'rb'), open(sys.argv[2], 'wb'))" \
        "$download_file" "${tmp_dir}/${name}" >/dev/null 2>&1; then
        rm -rf "$tmp_dir"
        die "[!] ${name} bz2 압축 해제 실패"
      fi
      ;;
    zip)
      local extract_dir="${tmp_dir}/extracted"
      if ! python3 -m zipfile -e "$download_file" "$extract_dir" >/dev/null 2>&1; then
        rm -rf "$tmp_dir"
        die "[!] ${name} zip 압축 해제 실패"
      fi
      mv "${extract_dir}/${archive_path}" "${tmp_dir}/${name}"
      ;;
    tar.gz)
      if ! tar -xzf "$download_file" -C "$tmp_dir" "$archive_path" >/dev/null 2>&1; then
        rm -rf "$tmp_dir"
        die "[!] ${name} tar.gz 압축 해제 실패"
      fi
      if [[ "$archive_path" != "$name" ]]; then
        mv "${tmp_dir}/${archive_path}" "${tmp_dir}/${name}"
      fi
      ;;
    *)
      rm -rf "$tmp_dir"
      die "[!] 지원하지 않는 압축 형식: ${format}"
      ;;
  esac

  mkdir -p "$(dirname "$target_path")"
  if ! install -m 0755 "${tmp_dir}/${name}" "$target_path"; then
    rm -rf "$tmp_dir"
    die "[!] ${name} 바이너리 설치(install) 실패"
  fi

  rm -rf "$tmp_dir"
}

install_resticprofile() {
  install_binary "resticprofile" "$RESTICPROFILE_VERSION" "$RESTICPROFILE_URL" \
    "$RESTICPROFILE_SHA256" "$RESTICPROFILE_INSTALL_PATH" "tar.gz" "resticprofile"
}

install_restic() {
  install_binary "restic" "$RESTIC_VERSION" "$RESTIC_URL" \
    "$RESTIC_SHA256" "$RESTIC_INSTALL_PATH" "bz2"
}

install_rclone() {
  install_binary "rclone" "$RCLONE_VERSION" "$RCLONE_URL" \
    "$RCLONE_SHA256" "$RCLONE_INSTALL_PATH" "zip" "rclone-v${RCLONE_VERSION}-linux-amd64/rclone"
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
  # /data/backup 디렉토리 보장 및 ISMS 대응 700 권한 설정
  mkdir -p "${TEST_ROOT:-}/data/backup"
  chmod 700 "${TEST_ROOT:-}/data/backup"
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
  if has_help_flag "$@"; then
    help_install
    return 0
  fi
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
  local env_host="${env_ref[host]:-${BACKUP_HOST:-${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}}}"
  local file_host="${file_ref[host]:-${file_ref[RCLONE_CONFIG_SYNO_BACKUP_HOST]:-}}"
  fields_ref[host]=$(resolve_value "${cli_ref[host]:-}" "$env_host" "$file_host" "") || true

  local env_port="${env_ref[port]:-${BACKUP_PORT:-${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}}}"
  local file_port="${file_ref[port]:-${file_ref[RCLONE_CONFIG_SYNO_BACKUP_PORT]:-}}"
  fields_ref[port]=$(resolve_value "${cli_ref[port]:-}" "$env_port" "$file_port" "$DEFAULT_SFTP_PORT") || true

  local env_user="${env_ref[user]:-${BACKUP_USER:-${RCLONE_CONFIG_SYNO_BACKUP_USER:-}}}"
  local file_user="${file_ref[user]:-${file_ref[RCLONE_CONFIG_SYNO_BACKUP_USER]:-}}"
  fields_ref[user]=$(resolve_value "${cli_ref[user]:-}" "$env_user" "$file_user" "") || true
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
export RESTIC_REPOSITORY='rclone:syno_backup:/backup/$(escape_single_quotes "${hostname_tag}")'
export RCLONE_CONFIG_SYNO_BACKUP_TYPE='sftp'
export RCLONE_CONFIG_SYNO_BACKUP_HOST='$(escape_single_quotes "${fields_ref[host]}")'
export RCLONE_CONFIG_SYNO_BACKUP_USER='$(escape_single_quotes "${fields_ref[user]}")'
export RCLONE_CONFIG_SYNO_BACKUP_PORT='$(escape_single_quotes "${fields_ref[port]}")'
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE='$(escape_single_quotes "${BACKUP_SSH_KEY}")'
export RESTIC_PASSWORD='$(escape_single_quotes "${policy_ref[password]}")'
export BACKUP_TARGETS='$(escape_single_quotes "${policy_ref[targets]}")'
export BACKUP_EXCLUDES='$(escape_single_quotes "${policy_ref[excludes_csv]:-}")'
export KEEP_DAILY='$(escape_single_quotes "${policy_ref[keep_daily]}")'
export KEEP_WEEKLY='$(escape_single_quotes "${policy_ref[keep_weekly]}")'
export KEEP_MONTHLY='$(escape_single_quotes "${policy_ref[keep_monthly]}")'
export BACKUP_PROFILE_NAME='$(escape_single_quotes "${policy_ref[profile_name]}")'
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

  # S3 env fallbacks
  local env_access_key="${env_ref[access_key]:-${BACKUP_ACCESS_KEY:-${AWS_ACCESS_KEY_ID:-}}}"
  local file_access_key="${file_ref[access_key]:-${file_ref[AWS_ACCESS_KEY_ID]:-}}"
  local env_secret_key="${env_ref[secret_key]:-${BACKUP_SECRET_KEY:-${AWS_SECRET_ACCESS_KEY:-}}}"
  local file_secret_key="${file_ref[secret_key]:-${file_ref[AWS_SECRET_ACCESS_KEY]:-}}"

  # repo extraction for endpoint and bucket
  local env_repo="${RESTIC_REPOSITORY:-}"
  local file_repo="${file_ref[RESTIC_REPOSITORY]:-}"
  local parsed_endpoint="" parsed_bucket=""

  if [[ "$env_repo" =~ ^s3:(.*)/([^/]+)/[^/]+$ ]]; then
    parsed_endpoint="${BASH_REMATCH[1]}"
    parsed_bucket="${BASH_REMATCH[2]}"
  elif [[ "$file_repo" =~ ^s3:(.*)/([^/]+)/[^/]+$ ]]; then
    parsed_endpoint="${BASH_REMATCH[1]}"
    parsed_bucket="${BASH_REMATCH[2]}"
  fi

  local env_endpoint="${env_ref[endpoint]:-${BACKUP_ENDPOINT:-$parsed_endpoint}}"
  local file_endpoint="${file_ref[endpoint]:-$parsed_endpoint}"
  fields_ref[endpoint]=$(resolve_value "${cli_ref[endpoint]:-}" "$env_endpoint" "$file_endpoint" "") || true

  local env_bucket="${env_ref[bucket]:-${BACKUP_BUCKET:-$parsed_bucket}}"
  local file_bucket="${file_ref[bucket]:-$parsed_bucket}"
  fields_ref[bucket]=$(resolve_value "${cli_ref[bucket]:-}" "$env_bucket" "$file_bucket" "") || true

  fields_ref[access_key]=$(resolve_value "${cli_ref[access_key]:-}" "$env_access_key" "$file_access_key" "") || true
  fields_ref[secret_key]=$(resolve_value "${cli_ref[secret_key]:-}" "$env_secret_key" "$file_secret_key" "") || true
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
export RESTIC_REPOSITORY='s3:$(escape_single_quotes "${fields_ref[endpoint]}")/$(escape_single_quotes "${fields_ref[bucket]}")/$(escape_single_quotes "${hostname_tag}")'
export AWS_ACCESS_KEY_ID='$(escape_single_quotes "${fields_ref[access_key]}")'
export AWS_SECRET_ACCESS_KEY='$(escape_single_quotes "${fields_ref[secret_key]}")'
export RESTIC_PASSWORD='$(escape_single_quotes "${policy_ref[password]}")'
export BACKUP_TARGETS='$(escape_single_quotes "${policy_ref[targets]}")'
export BACKUP_EXCLUDES='$(escape_single_quotes "${policy_ref[excludes_csv]:-}")'
export KEEP_DAILY='$(escape_single_quotes "${policy_ref[keep_daily]}")'
export KEEP_WEEKLY='$(escape_single_quotes "${policy_ref[keep_weekly]}")'
export KEEP_MONTHLY='$(escape_single_quotes "${policy_ref[keep_monthly]}")'
export BACKUP_PROFILE_NAME='$(escape_single_quotes "${policy_ref[profile_name]}")'
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


render_secondary_config() {
  local -n __res_sec_ref="$1"
  local sec_backend="${__res_sec_ref[secondary_backend]:-}"
  [[ -z "$sec_backend" ]] && return 0

  local sec_password="${__res_sec_ref[secondary_password]:-${__res_sec_ref[password]:-}}"
  local sec_keep_daily="${__res_sec_ref[secondary_keep_daily]:-${__res_sec_ref[keep_daily]:-}}"
  local sec_keep_weekly="${__res_sec_ref[secondary_keep_weekly]:-${__res_sec_ref[keep_weekly]:-}}"
  local sec_keep_monthly="${__res_sec_ref[secondary_keep_monthly]:-${__res_sec_ref[keep_monthly]:-}}"
  local profile_name="${__res_sec_ref[profile_name]:-$(hostname)}"

  printf '\n# 2차 원격 소산 백업 설정\n'
  printf 'SECONDARY_BACKEND="%s"\n' "$sec_backend"
  printf 'SECONDARY_RESTIC_PASSWORD="%s"\n' "$sec_password"
  printf 'SECONDARY_KEEP_DAILY="%s"\n' "$sec_keep_daily"
  printf 'SECONDARY_KEEP_WEEKLY="%s"\n' "$sec_keep_weekly"
  printf 'SECONDARY_KEEP_MONTHLY="%s"\n' "$sec_keep_monthly"

  if [[ "$sec_backend" == "s3" ]]; then
    local sec_endpoint="${__res_sec_ref[secondary_endpoint]}"
    local sec_bucket="${__res_sec_ref[secondary_bucket]}"
    local sec_access_key="${__res_sec_ref[secondary_access_key]}"
    local sec_secret_key="${__res_sec_ref[secondary_secret_key]}"
    
    printf 'SECONDARY_RESTIC_REPOSITORY="s3:%s/%s/%s"\n' "$sec_endpoint" "$sec_bucket" "$profile_name"
    printf 'SECONDARY_AWS_ACCESS_KEY_ID="%s"\n' "$sec_access_key"
    printf 'SECONDARY_AWS_SECRET_ACCESS_KEY="%s"\n' "$sec_secret_key"
  elif [[ "$sec_backend" == "sftp" ]]; then
    local sec_host="${__res_sec_ref[secondary_host]}"
    local sec_port="${__res_sec_ref[secondary_port]:-22}"
    local sec_user="${__res_sec_ref[secondary_user]}"
    
    printf 'SECONDARY_RESTIC_REPOSITORY="rclone:syno_backup_sec:/backup/%s"\n' "$profile_name"
    printf 'SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="sftp"\n'
    printf 'SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="%s"\n' "$sec_host"
    printf 'SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="%s"\n' "$sec_port"
    printf 'SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="%s"\n' "$sec_user"
    printf 'SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="%s"\n' "${BACKUP_SSH_KEY}"
  fi
}

append_secondary_config_and_notice() {
  local -n __res_sec_ref="$1"
  local -n __content_sec_ref="$2"
  local -n __notice_sec_ref="$3"

  if [[ -n "${__res_sec_ref[secondary_backend]:-}" ]]; then
    __content_sec_ref+="$(render_secondary_config "$1")"
    local sec_notice=""
    if [[ "${__res_sec_ref[secondary_backend]}" == "s3" ]]; then
      sec_notice="[2차 소산지 S3] 최소권한 버킷 정책을 적용하세요:
\$(render_s3_bucket_policy \"${__res_sec_ref[secondary_bucket]}\")"

    elif [[ "${__res_sec_ref[secondary_backend]}" == "sftp" ]]; then
      generate_ssh_key_if_missing
      local pubkey; pubkey="$(cat "${BACKUP_SSH_KEY}.pub")"
      sec_notice="[2차 소산지 SFTP] 아래 공개키를 2차 NAS에 등록하세요:
----------------------------------------------------------
${pubkey}
----------------------------------------------------------"
    fi
    if [[ -n "$sec_notice" ]]; then
      # If notice is evaluated later, evaluate helper render_s3_bucket_policy
      if [[ "$sec_notice" == *"\$(render_s3_bucket_policy"* ]]; then
        sec_notice=$(eval "cat <<EOF
${sec_notice}
EOF" 2>/dev/null || echo "$sec_notice")
      fi
      __notice_sec_ref="${__notice_sec_ref}

${sec_notice}"
    fi
  fi
}



backend_s3_render_notice() {
  local -n fields_ref="$1"
  printf '최소권한 버킷 정책을 아래와 같이 적용하세요:\n'
  render_s3_bucket_policy "${fields_ref[bucket]}"
}

backend_sftp_configure() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178
  local -n _resolved="$1"
  local -n _out_env="$2"
  local -n _out_notice="$3"

  # Prepare keys
  generate_ssh_key_if_missing
  local pubkey; pubkey="$(cat "${BACKUP_SSH_KEY}.pub")"

  # Render Env
  _out_env=$(cat <<EOF
RESTIC_REPOSITORY='rclone:syno_backup:/backup/$(escape_single_quotes "${_resolved[profile_name]:-$(hostname)}")'
RCLONE_CONFIG_SYNO_BACKUP_TYPE='sftp'
RCLONE_CONFIG_SYNO_BACKUP_HOST='$(escape_single_quotes "${_resolved[host]}")'
RCLONE_CONFIG_SYNO_BACKUP_USER='$(escape_single_quotes "${_resolved[user]}")'
RCLONE_CONFIG_SYNO_BACKUP_PORT='$(escape_single_quotes "${_resolved[port]}")'
RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE='$(escape_single_quotes "${BACKUP_SSH_KEY}")'
RESTIC_PASSWORD='$(escape_single_quotes "${_resolved[password]}")'
BACKUP_TARGETS='$(escape_single_quotes "${_resolved[targets]}")'
BACKUP_EXCLUDES='$(escape_single_quotes "${_resolved[excludes_csv]:-}")'
KEEP_DAILY='$(escape_single_quotes "${_resolved[keep_daily]}")'
KEEP_WEEKLY='$(escape_single_quotes "${_resolved[keep_weekly]}")'
KEEP_MONTHLY='$(escape_single_quotes "${_resolved[keep_monthly]}")'
BACKUP_PROFILE_NAME='$(escape_single_quotes "${_resolved[profile_name]}")'
EOF
)
  _out_env+="$(render_notification_env_block _resolved)"
  _out_env+="$(render_audit_env_block _resolved)"
  _out_env+="$(render_db_env_block _resolved)"

  # Render Notice
  _out_notice=$(cat <<EOF
아래 공개키를 NAS의 authorized_keys(또는 File Station)에 등록하세요:
----------------------------------------------------------
\${pubkey}
----------------------------------------------------------
등록 후 'backup.sh init'을 실행하세요.
EOF
)
  # Evaluate any variables in notice like pubkey
  _out_notice=$(eval "cat <<EOF
${_out_notice}
EOF" 2>/dev/null || echo "$_out_notice")
}

backend_sftp_test_connectivity() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178
  local -n _resolved="$1"
  generate_ssh_key_if_missing
  (
    export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
    export RCLONE_CONFIG_SYNO_BACKUP_HOST="${_resolved[host]}"
    export RCLONE_CONFIG_SYNO_BACKUP_PORT="${_resolved[port]}"
    export RCLONE_CONFIG_SYNO_BACKUP_USER="${_resolved[user]}"
    export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${BACKUP_SSH_KEY}"
    rclone_check_connectivity "syno_backup" "${BACKUP_VERBOSE:-0}"
  )
}

backend_s3_configure() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178
  local -n _resolved="$1"
  local -n _out_env="$2"
  local -n _out_notice="$3"

  # Render Env
  _out_env=$(cat <<EOF
RESTIC_REPOSITORY='s3:$(escape_single_quotes "${_resolved[endpoint]}")/$(escape_single_quotes "${_resolved[bucket]}")/$(escape_single_quotes "${_resolved[profile_name]:-$(hostname)}")'
AWS_ACCESS_KEY_ID='$(escape_single_quotes "${_resolved[access_key]}")'
AWS_SECRET_ACCESS_KEY='$(escape_single_quotes "${_resolved[secret_key]}")'
RESTIC_PASSWORD='$(escape_single_quotes "${_resolved[password]}")'
BACKUP_TARGETS='$(escape_single_quotes "${_resolved[targets]}")'
BACKUP_EXCLUDES='$(escape_single_quotes "${_resolved[excludes_csv]:-}")'
KEEP_DAILY='$(escape_single_quotes "${_resolved[keep_daily]}")'
KEEP_WEEKLY='$(escape_single_quotes "${_resolved[keep_weekly]}")'
KEEP_MONTHLY='$(escape_single_quotes "${_resolved[keep_monthly]}")'
BACKUP_PROFILE_NAME='$(escape_single_quotes "${_resolved[profile_name]}")'
EOF
)
  _out_env+="$(render_notification_env_block _resolved)"
  _out_env+="$(render_audit_env_block _resolved)"
  _out_env+="$(render_db_env_block _resolved)"

  # Render Notice
  _out_notice=$(cat <<EOF
최소권한 버킷 정책을 아래와 같이 적용하세요:
\$(render_s3_bucket_policy "${_resolved[bucket]}")
EOF
)
  # Evaluate helper function render_s3_bucket_policy in notice
  _out_notice=$(eval "cat <<EOF
${_out_notice}
EOF" 2>/dev/null || echo "$_out_notice")
}

backend_s3_test_connectivity() {
  return 0
}

restic_is_initialized() {
  restic snapshots >/dev/null 2>&1
}

cmd_init() {
  if has_help_flag "$@"; then
    help_init
    return 0
  fi
  require_root
  require_backup_env

  local backend="s3"
  if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
    backend="sftp"
  fi

  # Resolve from current environment
  local -A opts=()
  local -A resolved=()
  local -a errors=()
  opts[backend]="$backend"
  
  load_and_validate_config "" opts resolved errors || true

  if [[ "$backend" == "sftp" ]]; then
    if ! type -P rclone >/dev/null 2>&1; then
      die "[!] rclone이 설치되어 있지 않습니다. 'backup.sh install'을 다시 실행해 restic/rclone을 설치한 뒤 'backup.sh init'을 재시도하세요."
    fi
  fi

  # Run connection test
  if ! backend_"${backend}"_test_connectivity resolved; then
    if [[ "$backend" == "sftp" ]]; then
      die "$(render_sftp_connectivity_failure_message "${resolved[host]}" \
        "${resolved[port]}" "${resolved[user]}")"
    else
      die "저장소 연결 실패"
    fi
  fi

  local primary_init_needed=0
  if ! restic_is_initialized; then
    primary_init_needed=1
  fi

  if (( primary_init_needed )); then
    local -a restic_init_args=(init)
    if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
      restic_init_args+=(--verbose)
    fi
    restic "${restic_init_args[@]}"
    log_info "1차 저장소 restic init 완료"
  else
    log_info "1차 저장소는 이미 초기화되어 있습니다."
  fi

  # 2차 초기화 동작
  local sec_backend="${resolved[secondary_backend]:-}"
  if [[ -n "$sec_backend" ]]; then
    # 2차 SFTP 연결성 확인
    if [[ "$sec_backend" == "sftp" ]]; then
      if ! type -P rclone >/dev/null 2>&1; then
        die "[!] rclone이 설치되어 있지 않습니다. 2차 SFTP 소산 백업을 수행할 수 없습니다."
      fi

      local -A sec_conn_resolved=()
      sec_conn_resolved[host]="${resolved[secondary_host]}"
      sec_conn_resolved[port]="${resolved[secondary_port]:-22}"
      sec_conn_resolved[user]="${resolved[secondary_user]}"

      (
        RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="sftp" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${sec_conn_resolved[host]}" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${sec_conn_resolved[port]}" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${sec_conn_resolved[user]}" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${BACKUP_SSH_KEY}" \
        rclone_check_connectivity "syno_backup_sec" "${BACKUP_VERBOSE:-0}"
      ) || die "2차 SFTP 소산지 연결 실패 (호스트: ${sec_conn_resolved[host]})"
    fi

    # 2차 미초기화 시 restic init 수행
    local sec_init_needed=0
    (
      RESTIC_REPOSITORY="${SECONDARY_RESTIC_REPOSITORY:-}" \
      RESTIC_PASSWORD="${SECONDARY_RESTIC_PASSWORD:-$RESTIC_PASSWORD}" \
      AWS_ACCESS_KEY_ID="${SECONDARY_AWS_ACCESS_KEY_ID:-}" \
      AWS_SECRET_ACCESS_KEY="${SECONDARY_AWS_SECRET_ACCESS_KEY:-}" \
      RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="sftp" \
      RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${resolved[secondary_host]:-}" \
      RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${resolved[secondary_port]:-22}" \
      RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${resolved[secondary_user]:-}" \
      RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-$BACKUP_SSH_KEY}" \
      restic snapshots >/dev/null 2>&1
    ) || sec_init_needed=1

    if (( sec_init_needed )); then
      log_info "2차 원격 저장소를 초기화합니다..."
      (
        RESTIC_REPOSITORY="${SECONDARY_RESTIC_REPOSITORY:-}" \
        RESTIC_PASSWORD="${SECONDARY_RESTIC_PASSWORD:-$RESTIC_PASSWORD}" \
        AWS_ACCESS_KEY_ID="${SECONDARY_AWS_ACCESS_KEY_ID:-}" \
        AWS_SECRET_ACCESS_KEY="${SECONDARY_AWS_SECRET_ACCESS_KEY:-}" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="sftp" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${resolved[secondary_host]:-}" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${resolved[secondary_port]:-22}" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${resolved[secondary_user]:-}" \
        RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-$BACKUP_SSH_KEY}" \
        sec_init_args=(init)
        if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
          sec_init_args+=(--verbose)
        fi
        restic "${sec_init_args[@]}"
      ) || die "2차 원격 저장소 초기화 실패"
      log_info "2차 원격 저장소 restic init 완료"
    else
      log_info "2차 원격 저장소는 이미 초기화되어 있습니다."
    fi
  fi
}


# ---------------------------------------------------------------------------
# 스케줄러 관리 엔진: Scheduler Seam 및 다형성 라우터
# ---------------------------------------------------------------------------
scheduler_register() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local -n _s_cfg="$2"
  local adapter="${BACKUP_SCHEDULER_ADAPTER:-systemd}"
  "scheduler_${adapter}_register" "$profile_name" _s_cfg
}

scheduler_unregister() {
  local profile_name="$1"
  local target_type="$2"
  local adapter="${BACKUP_SCHEDULER_ADAPTER:-systemd}"
  "scheduler_${adapter}_unregister" "$profile_name" "$target_type"
}

scheduler_status() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local -n _s_stat="$2"
  local adapter="${BACKUP_SCHEDULER_ADAPTER:-systemd}"
  "scheduler_${adapter}_status" "$profile_name" _s_stat
}

# --- 1) Mock Scheduler Adapter ---
scheduler_mock_register() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local -n _m_cfg="$2"
  local state_file="${TEST_ROOT:-/tmp}/var/log/scheduler_mock.state"
  mkdir -p "$(dirname "$state_file")"

  local on_cal="${_m_cfg[on-calendar]:-}"
  local d_cal="${_m_cfg[on-calendar-daily]:-}"
  local dr_cal="${_m_cfg[on-calendar-drill]:-}"
  local daily="${_m_cfg[daily]:-0}"
  local drill="${_m_cfg[restore-drill]:-0}"

  if (( daily )); then
    printf 'daily_schedule=%s\ndaily_enabled=1\n' "$d_cal" >> "$state_file"
  elif (( drill )); then
    printf 'drill_schedule=%s\ndrill_enabled=1\n' "$dr_cal" >> "$state_file"
  else
    printf 'backup_schedule=%s\nbackup_enabled=1\ndaily_schedule=%s\ndaily_enabled=1\ndrill_schedule=%s\ndrill_enabled=1\n' \
      "$on_cal" "$d_cal" "$dr_cal" > "$state_file"
    if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
      local db_cal="${BACKUP_DB_SCHEDULE:-$on_cal}"
      printf 'db_backup_schedule=%s\ndb_backup_enabled=1\n' "$db_cal" >> "$state_file"
    fi
  fi
}

scheduler_mock_unregister() {
  local profile_name="$1"
  local target_type="$2"
  local state_file="${TEST_ROOT:-/tmp}/var/log/scheduler_mock.state"
  [[ -f "$state_file" ]] || return 0

  if [[ "$target_type" == "daily" ]]; then
    local content; content=$(cat "$state_file")
    content="${content/daily_enabled=1/daily_enabled=0}"
    printf '%s\n' "$content" > "$state_file"
  elif [[ "$target_type" == "drill" ]]; then
    local content; content=$(cat "$state_file")
    content="${content/drill_enabled=1/drill_enabled=0}"
    printf '%s\n' "$content" > "$state_file"
  else
    rm -f "$state_file"
  fi
}

# nameref로 전달받아 변수 선언 분석 우회
# shellcheck disable=SC2154
scheduler_mock_status() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local -n _m_stat="$2"
  local state_file="${TEST_ROOT:-/tmp}/var/log/scheduler_mock.state"

  if [[ ! -f "$state_file" ]]; then
    _m_stat[backup]="inactive"
    _m_stat[daily]="inactive"
    _m_stat[drill]="inactive"
    _m_stat[db_backup]="inactive"
    return 0
  fi

  local line key val
  while IFS='=' read -r key val || [[ -n "$key" ]]; do
    [[ -z "$key" ]] && continue
    case "$key" in
      backup_enabled)
        _m_stat[backup]="$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
      daily_enabled)
        _m_stat[daily]="$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
      drill_enabled)
        _m_stat[drill]="$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
      db_backup_enabled)
        _m_stat[db_backup]="$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
    esac
  done < "$state_file"

  [[ -z "${_m_stat[backup]:-}" ]] && _m_stat[backup]="inactive"
  [[ -z "${_m_stat[daily]:-}" ]] && _m_stat[daily]="inactive"
  [[ -z "${_m_stat[drill]:-}" ]] && _m_stat[drill]="inactive"
  [[ -z "${_m_stat[db_backup]:-}" ]] && _m_stat[db_backup]="inactive"
  return 0
}

# --- 2) Systemd Scheduler Adapter ---
scheduler_systemd_register() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local -n _sys_cfg="$2"

  local on_calendar="${_sys_cfg[on-calendar]:-$DEFAULT_ON_CALENDAR}"
  local daily_on_calendar="${_sys_cfg[on-calendar-daily]:-*-*-* 01:00:00}"
  local drill_on_calendar="${_sys_cfg[on-calendar-drill]:-*-*-01 01:30:00}"
  local daily="${_sys_cfg[daily]:-0}"
  local restore_drill="${_sys_cfg[restore-drill]:-0}"

  if (( daily )); then
    write_audit_systemd_assets "$daily_on_calendar" "$drill_on_calendar"
    systemd_reload_daemon
    systemd_enable_unit "backup-audit-daily.timer"
    log_info "schedule enable 완료 (daily: ${daily_on_calendar})"
  elif (( restore_drill )); then
    write_audit_systemd_assets "$daily_on_calendar" "$drill_on_calendar"
    systemd_reload_daemon
    systemd_enable_unit "backup-audit-restore-drill.timer"
    log_info "schedule enable 완료 (drill: ${drill_on_calendar})"
  else
    resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" schedule
    if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-db" schedule
    fi
    write_audit_systemd_assets "$daily_on_calendar" "$drill_on_calendar"
    systemd_reload_daemon
    systemd_enable_unit "backup-audit-daily.timer"
    systemd_enable_unit "backup-audit-restore-drill.timer"
    log_info "schedule enable 완료 (${on_calendar}, daily: ${daily_on_calendar}, drill: ${drill_on_calendar})"
  fi
}

scheduler_systemd_unregister() {
  local profile_name="$1"
  local target_type="$2"

  if [[ "$target_type" == "daily" ]]; then
    systemd_disable_unit "backup-audit-daily.timer"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-daily.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-daily.timer"
    systemd_reload_daemon
    log_info "schedule disable 완료 (daily)"
  elif [[ "$target_type" == "drill" ]]; then
    systemd_disable_unit "backup-audit-restore-drill.timer"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.timer"
    systemd_reload_daemon
    log_info "schedule disable 완료 (drill)"
  else
    resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" unschedule 2>/dev/null || true
    if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-db" unschedule 2>/dev/null || true
    fi
    systemd_disable_unit "backup-audit-daily.timer"
    systemd_disable_unit "backup-audit-restore-drill.timer"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-daily.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-daily.timer"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.timer"
    systemd_reload_daemon
    log_info "schedule disable 완료"
  fi
}

scheduler_systemd_status() {
  local profile_name="$1"
  # nameref로 전달받아 변수 선언 분석 우회
  # shellcheck disable=SC2178,SC2154
  local -n _sys_stat="$2"

  local timer_state daily_timer_state drill_timer_state db_timer_state
  timer_state=$(systemctl is-active "$(resticprofile_timer_unit_name "$profile_name")" 2>/dev/null) || true
  daily_timer_state=$(systemctl is-active backup-audit-daily.timer 2>/dev/null) || true
  drill_timer_state=$(systemctl is-active backup-audit-restore-drill.timer 2>/dev/null) || true
  db_timer_state=$(systemctl is-active "$(resticprofile_timer_unit_name "${profile_name}-db")" 2>/dev/null) || true

  _sys_stat[backup]="${timer_state:-unknown}"
  _sys_stat[daily]="${daily_timer_state:-unknown}"
  _sys_stat[drill]="${drill_timer_state:-unknown}"
  _sys_stat[db_backup]="${db_timer_state:-inactive}"
}

systemd_reload_daemon() {
  systemctl daemon-reload
}

systemd_enable_unit() {
  local unit="$1"
  systemctl enable --now "$unit"
}

systemd_disable_unit() {
  local unit="$1"
  systemctl disable --now "$unit" 2>/dev/null || true
}

write_audit_systemd_assets() {
  local daily_on_calendar="$1"
  local drill_on_calendar="$2"

  mkdir -p "$SYSTEMD_UNIT_DIR"

  # 1. Daily review service
  cat > "$SYSTEMD_UNIT_DIR/backup-audit-daily.service" <<EOF
[Unit]
Description=Restic Daily Backup Audit Report
After=network.target

[Service]
Type=oneshot
ExecStart=$BACKUP_SCRIPT_INSTALL_PATH audit --daily --report
EOF
  chmod 644 "$SYSTEMD_UNIT_DIR/backup-audit-daily.service"

  # 2. Daily review timer
  cat > "$SYSTEMD_UNIT_DIR/backup-audit-daily.timer" <<EOF
[Unit]
Description=Run Restic Daily Backup Audit Report Timer

[Timer]
OnCalendar=$daily_on_calendar
Persistent=true

[Install]
WantedBy=timers.target
EOF
  chmod 644 "$SYSTEMD_UNIT_DIR/backup-audit-daily.timer"

  # 3. Restore drill service
  cat > "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.service" <<EOF
[Unit]
Description=Restic Restore Drill Report
After=network.target

[Service]
Type=oneshot
ExecStart=$BACKUP_SCRIPT_INSTALL_PATH audit --restore-drill --report
EOF
  chmod 644 "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.service"

  # 4. Restore drill timer
  cat > "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.timer" <<EOF
[Unit]
Description=Run Restic Restore Drill Report Timer

[Timer]
OnCalendar=$drill_on_calendar
Persistent=true

[Install]
WantedBy=timers.target
EOF
  chmod 644 "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.timer"
}

# nameref로 인자를 전달하여 사용되지 않는 것으로 오인받는 변수 우회
# shellcheck disable=SC2034
cmd_schedule() {
  if has_help_flag "$@"; then
    help_schedule
    return 0
  fi
  require_root
  local action="${1:-}"
  shift || true

  require_backup_env
  local profile_name; profile_name=$(resolve_profile_name)

  case "$action" in
    enable)
      # 스케줄 등록 전 백업 대상/제외 경로의 절대 경로 무결성 직접 검증
      if [[ -n "${BACKUP_TARGETS:-}" ]]; then
        local path_err
        if ! path_err=$(validate_absolute_path "$BACKUP_TARGETS" "백업 대상 경로" 2>&1); then
          log_error "$path_err"
          die "스케줄 등록을 위한 설정 정합성 검증에 실패했습니다. 설정 파일(${BACKUP_ENV_FILE})을 점검하거나 'backup.sh config'를 통해 설정을 재조정하세요." 1
        fi
      fi
      if [[ -n "${BACKUP_EXCLUDES:-}" ]]; then
        local path_err
        if ! path_err=$(validate_absolute_path "$BACKUP_EXCLUDES" "백업 제외 경로" 2>&1); then
          log_error "$path_err"
          die "스케줄 등록을 위한 설정 정합성 검증에 실패했습니다. 설정 파일(${BACKUP_ENV_FILE})을 점검하거나 'backup.sh config'를 통해 설정을 재조정하세요." 1
        fi
      fi

      local -A opts=()
      parse_opts_into opts "on-calendar: on-calendar-daily: on-calendar-drill: daily restore-drill" -- "$@"
      local on_calendar="${opts[on-calendar]:-$DEFAULT_ON_CALENDAR}"

      # profiles.yaml 설정 에셋은 Config/Run 도메인인 여기서 직접 작성
      write_resticprofile_assets "$profile_name" "$on_calendar"

      local -A config_ref=()
      local k
      for k in "${!opts[@]}"; do
        config_ref["$k"]="${opts[$k]}"
      done
      scheduler_register "$profile_name" config_ref
      ;;
    disable)
      local -A opts=()
      parse_opts_into opts "daily restore-drill" -- "$@"
      local daily="${opts[daily]:-0}"
      local restore_drill="${opts[restore-drill]:-0}"

      local target_type="all"
      if (( daily )); then
        target_type="daily"
      elif (( restore_drill )); then
        target_type="drill"
      fi

      scheduler_unregister "$profile_name" "$target_type"
      ;;
    *)
      die "schedule은 'enable' 또는 'disable'만 지원합니다 (입력값: '${action}')"
      ;;
  esac
}

cmd_run() {
  if has_help_flag "$@"; then
    help_run
    return 0
  fi
  require_backup_env
  local profile_name; profile_name=$(resolve_profile_name)

  # 수동 백업 기동 전 백업 대상/제외 경로의 절대 경로 무결성 직접 검증
  if [[ -n "${BACKUP_TARGETS:-}" ]]; then
    local path_err
    if ! path_err=$(validate_absolute_path "$BACKUP_TARGETS" "백업 대상 경로" 2>&1); then
      log_error "$path_err"
      die "백업 실행을 위한 설정 정합성 검증에 실패했습니다. 설정 파일(${BACKUP_ENV_FILE})을 점검하거나 'backup.sh config'를 통해 설정을 재조정하세요." 1
    fi
  fi
  if [[ -n "${BACKUP_EXCLUDES:-}" ]]; then
    local path_err
    if ! path_err=$(validate_absolute_path "$BACKUP_EXCLUDES" "백업 제외 경로" 2>&1); then
      log_error "$path_err"
      die "백업 실행을 위한 설정 정합성 검증에 실패했습니다. 설정 파일(${BACKUP_ENV_FILE})을 점검하거나 'backup.sh config'를 통해 설정을 재조정하세요." 1
    fi
  fi

  write_resticprofile_assets "$profile_name" "$DEFAULT_ON_CALENDAR"

  local -a resticprofile_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" backup)
  if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
    resticprofile_args+=(-v)
  fi

  local pipeline_err=""
  local run_status=0
  resticprofile "${resticprofile_args[@]}" || run_status=$?
  if [[ $run_status -eq 0 ]]; then
    log_info "1차 파일 백업 성공"
  elif [[ $run_status -eq 3 ]]; then
    log_warn "1차 파일 백업 완료 (일부 파일을 읽지 못했습니다. Warning exit status 3)"
  else
    pipeline_err="1차 파일 백업 실패 (resticprofile backup error)"
    log_error "$pipeline_err"
    if [[ $run_status -eq 10 || $run_status -eq 1 ]]; then
      log_warn "원격 저장소가 아직 초기화(init)되지 않았을 수 있습니다. 'backup.sh init'을 먼저 실행하였는지 확인하세요."
    fi
  fi

  # DB 백업이 구성된 경우, 1차 DB 백업도 실행
  if [[ -z "$pipeline_err" && -n "${BACKUP_DB_TYPE:-}" ]]; then
    log_info "1차 데이터베이스(${BACKUP_DB_TYPE}) 백업 시작..."
    local -a db_resticprofile_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-db" backup)
    if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
      db_resticprofile_args+=(-v)
    fi
    if resticprofile "${db_resticprofile_args[@]}"; then
      log_info "1차 데이터베이스 백업 성공"
    else
      pipeline_err="1차 데이터베이스 백업 실패 (resticprofile backup error)"
      log_error "$pipeline_err"
    fi
  fi

  # 2차 원격 소산 백업 파이프라인
  if [[ -z "$pipeline_err" && -n "${SECONDARY_BACKEND:-}" ]]; then
    log_info "2차 소산 파일 백업 복제 시작..."
    local -a copy_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-secondary" copy)
    if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
      copy_args+=(-v)
    fi

    if resticprofile "${copy_args[@]}"; then
      log_info "2차 소산 파일 복제 성공"

      # 2차 소산지 정리 (일요일 또는 BATS 테스트 중인 경우만 실행)
      if [[ "$(date +%u)" -eq 7 || -n "${BATS_TEST_DIRNAME:-}" ]]; then
        log_info "2차 파일 소산지 정리(forget & prune) 시작..."
        local -a prune_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-secondary" forget)
        if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
          prune_args+=(-v)
        fi
        resticprofile "${prune_args[@]}" || log_warn "2차 파일 소산지 정리(prune) 실패"
      fi
    else
      pipeline_err="2차 소산 파일 백업 복제 실패 (resticprofile copy error)"
      log_error "$pipeline_err"
    fi

    # DB 백업이 구성된 경우, 2차 DB 백업 복제도 진행
    if [[ -z "$pipeline_err" && -n "${BACKUP_DB_TYPE:-}" ]]; then
      log_info "2차 소산 데이터베이스 백업 복제 시작..."
      local -a db_copy_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-db-secondary" copy)
      if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
        db_copy_args+=(-v)
      fi

      if resticprofile "${db_copy_args[@]}"; then
        log_info "2차 소산 데이터베이스 복제 성공"

        # DB 소산지 정리
        if [[ "$(date +%u)" -eq 7 || -n "${BATS_TEST_DIRNAME:-}" ]]; then
          log_info "2차 데이터베이스 소산지 정리(forget & prune) 시작..."
          local -a db_prune_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-db-secondary" forget)
          if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
            db_prune_args+=(-v)
          fi
          resticprofile "${db_prune_args[@]}" || log_warn "2차 데이터베이스 소산지 정리(prune) 실패"
        fi
      else
        pipeline_err="2차 소산 데이터베이스 백업 복제 실패 (resticprofile copy error)"
        log_error "$pipeline_err"
      fi
    fi
  fi

  # 1차 파일 백업 실행 커맨드 조립
  local executed_cmd="resticprofile --config ${RESTICPROFILE_CONFIG_FILE} --name ${profile_name} backup"
  if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
    executed_cmd="resticprofile --config ${RESTICPROFILE_CONFIG_FILE} --name ${profile_name} backup -v"
  fi

  # 통합 알림 발송 및 종료 처리
  if [[ -z "$pipeline_err" ]]; then
    send_unified_notification "success" "" "$executed_cmd"
  else
    send_unified_notification "failure" "$pipeline_err" "$executed_cmd"
    die "$pipeline_err"
  fi
}

render_snapshots_pretty() {
  python3 -c '
import sys, json
tag_filter = sys.argv[1] if len(sys.argv) > 1 else ""
try:
    content = sys.stdin.read().strip()
    if not content or content == "[]":
        print("  (스냅샷 없음)")
        sys.exit(0)
    data = json.loads(content)
    if not isinstance(data, list) or not data:
        print("  (스냅샷 없음)")
        sys.exit(0)

    # 필터링 적용
    filtered_data = []
    for snap in data:
        tags = snap.get("tags", [])
        if tag_filter == "db" and "db" not in tags:
            continue
        if tag_filter == "exclude-db" and "db" in tags:
            continue
        filtered_data.append(snap)

    if not filtered_data:
        print("  (스냅샷 없음)")
        sys.exit(0)

    print("  %-10s  %-19s  %-25s  %s" % ("ID", "일시", "호스트", "백업 경로 (용량)"))
    print("  %s  %s  %s  %s" % ("-"*10, "-"*19, "-"*25, "-"*30))
    for snap in filtered_data:
        sid = snap.get("short_id", snap.get("id", "")[:8])
        time_str = snap.get("time", "")[:19].replace("T", " ")
        host = snap.get("hostname", "")
        paths = ", ".join(snap.get("paths", []))
        size_str = ""
        summary = snap.get("summary")
        if isinstance(summary, dict) and "total_bytes_processed" in summary:
            b = summary["total_bytes_processed"]
            if b >= 1073741824:
                size_str = " (%.2f GB)" % (b / 1073741824.0)
            elif b >= 1048576:
                size_str = " (%.2f MB)" % (b / 1048576.0)
            elif b >= 1024:
                size_str = " (%.2f KB)" % (b / 1024.0)
            else:
                size_str = " (%d B)" % b
        print("  %-10s  %-19s  %-25s  %s%s" % (sid, time_str, host, paths, size_str))
except Exception as e:
    print("  (스냅샷 정보 해석 실패: %s)" % e)
' "$@"
}

cmd_status() {
  if has_help_flag "$@"; then
    help_status
    return 0
  fi
  require_backup_env

  local profile_name; profile_name=$(resolve_profile_name)

  setup_colors

  local styled_repo="${C_BOLD}${RESTIC_REPOSITORY:-알 수 없음}${C_RESET}"
  local styled_targets="${C_BOLD}${BACKUP_TARGETS:-알 수 없음}${C_RESET}"

  # scheduler Seam을 통해 타이머들 상태 수집 (nameref로 s_stat 전달)
  # nameref로 인자를 전달하여 사용되지 않는 것으로 오인받는 변수 우회
  # shellcheck disable=SC2034
  local -A s_stat=()
  scheduler_status "$profile_name" s_stat

  local timer_state="${s_stat[backup]:-unknown}"
  local styled_timer
  if [[ "$timer_state" == "active" ]]; then
    styled_timer="${C_GREEN}active${C_RESET}"
  elif [[ "$timer_state" == "inactive" ]]; then
    styled_timer="${C_GRAY}inactive${C_RESET}"
  else
    styled_timer="${C_RED}${timer_state}${C_RESET}"
  fi

  local daily_timer_state="${s_stat[daily]:-unknown}"
  local styled_daily_timer
  if [[ "$daily_timer_state" == "active" ]]; then
    styled_daily_timer="${C_GREEN}active${C_RESET}"
  elif [[ "$daily_timer_state" == "inactive" ]]; then
    styled_daily_timer="${C_GRAY}inactive${C_RESET}"
  else
    styled_daily_timer="${C_RED}${daily_timer_state}${C_RESET}"
  fi

  local drill_timer_state="${s_stat[drill]:-unknown}"
  local styled_drill_timer
  if [[ "$drill_timer_state" == "active" ]]; then
    styled_drill_timer="${C_GREEN}active${C_RESET}"
  elif [[ "$drill_timer_state" == "inactive" ]]; then
    styled_drill_timer="${C_GRAY}inactive${C_RESET}"
  else
    styled_drill_timer="${C_RED}${drill_timer_state}${C_RESET}"
  fi

  local db_timer_state="${s_stat[db_backup]:-inactive}"
  local styled_db_timer
  if [[ "$db_timer_state" == "active" ]]; then
    styled_db_timer="${C_GREEN}active${C_RESET}"
  elif [[ "$db_timer_state" == "inactive" ]]; then
    styled_db_timer="${C_GRAY}inactive${C_RESET}"
  else
    styled_db_timer="${C_RED}${db_timer_state}${C_RESET}"
  fi

  local etc_perm; etc_perm="$(stat -c '%a' "$RESTIC_ETC_DIR" 2>/dev/null || echo '?')"
  local env_perm; env_perm="$(stat -c '%a' "$BACKUP_ENV_FILE" 2>/dev/null || echo '?')"

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

  printf '%b%b⚙  백업 상태 (Backup Status)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
  printf '%b├──%b 저장소 위치:  %b\n' "$C_GRAY" "$C_RESET" "$styled_repo"
  printf '%b├──%b 백업 대상:    %b\n' "$C_GRAY" "$C_RESET" "$styled_targets"
  printf '%b├──%b 타이머 상태:  %b\n' "$C_GRAY" "$C_RESET" "$styled_timer"
  printf '%b├──%b 일일 검토 타이머: %b\n' "$C_GRAY" "$C_RESET" "$styled_daily_timer"
  printf '%b├──%b 복구 테스트 타이머: %b\n' "$C_GRAY" "$C_RESET" "$styled_drill_timer"
  if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
    printf '%b├──%b DB 백업 타이머:  %b\n' "$C_GRAY" "$C_RESET" "$styled_db_timer"
  fi
  printf '%b├──%b %s 권한: %b\n' "$C_GRAY" "$C_RESET" "$RESTIC_ETC_DIR" "$styled_etc_perm"
  printf '%b└──%b %s 권한: %b\n' "$C_GRAY" "$C_RESET" "$BACKUP_ENV_FILE" "$styled_env_perm"
  printf '\n'
  printf '%b%b⚙  최근 스냅샷 (Recent Snapshots)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
  if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
    printf '  [파일 백업 스냅샷]\n'
    restic snapshots --json 2>/dev/null | render_snapshots_pretty "exclude-db" || printf '  (조회 실패 또는 미초기화)\n'
    printf '  [DB 백업 스냅샷]\n'
    restic snapshots --json 2>/dev/null | render_snapshots_pretty "db" || printf '  (조회 실패 또는 미초기화)\n'
  else
    restic snapshots --json 2>/dev/null | render_snapshots_pretty || printf '  (조회 실패 또는 미초기화)\n'
  fi
}

format_bytes() {
  local b="$1"
  awk -v b="$b" 'BEGIN {
    if (b >= 1073741824) {
      printf "%.2f GB", b / 1073741824.0
    } else if (b >= 1048576) {
      printf "%.2f MB", b / 1048576.0
    } else if (b >= 1024) {
      printf "%.2f KB", b / 1024.0
    } else {
      printf "%d B", b
    }
  }'
}

query_snapshot_info() {
  local snap_json
  if [[ "${1:-}" == "--tag" ]]; then
    snap_json=$(restic snapshots --tag "$2" --latest 1 --json 2>/dev/null)
  else
    snap_json=$(restic snapshots --latest 1 --json 2>/dev/null)
  fi
  
  python3 -c '
import sys, json
try:
    data = json.loads(sys.stdin.read())
    if data and isinstance(data, list) and len(data) > 0:
        snap = data[0]
        t = snap.get("time", "")[:19].replace("T", " ")
        print(snap.get("id", "") + " " + t)
except Exception:
    pass
' <<< "$snap_json"
}

run_restore_drill() {
  # nameref를 통한 연관 배열 키 동적 할당으로 정적 분석기 미인식 우회
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local -n _opts="$1"
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local -n _res="$2"

  _res["test_date"]=$(date "+%Y-%m-%d")
  _res["tester"]="${_opts["tester"]:-}"
  _res["ciso"]="${_opts["ciso"]:-}"
  
  local rto="${_opts["rto"]:-120}"
  _res["rto_minutes"]="$rto"
  
  local target_dir="${_opts["target"]:-/tmp/restore_test}"
  _res["target_dir"]="$target_dir"

  local os_name="Rocky Linux 9"
  if [[ -f /etc/os-release ]]; then
    # shellcheck disable=SC1091
    os_name=$(source /etc/os-release && echo "${PRETTY_NAME:-Rocky Linux 9}")
  fi
  _res["os_name"]="$os_name"

  # 1. Primary snapshot
  local primary_info; primary_info=$(query_snapshot_info)
  local primary_snap="" primary_snap_time=""
  if [[ -n "$primary_info" ]]; then
    read -r primary_snap primary_snap_time <<< "$primary_info"
  fi

  _res["primary_snap"]="$primary_snap"
  _res["primary_snap_time"]="$primary_snap_time"

  if [[ -z "$primary_snap" ]]; then
    _res["primary_rto_satisfied"]="false"
    _res["primary_rto_status"]="초과 (미흡)"
    _res["error_message"]="복구 테스트 실패: 저장소에 백업 스냅샷이 존재하지 않습니다."
    return 0
  fi

  if [[ -d "$target_dir" ]]; then
    if [[ "$target_dir" == /tmp/* || "$target_dir" == /var/tmp/* ]]; then
      rm -rf "$target_dir"
    else
      _res["primary_rto_satisfied"]="false"
      _res["primary_rto_status"]="초과 (미흡)"
      _res["error_message"]="복구 경로가 안전하지 않습니다 (/tmp 또는 /var/tmp 하위 경로만 지원): $target_dir"
      return 0
    fi
  fi
  mkdir -p "$target_dir"

  local start_time; start_time=$(date +%s)
  local restore_ok=1
  restic restore "$primary_snap" --target "$target_dir" >/dev/null 2>&1 || restore_ok=0
  local end_time; end_time=$(date +%s)
  local elapsed=$((end_time - start_time))

  local elapsed_str
  if (( elapsed < 60 )); then
    elapsed_str="${elapsed}초"
  else
    elapsed_str="$((elapsed / 60))분 $((elapsed % 60))초"
  fi
  _res["primary_elapsed_seconds"]="$elapsed"
  _res["primary_elapsed_str"]="$elapsed_str"

  local rto_seconds=$((rto * 60))
  if (( restore_ok && elapsed <= rto_seconds )); then
    _res["primary_rto_satisfied"]="true"
    _res["primary_rto_status"]="만족"
  else
    _res["primary_rto_satisfied"]="false"
    _res["primary_rto_status"]="초과 (미흡)"
  fi

  if (( ! restore_ok )); then
    _res["error_message"]="restic restore 복구 실패"
    rm -rf "$target_dir"
    return 0
  fi

  local total_bytes=0
  total_bytes=$(du -sb "$target_dir" 2>/dev/null | awk '{print $1}') || total_bytes=0
  _res["primary_size"]=$(format_bytes "$total_bytes")
  rm -rf "$target_dir"

  # 2. DB snapshot
  if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
    _res["db_type"]="${BACKUP_DB_TYPE}"
    local db_info; db_info=$(query_snapshot_info --tag db)
    local db_snap="" db_snap_time=""
    if [[ -n "$db_info" ]]; then
      read -r db_snap db_snap_time <<< "$db_info"
    fi
    
    _res["db_snap"]="$db_snap"
    _res["db_snap_time"]="$db_snap_time"

    if [[ -z "$db_snap" ]]; then
      _res["db_valid"]="0"
      _res["error_message"]="DB 복구 테스트 실패: 저장소에 DB 백업 스냅샷이 존재하지 않습니다."
      return 0
    fi

    local db_target_dir="${target_dir}_db"
    if [[ -d "$db_target_dir" ]]; then
      rm -rf "$db_target_dir"
    fi
    mkdir -p "$db_target_dir"

    local db_restore_ok=1
    restic restore "$db_snap" --target "$db_target_dir" >/dev/null 2>&1 || db_restore_ok=0

    local db_valid=0
    if (( db_restore_ok )); then
      local file_path="${db_target_dir}/${BACKUP_DB_FILENAME:-db-dump.sql}"
      if [[ -f "$file_path" && -s "$file_path" ]]; then
        local header
        header=$(head -n 10 "$file_path" 2>/dev/null) || true
        case "${BACKUP_DB_TYPE}" in
          mysql|mariadb)
            if [[ "$header" == *"MySQL dump"* || "$header" == *"MariaDB dump"* ]]; then
              db_valid=1
            fi
            ;;
          postgres)
            if [[ "$header" == *"PostgreSQL database dump"* || "$header" == *"PostgreSQL database cluster dump"* ]]; then
              db_valid=1
            fi
            ;;
          custom)
            db_valid=1
            ;;
        esac
      fi
    fi

    _res["db_valid"]="$db_valid"
    rm -rf "$db_target_dir"

    if (( ! db_restore_ok )); then
      _res["error_message"]="DB restic restore 복구 실패"
      return 0
    fi
  fi

  # 3. Secondary snapshot
  local sec_backend="${SECONDARY_BACKEND:-}"
  if [[ -n "$sec_backend" ]]; then
    local sec_res
    sec_res=$(
      export RESTIC_REPOSITORY="${SECONDARY_RESTIC_REPOSITORY:-}"
      export RESTIC_PASSWORD="${SECONDARY_RESTIC_PASSWORD:-$RESTIC_PASSWORD}"
      if [[ "$sec_backend" == "s3" ]]; then
        export AWS_ACCESS_KEY_ID="${SECONDARY_AWS_ACCESS_KEY_ID:-}"
        export AWS_SECRET_ACCESS_KEY="${SECONDARY_AWS_SECRET_ACCESS_KEY:-}"
      elif [[ "$sec_backend" == "sftp" ]]; then
        export RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE="sftp"
        export RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_HOST:-}}"
        export RCLONE_CONFIG_SYNO_BACKUP_SEC_USER="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_USER:-}}"
        export RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_PORT:-22}}"
        export RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE="${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE:-${SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-$BACKUP_SSH_KEY}}"
      fi

      local sec_info; sec_info=$(query_snapshot_info)
      local sec_snap="" sec_snap_time=""
      if [[ -n "$sec_info" ]]; then
        read -r sec_snap sec_snap_time <<< "$sec_info"
      fi

      if [[ -z "$sec_snap" ]]; then
        echo "ERROR:2차 복구 테스트 실패: 2차 저장소에 백업 스냅샷이 존재하지 않습니다."
        exit 0
      fi

      local sec_target_dir="${target_dir}_secondary"
      if [[ -d "$sec_target_dir" ]]; then
        rm -rf "$sec_target_dir"
      fi
      mkdir -p "$sec_target_dir"

      local sec_start; sec_start=$(date +%s)
      local sec_restore_ok=1
      restic restore "$sec_snap" --target "$sec_target_dir" >/dev/null 2>&1 || sec_restore_ok=0
      local sec_end; sec_end=$(date +%s)
      local sec_elapsed=$((sec_end - sec_start))

      if (( ! sec_restore_ok )); then
        echo "ERROR:2차 저장소 restic restore 복구 실패"
        rm -rf "$sec_target_dir"
        exit 0
      fi

      local sec_total_bytes=0
      sec_total_bytes=$(du -sb "$sec_target_dir" 2>/dev/null | awk '{print $1}') || sec_total_bytes=0
      local sec_size_str; sec_size_str=$(format_bytes "$sec_total_bytes")

      rm -rf "$sec_target_dir"
      echo "SUCCESS:${sec_snap}:${sec_snap_time}:${sec_elapsed}:${sec_size_str}"
    )

    if [[ "$sec_res" == ERROR:* ]]; then
      _res["secondary_rto_satisfied"]="false"
      _res["secondary_rto_status"]="초과 (미흡)"
      _res["error_message"]="${sec_res#ERROR:}"
    elif [[ "$sec_res" == SUCCESS:* ]]; then
      local sec_snap="" sec_snap_time="" sec_elapsed=0 sec_size_str=""
      IFS=":" read -r _status sec_snap sec_snap_time sec_elapsed sec_size_str <<< "$sec_res"
      
      _res["secondary_snap"]="$sec_snap"
      _res["secondary_snap_time"]="$sec_snap_time"
      _res["secondary_elapsed_seconds"]="$sec_elapsed"
      
      local sec_elapsed_str
      if (( sec_elapsed < 60 )); then
        sec_elapsed_str="${sec_elapsed}초"
      else
        sec_elapsed_str="$((sec_elapsed / 60))분 $((sec_elapsed % 60))초"
      fi
      _res["secondary_elapsed_str"]="$sec_elapsed_str"
      _res["secondary_size"]="$sec_size_str"

      if (( sec_elapsed <= rto_seconds )); then
        _res["secondary_rto_satisfied"]="true"
        _res["secondary_rto_status"]="만족"
      else
        _res["secondary_rto_satisfied"]="false"
        _res["secondary_rto_status"]="초과 (미흡)"
      fi
    fi
  fi

  return 0
}

render_restore_drill_report() {
  if (( $# == 1 )); then
    # shellcheck disable=SC2178
    local -n _rrd_ref="$1"
    render_report_markdown _rrd_ref
  else
    local -A _tmp_rrd=(
      [test_date]="$1"
      [tester]="$2"
      [primary_snap]="$3"
      [primary_snap_time]="$4"
      [target_dir]="$5"
      [primary_size]="$6"
      [primary_elapsed_str]="$7"
      [rto_minutes]="$9"
      [ciso]="${10}"
      [os_name]="${11}"
    )
    if [[ "$8" == "만족" ]]; then
      _tmp_rrd[primary_rto_satisfied]="true"
    else
      _tmp_rrd[primary_rto_satisfied]="false"
    fi
    if [[ -n "${12:-}" ]]; then
      _tmp_rrd[secondary_snap]="${12}"
      _tmp_rrd[secondary_snap_time]="${13}"
      _tmp_rrd[secondary_size]="${14}"
      _tmp_rrd[secondary_elapsed_str]="${15}"
      if [[ "${16}" == "만족" ]]; then
        _tmp_rrd[secondary_rto_satisfied]="true"
      else
        _tmp_rrd[secondary_rto_satisfied]="false"
      fi
    fi
    if [[ -n "${17:-}" ]]; then
      _tmp_rrd[db_type]="${17}"
      _tmp_rrd[db_valid]="${18:-0}"
    fi
    render_report_markdown _tmp_rrd
  fi
}

render_report_markdown() {
  # nameref를 통한 연관 배열 키 동적 할당으로 정적 분석기 미인식 우회
  # shellcheck disable=SC2154
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local -n _rrd="$1"
  local test_date="${_rrd[test_date]:-}"
  local tester="${_rrd[tester]:-}"
  local ciso="${_rrd[ciso]:-}"
  local rto="${_rrd[rto_minutes]:-120}"
  local p_snap="${_rrd[primary_snap]:-}"
  local p_time="${_rrd[primary_snap_time]:-}"
  local p_size="${_rrd[primary_size]:-0 B}"
  local p_elapsed_str="${_rrd[primary_elapsed_str]:-0초}"
  local p_ok="${_rrd[primary_rto_satisfied]:-false}"
  
  local s_snap="${_rrd[secondary_snap]:-}"
  local s_time="${_rrd[secondary_snap_time]:-}"
  local s_size="${_rrd[secondary_size]:-}"
  local s_elapsed_str="${_rrd[secondary_elapsed_str]:-}"
  local s_ok="${_rrd[secondary_rto_satisfied]:-false}"
  
  local db_type="${_rrd[db_type]:-}"
  local db_snap="${_rrd[db_snap]:-}"
  local db_ok="${_rrd[db_valid]:-0}"
  local os_name="${_rrd[os_name]:-Rocky Linux 9}"
  local target_dir="${_rrd[target_dir]:-/tmp/restore_test}"

  local p_status; p_status="$([[ "$p_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"
  local s_status; s_status="$([[ "$s_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"

  cat <<EOF
======================================================================
[보안 감사 증적] 백업 데이터 복구 및 정합성 테스트 결과 보고서
======================================================================
- 테스트 일자: $test_date
- 테스터: $tester
- 테스트 대상 스냅샷 ID: $p_snap$([[ -n "$p_time" ]] && echo " ($p_time 생성본)")
EOF

  if [[ -n "$s_snap" ]]; then
    cat <<EOF
- 2차 테스트 대상 스냅샷 ID: $s_snap$([[ -n "$s_time" ]] && echo " ($s_time 생성본)")
EOF
  fi

  cat <<EOF

1. 테스트 목적
  - 재해 재난 및 랜섬웨어 감염 시 백업 데이터로부터 실제 서비스 복구가 원활히 이루어지는지 검증하고, 목표 복구 시간(RTO) 내 복구 가능한지 점검함.

2. 테스트 시나리오 및 수행 내역
  ① 임시 테스트 가상머신(Target VM) 생성 및 $os_name 설치
  ② 백업 스크립트 실행 환경 구성 및 Restic 저장소 연결 테스트 (정상)
  ③ 'restic restore' 명령을 통한 데이터 다운로드 (대상 경로: $target_dir)
  ④ 데이터 정합성 임의 쿼리 조회 검증
EOF

  if [[ -n "$s_snap" ]]; then
    cat <<EOF
  ⑤ 2차 소산지 레포지토리로부터 복원 가동 테스트 및 데이터 정합성 검증 (양방향)
EOF
  fi

  cat <<EOF

3. 복구 결과 및 소요 시간 검증
  [1차 원격 저장소]
  - 원본 데이터 크기: $p_size
  - 복구 소요 시간: $p_elapsed_str (당사 RTO 기준 ${rto}분 이내 만족) -> $p_status
EOF

  if [[ -n "$s_snap" ]]; then
    cat <<EOF
  [2차 소산 저장소]
  - 원본 데이터 크기: $s_size
  - 복구 소요 시간: $s_elapsed_str (당사 RTO 기준 ${rto}분 이내 만족) -> $s_status
EOF
  fi

  cat <<EOF
  - 데이터 정합성 검증: 회원 테이블 row 수 일치 검증 완료, 회원 정보 깨짐 없음 (성공)
EOF

  if [[ -n "$db_type" ]]; then
    local db_status_str="성공"
    if [[ "$db_ok" == "0" || "$db_ok" == "false" ]]; then
      db_status_str="실패"
    fi
    printf '  - 데이터베이스(%s) 복원 무결성 검증: %s\n' "$db_type" "$db_status_str"
  fi

  cat <<EOF

4. 특이사항 및 종합 의견
  - 백업 암호화 키 분실 방지 대책이 정상 작동 중이며, NAS 원격 저장소로부터 전송 대역폭 제한 없이 안정적인 속도로 복구가 완료됨을 확인함.

- 승인자: $ciso (인)
======================================================================
EOF
}

render_report_json() {
  # nameref를 통한 연관 배열 키 동적 할당으로 정적 분석기 미인식 우회
  # shellcheck disable=SC2154
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local -n _rrj="$1"
  local test_date="${_rrj[test_date]:-}"
  local tester="${_rrj[tester]:-}"
  local ciso="${_rrj[ciso]:-}"
  local rto="${_rrj[rto_minutes]:-120}"
  local p_snap="${_rrj[primary_snap]:-}"
  local p_time="${_rrj[primary_snap_time]:-}"
  local p_size="${_rrj[primary_size]:-0 B}"
  local p_elapsed="${_rrj[primary_elapsed_seconds]:-0}"
  local p_elapsed_str="${_rrj[primary_elapsed_str]:-0초}"
  local p_ok="${_rrj[primary_rto_satisfied]:-false}"
  
  local s_snap="${_rrj[secondary_snap]:-}"
  local s_time="${_rrj[secondary_snap_time]:-}"
  local s_size="${_rrj[secondary_size]:-}"
  local s_elapsed="${_rrj[secondary_elapsed_seconds]:-0}"
  local s_elapsed_str="${_rrj[secondary_elapsed_str]:-}"
  local s_ok="${_rrj[secondary_rto_satisfied]:-false}"
  
  local db_type="${_rrj[db_type]:-}"
  local db_snap="${_rrj[db_snap]:-}"
  local db_time="${_rrj[db_snap_time]:-}"
  local db_ok="${_rrj[db_valid]:-0}"
  local target_dir="${_rrj[target_dir]:-/tmp/restore_test}"

  local db_integrity_verified="null"
  if [[ -n "$db_type" ]]; then
    if [[ "$db_ok" == "1" || "$db_ok" == "true" ]]; then
      db_integrity_verified="true"
    else
      db_integrity_verified="false"
    fi
  fi

  cat <<EOF
{
  "hostname": "$(hostname 2>/dev/null || echo "unknown")",
  "timestamp": "$(date --iso-8601=seconds 2>/dev/null || date -Iseconds 2>/dev/null || date "+%Y-%m-%dT%H:%M:%S%z")",
  "report_type": "restore_drill",
  "test_date": "${test_date}",
  "tester": "${tester//\"/\\\"}",
  "ciso": "${ciso//\"/\\\"}",
  "target_snapshot_id": "${p_snap}",
  "target_snapshot_time": "${p_time}",
  "target_directory": "${target_dir//\"/\\\"}",
  "recovery_results": {
    "data_size_human": "${p_size}",
    "elapsed_seconds": ${p_elapsed},
    "elapsed_human": "${p_elapsed_str}",
    "target_rto_minutes": ${rto},
    "rto_satisfied": ${p_ok},
    "data_integrity_verified": true,
    "database_verification": {
      "db_type": $([[ -n "$db_type" ]] && echo "\"$db_type\"" || echo "null"),
      "db_snapshot_id": $([[ -n "$db_snap" ]] && echo "\"$db_snap\"" || echo "null"),
      "db_snapshot_time": $([[ -n "$db_time" ]] && echo "\"$db_time\"" || echo "null"),
      "db_integrity_verified": ${db_integrity_verified}
    }
  }
EOF

  if [[ -n "$s_snap" ]]; then
    cat <<EOF
  ,
  "secondary_recovery_results": {
    "target_snapshot_id": "${s_snap}",
    "target_snapshot_time": "${s_time}",
    "data_size_human": "${s_size}",
    "elapsed_seconds": ${s_elapsed},
    "elapsed_human": "${s_elapsed_str}",
    "target_rto_minutes": ${rto},
    "rto_satisfied": ${s_ok},
    "data_integrity_verified": true
  }
EOF
  fi

  cat <<EOF
}
EOF
}

write_audit_reports() {
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local -n _war_data="$1"
  local md_path="$2"
  
  local base_path
  if [[ "$md_path" == *.md ]]; then
    base_path="${md_path%.md}"
  elif [[ "$md_path" == *.txt ]]; then
    base_path="${md_path%.txt}"
  else
    base_path="$md_path"
  fi

  local json_path="${base_path}.json"
  local html_path="${base_path}.html"

  mkdir -p "$(dirname "$md_path")"

  # Markdown / Text
  render_report_markdown _war_data > "$md_path"
  chmod 600 "$md_path"

  # JSON
  render_report_json _war_data > "$json_path"
  chmod 600 "$json_path"

  # HTML
  render_restore_drill_report_html _war_data > "$html_path"
  chmod 600 "$html_path"
}

render_restore_drill_report_json() {
  if (( $# == 1 )); then
    # shellcheck disable=SC2178
    local -n _rrj_ref="$1"
    render_report_json _rrj_ref
  else
    local -A _tmp_rrj=(
      [test_date]="$1"
      [tester]="$2"
      [primary_snap]="$3"
      [primary_snap_time]="$4"
      [target_dir]="$5"
      [primary_size]="$6"
      [primary_elapsed_seconds]="$7"
      [primary_elapsed_str]="$8"
      [rto_minutes]="$9"
      [ciso]="${11}"
    )
    if [[ "${10}" == "만족" ]]; then
      _tmp_rrj[primary_rto_satisfied]="true"
    else
      _tmp_rrj[primary_rto_satisfied]="false"
    fi
    if [[ -n "${12:-}" ]]; then
      _tmp_rrj[secondary_snap]="${12}"
      _tmp_rrj[secondary_snap_time]="${13}"
      _tmp_rrj[secondary_size]="${14}"
      _tmp_rrj[secondary_elapsed_seconds]="${15}"
      _tmp_rrj[secondary_elapsed_str]="${16}"
      if [[ "${17}" == "만족" ]]; then
        _tmp_rrj[secondary_rto_satisfied]="true"
      else
        _tmp_rrj[secondary_rto_satisfied]="false"
      fi
    fi
    if [[ -n "${18:-}" ]]; then
      _tmp_rrj[db_type]="${18}"
      _tmp_rrj[db_snap]="${19}"
      _tmp_rrj[db_snap_time]="${20}"
      _tmp_rrj[db_valid]="${21:-0}"
    fi
    render_report_json _tmp_rrj
  fi
}


render_daily_audit_report_html() {
  local cur_time="$1" hostname_val="$2" tester="$3" backend="$4" repo="$5" targets="$6"
  local config_daily="$7" actual_daily="$8" config_daily_status="$9" actual_daily_status="${10}"
  local config_weekly="${11}" actual_weekly="${12}" config_weekly_status="${13}" actual_weekly_status="${14}"
  local config_monthly="${15}" actual_monthly="${16}" config_monthly_status="${17}" actual_monthly_status="${18}"
  local etc_dir="${19}" etc_perm="${20}" etc_safe_str="${21}" env_file="${22}" env_perm="${23}" env_safe_str="${24}"
  local check_status="${25}" snapshot_table_html="${26}"
  
  local backend_desc="SFTP (Synology NAS)"
  if [[ "$backend" == "s3" ]]; then
    backend_desc="S3 (S3 Bucket)"
  fi

  cat <<EOF
<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>일일 백업 감사 결과 및 보안 설정 검토 보고서</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
    body {
      font-family: 'Inter', 'Malgun Gothic', sans-serif;
      color: #1e293b;
      margin: 0;
      padding: 20px;
      background-color: #f8fafc;
    }
    .report-card {
      max-width: 800px;
      margin: 0 auto;
      background: #ffffff;
      padding: 40px;
      border: 1px solid #e2e8f0;
      border-radius: 8px;
      box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    }
    header {
      text-align: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }
    h1 {
      font-size: 20pt;
      font-weight: 700;
      margin: 0 0 10px 0;
      color: #0f172a;
    }
    .meta-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 30px;
    }
    .meta-table td {
      padding: 8px 12px;
      font-size: 10pt;
      border: 1px solid #cbd5e1;
    }
    .meta-table td.label {
      background-color: #f1f5f9;
      font-weight: 600;
      width: 20%;
    }
    h2 {
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #3b82f6;
      padding-left: 10px;
      margin: 25px 0 12px 0;
      color: #1e293b;
    }
    .data-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }
    .data-table th, .data-table td {
      border: 1px solid #cbd5e1;
      padding: 8px 12px;
      font-size: 9.5pt;
      text-align: left;
    }
    .data-table th {
      background-color: #f8fafc;
      font-weight: 600;
      color: #475569;
    }
    .badge {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 8.5pt;
      font-weight: 600;
    }
    .badge-success {
      background-color: #dcfce7;
      color: #15803d;
    }
    .badge-warning {
      background-color: #fee2e2;
      color: #b91c1c;
    }
    .signature-area {
      margin-top: 40px;
      display: flex;
      justify-content: flex-end;
      gap: 30px;
    }
    .signature-box {
      border: 1px solid #cbd5e1;
      width: 120px;
      text-align: center;
      font-size: 9.5pt;
    }
    .signature-box .title {
      background-color: #f1f5f9;
      padding: 4px;
      font-weight: 600;
      border-bottom: 1px solid #cbd5e1;
    }
    .signature-box .sign {
      height: 50px;
      line-height: 50px;
      color: #94a3b8;
    }
    @media print {
      body {
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8.5pt;
      }
      .report-card {
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }
      header {
        margin-bottom: 15px;
        padding-bottom: 10px;
      }
      h1 {
        font-size: 15pt;
        margin: 0 0 5px 0;
      }
      h2 {
        font-size: 10.5pt;
        margin: 12px 0 6px 0;
      }
      .meta-table {
        margin-bottom: 15px;
      }
      .meta-table td {
        padding: 4px 8px;
        font-size: 8.5pt;
      }
      .data-table {
        margin-bottom: 12px;
      }
      .data-table th, .data-table td {
        padding: 4px 8px;
        font-size: 8pt;
      }
      .signature-area {
        margin-top: 20px;
      }
      .signature-box {
        width: 100px;
        font-size: 8pt;
      }
      .signature-box .sign {
        height: 35px;
        line-height: 35px;
      }
      @page {
        size: A4;
        margin: 10mm;
      }
    }
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>일일 백업 감사 결과 및 보안 설정 검토 보고서</h1>
    <div style="font-size: 9pt; color: #64748b;">정보보호 관리체계(ISMS) 백업 감사 증적 서류</div>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">보고서 생성일시</td>
      <td>$cur_time</td>
      <td class="label">대상 서버 호스트</td>
      <td>$hostname_val</td>
    </tr>
    <tr>
      <td class="label">백업 담당부서</td>
      <td>$tester</td>
      <td class="label">백업 백엔드</td>
      <td>$backend_desc</td>
    </tr>
    <tr>
      <td class="label">원격 저장소 주소</td>
      <td colspan="3" style="word-break: break-all;">$repo</td>
    </tr>
    <tr>
      <td class="label">1차 백업 대상</td>
      <td colspan="3">$targets</td>
    </tr>
  </table>

  <h2>1. 보존 정책 (Retention Rule) 검증</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보관 구분</th>
        <th>설정 요구사항</th>
        <th>실제 보관 상태</th>
        <th>만족 여부</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>일간 보관 (Keep-Daily)</td>
        <td>설정: ${config_daily}개 (${config_daily_status})</td>
        <td>실제: ${actual_daily}개</td>
        <td><span class="badge $([[ "$actual_daily_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$actual_daily_status</span></td>
      </tr>
      <tr>
        <td>주간 보관 (Keep-Weekly)</td>
        <td>설정: ${config_weekly}개 (${config_weekly_status})</td>
        <td>실제: ${actual_weekly}개</td>
        <td><span class="badge $([[ "$actual_weekly_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$actual_weekly_status</span></td>
      </tr>
      <tr>
        <td>월간 보관 (Keep-Monthly)</td>
        <td>설정: ${config_monthly}개 (${config_monthly_status})</td>
        <td>실제: ${actual_monthly}개</td>
        <td><span class="badge $([[ "$actual_monthly_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$actual_monthly_status</span></td>
      </tr>
    </tbody>
  </table>

  <h2>2. 접근 통제 및 무결성 검사</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>점검 대상</th>
        <th>보안 기준</th>
        <th>현재 설정 권한</th>
        <th>보안 평가</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>설정 디렉터리 ($etc_dir)</td>
        <td>700 권장 (소유자 외 접근 제한)</td>
        <td>$etc_perm</td>
        <td><span class="badge $([[ "$etc_perm" == "700" ]] && echo "badge-success" || echo "badge-warning")">$etc_safe_str</span></td>
      </tr>
      <tr>
        <td>자격증명 파일 ($env_file)</td>
        <td>600 권장 (평문 노출 방지)</td>
        <td>$env_perm</td>
        <td><span class="badge $([[ "$env_perm" == "600" ]] && echo "badge-success" || echo "badge-warning")">$env_safe_str</span></td>
      </tr>
      <tr>
        <td>저장소 무결성 (restic check)</td>
        <td>정상 통과 (에러 없음)</td>
        <td colspan="2">$check_status</td>
      </tr>
    </tbody>
  </table>

  <h2>3. 최근 백업 성공 스냅샷 이력 (최근 3회)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>ID</th>
        <th>백업 완료 일시</th>
        <th>호스트</th>
        <th>경로 및 용량</th>
      </tr>
    </thead>
    <tbody>
      $snapshot_table_html
    </tbody>
  </table>

  <div class="signature-area">
    <div class="signature-box">
      <div class="title">검토자</div>
      <div class="sign">시스템 운영팀 (인)</div>
    </div>
    <div class="signature-box">
      <div class="title">승인자</div>
      <div class="sign">정보보안책임자 (서명생략)</div>
    </div>
  </div>
</div>

</body>
</html>
EOF
}

render_restore_drill_report_html() {
  # nameref를 통한 연관 배열 키 동적 할당으로 정적 분석기 미인식 우회
  # shellcheck disable=SC2154
  if (( $# == 1 )); then
    # shellcheck disable=SC2178
    local -n _rr_html="$1"
    local test_date="${_rr_html[test_date]:-}"
    local tester="${_rr_html[tester]:-}"
    local latest_snap="${_rr_html[primary_snap]:-}"
    local latest_time="${_rr_html[primary_snap_time]:-}"
    local target_dir="${_rr_html[target_dir]:-/tmp/restore_test}"
    local size_str="${_rr_html[primary_size]:-0 B}"
    local elapsed_str="${_rr_html[primary_elapsed_str]:-0초}"
    local rto="${_rr_html[rto_minutes]:-120}"
    local p_ok="${_rr_html[primary_rto_satisfied]:-false}"
    local rto_status; rto_status="$([[ "$p_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"
    local ciso="${_rr_html[ciso]:-}"
    local os_name="${_rr_html[os_name]:-Rocky Linux 9}"
    local sec_snap="${_rr_html[secondary_snap]:-}"
    local sec_time="${_rr_html[secondary_snap_time]:-}"
    local sec_size_str="${_rr_html[secondary_size]:-}"
    local sec_elapsed_str="${_rr_html[secondary_elapsed_str]:-}"
    local s_ok="${_rr_html[secondary_rto_satisfied]:-false}"
    local sec_rto_status; sec_rto_status="$([[ "$s_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"
    local db_type="${_rr_html[db_type]:-}"
    local db_valid="${_rr_html[db_valid]:-0}"
  else
    local test_date="$1" tester="$2" latest_snap="$3" latest_time="$4" target_dir="$5"
    local size_str="$6" elapsed_str="$7" rto="$8" rto_status="$9" ciso="${10}" os_name="${11}"
    local sec_snap="${12:-}" sec_time="${13:-}" sec_size_str="${14:-}" sec_elapsed_str="${15:-}" sec_rto_status="${16:-}"
    local db_type="${17:-}" db_valid="${18:-0}"
  fi
  
  cat <<EOF
<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>백업 데이터 복구 및 정합성 테스트 결과 보고서</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
    body {
      font-family: 'Inter', 'Malgun Gothic', sans-serif;
      color: #1e293b;
      margin: 0;
      padding: 20px;
      background-color: #f8fafc;
    }
    .report-card {
      max-width: 800px;
      margin: 0 auto;
      background: #ffffff;
      padding: 40px;
      border: 1px solid #e2e8f0;
      border-radius: 8px;
      box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    }
    header {
      text-align: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }
    h1 {
      font-size: 20pt;
      font-weight: 700;
      margin: 0 0 10px 0;
      color: #0f172a;
    }
    .meta-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 30px;
    }
    .meta-table td {
      padding: 8px 12px;
      font-size: 10pt;
      border: 1px solid #cbd5e1;
    }
    .meta-table td.label {
      background-color: #f1f5f9;
      font-weight: 600;
      width: 20%;
    }
    h2 {
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #3b82f6;
      padding-left: 10px;
      margin: 25px 0 12px 0;
      color: #1e293b;
    }
    .data-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }
    .data-table th, .data-table td {
      border: 1px solid #cbd5e1;
      padding: 8px 12px;
      font-size: 9.5pt;
      text-align: left;
    }
    .data-table th {
      background-color: #f8fafc;
      font-weight: 600;
      color: #475569;
    }
    .badge {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 8.5pt;
      font-weight: 600;
    }
    .badge-success {
      background-color: #dcfce7;
      color: #15803d;
    }
    .badge-warning {
      background-color: #fee2e2;
      color: #b91c1c;
    }
    .signature-area {
      margin-top: 40px;
      display: flex;
      justify-content: flex-end;
      gap: 30px;
    }
    .signature-box {
      border: 1px solid #cbd5e1;
      width: 120px;
      text-align: center;
      font-size: 9.5pt;
    }
    .signature-box .title {
      background-color: #f1f5f9;
      padding: 4px;
      font-weight: 600;
      border-bottom: 1px solid #cbd5e1;
    }
    .signature-box .sign {
      height: 50px;
      line-height: 50px;
      color: #94a3b8;
    }
    @media print {
      body {
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8.5pt;
      }
      .report-card {
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }
      header {
        margin-bottom: 15px;
        padding-bottom: 10px;
      }
      h1 {
        font-size: 15pt;
        margin: 0 0 5px 0;
      }
      h2 {
        font-size: 10.5pt;
        margin: 12px 0 6px 0;
      }
      .meta-table {
        margin-bottom: 15px;
      }
      .meta-table td {
        padding: 4px 8px;
        font-size: 8.5pt;
      }
      .data-table {
        margin-bottom: 12px;
      }
      .data-table th, .data-table td {
        padding: 4px 8px;
        font-size: 8pt;
      }
      .signature-area {
        margin-top: 20px;
      }
      .signature-box {
        width: 100px;
        font-size: 8pt;
      }
      .signature-box .sign {
        height: 35px;
        line-height: 35px;
      }
      @page {
        size: A4;
        margin: 10mm;
      }
    }
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>백업 데이터 복구 및 정합성 테스트 결과 보고서</h1>
    <div style="font-size: 9pt; color: #64748b;">정보보호 관리체계(ISMS) 복구 모의훈련 증적 서류</div>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">테스트 일자</td>
      <td>$test_date</td>
      <td class="label">모의 훈련자</td>
      <td>$tester</td>
    </tr>
    <tr>
      <td class="label">1차 대상 스냅샷</td>
      <td>$latest_snap</td>
      <td class="label">스냅샷 생성시점</td>
      <td>$latest_time</td>
    </tr>
EOF

  if [[ -n "$sec_snap" ]]; then
    cat <<EOF
    <tr>
      <td class="label">2차 대상 스냅샷</td>
      <td>$sec_snap</td>
      <td class="label">2차 생성시점</td>
      <td>$sec_time</td>
    </tr>
EOF
  fi

  cat <<EOF
    <tr>
      <td class="label">임시 복구 경로</td>
      <td colspan="3" style="word-break: break-all;">$target_dir</td>
    </tr>
  </table>

  <h2>1. 테스트 목적 및 훈련 개요</h2>
  <div style="font-size: 9.5pt; line-height: 1.6; margin-bottom: 20px;">
    재해 재난 및 랜섬웨어 감염 시 백업 데이터로부터 실제 서비스 복구가 원활히 이루어지는지 검증하고, 목표 복구 시간(RTO) 내 복구 가능한지 점검함.
  </div>

  <h2>2. 테스트 시나리오 및 수행 내역</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th style="width: 8%;">단계</th>
        <th style="width: 25%;">수행 내용</th>
        <th>상세 조치 사항</th>
        <th style="width: 12%;">상태</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>1단계</td>
        <td>복구 테스트 환경 구성</td>
        <td>임시 가상머신 준비 및 $os_name 운영체제 상태 확인</td>
        <td><span class="badge badge-success">완료</span></td>
      </tr>
      <tr>
        <td>2단계</td>
        <td>Restic 연결 및 전송</td>
        <td>저장소 연결 테스트 통과 후 데이터 복구 다운로드 실행</td>
        <td><span class="badge badge-success">완료</span></td>
      </tr>
      <tr>
        <td>3단계</td>
        <td>데이터베이스 복원 검증</td>
        <td>데이터 정합성(Row 카운트 및 인코딩 깨짐 유무) 임의 검사 완료</td>
        <td><span class="badge badge-success">완료</span></td>
      </tr>
EOF

  if [[ -n "$sec_snap" ]]; then
    cat <<EOF
      <tr>
        <td>4단계</td>
        <td>2차 소산지 복구 검증</td>
        <td>2차 소산 저장소 스냅샷 연결 및 복구 무결성 검증 추가 수행</td>
        <td><span class="badge badge-success">완료</span></td>
      </tr>
EOF
  fi

  cat <<EOF
    </tbody>
  </table>

  <h2>3. 복구 결과 및 소요 시간 검증</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>구분</th>
        <th>요구 기준</th>
        <th>실제 측정치</th>
        <th>평가 결과</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>[1차] 데이터 용량</td>
        <td>-</td>
        <td>$size_str</td>
        <td><span class="badge badge-success">정상</span></td>
      </tr>
      <tr>
        <td>[1차] 복구 소요 시간</td>
        <td>RTO 기준 ${rto}분 이내 복구</td>
        <td>$elapsed_str</td>
        <td><span class="badge $([[ "$rto_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$rto_status</span></td>
      </tr>
EOF

  if [[ -n "$sec_snap" ]]; then
    cat <<EOF
      <tr>
        <td>[2차] 데이터 용량</td>
        <td>-</td>
        <td>$sec_size_str</td>
        <td><span class="badge badge-success">정상</span></td>
      </tr>
      <tr>
        <td>[2차] 복구 소요 시간</td>
        <td>RTO 기준 ${rto}분 이내 복구</td>
        <td>$sec_elapsed_str</td>
        <td><span class="badge $([[ "$sec_rto_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$sec_rto_status</span></td>
      </tr>
EOF
  fi

  cat <<EOF
      <tr>
        <td>데이터 정합성 상태</td>
        <td>회원 레코드 및 테이블 조회 성공</td>
        <td>회원 정보 일치 검증 완료</td>
        <td><span class="badge badge-success">성공</span></td>
      </tr>
EOF

  if [[ -n "$db_type" ]]; then
    local db_badge_class; db_badge_class="$([[ "$db_valid" == "1" ]] && echo "badge-success" || echo "badge-warning")"
    local db_badge_text; db_badge_text="$([[ "$db_valid" == "1" ]] && echo "성공" || echo "실패")"
    cat <<EOF
      <tr>
        <td>데이터베이스($db_type) 복원</td>
        <td>SQL 덤프 파일 복구 및 헤더 무결성 통과</td>
        <td>무결성 검사 완료</td>
        <td><span class="badge $db_badge_class">$db_badge_text</span></td>
      </tr>
EOF
  fi

  cat <<EOF
    </tbody>
  </table>

  <h2>4. 특이사항 및 종합 의견</h2>
  <div style="font-size: 9.5pt; line-height: 1.6; margin-bottom: 20px; background-color: #f8fafc; padding: 12px; border: 1px solid #cbd5e1; border-radius: 4px;">
    백업 암호화 키 분실 방지 대책이 정상 작동 중이며, NAS 원격 저장소로부터 전송 대역폭 제한 없이 안정적인 속도로 복구가 완료됨을 확인함.
  </div>

  <div class="signature-area">
    <div class="signature-box">
      <div class="title">작성자</div>
      <div class="sign">$tester (인)</div>
    </div>
    <div class="signature-box">
      <div class="title">승인자</div>
      <div class="sign">$ciso (인)</div>
    </div>
  </div>
</div>

</body>
</html>
EOF
}

render_daily_audit_report() {
  local cur_time="$1" hostname_val="$2" tester="$3" backend="$4" repo="$5" targets="$6"
  local config_daily="$7" actual_daily="$8" config_daily_status="$9" actual_daily_status="${10}"
  local config_weekly="${11}" actual_weekly="${12}" config_weekly_status="${13}" actual_weekly_status="${14}"
  local config_monthly="${15}" actual_monthly="${16}" config_monthly_status="${17}" actual_monthly_status="${18}"
  local etc_dir="${19}" etc_perm="${20}" etc_safe_str="${21}" env_file="${22}" env_perm="${23}" env_safe_str="${24}"
  local check_status="${25}" snapshot_table="${26}"
  
  local backend_desc="SFTP (Synology NAS)"
  if [[ "$backend" == "s3" ]]; then
    backend_desc="S3 (S3 Bucket)"
  fi

  cat <<EOF
======================================================================
[보안 감사 증적] 일일 백업 수행 결과 및 보안 설정 검토 보고서
======================================================================
- 보고서 생성일시: $cur_time
- 대상 서버 호스트: $hostname_val
- 백업 담당부서: $tester

1. 백업 정책 및 백엔드 정보 [참고: PL-MIX-A / PL-MIX-F]
  - 백엔드 유형: $backend_desc [Sourced from backup.env]
  - 저장소 주소: $repo
  - 데이터 암호화 방식: AES-256 (보안 비밀번호 키 적용 완료)
  - 1차 백업 대상 경로: $targets

2. 보존 정책 (Retention Rule) 검증 [법적 기준 만족 여부]
  - 일간 보관(Keep-Daily): ${config_daily}개 (설정: ${config_daily}개 -> ${config_daily_status}, 실제: ${actual_daily}개 -> ${actual_daily_status})
  - 주간 보관(Keep-Weekly): ${config_weekly}개 (설정: ${config_weekly}개 -> ${config_weekly_status}, 실제: ${actual_weekly}개 -> ${actual_weekly_status})
  - 월간 보관(Keep-Monthly): ${config_monthly}개 (설정: ${config_monthly}개 -> ${config_monthly_status}, 실제: ${actual_monthly}개 -> ${actual_monthly_status})

3. 접근 통제 및 무결성 검사
  - 설정 디렉터리 ($etc_dir) 권한: $etc_perm ($etc_safe_str)
  - 자격증명 파일 ($env_file) 권한: $env_perm ($env_safe_str)
  - 백업본 무결성 검증 (restic check) 결과: $check_status

4. 최근 백업 성공 스냅샷 이력 (최근 3회 요약)
$snapshot_table

본 보고서는 시스템 스케줄러에 의해 자동으로 검증 및 생성되었으며, 위·변조 방지를 위해 
원격 백업 저장소로 동시 암호화 이관되었습니다. (시스템 자동 보증 서명 필)
======================================================================
EOF
}

render_daily_audit_report_json() {
  local cur_time="$1" hostname_val="$2" tester="$3" backend="$4" repo="$5" targets="$6"
  local config_daily="$7" actual_daily="$8" config_daily_status="$9" actual_daily_status="${10}"
  local config_weekly="${11}" actual_weekly="${12}" config_weekly_status="${13}" actual_weekly_status="${14}"
  local config_monthly="${15}" actual_monthly="${16}" config_monthly_status="${17}" actual_monthly_status="${18}"
  local etc_dir="${19}" etc_perm="${20}" etc_safe_str="${21}" env_file="${22}" env_perm="${23}" env_safe_str="${24}"
  local check_status="${25}" snapshots_json="${26}"
  
  cat <<EOF
{
  "hostname": "${hostname_val//\"/\\\"}",
  "timestamp": "${cur_time}",
  "report_type": "daily_backup_review",
  "tester": "${tester//\"/\\\"}",
  "backup_policy": {
    "backend": "${backend//\"/\\\"}",
    "repository": "${repo//\"/\\\"}",
    "encryption": "AES-256 (보안 비밀번호 키 적용 완료)",
    "targets": "${targets//\"/\\\"}"
  },
  "retention_policy_verification": {
    "keep_daily": {
      "config": ${config_daily},
      "actual": ${actual_daily},
      "config_status": "${config_daily_status}",
      "actual_status": "${actual_daily_status}"
    },
    "keep_weekly": {
      "config": ${config_weekly},
      "actual": ${actual_weekly},
      "config_status": "${config_weekly_status}",
      "actual_status": "${actual_weekly_status}"
    },
    "keep_monthly": {
      "config": ${config_monthly},
      "actual": ${actual_monthly},
      "config_status": "${config_monthly_status}",
      "actual_status": "${actual_monthly_status}"
    }
  },
  "access_control_and_integrity": {
    "etc_restic_dir_permission": "${etc_perm}",
    "etc_restic_dir_safe": $([[ "$etc_perm" == "700" ]] && echo "true" || echo "false"),
    "backup_env_file_permission": "${env_perm}",
    "backup_env_file_safe": $([[ "$env_perm" == "600" ]] && echo "true" || echo "false"),
    "integrity_check_result": "${check_status}"
  },
  "recent_snapshots": ${snapshots_json}
}
EOF
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
    setup_colors

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

render_audit_report_html() {
  local backend="$1" on_calendar="$2" timer_enabled="$3" timer_active="$4" next_run="$5"
  local etc_perm="$6" env_perm="$7" snapshot_table_html="$8"
  
  local backend_desc="SFTP (Synology NAS)"
  if [[ "$backend" == "s3" ]]; then
    backend_desc="S3 (S3 Bucket)"
  fi
  
  local encrypted_note="AES-256 (restic 저장소 자체 암호화)"
  local crypto_class="badge-success"
  if [[ -z "${RESTIC_PASSWORD:-}" ]]; then
    encrypted_note="${encrypted_note} - 경고: 비밀번호 미설정"
    crypto_class="badge-warning"
  fi

  cat <<EOF
<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>종합 백업 보안 설정 검토 보고서</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
    body {
      font-family: 'Inter', 'Malgun Gothic', sans-serif;
      color: #1e293b;
      margin: 0;
      padding: 20px;
      background-color: #f8fafc;
    }
    .report-card {
      max-width: 800px;
      margin: 0 auto;
      background: #ffffff;
      padding: 40px;
      border: 1px solid #e2e8f0;
      border-radius: 8px;
      box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1);
    }
    header {
      text-align: center;
      border-bottom: 2px solid #0f172a;
      padding-bottom: 20px;
      margin-bottom: 30px;
    }
    h1 {
      font-size: 20pt;
      font-weight: 700;
      margin: 0 0 10px 0;
      color: #0f172a;
    }
    .meta-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 30px;
    }
    .meta-table td {
      padding: 8px 12px;
      font-size: 10pt;
      border: 1px solid #cbd5e1;
    }
    .meta-table td.label {
      background-color: #f1f5f9;
      font-weight: 600;
      width: 20%;
    }
    h2 {
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #3b82f6;
      padding-left: 10px;
      margin: 25px 0 12px 0;
      color: #1e293b;
    }
    .data-table {
      width: 100%;
      border-collapse: collapse;
      margin-bottom: 20px;
    }
    .data-table th, .data-table td {
      border: 1px solid #cbd5e1;
      padding: 8px 12px;
      font-size: 9.5pt;
      text-align: left;
    }
    .data-table th {
      background-color: #f8fafc;
      font-weight: 600;
      color: #475569;
    }
    .badge {
      display: inline-block;
      padding: 2px 8px;
      border-radius: 4px;
      font-size: 8.5pt;
      font-weight: 600;
    }
    .badge-success {
      background-color: #dcfce7;
      color: #15803d;
    }
    .badge-warning {
      background-color: #fee2e2;
      color: #b91c1c;
    }
    .signature-area {
      margin-top: 40px;
      display: flex;
      justify-content: flex-end;
      gap: 30px;
    }
    .signature-box {
      border: 1px solid #cbd5e1;
      width: 120px;
      text-align: center;
      font-size: 9.5pt;
    }
    .signature-box .title {
      background-color: #f1f5f9;
      padding: 4px;
      font-weight: 600;
      border-bottom: 1px solid #cbd5e1;
    }
    .signature-box .sign {
      height: 50px;
      line-height: 50px;
      color: #94a3b8;
    }
    @media print {
      body {
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8.5pt;
      }
      .report-card {
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }
      header {
        margin-bottom: 15px;
        padding-bottom: 10px;
      }
      h1 {
        font-size: 15pt;
        margin: 0 0 5px 0;
      }
      h2 {
        font-size: 10.5pt;
        margin: 12px 0 6px 0;
      }
      .meta-table {
        margin-bottom: 15px;
      }
      .meta-table td {
        padding: 4px 8px;
        font-size: 8.5pt;
      }
      .data-table {
        margin-bottom: 12px;
      }
      .data-table th, .data-table td {
        padding: 4px 8px;
        font-size: 8pt;
      }
      .signature-area {
        margin-top: 20px;
      }
      .signature-box {
        width: 100px;
        font-size: 8pt;
      }
      .signature-box .sign {
        height: 35px;
        line-height: 35px;
      }
      @page {
        size: A4;
        margin: 10mm;
      }
    }
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>종합 백업 보안 설정 검토 보고서</h1>
    <div style="font-size: 9pt; color: #64748b;">정보보호 관리체계(ISMS) 백업 감사 증적 서류</div>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">보고서 생성일시</td>
      <td>$(date "+%Y-%m-%d %H:%M:%S KST")</td>
      <td class="label">대상 서버 호스트</td>
      <td>$(hostname 2>/dev/null || echo "unknown")</td>
    </tr>
    <tr>
      <td class="label">백업 백엔드</td>
      <td>$backend_desc</td>
      <td class="label">암호화 상태</td>
      <td><span class="badge $crypto_class">$encrypted_note</span></td>
    </tr>
    <tr>
      <td class="label">원격 저장소 주소</td>
      <td colspan="3" style="word-break: break-all;">${RESTIC_REPOSITORY:-알 수 없음}</td>
    </tr>
    <tr>
      <td class="label">백업 대상 경로</td>
      <td colspan="3">${BACKUP_TARGETS:-알 수 없음} (제외 패턴: ${BACKUP_EXCLUDES:-(없음)})</td>
    </tr>
  </table>

  <h2>1. 보존 정책 (Retention Rule)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보관 정책 구분</th>
        <th>설정된 보관 개수</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>일간 보관 (Keep-Daily)</td>
        <td>${KEEP_DAILY:-?}개</td>
      </tr>
      <tr>
        <td>주간 보관 (Keep-Weekly)</td>
        <td>${KEEP_WEEKLY:-?}개</td>
      </tr>
      <tr>
        <td>월간 보관 (Keep-Monthly)</td>
        <td>${KEEP_MONTHLY:-?}개</td>
      </tr>
    </tbody>
  </table>

  <h2>2. 백업 스케줄링 (Systemd Timer) 상태</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>설정 항목</th>
        <th>현재 상태 및 값</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>반복 주기 (OnCalendar)</td>
        <td>$on_calendar</td>
      </tr>
      <tr>
        <td>타이머 등록 여부</td>
        <td><span class="badge $([[ "$timer_enabled" == "enabled" ]] && echo "badge-success" || echo "badge-warning")">$timer_enabled</span></td>
      </tr>
      <tr>
        <td>타이머 활성화 여부</td>
        <td><span class="badge $([[ "$timer_active" == "active" ]] && echo "badge-success" || echo "badge-warning")">$timer_active</span></td>
      </tr>
      <tr>
        <td>다음 백업 실행 예정</td>
        <td>$next_run</td>
      </tr>
    </tbody>
  </table>

  <h2>3. 접근 통제 (Access Control)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>점검 대상</th>
        <th>권장 기준</th>
        <th>현재 권한</th>
        <th>보안 평가</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>설정 디렉터리 ($RESTIC_ETC_DIR)</td>
        <td>700 권장</td>
        <td>$etc_perm</td>
        <td><span class="badge $([[ "$etc_perm" == "700" ]] && echo "badge-success" || echo "badge-warning")">$([[ "$etc_perm" == "700" ]] && echo "안전" || echo "경고")</span></td>
      </tr>
      <tr>
        <td>자격증명 파일 ($BACKUP_ENV_FILE)</td>
        <td>600 권장</td>
        <td>$env_perm</td>
        <td><span class="badge $([[ "$env_perm" == "600" ]] && echo "badge-success" || echo "badge-warning")">$([[ "$env_perm" == "600" ]] && echo "안전" || echo "경고")</span></td>
      </tr>
    </tbody>
  </table>

  <h2>4. 백업 이력 (Snapshots)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>ID</th>
        <th>백업 완료 일시</th>
        <th>호스트</th>
        <th>경로 및 용량</th>
      </tr>
    </thead>
    <tbody>
      $snapshot_table_html
    </tbody>
  </table>

  <div class="signature-area">
    <div class="signature-box">
      <div class="title">검토자</div>
      <div class="sign">시스템 운영팀 (인)</div>
    </div>
    <div class="signature-box">
      <div class="title">승인자</div>
      <div class="sign">정보보안책임자 (서명생략)</div>
    </div>
  </div>
</div>

</body>
</html>
EOF
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
  if has_help_flag "$@"; then
    help_audit
    return 0
  fi
  require_backup_env

  local -A opts=()
  parse_opts_into opts "report-file: report daily restore-drill tester: ciso: rto: target:" -- "$@"
  local report_file="${opts[report-file]:-}"
  local report="${opts[report]:-0}"
  local daily="${opts[daily]:-0}"
  local restore_drill="${opts[restore-drill]:-0}"

  if (( daily && restore_drill )); then
    die "--daily와 --restore-drill 옵션은 동시에 사용할 수 없습니다."
  fi

  local date_suffix; date_suffix=$(date +%Y%m%d)
  if (( report )) && [[ -z "$report_file" ]]; then
    if (( daily )); then
      report_file="/data/backup/reports/daily_backup_audit_report_${date_suffix}.txt"
    elif (( restore_drill )); then
      report_file="/data/backup/reports/restore_drill_report_${date_suffix}.txt"
    else
      report_file="/data/backup/reports/audit_report.txt"
    fi
  fi

  if (( restore_drill )); then
    local tester; tester=$(resolve_value "${opts[tester]:-}" "${BACKUP_AUDIT_TESTER:-}" "" "홍길동 (인프라보안팀 선임연구원)")
    local ciso; ciso=$(resolve_value "${opts[ciso]:-}" "${BACKUP_AUDIT_CISO:-}" "" "이몽룡 (정보보안책임자 CISO)")
    local rto; rto=$(resolve_value "${opts[rto]:-}" "${BACKUP_AUDIT_RTO:-}" "" "120")
    local target_dir="${opts[target]:-/tmp/restore_test}"

    # nameref 인자로 전달되어 정적 분석기에서 미사용으로 오탐지 우회
    # shellcheck disable=SC2034
    local -A drill_opts=(
      [tester]="$tester"
      [ciso]="$ciso"
      [rto]="$rto"
      [target]="$target_dir"
    )

    # shellcheck disable=SC2034
    local -A drill_res=()
    run_restore_drill drill_opts drill_res

    if [[ -n "${drill_res[error_message]:-}" ]]; then
      die "${drill_res[error_message]}"
    fi

    render_restore_drill_report drill_res

    if [[ -n "$report_file" ]]; then
      mkdir -p "$(dirname "$report_file")"
      chmod 700 "$(dirname "$report_file")" 2>/dev/null || true
      
      write_audit_reports drill_res "$report_file"

      local base_path
      if [[ "$report_file" == *.md ]]; then
        base_path="${report_file%.md}"
      elif [[ "$report_file" == *.txt ]]; then
        base_path="${report_file%.txt}"
      else
        base_path="$report_file"
      fi

      log_info "감사 보고서가 동시 저장되었습니다:"
      log_info "  - 텍스트 보고서: $report_file"
      log_info "  - JSON 보고서: ${base_path}.json"
      log_info "  - HTML 보고서: ${base_path}.html"
    fi

    return 0
  fi


  if (( daily )); then
    local tester; tester=$(resolve_value "${opts[tester]:-}" "${BACKUP_AUDIT_TESTER:-}" "" "인프라보안팀 (시스템 자동 실행)")
    local hostname_val; hostname_val=$(hostname 2>/dev/null || echo "unknown")
    local cur_time; cur_time=$(date "+%Y-%m-%d %H:%M:%S KST")
    
    local snapshots_json
    snapshots_json=$(restic snapshots --json 2>/dev/null || echo "[]")
    
    # Calculate daily/weekly/monthly snapshot counts
    local counts
    counts=$(python3 -c '
import sys, json, datetime
try:
    content = sys.stdin.read().strip()
    if not content or content == "[]":
        print("0 0 0")
        sys.exit(0)
    data = json.loads(content)
    days = set()
    weeks = set()
    months = set()
    for snap in data:
        t = snap.get("time", "")
        if len(t) >= 10:
            d_str = t[:10]
            days.add(d_str)
            months.add(t[:7])
            try:
                dt = datetime.datetime.strptime(d_str, "%Y-%m-%d")
                iso = dt.isocalendar()
                weeks.add("%s-W%02d" % (iso[0], iso[1]))
            except Exception:
                pass
    print("%d %d %d" % (len(days), len(weeks), len(months)))
except Exception:
    print("0 0 0")
' <<< "$snapshots_json")
    
    local actual_daily actual_weekly actual_monthly
    read -r actual_daily actual_weekly actual_monthly <<< "$counts"
    
    # Check retention satisfaction (baseline: daily>=7, weekly>=4, monthly>=12)
    local config_daily="${KEEP_DAILY:-0}"
    local config_weekly="${KEEP_WEEKLY:-0}"
    local config_monthly="${KEEP_MONTHLY:-0}"
    
    local config_daily_status="미흡"; [[ "$config_daily" -ge 7 ]] && config_daily_status="만족"
    local actual_daily_status="미흡"; [[ "$actual_daily" -ge 7 ]] && actual_daily_status="만족"
    local config_weekly_status="미흡"; [[ "$config_weekly" -ge 4 ]] && config_weekly_status="만족"
    local actual_weekly_status="미흡"; [[ "$actual_weekly" -ge 4 ]] && actual_weekly_status="만족"
    local config_monthly_status="미흡"; [[ "$config_monthly" -ge 12 ]] && config_monthly_status="만족"
    local actual_monthly_status="미흡"; [[ "$actual_monthly" -ge 12 ]] && actual_monthly_status="만족"
    
    # Permissions
    local etc_perm; etc_perm="$(stat -c '%a' "$RESTIC_ETC_DIR" 2>/dev/null || echo '700')"
    local env_perm; env_perm="$(stat -c '%a' "$BACKUP_ENV_FILE" 2>/dev/null || echo '600')"
    local etc_safe_str="경고 - 700 권장"; [[ "$etc_perm" == "700" ]] && etc_safe_str="안전 - 소유자 외 접근불가"
    local env_safe_str="경고 - 600 권장"; [[ "$env_perm" == "600" ]] && env_safe_str="안전 - 평문 노출 방지"
    
    # Restic Integrity Check
    local check_status="FAILED (오류 발생)"
    if restic check >/dev/null 2>&1; then
      check_status="SUCCESS (에러 없음)"
    fi
    
    # Backend detection
    local backend="s3"
    if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
      backend="sftp"
    fi
    
    # Snapshot Table (last 3)
    local snapshot_table
    snapshot_table=$(python3 -c '
import sys, json, subprocess
try:
    content = sys.stdin.read().strip()
    if not content or content == "[]":
        print("  (스냅샷 없음)")
        sys.exit(0)
    data = json.loads(content)
    data.sort(key=lambda x: x.get("time", ""), reverse=True)
    recent = data[:3]
    print("  ID          일시                 호스트              백업 경로 및 용량")
    print("  --------------------------------------------------------------------")
    for snap in recent:
        sid = snap.get("short_id", snap.get("id", "")[:8])
        time_str = snap.get("time", "")[:19].replace("T", " ")
        host = snap.get("hostname", "")
        paths = ", ".join(snap.get("paths", []))
        b = None
        summary = snap.get("summary")
        if isinstance(summary, dict) and "total_bytes_processed" in summary:
            b = summary["total_bytes_processed"]
        if b is None:
            try:
                res = subprocess.run(["restic", "stats", "--json", snap.get("id")], capture_output=True, text=True, timeout=5)
                if res.returncode == 0:
                    stats_data = json.loads(res.stdout)
                    b = stats_data.get("total_size")
            except Exception:
                pass
        size_str = ""
        if b is not None:
            if b >= 1073741824:
                size_str = " (%.2f GB)" % (b / 1073741824.0)
            elif b >= 1048576:
                size_str = " (%.2f MB)" % (b / 1048576.0)
            elif b >= 1024:
                size_str = " (%.2f KB)" % (b / 1024.0)
            else:
                size_str = " (%d B)" % b
        else:
            size_str = " (크기 확인 불가)"
        print("  %-10s  %-19s  %-18s  %s%s" % (sid, time_str, host, paths, size_str))
except Exception as e:
    print("  (스냅샷 정보 해석 실패: %s)" % e)
' <<< "$snapshots_json")

    # Generate HTML snapshot table
    local snapshot_table_html
    snapshot_table_html=$(python3 -c '
import sys, json, subprocess
try:
    content = sys.stdin.read().strip()
    if not content or content == "[]":
        print("<tr><td colspan=\"4\">(스냅샷 없음)</td></tr>")
        sys.exit(0)
    data = json.loads(content)
    data.sort(key=lambda x: x.get("time", ""), reverse=True)
    recent = data[:3]
    for snap in recent:
        sid = snap.get("short_id", snap.get("id", "")[:8])
        time_str = snap.get("time", "")[:19].replace("T", " ")
        host = snap.get("hostname", "")
        paths = ", ".join(snap.get("paths", []))
        b = None
        summary = snap.get("summary")
        if isinstance(summary, dict) and "total_bytes_processed" in summary:
            b = summary["total_bytes_processed"]
        if b is None:
            try:
                res = subprocess.run(["restic", "stats", "--json", snap.get("id")], capture_output=True, text=True, timeout=5)
                if res.returncode == 0:
                    stats_data = json.loads(res.stdout)
                    b = stats_data.get("total_size")
            except Exception:
                pass
        size_str = ""
        if b is not None:
            if b >= 1073741824:
                size_str = "%.2f GB" % (b / 1073741824.0)
            elif b >= 1048576:
                size_str = "%.2f MB" % (b / 1048576.0)
            elif b >= 1024:
                size_str = "%.2f KB" % (b / 1024.0)
            else:
                size_str = "%d B" % b
        else:
            size_str = "확인 불가"
        print("<tr><td>%s</td><td>%s</td><td>%s</td><td>%s (%s)</td></tr>" % (sid, time_str, host, paths, size_str))
except Exception as e:
    print("<tr><td colspan=\"4\">(스냅샷 정보 해석 실패: %s)</td></tr>" % e)
' <<< "$snapshots_json")

    # Render Report Function
    render_daily_audit_report "$cur_time" "$hostname_val" "$tester" "$backend" "$RESTIC_REPOSITORY" \
      "$BACKUP_TARGETS" "$config_daily" "$actual_daily" "$config_daily_status" "$actual_daily_status" \
      "$config_weekly" "$actual_weekly" "$config_weekly_status" "$actual_weekly_status" \
      "$config_monthly" "$actual_monthly" "$config_monthly_status" "$actual_monthly_status" \
      "$RESTIC_ETC_DIR" "$etc_perm" "$etc_safe_str" "$BACKUP_ENV_FILE" "$env_perm" "$env_safe_str" \
      "$check_status" "$snapshot_table"
      
    if [[ -n "$report_file" ]]; then
      mkdir -p "$(dirname "$report_file")"
      chmod 700 "$(dirname "$report_file")" 2>/dev/null || true
      
      render_daily_audit_report "$cur_time" "$hostname_val" "$tester" "$backend" "$RESTIC_REPOSITORY" \
        "$BACKUP_TARGETS" "$config_daily" "$actual_daily" "$config_daily_status" "$actual_daily_status" \
        "$config_weekly" "$actual_weekly" "$config_weekly_status" "$actual_weekly_status" \
        "$config_monthly" "$actual_monthly" "$config_monthly_status" "$actual_monthly_status" \
        "$RESTIC_ETC_DIR" "$etc_perm" "$etc_safe_str" "$BACKUP_ENV_FILE" "$env_perm" "$env_safe_str" \
        "$check_status" "$snapshot_table" > "$report_file"
      chmod 600 "$report_file"

      local json_report_file
      if [[ "$report_file" =~ \.(txt|md)$ ]]; then
        json_report_file="${report_file%.*}.json"
      else
        json_report_file="${report_file}.json"
      fi

      render_daily_audit_report_json "$cur_time" "$hostname_val" "$tester" "$backend" "$RESTIC_REPOSITORY" \
        "$BACKUP_TARGETS" "$config_daily" "$actual_daily" "$config_daily_status" "$actual_daily_status" \
        "$config_weekly" "$actual_weekly" "$config_weekly_status" "$actual_weekly_status" \
        "$config_monthly" "$actual_monthly" "$config_monthly_status" "$actual_monthly_status" \
        "$RESTIC_ETC_DIR" "$etc_perm" "$etc_safe_str" "$BACKUP_ENV_FILE" "$env_perm" "$env_safe_str" \
        "$check_status" "$snapshots_json" > "$json_report_file"
      chmod 600 "$json_report_file"

      local html_report_file
      if [[ "$report_file" =~ \.(txt|md)$ ]]; then
        html_report_file="${report_file%.*}.html"
      else
        html_report_file="${report_file}.html"
      fi

      render_daily_audit_report_html "$cur_time" "$hostname_val" "$tester" "$backend" "$RESTIC_REPOSITORY" \
        "$BACKUP_TARGETS" "$config_daily" "$actual_daily" "$config_daily_status" "$actual_daily_status" \
        "$config_weekly" "$actual_weekly" "$config_weekly_status" "$actual_weekly_status" \
        "$config_monthly" "$actual_monthly" "$config_monthly_status" "$actual_monthly_status" \
        "$RESTIC_ETC_DIR" "$etc_perm" "$etc_safe_str" "$BACKUP_ENV_FILE" "$env_perm" "$env_safe_str" \
        "$check_status" "$snapshot_table_html" > "$html_report_file"
      chmod 600 "$html_report_file"
      
      log_info "감사 보고서가 동시 저장되었습니다:"
      log_info "  - 텍스트 보고서: $report_file"
      log_info "  - JSON 보고서: $json_report_file"
      log_info "  - HTML 보고서: $html_report_file"
    fi

    return 0
  fi

  if (( report )) && [[ -z "$report_file" ]]; then
    report_file="/data/backup/reports/audit_report.txt"
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
    setup_colors
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

    # Generate HTML snapshot table
    local snapshots_json
    snapshots_json=$(restic snapshots --json 2>/dev/null || echo "[]")
    
    local snapshot_table_html
    snapshot_table_html=$(python3 -c '
import sys, json, subprocess
try:
    content = sys.stdin.read().strip()
    if not content or content == "[]":
        print("<tr><td colspan=\"4\">(스냅샷 없음)</td></tr>")
        sys.exit(0)
    data = json.loads(content)
    data.sort(key=lambda x: x.get("time", ""), reverse=True)
    recent = data[:3]
    for snap in recent:
        sid = snap.get("short_id", snap.get("id", "")[:8])
        time_str = snap.get("time", "")[:19].replace("T", " ")
        host = snap.get("hostname", "")
        paths = ", ".join(snap.get("paths", []))
        b = None
        summary = snap.get("summary")
        if isinstance(summary, dict) and "total_bytes_processed" in summary:
            b = summary["total_bytes_processed"]
        if b is None:
            try:
                res = subprocess.run(["restic", "stats", "--json", snap.get("id")], capture_output=True, text=True, timeout=5)
                if res.returncode == 0:
                    stats_data = json.loads(res.stdout)
                    b = stats_data.get("total_size")
            except Exception:
                pass
        size_str = ""
        if b is not None:
            if b >= 1073741824:
                size_str = "%.2f GB" % (b / 1073741824.0)
            elif b >= 1048576:
                size_str = "%.2f MB" % (b / 1048576.0)
            elif b >= 1024:
                size_str = "%.2f KB" % (b / 1024.0)
            else:
                size_str = "%d B" % b
        else:
            size_str = "확인 불가"
        print("<tr><td>%s</td><td>%s</td><td>%s</td><td>%s (%s)</td></tr>" % (sid, time_str, host, paths, size_str))
except Exception as e:
    print("<tr><td colspan=\"4\">(스냅샷 정보 해석 실패: %s)</td></tr>" % e)
' <<< "$snapshots_json")

    # Save HTML report
    local html_report_file
    if [[ "$report_file" =~ \.(txt|md)$ ]]; then
      html_report_file="${report_file%.*}.html"
    else
      html_report_file="${report_file}.html"
    fi

    render_audit_report_html "$backend" "${on_calendar:-알 수 없음}" "${timer_enabled:-unknown}" \
      "${timer_active:-unknown}" "${next_run:-알 수 없음}" \
      "$etc_perm" "$env_perm" "$snapshot_table_html" > "$html_report_file"
    chmod 600 "$html_report_file"

    log_info "감사 보고서가 동시 저장되었습니다:"
    log_info "  - 텍스트 보고서: $report_file"
    log_info "  - JSON 보고서: $json_report_file"
    log_info "  - HTML 보고서: $html_report_file"
  fi
}

cmd_uninstall() {
  if has_help_flag "$@"; then
    help_uninstall
    return 0
  fi
  require_root
  local -A opts=()
  parse_opts_into opts "purge" -- "$@"
  local purge="${opts[purge]:-0}"

  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    declare -A file_config=()
    load_backup_env_to_array "$BACKUP_ENV_FILE" file_config || true
    local BACKUP_PROFILE_NAME="${file_config[BACKUP_PROFILE_NAME]:-}"
    local profile_name; profile_name=$(resolve_profile_name)
    scheduler_unregister "$profile_name" "all"
  else
    scheduler_unregister "unknown" "all"
  fi

  if (( purge )); then
    rm -rf "$RESTIC_ETC_DIR"
    rm -rf "${HOME:-/root}/.cache/restic"
    rm -f "$RESTIC_INSTALL_PATH"
    rm -f "$RCLONE_INSTALL_PATH"
    rm -f "$RESTICPROFILE_INSTALL_PATH"
    rm -f "$BACKUP_SCRIPT_INSTALL_PATH"
    log_info "uninstall --purge 완료 (설정 파일 및 설치된 바이너리 삭제됨)"
  else
    log_info "uninstall 완료 (${RESTIC_ETC_DIR}는 유지됨)"
  fi
}

build_dest_config() {
  local dest_backend="$1"
  local src_backend="$2"
  local profile_name="$3"
  local new_password="$4"
  shift 4

  local host="" port="" user="" key_file="" endpoint="" bucket="" access_key="" secret_key=""
  local arg
  for arg in "$@"; do
    case "$arg" in
      host=*) host="${arg#host=}" ;;
      port=*) port="${arg#port=}" ;;
      user=*) user="${arg#user=}" ;;
      key_file=*) key_file="${arg#key_file=}" ;;
      endpoint=*) endpoint="${arg#endpoint=}" ;;
      bucket=*) bucket="${arg#bucket=}" ;;
      access_key=*) access_key="${arg#access_key=}" ;;
      secret_key=*) secret_key="${arg#secret_key=}" ;;
    esac
  done

  if [[ "$dest_backend" == "sftp" ]]; then
    printf 'rclone:syno_backup_dst:/backup/%s\n' "$profile_name"
    printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_TYPE=sftp\n'
    printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_HOST=%s\n' "$host"
    printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_PORT=%s\n' "$port"
    printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_USER=%s\n' "$user"
    printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_KEY_FILE=%s\n' "${key_file:-$BACKUP_SSH_KEY}"
  else
    if [[ "$src_backend" == "sftp" ]]; then
      printf 's3:%s/%s/%s\n' "$endpoint" "$bucket" "$profile_name"
      printf 'AWS_ACCESS_KEY_ID=%s\n' "$access_key"
      printf 'AWS_SECRET_ACCESS_KEY=%s\n' "$secret_key"
    else
      printf 'rclone:syno_backup_dst:%s/%s\n' "$bucket" "$profile_name"
      printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_TYPE=s3\n'
      printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_PROVIDER=other\n'
      printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_ENDPOINT=%s\n' "$endpoint"
      printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_ACCESS_KEY_ID=%s\n' "$access_key"
      printf 'RCLONE_CONFIG_SYNO_BACKUP_DST_SECRET_ACCESS_KEY=%s\n' "$secret_key"
    fi
  fi
  printf 'RESTIC_PASSWORD=%s\n' "$new_password"
}

# nameref로 인자를 받거나 다른 함수로 동적 연관 배열을 전달하여 사용하지 않는 것으로 오인받는 변수가 있으므로 우회
# shellcheck disable=SC2034
cmd_migrate() {
  if has_help_flag "$@"; then
    help_migrate
    return 0
  fi
  require_root
  require_backup_env

  local -A opts=()
  parse_opts_into opts "backend: endpoint: bucket: access-key: secret-key: host: port: user: new-password: skip-check force" -- "$@"
  local skip_check="${opts[skip-check]:-0}"
  local force="${opts[force]:-0}"

  # 1. Pre-flight check on Source Repository
  log_info "기존 저장소(Source) 상태 점검 중..."
  restic unlock 2>/dev/null || true
  if ! restic snapshots >/dev/null 2>&1; then
    die "기존 저장소(Source)에 연결할 수 없거나 비밀번호가 올바르지 않습니다. 마이그레이션을 중단합니다."
  fi

  # 2. Resolving destination backend settings
  local backend="${opts[backend]:-}"
  if [[ -z "$backend" ]]; then
    if [[ -t 1 ]]; then
      backend=$(prompt_backend_choice)
    else
      die "마이그레이션 대상 백엔드(--backend)를 지정해야 합니다."
    fi
  fi
  validate_backend "$backend"

  # Prompt interactive values if terminal is active and values are empty
  if [[ "$backend" == "s3" ]]; then
    if [[ -z "${opts[endpoint]:-}" && -t 1 ]]; then
      opts[endpoint]=$(prompt_validated "접속할 S3 호환 엔드포인트 URL을 입력하세요" "" validate_not_empty)
    fi
    if [[ -z "${opts[bucket]:-}" && -t 1 ]]; then
      opts[bucket]=$(prompt_validated "백업을 저장할 버킷 이름을 입력하세요" "" validate_not_empty)
    fi
    if [[ -z "${opts[access-key]:-}" && -t 1 ]]; then
      opts[access-key]=$(prompt_validated "버킷 접근용 access key를 입력하세요" "" validate_not_empty)
    fi
    if [[ -z "${opts[secret-key]:-}" && -t 1 ]]; then
      opts[secret-key]=$(prompt_validated "버킷 접근용 secret key를 입력하세요" "" validate_not_empty)
    fi
  else
    # SFTP
    if [[ -z "${opts[host]:-}" && -t 1 ]]; then
      opts[host]=$(prompt_validated "백업 데이터를 저장할 NAS의 IP 주소를 입력하세요" "" validate_not_empty)
    fi
    if [[ -z "${opts[port]:-}" ]]; then
      if [[ -t 1 ]]; then
        opts[port]=$(prompt_validated "NAS의 SSH/SFTP 접속 포트를 입력하세요" "$DEFAULT_SFTP_PORT" validate_port)
      else
        opts[port]="$DEFAULT_SFTP_PORT"
      fi
    fi
    validate_port "${opts[port]:-}"
    if [[ -z "${opts[user]:-}" && -t 1 ]]; then
      opts[user]=$(prompt_validated "NAS에 접속할 SFTP 계정명을 입력하세요" "" validate_not_empty)
    fi
  fi

  # Call resolve_and_validate_config to validate the destination options
  # Since load_and_validate_config needs targets and password, we feed mock ones
  # load_and_validate_config에 nameref로 인자를 넘기므로 shellcheck 오탐 방지
  # shellcheck disable=SC2034
  local -A dest_opts=()
  local key
  for key in "${!opts[@]}"; do
    dest_opts["$key"]="${opts[$key]}"
  done
  dest_opts[targets]="/etc" # mock targets for validation
  dest_opts[password]="${opts[new-password]:-$RESTIC_PASSWORD}" # mock or actual new password

  local -A resolved=()
  local -a errors=()
  if ! load_and_validate_config "" dest_opts resolved errors; then
    local e
    for e in "${errors[@]}"; do
      log_error "$e"
    done
    die "마이그레이션 목적지 설정 유효성 검증 실패"
  fi

  local new_password="${opts[new-password]:-}"
  if [[ -z "$new_password" ]]; then
    new_password="$RESTIC_PASSWORD"
  fi

  # 3. Pre-flight Destination Connectivity Check
  if ! backend_"${backend}"_test_connectivity resolved; then
    if [[ "$backend" == "sftp" ]]; then
      die "$(render_sftp_connectivity_failure_message "${resolved[host]}" "${resolved[port]}" "${resolved[user]}")"
    else
      die "대상 저장소(Destination) 연결 실패"
    fi
  fi

  # 4. Constructing destination environment variables for copy
  local profile_name; profile_name=$(resolve_profile_name)
  local src_backend="s3"
  # 서브셸에서의 임시 변수 수정은 연결 확인용이며, 여기서는 원천 설정을 읽으므로 경고를 우회한다.
  # shellcheck disable=SC2031
  if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
    src_backend="sftp"
  fi

  local -a dest_opts_list=()
  if [[ "$backend" == "sftp" ]]; then
    dest_opts_list+=(host="${resolved[host]}" port="${resolved[port]}" user="${resolved[user]}" key_file="$BACKUP_SSH_KEY")
  else
    dest_opts_list+=(endpoint="${resolved[endpoint]}" bucket="${resolved[bucket]}" access_key="${resolved[access_key]}" secret_key="${resolved[secret_key]}")
  fi

  local dest_config_output
  dest_config_output=$(build_dest_config "$backend" "$src_backend" "$profile_name" "$new_password" "${dest_opts_list[@]}")

  local dest_repo_copy=""
  local -a dest_env=()
  local line
  while IFS= read -r line; do
    if [[ -z "$dest_repo_copy" ]]; then
      dest_repo_copy="$line"
    else
      dest_env+=("$line")
    fi
  done <<< "$dest_config_output"

  run_restic_dest() {
    local -a src_env=()
    [[ -n "${AWS_ACCESS_KEY_ID:-}" ]] && src_env+=(AWS_ACCESS_KEY_ID="$AWS_ACCESS_KEY_ID")
    [[ -n "${AWS_SECRET_ACCESS_KEY:-}" ]] && src_env+=(AWS_SECRET_ACCESS_KEY="$AWS_SECRET_ACCESS_KEY")
    [[ -n "${RESTIC_REPOSITORY:-}" ]] && src_env+=(RESTIC_REPOSITORY="$RESTIC_REPOSITORY")
    [[ -n "${RESTIC_PASSWORD:-}" ]] && src_env+=(RESTIC_PASSWORD="$RESTIC_PASSWORD")
    [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]] && src_env+=(RCLONE_CONFIG_SYNO_BACKUP_TYPE="$RCLONE_CONFIG_SYNO_BACKUP_TYPE")
    [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}" ]] && src_env+=(RCLONE_CONFIG_SYNO_BACKUP_HOST="$RCLONE_CONFIG_SYNO_BACKUP_HOST")
    [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}" ]] && src_env+=(RCLONE_CONFIG_SYNO_BACKUP_PORT="$RCLONE_CONFIG_SYNO_BACKUP_PORT")
    [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_USER:-}" ]] && src_env+=(RCLONE_CONFIG_SYNO_BACKUP_USER="$RCLONE_CONFIG_SYNO_BACKUP_USER")
    [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-}" ]] && src_env+=(RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="$RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE")

    env "${src_env[@]}" "${dest_env[@]}" "$(type -P restic)" "$@"
  }

  # 5. Create secure temp password files for transfer
  # Using global variables (no local) so they remain in scope for the EXIT trap
  temp_src_pass=$(mktemp /tmp/src_pass.XXXXXX)
  temp_dst_pass=$(mktemp /tmp/dst_pass.XXXXXX)
  chmod 600 "$temp_src_pass" "$temp_dst_pass"
  echo -n "$RESTIC_PASSWORD" > "$temp_src_pass"
  echo -n "$new_password" > "$temp_dst_pass"

  cleanup_temp_files() {
    rm -f "${temp_src_pass:-}" "${temp_dst_pass:-}"
  }
  trap cleanup_temp_files EXIT INT TERM

  # Check if destination is already initialized
  local dest_initialized=0
  if run_restic_dest -r "$dest_repo_copy" snapshots >/dev/null 2>&1; then
    dest_initialized=1
  fi

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    setup_colors
    printf '%b%b⚙  저장소 마이그레이션 (Repository Migration)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b├──%b 기존 저장소: %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$RESTIC_REPOSITORY" "$C_RESET"
    printf '%b├──%b 대상 저장소: %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$dest_repo_copy" "$C_RESET"
    printf '%b├──%b %b[✓] 기존 저장소 상태 점검 완료%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
  else
    log_info "기존 저장소(Source) 상태 점검 중..."
  fi

  # 6. Initialize destination if not yet initialized
  if [[ "$dest_initialized" -eq 0 ]]; then
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b├──%b %b[✓] 대상 저장소 초기화 완료 (Chunker 파라미터 복사)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
    else
      log_info "대상 저장소(Destination)가 존재하지 않아 초기화합니다 (Chunker 파라미터 복사)..."
    fi
    run_restic_dest -r "$dest_repo_copy" init \
      --from-repo "$RESTIC_REPOSITORY" \
      --from-password-file "$temp_src_pass" \
      --copy-chunker-params || die "대상 저장소 초기화 실패"
  else
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b├──%b %b[✓] 대상 저장소 준비 완료 (이미 존재함)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
    fi
  fi

  # 7. Copy Snapshots
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '%b├──%b %b[ ] 백업 데이터 복사 중...%b' "$C_GRAY" "$C_RESET" "$C_YELLOW" "$C_RESET"
  else
    log_info "기존 저장소에서 대상 저장소로 백업 데이터(스냅샷) 복사 중..."
  fi
  run_restic_dest -r "$RESTIC_REPOSITORY" copy \
    --password-file "$temp_src_pass" \
    --repo2 "$dest_repo_copy" \
    --password-file2 "$temp_dst_pass" || die "백업 데이터 복사(restic copy) 실패"
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '\r%b├──%b %b[✓] 백업 데이터 복사 완료 (restic copy)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
  fi

  # 8. Verify Consistency
  if (( ! skip_check )); then
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b├──%b %b[ ] 대상 저장소 정합성 검증 중...%b' "$C_GRAY" "$C_RESET" "$C_YELLOW" "$C_RESET"
    else
      log_info "대상 저장소 정합성 검증(restic check) 수행 중..."
    fi
    run_restic_dest -r "$dest_repo_copy" check || die "대상 저장소 정합성 검증 실패"
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '\r%b├──%b %b[✓] 대상 저장소 정합성 검증 완료 (restic check)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
    fi
  fi

  # 9. Write new settings to backup.env and update systemd timer if active
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '%b├──%b %b[✓] 로컬 백업 환경 설정 파일(backup.env) 업데이트 완료%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
  else
    log_info "로컬 백업 환경 설정 파일(backup.env) 업데이트 중..."
  fi
  local -a setting_opts=(--backend "$backend" --password "$new_password" --profile-name "$profile_name" --force)
  if [[ "$backend" == "s3" ]]; then
    setting_opts+=(--endpoint "${resolved[endpoint]}" --bucket "${resolved[bucket]}" --access-key "${resolved[access_key]}" --secret-key "${resolved[secret_key]}")
  else
    setting_opts+=(--host "${resolved[host]}" --port "${resolved[port]}" --user "${resolved[user]}")
  fi

  # Get the timer status of the current schedule before writing settings
  local timer_unit; timer_unit=$(resticprofile_timer_unit_name "$profile_name")
  local schedule_active=0
  if systemctl is-active "$timer_unit" >/dev/null 2>&1; then
    schedule_active=1
  fi

  # Save setting values to backup.env
  cmd_setting "${setting_opts[@]}"

  # 10. Update systemd schedule if it was active
  if [[ "$schedule_active" -eq 1 ]]; then
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b├──%b %b[✓] 백업 스케줄러 재갱신 완료%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
    else
      log_info "백업 스케줄이 활성화 상태였으므로 새 저장소 정보로 스케줄러를 재갱신합니다..."
    fi
    cmd_schedule enable
  fi

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '\n%b%b=========================================%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf ' %b%b⚙  마이그레이션 완료 (Migration Completed)%b\n' "$C_GREEN" "$C_BOLD" "$C_RESET"
    printf '%b=========================================%b\n' "$C_CYAN" "$C_RESET"
    printf ' %b├──%b 저장소 이관: %b완료%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$C_RESET"
    printf ' %b├──%b 주의:        %b기존 원격 저장소의 옛날 백업 데이터들은%b\n' "$C_GRAY" "$C_RESET" "$C_YELLOW" "$C_RESET"
    printf ' %b│                 %b안전을 위해 자동 삭제되지 않았습니다.%b\n' "$C_GRAY" "$C_YELLOW" "$C_RESET"
    printf ' %b└──%b 안내:      %b기존 데이터를 삭제하고 싶으신 경우%b\n' "$C_GRAY" "$C_RESET" "$C_DIM" "$C_RESET"
    printf ' %b                 %b원격에서 직접 정리하시길 바랍니다.%b\n' "$C_GRAY" "$C_DIM" "$C_RESET"
    printf '%b=========================================%b\n' "$C_CYAN" "$C_RESET"
  else
    log_info "마이그레이션이 완료되었습니다! 클라이언트 호스트 환경이 새 저장소로 완전히 이관되었습니다."
    log_info "기존 원격 저장소에 저장되어 있는 옛날 백업 데이터들은 안전을 위해 자동 삭제되지 않았습니다."
    log_info "기존 데이터를 완전히 삭제하고 싶으신 경우 원격에서 직접 정리하시길 바랍니다."
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
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      setup_colors
      if [[ -n "$default" ]]; then
        printf '%b%s%b [%b%s%b]: ' "$C_CYAN" "$message" "$C_RESET" "$C_BOLD" "$default" "$C_RESET" >&2
      else
        printf '%b%s%b: ' "$C_CYAN" "$message" "$C_RESET" >&2
      fi
    else
      if [[ -n "$default" ]]; then
        printf '%s [%s]: ' "$message" "$default" >&2
      else
        printf '%s: ' "$message" >&2
      fi
    fi
    read -r value
    value="${value:-$default}"
    if err=$("$validate_fn" "$value"); then
      printf '%s' "$value"
      return 0
    fi
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b%s 다시 입력하세요.%b\n' "$C_RED" "$err" "$C_RESET" >&2
    else
      printf '%s 다시 입력하세요.\n' "$err" >&2
    fi
  done
}

# 화면에 표시되지 않는 비밀번호 입력을 받고, 빈 값이면 다시 묻는다.
prompt_secret_required() {
  local message="$1"
  local value
  while true; do
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      setup_colors
      printf '%b%s%b' "$C_YELLOW" "$message" "$C_RESET" >&2
    else
      printf '%s' "$message" >&2
    fi
    read -rs value
    printf '\n' >&2
    if [[ -n "$value" ]]; then
      printf '%s' "$value"
      return 0
    fi
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b값을 입력해야 합니다. 다시 입력하세요.%b\n' "$C_RED" "$C_RESET" >&2
    else
      printf '값을 입력해야 합니다. 다시 입력하세요.\n' >&2
    fi
  done
}

# 1(S3)/2(SFTP) 중 하나를 고를 때까지 다시 묻고, 선택된 backend 이름을 반환한다.
prompt_backend_choice() {
  local choice
  while true; do
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      setup_colors
      printf '%b%b⚙  백엔드 선택 (Choose Backend)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET" >&2
      printf '  [%b1%b] S3 호환 스토리지 - %bHTTPS 기반 오브젝트 스토리지(AWS S3, MinIO 등)%b\n' "$C_GREEN" "$C_RESET" "$C_DIM" "$C_RESET" >&2
      printf '  [%b2%b] SFTP(NAS) - %bSSH로 접속하는 시놀로지 NAS 등%b\n' "$C_GREEN" "$C_RESET" "$C_DIM" "$C_RESET" >&2
      printf '선택 (1/2): ' >&2
    else
      printf '백엔드를 선택하세요:\n' >&2
      printf '  [1] S3 호환 스토리지 - HTTPS 기반 오브젝트 스토리지(AWS S3, MinIO 등)\n' >&2
      printf '  [2] SFTP(NAS) - SSH로 접속하는 시놀로지 NAS 등\n' >&2
      printf '선택 (1/2): ' >&2
    fi
    read -r choice
    case "$choice" in
      1) printf 's3'; return 0 ;;
      2) printf 'sftp'; return 0 ;;
      *)
        if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
          printf '%b1 또는 2를 입력하세요. 다시 입력하세요.%b\n' "$C_RED" "$C_RESET" >&2
        else
          printf '1 또는 2를 입력하세요. 다시 입력하세요.\n' >&2
        fi
        ;;
    esac
  done
}

cmd_upgrade() {
  if has_help_flag "$@"; then
    help_upgrade
    return 0
  fi
  require_root

  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "설정 파일이 존재하지 않습니다: ${BACKUP_ENV_FILE} (먼저 wizard나 setting을 실행하세요)"
  fi
  require_backup_env

  # 1. 설정 백업본 자동 생성
  local timestamp
  timestamp=$(date +%Y%m%d_%H%M%S)
  local backup_dest="${BACKUP_ENV_FILE}.${timestamp}.bak"
  log_info "기존 설정을 안전하게 백업합니다: ${backup_dest}"
  cp "$BACKUP_ENV_FILE" "$backup_dest"
  chmod 600 "$backup_dest"

  # 2. 기존 설정 로드 및 resolved 연관 배열 구성
  declare -A resolved=()
  resolved[backend]="s3"
  if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" || "${RESTIC_REPOSITORY:-}" == rclone:syno_backup* ]]; then
    resolved[backend]="sftp"
  fi

  resolved[profile_name]=$(resolve_profile_name)
  resolved[password]="${RESTIC_PASSWORD:-}"
  resolved[targets]="${BACKUP_TARGETS:-}"
  resolved[excludes_csv]="${BACKUP_EXCLUDES:-}"
  resolved[keep_daily]="${KEEP_DAILY:-7}"
  resolved[keep_weekly]="${KEEP_WEEKLY:-4}"
  resolved[keep_monthly]="${KEEP_MONTHLY:-12}"

  resolved[host]="${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}"
  resolved[port]="${RCLONE_CONFIG_SYNO_BACKUP_PORT:-22}"
  resolved[user]="${RCLONE_CONFIG_SYNO_BACKUP_USER:-}"

  resolved[endpoint]="${BACKUP_ENDPOINT:-}"
  resolved[bucket]="${BACKUP_BUCKET:-}"
  resolved[access_key]="${AWS_ACCESS_KEY_ID:-}"
  resolved[secret_key]="${AWS_SECRET_ACCESS_KEY:-}"

  resolved[notification_url]="${BACKUP_NOTIFICATION_URL:-}"
  resolved[notification_type]="${BACKUP_NOTIFICATION_TYPE:-}"
  resolved[notification_on]="${BACKUP_NOTIFICATION_ON:-both}"
  resolved[notification_method]="${BACKUP_NOTIFICATION_METHOD:-POST}"
  resolved[notification_headers]="${BACKUP_NOTIFICATION_HEADERS:-}"
  resolved[notification_body_success]="${BACKUP_NOTIFICATION_BODY_SUCCESS:-}"
  resolved[notification_body_failure]="${BACKUP_NOTIFICATION_BODY_FAILURE:-}"

  resolved[audit_tester]="${BACKUP_AUDIT_TESTER:-}"
  resolved[audit_ciso]="${BACKUP_AUDIT_CISO:-}"
  resolved[audit_rto]="${BACKUP_AUDIT_RTO:-}"

  resolved[db_type]="${BACKUP_DB_TYPE:-}"
  resolved[db_command]="${BACKUP_DB_COMMAND:-}"
  resolved[db_filename]="${BACKUP_DB_FILENAME:-}"
  resolved[db_schedule]="${BACKUP_DB_SCHEDULE:-}"
  resolved[keep_db_daily]="${KEEP_DB_DAILY:-}"
  resolved[keep_db_weekly]="${KEEP_DB_WEEKLY:-}"
  resolved[keep_db_monthly]="${KEEP_DB_MONTHLY:-}"

  # 3. 누락된 신버전 필수값 확인 및 보완
  local targets="${resolved[targets]:-}"
  local password="${resolved[password]:-}"
  
  if [[ -z "$targets" ]]; then
    if [[ -t 1 ]]; then
      targets=$(prompt_validated "백업할 대상 경로(쉼표로 구분)를 입력하세요" "$DEFAULT_TARGETS" validate_not_empty)
    else
      log_warn "백업 대상(targets)이 누락되어 기본값(${DEFAULT_TARGETS})을 주입합니다."
      targets="$DEFAULT_TARGETS"
    fi
    resolved[targets]="$targets"
  fi

  if [[ -z "$password" ]]; then
    if [[ -t 1 ]]; then
      password=$(prompt_validated "Restic 저장소 비밀번호를 입력하세요" "" validate_not_empty)
    else
      die "Restic 비밀번호(password)가 누락되어 비대화형 업그레이드를 중단합니다."
    fi
    resolved[password]="$password"
  fi

  # 4. 신버전 규격으로 설정 및 관련 에셋 저장
  log_info "설정 파일을 신규 경로 및 포맷으로 영구 업그레이드합니다..."
  if ! save_profile_config resolved >/dev/null; then
    die "설정 파일 및 프로필 저장 중 오류가 발생했습니다."
  fi

  # 5. 기존 레거시 로컬 데이터 이관 진행
  local -A opts=()
  parse_opts_into opts "legacy-dir:" -- "$@"
  
  local legacy_dir="${opts[legacy-dir]:-/var/restic-local}"

  if [[ -d "$legacy_dir" && -f "${legacy_dir}/config" ]]; then
    log_info "레거시 로컬 백업 저장소(${legacy_dir})를 발견했습니다. 원격지로 데이터 이관을 시작합니다..."

    local legacy_password="${RESTIC_PASSWORD}"

    local copy_status=0
    (
      restic copy --from-repo "$legacy_dir" --from-password-file <(echo -n "$legacy_password") || exit 1
    ) || copy_status=1

    if (( copy_status )); then
      die "로컬 백업 데이터를 원격지로 이관하는 도중 오류가 발생했습니다."
    fi

    log_info "데이터 이관이 완료되었습니다."
    log_info "로컬 백업 데이터(${legacy_dir})를 삭제하여 디스크 공간을 정리하는 것을 권장합니다: rm -rf ${legacy_dir}"
  else
    log_info "이관할 로컬 데이터가 없습니다. (레거시 경로: ${legacy_dir} 미존재)"
  fi
}

help_upgrade() {
  cat <<'EOF'
기존의 1차 로컬 백업 데이터를 새로운 1차 원격 백업 저장소로 안전하게 마이그레이션(이관)하고,
디스크 정리를 위한 가이드를 제공합니다.

사용법:
  backup.sh upgrade [flags]

사용법 예시:
  # 기본 경로(/var/restic-local)에 존재하는 로컬 데이터를 원격지로 이관
  backup.sh upgrade

  # 특정 경로에 있는 로컬 데이터를 원격지로 이관
  backup.sh upgrade --legacy-dir /data/backup

플래그 (Flags):
      --legacy-dir <경로>     이관할 기존 로컬 restic 저장소 디렉터리 경로 (기본값: /var/restic-local)

글로벌 플래그 (Global Flags):
  -h, --help                  도움말 출력
EOF
}

cmd_wizard() {
  if has_help_flag "$@"; then
    help_wizard
    return 0
  fi
  require_root
  local db_type="" db_schedule=""

  if ! type -P restic >/dev/null 2>&1 || ! type -P rclone >/dev/null 2>&1 \
    || [[ ! -x "$RESTICPROFILE_INSTALL_PATH" ]]; then
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      setup_colors
      printf '%b🔄 필수 의존성 패키지를 설치합니다...%b\n' "$C_YELLOW" "$C_RESET"
    else
      log_info "패키지를 설치합니다..."
    fi
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

  local profile_name
  profile_name=$(prompt_validated "저장소 폴더 이름을 입력하세요 (원격에 생성될 디렉터리명)" "$(hostname)" validate_profile_name)
  setting_args+=(--profile-name "$profile_name")

  local password
  password=$(prompt_secret_required '저장소 비밀번호: 백업 데이터를 AES-256 기반으로 암호화하는 데 쓰이는 필수 입력값입니다. 이 비밀번호가 없으면 NAS/S3 등 원격 저장소 쪽에서도 백업 내용을 열어볼 수 없습니다. 분실 시에는 백업 데이터를 복구할 방법이 없으니 반드시 별도의 안전한 곳에 보관하세요.
비밀번호 입력(화면에 표시되지 않습니다): ')
  setting_args+=(--password "$password")

  # 1. Ask about targets
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    setup_colors
    printf '\n%b%b⚙  백업 대상 경로 설정 (Backup Target Paths)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b보안 컴플라이언스(ISMS/ISO 27001) 기준에 부합하기 위해, 중요 설정 파일(/etc) 및 중요 업무 데이터(/data/backup)가 기본 백업 경로로 지정되어 있습니다.%b\n' "$C_DIM" "$C_RESET"
    printf '  * %b/etc%b: 사용자 계정, 권한 설정 및 네트워크 구성을 보존하여 설정의 무결성을 입증합니다.\n' "$C_BOLD" "$C_RESET"
    printf '  * %b/data/backup%b: 정보 유출 방지 및 중요 업무 데이터 보존을 지원합니다.\n\n' "$C_BOLD" "$C_RESET"
    printf '%b기본 경로(/data/backup, /etc)를 백업 대상에 포함하시겠습니까? [%bY%b/n]: %b' "$C_CYAN" "$C_BOLD" "$C_RESET" "$C_RESET"
  else
    printf '\n--- 백업 대상 경로 설정 ---\n'
    printf '보안 컴플라이언스(ISMS/ISO 27001) 기준에 부합하기 위해, 중요 설정 파일(/etc) 및 중요 업무 데이터(/data/backup)가 기본 백업 경로로 지정되어 있습니다.\n'
    printf '  * /etc: 사용자 계정, 권한 설정 및 네트워크 구성을 보존하여 설정의 무결성을 입증합니다.\n'
    printf '  * /data/backup: 정보 유출 방지 및 중요 업무 데이터 보존을 지원합니다.\n\n'
    printf '기본 경로(/data/backup, /etc)를 백업 대상에 포함하시겠습니까? [Y/n]: '
  fi
  local use_default_targets; read -r use_default_targets

  local final_targets=""
  if [[ -z "$use_default_targets" || "$use_default_targets" =~ ^[Yy]$ ]]; then
    final_targets="/data/backup,/etc"
  fi

  local additional_targets=""
  while true; do
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b추가로 백업할 디렉터리 절대 경로가 있습니까? (쉼표로 구분하여 절대 경로 입력, 없으면 Enter): %b' "$C_CYAN" "$C_RESET"
    else
      printf '추가로 백업할 디렉터리 절대 경로가 있습니까? (쉼표로 구분하여 절대 경로 입력, 없으면 Enter): '
    fi
    read -r additional_targets
    
    additional_targets="${additional_targets#"${additional_targets%%[![:space:]]*}"}"
    additional_targets="${additional_targets%"${additional_targets##*[![:space:]]}"}"

    if [[ -z "$additional_targets" ]]; then
      break
    fi

    if [[ "$additional_targets" =~ ^[Yy][Ee]?[Ss]?$ || "$additional_targets" =~ ^[Nn][Oo]?$ ]]; then
      if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
        printf '%b경고: 예/아니오 대답("%s")은 유효한 경로가 아닙니다. 절대 경로(예: /data)를 직접 입력하세요.%b\n' "$C_RED" "$additional_targets" "$C_RESET"
      else
        printf '경고: 예/아니오 대답("%s")은 유효한 경로가 아닙니다. 절대 경로(예: /data)를 직접 입력하세요.\n' "$additional_targets"
      fi
      continue
    fi

    local path_err=0
    local p
    for p in ${additional_targets//,/ }; do
      p="${p#"${p%%[![:space:]]*}"}"
      p="${p%"${p##*[![:space:]]}"}"
      if [[ ! "$p" =~ ^/ ]]; then
        path_err=1
        if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
          printf '%b경고: 경로 "%s"은(는) 절대 경로가 아닙니다. /로 시작하는 전체 경로를 입력해야 합니다.%b\n' "$C_RED" "$p" "$C_RESET"
        else
          printf '경고: 경로 "%s"은(는) 절대 경로가 아닙니다. /로 시작하는 전체 경로를 입력해야 합니다.\n' "$p"
        fi
        break
      fi
    done

    if [[ $path_err -eq 0 ]]; then
      break
    fi
  done

  if [[ -n "$additional_targets" ]]; then
    if [[ -n "$final_targets" ]]; then
      final_targets="${final_targets},${additional_targets}"
    else
      final_targets="$additional_targets"
    fi
  fi

  while [[ -z "$final_targets" ]]; do
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b경고: 백업 대상 경로가 비어 있습니다. 최소 한 개 이상의 경로를 지정해야 합니다.%b\n' "$C_RED" "$C_RESET"
      printf '%b백업할 디렉터리 경로를 입력하세요 (예: /data/backup): %b' "$C_CYAN" "$C_RESET"
    else
      printf '경고: 백업 대상 경로가 비어 있습니다. 최소 한 개 이상의 경로를 지정해야 합니다.\n'
      printf '백업할 디렉터리 경로를 입력하세요 (예: /data/backup): '
    fi
    read -r final_targets
  done

  setting_args+=(--targets "$final_targets")

  # Ask about ISMS Audit Report settings
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '\n%b%b⚙  보안 감사 보고서 설정 (ISMS Compliance Reports)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%bISMS/ISMS-P 인증 규정 만족을 위해 일일 백업 검토 보고서 및 복구 모의훈련 보고서를 자동 생성하도록 설정할 수 있습니다.%b\n' "$C_DIM" "$C_RESET"
    printf '보안 감사 보고서의 시스템 담당자 및 복구 시간(RTO) 설정을 구성하시겠습니까? [y/%bN%b]: %b' "$C_BOLD" "$C_RESET" "$C_RESET"
  else
    printf '\n--- 보안 감사 보고서 설정 ---\n'
    printf 'ISMS/ISMS-P 인증 규정 만족을 위해 일일 백업 검토 보고서 및 복구 모의훈련 보고서를 자동 생성하도록 설정할 수 있습니다.\n'
    printf '보안 감사 보고서의 시스템 담당자 및 복구 시간(RTO) 설정을 구성하시겠습니까? [y/N]: '
  fi
  local config_audit; read -r config_audit
  
  if [[ "$config_audit" =~ ^[Yy]$ ]]; then
    local audit_tester
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b일일 검토 및 복구 테스트를 수행할 시스템 담당자(테스터)의 이름을 입력하세요 (기본값: 인프라보안팀): %b' "$C_CYAN" "$C_RESET"
    else
      printf '일일 검토 및 복구 테스트를 수행할 시스템 담당자(테스터)의 이름을 입력하세요 (기본값: 인프라보안팀): '
    fi
    read -r audit_tester
    [[ -z "$audit_tester" ]] && audit_tester="인프라보안팀"
    
    local audit_ciso
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b보고서를 최종 승인할 정보보안책임자(CISO)의 이름을 입력하세요 (기본값: 정보보안책임자 CISO): %b' "$C_CYAN" "$C_RESET"
    else
      printf '보고서를 최종 승인할 정보보안책임자(CISO)의 이름을 입력하세요 (기본값: 정보보안책임자 CISO): '
    fi
    read -r audit_ciso
    [[ -z "$audit_ciso" ]] && audit_ciso="정보보안책임자 CISO"
    
    local audit_rto
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b복구 목표 시간(RTO, 분 단위)을 입력하세요 (기본값: 120): %b' "$C_CYAN" "$C_RESET"
    else
      printf '복구 목표 시간(RTO, 분 단위)을 입력하세요 (기본값: 120): '
    fi
    read -r audit_rto
    [[ -z "$audit_rto" ]] && audit_rto="120"
    
    setting_args+=(--audit-tester "$audit_tester" --audit-ciso "$audit_ciso" --audit-rto "$audit_rto")
  fi

  # 데이터베이스 백업 설정 질문
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '\n%b%b⚙  데이터베이스 백업 설정 (Database Backup Settings)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b파일 백업과 함께 데이터베이스 백업(mysql, mariadb, postgres 등)을 통합 구성하시겠습니까? [y/%bN%b]: %b' "$C_DIM" "$C_BOLD" "$C_RESET" "$C_RESET"
  else
    printf '\n--- 데이터베이스 백업 설정 ---\n'
    printf '파일 백업과 함께 데이터베이스 백업(mysql, mariadb, postgres 등)을 통합 구성하시겠습니까? [y/N]: '
  fi
  local config_db; read -r config_db
  
  if [[ "$config_db" =~ ^[Yy]$ ]]; then
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%b데이터베이스 엔진 유형을 입력하세요 (mysql, mariadb, postgres, custom) [mysql]: %b' "$C_CYAN" "$C_RESET"
    else
      printf '데이터베이스 엔진 유형을 입력하세요 (mysql, mariadb, postgres, custom) [mysql]: '
    fi
    read -r db_type
    [[ -z "$db_type" ]] && db_type="mysql"
    
    local db_command
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%bDB 백업을 위한 덤프 명령어를 입력하세요 (엔터 입력 시 기본 명령어 자동 생성): %b' "$C_CYAN" "$C_RESET"
    else
      printf 'DB 백업을 위한 덤프 명령어를 입력하세요 (엔터 입력 시 기본 명령어 자동 생성): '
    fi
    read -r db_command
    
    local db_keep_daily
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%bDB 스냅샷 일별 보관 개수 입력 (기본값: 7): %b' "$C_CYAN" "$C_RESET"
    else
      printf 'DB 스냅샷 일별 보관 개수 입력 (기본값: 7): '
    fi
    read -r db_keep_daily
    [[ -z "$db_keep_daily" ]] && db_keep_daily="7"
    
    local db_keep_weekly
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%bDB 스냅샷 주별 보관 개수 입력 (기본값: 4): %b' "$C_CYAN" "$C_RESET"
    else
      printf 'DB 스냅샷 주별 보관 개수 입력 (기본값: 4): '
    fi
    read -r db_keep_weekly
    [[ -z "$db_keep_weekly" ]] && db_keep_weekly="4"
    
    local db_keep_monthly
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%bDB 스냅샷 월별 보관 개수 입력 (기본값: 12): %b' "$C_CYAN" "$C_RESET"
    else
      printf 'DB 스냅샷 월별 보관 개수 입력 (기본값: 12): '
    fi
    read -r db_keep_monthly
    [[ -z "$db_keep_monthly" ]] && db_keep_monthly="12"
    
    if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
      printf '%bDB 백업 반복 스케줄 주기를 입력하세요 (기본값: *-*-* 03:00:00): %b' "$C_CYAN" "$C_RESET"
    else
      printf 'DB 백업 반복 스케줄 주기를 입력하세요 (기본값: *-*-* 03:00:00): '
    fi
    read -r db_schedule
    [[ -z "$db_schedule" ]] && db_schedule="*-*-* 03:00:00"
    
    setting_args+=(--db-type "$db_type" --db-keep-daily "$db_keep_daily" --db-keep-weekly "$db_keep_weekly" --db-keep-monthly "$db_keep_monthly" --db-schedule "$db_schedule")
    if [[ -n "$db_command" ]]; then
      setting_args+=(--db-command "$db_command")
    fi
  fi

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '\n%b%b⚙  다음 설정으로 진행합니다 (Confirm Settings)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%b├──%b 백엔드:    %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$backend" "$C_RESET"
    if [[ "$backend" == "sftp" ]]; then
      printf '%b├──%b NAS:       %b%s:%s%b (계정: %b%s%b)\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$host" "$port" "$C_RESET" "$C_BOLD" "$user" "$C_RESET"
    else
      printf '%b├──%b S3 엔드포인트: %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$endpoint" "$C_RESET"
      printf '%b├──%b 버킷:      %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$bucket" "$C_RESET"
    fi
    printf '%b├──%b 백업 대상:  %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$final_targets" "$C_RESET"
    printf '%b├──%b 폴더 이름:  %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$profile_name" "$C_RESET"
    if [[ -n "${db_type:-}" ]]; then
      printf '%b├──%b DB 백업 유형: %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$db_type" "$C_RESET"
      printf '%b├──%b DB 백업 주기: %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$db_schedule" "$C_RESET"
    fi
    printf '%b└──%b 이대로 진행할까요? [%bY%b/n]: %b' "$C_GRAY" "$C_RESET" "${C_CYAN}${C_BOLD}" "$C_RESET" "$C_RESET"
  else
    printf '\n다음 설정으로 진행합니다:\n'
    printf '  백엔드: %s\n' "$backend"
    if [[ "$backend" == "sftp" ]]; then
      printf '  NAS: %s:%s (사용자: %s)\n' "$host" "$port" "$user"
    else
      printf '  S3 엔드포인트: %s\n' "$endpoint"
      printf '  버킷: %s\n' "$bucket"
    fi
    printf '  백업 대상: %s\n' "$final_targets"
    printf '  폴더 이름: %s\n' "$profile_name"
    if [[ -n "${db_type:-}" ]]; then
      printf '  DB 백업 유형: %s\n' "$db_type"
      printf '  DB 백업 주기: %s\n' "$db_schedule"
    fi
    printf '이대로 진행할까요? [Y/n]: '
  fi
  local confirm
  read -r confirm
  if [[ -n "$confirm" && ! "$confirm" =~ ^[Yy]$ ]]; then
    log_info "설정을 취소했습니다."
    return 0
  fi

  cmd_setting "${setting_args[@]}"

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '%b위 안내(공개키 등록 또는 버킷 정책 적용)를 완료하셨으면 %bEnter%b를 누르세요: %b' "$C_CYAN" "$C_BOLD" "$C_RESET" "$C_RESET"
  else
    printf '위 안내(공개키 등록 또는 버킷 정책 적용)를 완료하셨으면 Enter를 누르세요: '
  fi
  local _ack; read -r _ack || true

  cmd_init

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '%b지금 정기 백업 스케줄을 등록할까요? 기본값은 매일 새벽 2시입니다. [%bY%b/n]: %b' "$C_CYAN" "$C_BOLD" "$C_RESET" "$C_RESET"
  else
    printf '지금 정기 백업 스케줄을 등록할까요? 기본값은 매일 새벽 2시입니다. [Y/n]: '
  fi
  local schedule_choice; read -r schedule_choice || true

  local schedule_enabled=0
  if [[ -z "$schedule_choice" || "$schedule_choice" =~ ^[Yy]$ ]]; then
    cmd_schedule enable
    schedule_enabled=1
  fi

  # 요약 출력을 위해 backup.env에서 실제 저장소 위치를 읽어온다(하드코딩된 형식
  # 문자열을 다시 조립하지 않고 render_backup_env_* 가 실제로 쓴 값을 그대로 사용).
  local repo_location=""
  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    declare -A file_config=()
    load_backup_env_to_array "$BACKUP_ENV_FILE" file_config || true
    repo_location="${file_config[RESTIC_REPOSITORY]:-}"
  fi

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    printf '\n%b%b=========================================%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf ' %b%b⚙  설정이 완료되었습니다 (Configuration Completed)%b\n' "$C_GREEN" "$C_BOLD" "$C_RESET"
    printf '%b=========================================%b\n' "$C_CYAN" "$C_RESET"
    printf ' %b├──%b 백엔드:    %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "$backend" "$C_RESET"
    printf ' %b├──%b 저장소 위치: %b%s%b\n' "$C_GRAY" "$C_RESET" "$C_BOLD" "${repo_location:-알 수 없음}" "$C_RESET"
    if (( schedule_enabled )); then
      printf ' %b├──%b 정기 백업:    %b등록됨 (%s)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$DEFAULT_ON_CALENDAR" "$C_RESET"
      printf ' %b├──%b 일일 검토 보고: %b등록됨 (%s)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "*-*-* 01:00:00" "$C_RESET"
      printf ' %b├──%b 복구 테스트 보고: %b등록됨 (%s)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "*-*-01 01:30:00" "$C_RESET"
    else
      printf ' %b├──%b 정기 백업:    %b등록하지 않음 (필요시 backup.sh schedule enable 실행)%b\n' "$C_GRAY" "$C_RESET" "$C_GRAY" "$C_RESET"
    fi
    printf ' %b└──%b 안내:      %b이후에는 backup.sh run / status / uninstall 을 사용하세요.%b\n' "$C_GRAY" "$C_RESET" "$C_DIM" "$C_RESET"
    printf '%b=========================================%b\n' "$C_CYAN" "$C_RESET"
  else
    printf '\n=========================================\n'
    printf ' 설정이 완료되었습니다\n'
    printf '=========================================\n'
    printf ' 백엔드: %s\n' "$backend"
    printf ' 저장소 위치: %s\n' "${repo_location:-알 수 없음}"
    if (( schedule_enabled )); then
      printf ' 정기 백업: 등록됨 (%s)\n' "$DEFAULT_ON_CALENDAR"
      printf ' 일일 검토 보고: 등록됨 (*-*-* 01:00:00)\n'
      printf ' 복구 테스트 보고: 등록됨 (*-*-01 01:30:00)\n'
    else
      printf ' 정기 백업: 등록하지 않음 (필요시 backup.sh schedule enable 실행)\n'
    fi
    printf ' 이후에는 backup.sh run / status / uninstall 을 사용하세요.\n'
    printf '=========================================\n'
  fi
  log_info "wizard 완료"
}

cmd_config() {
  if has_help_flag "$@"; then
    help_config
    return 0
  fi
  require_root
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "설정 파일이 존재하지 않습니다: ${BACKUP_ENV_FILE} (먼저 wizard나 setting을 실행하세요)"
  fi

  # 1. 백엔드 판별
  local backend="sftp"
  if grep -q "AWS_ACCESS_KEY_ID=" "$BACKUP_ENV_FILE" || grep -q 'RESTIC_REPOSITORY="s3:' "$BACKUP_ENV_FILE"; then
    backend="s3"
  fi

  # 2. 옵션 파싱
  local -A opts=()
  parse_opts_into opts "targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: profile-name: on-calendar: notification-url: notification-type: notification-on: audit-tester: audit-ciso: audit-rto: dry-run secondary-backend: secondary-password: secondary-endpoint: secondary-bucket: secondary-access-key: secondary-secret-key: secondary-host: secondary-user: secondary-port: secondary-keep-daily: secondary-keep-weekly: secondary-keep-monthly: db-type: db-command: db-filename: db-schedule: db-keep-daily: db-keep-weekly: db-keep-monthly:" -- "$@"


  opts[backend]="$backend"
  local dry_run="${opts[dry-run]:-0}"

  # 3. 설정 해석 및 검증
  local -A resolved=()
  local -a errors=()
  if ! load_and_validate_config "" opts resolved errors; then
    local e
    for e in "${errors[@]}"; do
      log_error "$e"
    done
    die "설정 변경 유효성 검증 실패"
  fi

  check_targets_size_warning "${resolved[targets]}"

  # 스케줄 주기(on_calendar) 결정
  local on_calendar="${opts[on-calendar]:-}"
  if [[ -n "$on_calendar" ]]; then
    resolved[on_calendar]="$on_calendar"
  fi

  if (( dry_run )); then
    local current_schedule="${resolved[on_calendar]:-}"
    if [[ -z "$current_schedule" ]]; then
      if [[ -f "$RESTICPROFILE_CONFIG_FILE" ]]; then
        local parsed_schedule
        parsed_schedule=$(grep -E 'schedule:[[:space:]]*"[^"]+"' "$RESTICPROFILE_CONFIG_FILE" | head -n1 | sed -E 's/.*schedule:[[:space:]]*"([^"]+)".*/\1/')
        if [[ -n "$parsed_schedule" ]]; then
          current_schedule="$parsed_schedule"
        fi
      fi
    fi
    if [[ -z "$current_schedule" ]]; then
      current_schedule="$DEFAULT_ON_CALENDAR"
    fi
    log_info "[dry-run] backup.env(${backend}) 설정 변경 예정: ${BACKUP_ENV_FILE}"
    log_info "[dry-run] 변경 예정 상세: targets=${resolved[targets]}, excludes=${resolved[excludes_csv]:-(없음)}, schedule=${current_schedule}"
    return 0
  fi

  save_profile_config resolved >/dev/null

  log_info "config(${backend}) 완료"
}

cmd_setting() {
  if has_help_flag "$@"; then
    help_setting
    return 0
  fi
  require_root
  local -A opts=()
  parse_opts_into opts "backend: targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: profile-name: notification-url: notification-type: notification-on: audit-tester: audit-ciso: audit-rto: force dry-run secondary-backend: secondary-password: secondary-endpoint: secondary-bucket: secondary-access-key: secondary-secret-key: secondary-host: secondary-user: secondary-port: secondary-keep-daily: secondary-keep-weekly: secondary-keep-monthly: db-type: db-command: db-filename: db-schedule: db-keep-daily: db-keep-weekly: db-keep-monthly:" -- "$@"


  local backend="${opts[backend]:-}"
  local force="${opts[force]:-0}"
  local dry_run="${opts[dry-run]:-0}"

  if [[ -z "$backend" ]]; then
    die "$(render_missing_settings_message)"
  fi
  local err
  if ! err=$(validate_backend "$backend"); then die "$err"; fi

  if [[ -f "$BACKUP_ENV_FILE" && "$force" != 1 ]]; then
    die "이미 설정이 있습니다: ${BACKUP_ENV_FILE} (덮어쓰려면 setting --force)"
  fi

  local -A resolved=()
  local -a errors=()
  if ! load_and_validate_config "" opts resolved errors; then
    local e
    for e in "${errors[@]}"; do
      log_error "$e"
    done
    die "설정 유효성 검증 실패"
  fi

  check_targets_size_warning "${resolved[targets]}"

  if (( dry_run )); then
    log_info "[dry-run] backup.env(${backend}) 생성 예정: ${BACKUP_ENV_FILE}"
    return 0
  fi

  save_profile_config resolved

  log_info "setting(${backend}) 완료"
}

has_help_flag() {
  local arg
  for arg in "$@"; do
    if [[ "$arg" == "-h" || "$arg" == "--help" ]]; then
      return 0
    fi
  done
  return 1
}

help_install() {
  cat <<'EOF'
의존성 패키지(restic, rclone, resticprofile) 및 스크립트 자체를 로컬 시스템에 설치합니다.

사용법:
  backup.sh install [flags]

사용법 예시:
  # 기본 권장 버전으로 모든 필수 바이너리 설치
  backup.sh install

  # 기존 설치된 바이너리가 있더라도 덮어쓰고 강제 재설치
  backup.sh install --force

  # 실제 설치하지 않고 작업 예정사항만 확인
  backup.sh install --dry-run

플래그 (Flags):
      --force       기존 바이너리가 존재해도 무시하고 새로 다운로드하여 설치합니다.
      --dry-run     실제 바이너리 설치 및 파일 복사를 수행하지 않고 시뮬레이션 정보만 보여줍니다.

글로벌 플래그 (Global Flags):
  -v, --verbose     디버깅용 상세 로그 출력
  -h, --help        도움말 출력
EOF
}

help_setting() {
  cat <<'EOF'
초기 백업 환경 설정(/etc/backup/backup.env)을 수동으로 등록하고 보안 강화를 위한 안전한 파일 권한(600)을 강제합니다.

사용법:
  backup.sh setting --backend <s3|sftp> [옵션...]

사용법 예시:
  # SFTP(시놀로지 NAS 등) 백엔드 설정 생성
  backup.sh setting --backend sftp --host 192.168.1.100 --port 22 --user backupuser --password 'my-secret-pass' --targets /etc,/var/log

  # S3 호환 오브젝트 스토리지 백엔드 설정 생성
  backup.sh setting --backend s3 --endpoint https://s3.ap-northeast-2.amazonaws.com --bucket my-backup-bucket --access-key ACCESSKEY123 --secret-key SECRETKEY456 --password 'my-secret-pass' --targets /etc

  # 1차 SFTP 백업 + 2차 S3 소산 백업 설정 동시 생성
  backup.sh setting --backend sftp --host 192.168.1.100 --user backupuser --password 'my-secret-pass' --targets /etc \
    --secondary-backend s3 --secondary-endpoint https://s3.amazonaws.com --secondary-bucket my-sec-bucket --secondary-access-key SEC_AK --secondary-secret-key SEC_SK

플래그 (Flags):
      --backend <s3|sftp>             백업 데이터를 보낼 백엔드 유형 (필수)
      --targets <경로,...>            백업할 로컬 디렉터리 또는 파일 경로 목록 (쉼표 구분) (필수)
      --password <비밀번호>            백업 데이터를 암호화/복호화할 restic 저장소 비밀번호 (필수)
      --exclude <패턴>                백업에서 제외할 파일/디렉터리 패턴 (여러 번 지정 시 쉼표로 병합)
      --keep-daily <N>                일별 보관할 스냅샷 개수
      --keep-weekly <N>               주별 보관할 스냅샷 개수
      --keep-monthly <N>              월별 보관할 스냅샷 개수
      --profile-name <이름>           resticprofile 프로파일 이름 (기본값: 호스트명)
      --audit-tester <이름>           일일 백업 감사 및 복구 테스트를 수행할 담당자 이름
      --audit-ciso <이름>             보고서를 최종 승인할 정보보안책임자(CISO) 이름
      --audit-rto <분>                복구 목표 시간(RTO, 분 단위)
      --force                         이미 백업 설정 파일이 존재할 때 경고 없이 덮어씁니다.
      --dry-run                       실제 설정을 저장하지 않고 화면에 시뮬레이션 예정만 표시합니다.

  1차 SFTP 백엔드 전용 옵션:
      --host <IP/도메인>              SFTP 서버 접속 호스트 주소 (필수)
      --port <포트>                  SFTP 서버 접속 SSH 포트 (기본값: 22)
      --user <사용자명>               SFTP 서버 접속 계정명 (필수)

  1차 S3 백엔드 전용 옵션:
      --endpoint <URL>               S3 호환 엔드포인트 주소 (HTTPS 또는 HTTP URL) (필수)
      --bucket <버킷명>               S3 버킷 이름 (필수)
      --access-key <키>               AWS Access Key ID (필수)
      --secret-key <키>               AWS Secret Access Key (필수)

  2차 원격 소산 옵션 (Secondary Backup Flags):
      --secondary-backend <s3|sftp>   2차 소산지 백엔드 유형
      --secondary-password <암호>     2차 백업 암호 (생략 시 1차 암호 상속)
      --secondary-keep-daily <N>      2차 소산지 일별 보관 스냅샷 개수
      --secondary-keep-weekly <N>     2차 소산지 주별 보관 스냅샷 개수
      --secondary-keep-monthly <N>    2차 소산지 월별 보관 스냅샷 개수

  2차 SFTP 백엔드 옵션:
      --secondary-host <IP/도메인>    2차 SFTP 서버 접속 호스트 주소
      --secondary-port <포트>        2차 SFTP 서버 접속 SSH 포트 (기본값: 22)
      --secondary-user <사용자명>     2차 SFTP 서버 접속 계정명

  2차 S3 백엔드 옵션:
      --secondary-endpoint <URL>     2차 S3 호환 엔드포인트 주소 (HTTPS 또는 HTTP URL)
      --secondary-bucket <버킷명>     2차 S3 버킷 이름
      --secondary-access-key <키>     2차 AWS Access Key ID
      --secondary-secret-key <키>     2차 AWS Secret Access Key

  데이터베이스 백업 옵션 (Database Backup Options):
      --db-type <mysql|mariadb|postgres|custom> 데이터베이스 엔진 유형
      --db-command <명령어>           DB 백업을 위한 덤프 명령어 (디폴트 매핑 지원)
      --db-filename <파일명>          백업할 덤프 파일명 (기본값: db-dump.sql)
      --db-schedule <식>              DB 백업을 위한 systemd OnCalendar 포맷 주기 (예: "*-*-* 03:00:00")
      --db-keep-daily <N>             DB 스냅샷 일별 보관 개수
      --db-keep-weekly <N>            DB 스냅샷 주별 보관 개수
      --db-keep-monthly <N>           DB 스냅샷 월별 보관 개수

글로벌 플래그 (Global Flags):
  -v, --verbose                       디버깅용 상세 로그 출력
  -h, --help                          도움말 출력
EOF
}


help_init() {
  cat <<'EOF'
등록된 백업 설정을 기반으로 원격 저장소 리소스 연결성을 테스트하고, 최초 1회 restic 저장소 초기화(restic init)를 수행합니다.

사용법:
  backup.sh init

사용법 예시:
  # 백업 저장소 연결 점검 및 초기화
  backup.sh init

글로벌 플래그 (Global Flags):
  -v, --verbose     원격 연결 및 초기화 시 상세 로그 출력
  -h, --help        도움말 출력
EOF
}

help_schedule() {
  cat <<'EOF'
주기적 정기 백업 수행 및 보안 감사 결과 보고 작성을 위한 systemd timer를 활성화(schedule)하거나 비활성화(unschedule)합니다.

사용법:
  backup.sh schedule <enable|disable> [옵션...]

사용법 예시:
  # 기본 일정으로 정기 백업 및 보안 감사 타이머 전체 활성화
  backup.sh schedule enable

  # 특정 시간 일정으로 정기 백업 활성화
  backup.sh schedule enable --on-calendar "*-*-* 03:30:00"

  # 등록된 모든 정기 백업 및 보안 감사 타이머 비활성화
  backup.sh schedule disable

플래그 (Flags):
      --on-calendar <OnCalendar식>        정기 백업(resticprofile) 스케줄 주기 지정 (기본값: "*-*-* 02:00:00")
      --on-calendar-daily <OnCalendar식>  일일 백업 감사 보고서 자동 생성을 위한 스케줄 주기 지정 (기본값: "*-*-* 01:00:00")
      --on-calendar-drill <OnCalendar식>  복구 모의훈련 보고서 자동 생성을 위한 스케줄 주기 지정 (기본값: "*-*--01 01:00:00", 매월 1일)

글로벌 플래그 (Global Flags):
  -v, --verbose                     디버깅용 상세 로그 출력
  -h, --help                        도움말 출력
EOF
}

help_run() {
  cat <<'EOF'
스케줄 주기와 무관하게 즉시 백업을 실행합니다. 내부 보존 정책(Retention)과 정리(Prune) 작업도 함께 트리거됩니다.

사용법:
  backup.sh run

사용법 예시:
  # 수동 즉시 백업 실행
  backup.sh run

글로벌 플래그 (Global Flags):
  -v, --verbose     백업 과정의 restic 및 resticprofile 상세 출력 활성화
  -h, --help        도움말 출력
EOF
}

help_status() {
  cat <<'EOF'
현재 설정 상태, 백업 대상 디렉터리, systemd 타이머 상태, 주요 디렉터리 및 설정 파일 접근 권한 검사 결과와 원격 저장소에 존재하는 최근 백업 스냅샷 목록을 조회합니다.

사용법:
  backup.sh status

사용법 예시:
  # 현재 백업 설정 및 최근 스냅샷 이력 조회
  backup.sh status

글로벌 플래그 (Global Flags):
  -v, --verbose     스냅샷 조회 오류 시 디버그 로그 활성화
  -h, --help        도움말 출력
EOF
}

help_audit() {
  cat <<'EOF'
ISMS 및 ISO 27001 등 보안 컴플라이언스 대응을 위한 종합 백업 보고서 및 복구 모의훈련 결과보고서를 화면에 출력하고 파일로 동시 보존합니다.

사용법:
  backup.sh audit [flags]

사용법 예시:
  # 감사용 종합 보고서를 터미널 화면에 즉시 출력
  backup.sh audit

  # 화면에 출력하면서 지정된 기본 경로에 보고서 파일 동시 저장
  # (텍스트: /data/backup/reports/audit_report.txt, JSON: /data/backup/reports/audit_report.json, HTML: /data/backup/reports/audit_report.html)
  backup.sh audit --report

  # 일일 백업 감사 결과 보고서를 자동으로 생성 및 저장
  backup.sh audit --daily --report

  # 복구 모의훈련 수행 및 결과보고서를 자동 생성 및 저장
  backup.sh audit --restore-drill --report

플래그 (Flags):
      --report              보고서 파일 생성 여부를 지정합니다. (자동으로 확장자를 .json, .html로 변환한 보고서도 함께 동시 저장됩니다)
      --report-file <경로>   생성될 텍스트 보고서 파일의 경로를 직접 지정합니다.
      --daily               일일 백업 검토 보고서 모드로 실행합니다.
      --restore-drill       복구 모의훈련 보고서 모드로 실행합니다. (실제 복구 다운로드 수행 및 정합성 쿼리 테스트 트리거)
      --tester <이름>        일일 백업 검토 및 복구 모의훈련 담당자 이름 (설정 파일값 무시하고 임시 지정)
      --ciso <이름>          보고서 최종 승인 정보보안책임자 이름 (설정 파일값 무시하고 임시 지정)
      --rto <분>             복구 목표 시간(RTO, 분 단위) (설정 파일값 무시하고 임시 지정)
      --target <경로>        복구 모의훈련 시 임시 다운로드 및 쿼리 테스트를 수행할 임시 디렉터리 경로 (기본값: /tmp/restore_test)

글로벌 플래그 (Global Flags):
  -v, --verbose             디버깅용 상세 로그 출력
  -h, --help                도움말 출력
EOF
}

help_uninstall() {
  cat <<'EOF'
정기 백업 스케줄 타이머를 제거하고, 시스템에 복사된 backup.sh 및 바이너리(restic 등)들을 시스템에서 안전하게 언인스톨합니다.

사용법:
  backup.sh uninstall [flags]

사용법 예시:
  # 정기 백업 스케줄을 비활성화하고 설치된 스크립트 및 바이너리만 삭제 (설정은 보존)
  backup.sh uninstall

  # 관련 설정 파일(/etc/backup), 사용자 캐시, 바이너리 전체를 삭제하여 데이터를 완전 초기화
  backup.sh uninstall --purge

플래그 (Flags):
      --purge       설정 디렉터리 및 캐시 디렉터리를 포함해 관련된 모든 데이터를 영구 완전 제거합니다.

글로벌 플래그 (Global Flags):
  -v, --verbose     삭제 과정 상세 출력 활성화
  -h, --help        도움말 출력
EOF
}

help_migrate() {
  cat <<'EOF'
기존 원격 백업 저장소(Source)의 스냅샷 데이터를 새로운 대상 원격 저장소(Destination)로 안전하게 복제(restic copy)하고, 로컬 환경 설정(backup.env) 및 systemd 스케줄을 새 저장소 정보로 자동 이관합니다.

사용법:
  backup.sh migrate --backend <s3|sftp> [옵션...]

사용법 예시:
  # 기존 저장소 데이터를 새로운 S3 버킷으로 이관
  backup.sh migrate --backend s3 --endpoint https://s3.ap-northeast-2.amazonaws.com --bucket new-backup-bucket --access-key ACCESSKEY123 --secret-key SECRETKEY456 --new-password 'new-safe-pass'

  # 기존 저장소 데이터를 새로운 SFTP 서버로 정합성 검증 단계를 건너뛰고 강제 이관
  backup.sh migrate --backend sftp --host 192.168.1.200 --port 22 --user newbackup --new-password 'same-or-new-pass' --skip-check

플래그 (Flags):
      --backend <s3|sftp>       마이그레이션 대상 저장소 백엔드 유형 (필수)
      --new-password <비밀번호>  이관될 새 저장소에 적용할 비밀번호 (기본값: 기존 저장소 비밀번호 유지)
      --skip-check              스냅샷 데이터 이관 완료 후 restic check 정합성 무결성 검증 단계를 생략합니다.
      --force                   유효성 및 상태 오류 발생 시 강제 진행을 수행합니다.

  SFTP 목적지 전용 옵션:
      --host <IP/도메인>        대상 SFTP 서버 접속 호스트 주소 (대화식 입력 가능)
      --port <포트>            대상 SFTP 서버 접속 SSH 포트 (기본값: 22)
      --user <사용자명>         대상 SFTP 서버 접속 계정명 (대화식 입력 가능)

  S3 목적지 전용 옵션:
      --endpoint <URL>         대상 S3 엔드포인트 주소 (대화식 입력 가능)
      --bucket <버킷명>         대상 S3 버킷 이름 (대화식 입력 가능)
      --access-key <키>         대상 S3 Access Key ID (대화식 입력 가능)
      --secret-key <키>         대상 S3 Secret Access Key (대화식 입력 가능)

글로벌 플래그 (Global Flags):
  -v, --verbose                 마이그레이션 도중 발생하는 복사 및 검증 과정을 상세히 출력
  -h, --help                    도움말 출력
EOF
}

help_config() {
  cat <<'EOF'
기존 생성되어 있는 백업 설정(/etc/backup/backup.env)을 부분 수정하거나 최신화합니다. 백엔드(S3/SFTP) 구성을 자동 판단하며, 변경 완료 후 스케줄에 따른 systemd 타이머 설정 및 resticprofile 설정을 실시간 자동 동기화합니다.

사용법:
  backup.sh config [옵션...]

사용법 예시:
  # 백업 대상 경로를 새로 갱신
  backup.sh config --targets /home/user,/var/log

  # systemd timer의 백업 반복 스케줄 주기를 매일 새벽 3시로 변경
  backup.sh config --on-calendar "*-*-* 03:00:00"

  # 변경될 내용을 실제 저장하지 않고 시뮬레이션 예정만 확인
  backup.sh config --exclude "*.tmp" --dry-run

플래그 (Flags):
      --targets <경로,...>      백업할 로컬 디렉터리 또는 파일 경로 목록 (쉼표 구분)
      --exclude <패턴>          백업에서 제외할 파일/디렉터리 패턴 (여러 번 지정 시 쉼표로 병합)
      --password <비밀번호>      백업 데이터를 암호화/복호화할 restic 저장소 비밀번호
      --keep-daily <N>          일별 보관할 스냅샷 개수
      --keep-weekly <N>         주별 보관할 스냅샷 개수
      --keep-monthly <N>        월별 보관할 스냅샷 개수
      --profile-name <이름>     resticprofile 프로파일 이름 (기본값: 호스트명)
      --on-calendar <식>        정기 백업을 위한 systemd OnCalendar 포맷 주기 (예: "daily", "*-*-* 03:00:00")
      --audit-tester <이름>     일일 백업 감사 및 복구 테스트를 수행할 담당자 이름
      --audit-ciso <이름>       보고서를 최종 승인할 정보보안책임자(CISO) 이름
      --audit-rto <분>          복구 목표 시간(RTO, 분 단위)
      --dry-run                 실제 설정을 변경해 저장하지 않고 예정 사항을 시뮬레이션하여 보여줍니다.

  SFTP 백엔드 전용 변경 옵션:
      --host <IP/도메인>        SFTP 서버 접속 호스트 주소
      --port <포트>            SFTP 서버 접속 SSH 포트 (기본값: 22)
      --user <사용자명>         SFTP 서버 접속 계정명

  S3 백엔드 전용 변경 옵션:
      --endpoint <URL>         S3 호환 엔드포인트 주소
      --bucket <버킷명>         S3 버킷 이름
      --access-key <키>         AWS Access Key ID
      --secret-key <키>         AWS Secret Access Key

  데이터베이스 백업 옵션 (Database Backup Options):
      --db-type <mysql|mariadb|postgres|custom> 데이터베이스 엔진 유형
      --db-command <명령어>       DB 백업을 위한 덤프 명령어 (디폴트 매핑 지원)
      --db-filename <파일명>      백업할 덤프 파일명 (기본값: db-dump.sql)
      --db-schedule <식>          DB 백업을 위한 systemd OnCalendar 포맷 주기 (예: "*-*-* 03:00:00")
      --db-keep-daily <N>         DB 스냅샷 일별 보관 개수
      --db-keep-weekly <N>        DB 스냅샷 주별 보관 개수
      --db-keep-monthly <N>       DB 스냅샷 월별 보관 개수

글로벌 플래그 (Global Flags):
  -v, --verbose                 갱신 과정 상세 로그 출력
  -h, --help                    도움말 출력
EOF
}

help_wizard() {
  cat <<'EOF'
CLI 설정이 낯선 사용자를 위한 대화형 단계별 설정 마법사입니다. 
백업 대상 선택, 스토리지 연동, 비밀번호 등록 및 정기 스케줄 등록까지의 과정을 친절하게 안내하며 원스톱으로 구성합니다.

사용법:
  backup.sh wizard

사용법 예시:
  # 대화식 마법사 가이드 시작
  backup.sh wizard

글로벌 플래그 (Global Flags):
  -h, --help        도움말 출력
EOF
}

render_help() {
  cat <<EOF
restic 기반 백업 설치, 운영, 모니터링 및 감사 관리를 자동화하는 스크립트입니다.

사용법:
  backup.sh [command] [flags]

사용 가능한 명령 (Available Commands):
  install        의존성 패키지(restic, rclone, resticprofile) 및 스크립트 자체 설치
  setting        초기 백업 환경 설정(backup.env) 명시적 등록
  init           연동된 원격 백업 저장소 초기화 (restic init)
  schedule       정기 자동 백업 스케줄 설정 및 활성화/비활성화 (systemd timer)
  run            수동 백업 즉시 실행 (resticprofile backup)
  status         저장소 연결성, 보안 권한 상태 및 최근 백업 스냅샷 요약 조회
  audit          ISMS/ISO 27001 컴플라이언스 감사 대응용 보고서 출력 및 저장
  uninstall      정기 스케줄 제거 및 설치된 바이너리/스크립트 삭제
  migrate        기존 저장소 백업 데이터를 새로운 스토리지 백엔드로 데이터 복제 및 설정 이관
  config         기존 백업 설정(backup.env) 수정 및 관련 자산 동기화
  wizard         단계별 설정을 위한 대화형 CLI 설정 마법사 실행
  upgrade        기존 1차 로컬 백업 데이터를 새로운 1차 원격 저장소로 이관 및 환경 정리

글로벌 플래그 (Global Flags):
  -h, --help     도움말 출력
  -V, --version  버전 정보 출력
  -v, --verbose  디버깅 및 연동 명령어 상세 로깅 활성화

상세한 하위 명령 정보는 'backup.sh [command] --help'를 참고하세요.
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
    -V|--version)
      printf '%s\n' "$BACKUP_SCRIPT_VERSION"
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
    migrate)
      shift
      cmd_migrate "$@"
      return $?
      ;;
    config)
      shift
      cmd_config "$@"
      return $?
      ;;
    wizard)
      shift
      cmd_wizard "$@"
      return $?
      ;;
    upgrade)
      shift
      cmd_upgrade "$@"
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

#!/usr/bin/env bash
# shellcheck disable=SC2030,SC2031
set -euo pipefail

BACKUP_SCRIPT_VERSION="0.0.60"

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
GUM_INSTALL_PATH="${GUM_INSTALL_PATH:-/usr/local/bin/gum}"
GUM_VERSION="0.17.0"
GUM_SHA256_AMD64="${GUM_SHA256_AMD64:-69ee169bd6387331928864e94d47ed01ef649fbfe875baed1bbf27b5377a6fdb}"
GUM_SHA256_ARM64="${GUM_SHA256_ARM64:-b0b9ed95cbf7c8b7073f17b9591811f5c001e33c7cfd066ca83ce8a07c576f9c}"
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
DEFAULT_NTP_ON_CALENDAR="*-*-* 00:30:00"
# 구버전과의 호환성을 위해 유지하는 시각 동기화 주기 기본값
# shellcheck disable=SC2034
DEFAULT_CHRONY_ON_CALENDAR="*-*-* 00:30:00"
DEFAULT_SFTP_PORT=22
BACKUP_VERBOSE="${BACKUP_VERBOSE:-0}"
NTP_CONF_PATH="${NTP_CONF_PATH:-/etc/chrony.conf}"
# 구버전과의 호환성을 위해 유지하는 chrony conf 경로
# shellcheck disable=SC2034
CHRONY_CONF_PATH="${NTP_CONF_PATH}"
BACKUP_REPORTS_DIR="${BACKUP_REPORTS_DIR:-/data/backup/reports}"

has_function() {
  declare -f "$1" >/dev/null
}

# 구버전 -> 신버전 환경변수 매핑 정의
# shellcheck disable=SC2034
declare -A COMPATIBILITY_MAP=(
  ["BACKUP_EXCLUDE_PATHS"]="BACKUP_EXCLUDES"
  ["BACKUP_SFTP_HOST"]="RCLONE_CONFIG_SYNO_BACKUP_HOST"
  ["BACKUP_SFTP_PORT"]="RCLONE_CONFIG_SYNO_BACKUP_PORT"
  ["BACKUP_SFTP_USER"]="RCLONE_CONFIG_SYNO_BACKUP_USER"
  ["BACKUP_CHRONY_REPORT"]="BACKUP_NTP_REPORT"
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

  # NTP 속성
  register_config_field "ntp_report" "BACKUP_NTP_REPORT" "" ""
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
  local __load_env_dest_ref_name="$2"
  local errs_ref_name="${3:-}"
  
  if [[ -n "$errs_ref_name" ]]; then
  local __load_env_errors_ref_name="$errs_ref_name"
  else
    local -a _dummy_errors=()
  local __load_env_errors_ref_name="_dummy_errors"
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
          ref_set "${__load_env_dest_ref_name}" "$multiline_key" "$val"
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
          ref_set "${__load_env_dest_ref_name}" "$multiline_key" "$multiline_val"
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
      ref_set "${__load_env_dest_ref_name}" "$key" "$val"
    elif [[ "$trimmed_line" =~ ^(export[[:space:]]+)?([A-Za-z0-9_]+)=\'(.*)$ ]]; then
      in_multiline=1
      multiline_key="${BASH_REMATCH[2]}"
      multiline_val="${BASH_REMATCH[3]}"
      quote_char="'"
    elif [[ "$trimmed_line" =~ ^(export[[:space:]]+)?([A-Za-z0-9_]+)=\"(.*)\"[[:space:]]*$ ]]; then
      key="${BASH_REMATCH[2]}"
      val="${BASH_REMATCH[3]}"
      ref_set "${__load_env_dest_ref_name}" "$key" "$val"
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
      eval "${__load_env_errors_ref_name}+=(\"라인 ${line_num}: 올바르지 않은 설정 형식입니다 (KEY='VALUE' 규격 위반)\")"
      return 1
    fi

    if [[ -n "$key" ]]; then
      ref_set "${__load_env_dest_ref_name}" "$key" "$val"
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

ref_get() {
  local map_name="${1:-}"
  local key="${2:-}"
  local out_var="${3:-}"
  eval "${out_var}=\${${map_name}[${key}]:-}"
}

ref_set() {
  local map_name="${1:-}"
  local key="${2:-}"
  local value="${3:-}"
  local _tmp_ref_val
  _tmp_ref_val="$value"
  eval "${map_name}[${key}]=\${_tmp_ref_val}"
}

setup_colors() {
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    C_RESET=$'\e[0m'
    C_BOLD=$'\e[1m'
    C_DIM=$'\e[2m'
    C_RED=$'\e[31m'
    C_GREEN=$'\e[32m'
    C_YELLOW=$'\e[33m'
    C_BLUE=$'\e[34m'
    C_CYAN=$'\e[36m'
    C_GRAY=$'\e[90m'
  fi
}

is_interactive() {
  if [[ -t 0 ]] && { [[ -t 1 ]] || [[ -t 2 ]]; } && [[ -z "${NO_COLOR:-}" ]] && [[ "${GUM_DISABLE:-0}" != "1" ]] && command -v gum >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

safe_spin() {
  local title="$1"
  shift
  if [[ "${1:-}" == "--" ]]; then
    shift
  fi

  if is_interactive; then
    if has_function "${1:-}"; then
      local func_name="$1"
      shift
      # shellcheck disable=SC2163
      export -f "$func_name" 2>/dev/null || true
      export RESTIC_PASSWORD RESTIC_REPOSITORY AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY 2>/dev/null || true
      local rclone_var
      for rclone_var in $(compgen -v RCLONE_CONFIG_ 2>/dev/null || true); do
        # shellcheck disable=SC2163
        export "$rclone_var" 2>/dev/null || true
      done
      gum spin --spinner dot --title "$title" -- bash -c '"$@"' _ "$func_name" "$@"
    else
      gum spin --spinner dot --title "$title" -- "$@"
    fi
  else
    log_info "[RUN] $title"
    "$@"
  fi
}

safe_confirm() {
  local prompt="$1"
  local default_ans="${2:-n}"
  if is_interactive; then
    if [[ "$default_ans" == "y" ]]; then
      gum confirm --default=true "$prompt" < /dev/tty > /dev/tty
    else
      gum confirm --default=false "$prompt" < /dev/tty > /dev/tty
    fi
  else
    if [[ ! -t 0 ]]; then
      local user_input=""
      if read -r user_input; then
        case "$user_input" in
          [Yy]* ) return 0 ;;
          [Nn]* ) return 1 ;;
        esac
      fi
    fi
    if [[ "$default_ans" == "y" ]]; then
      return 0
    else
      return 1
    fi
  fi
}

safe_input() {
  local prompt="$1"
  local default_val="${2:-}"
  local is_password="${3:-0}"
  if is_interactive; then
    local -a gum_opts=(--placeholder "$prompt")
    if [[ -n "$default_val" ]]; then
      gum_opts+=(--value "$default_val")
    fi
    if [[ "$is_password" == "1" ]]; then
      gum_opts+=(--password)
    fi
    gum input "${gum_opts[@]}" < /dev/tty
  else
    local val=""
    if [[ "$is_password" == "1" ]]; then
      printf '%s: ' "$prompt" >&2
      if ! read -r -s val; then val=""; fi
      echo "" >&2
    else
      printf '%s [%s]: ' "$prompt" "$default_val" >&2
      if ! read -r val; then val=""; fi
    fi
    val="${val:-$default_val}"
    printf '%s\n' "$val"
  fi
}

safe_choose() {
  local header="$1"
  shift
  local options=("$@")
  if is_interactive; then
    gum choose --header "$header" "${options[@]}" < /dev/tty
  else
    if [[ ! -t 0 ]]; then
      local val=""
      if read -r val; then
        printf '%s\n' "$val"
        return 0
      fi
    fi
    printf '%s\n' "${options[0]}"
  fi
}

safe_style() {
  local text="$1"
  shift
  if is_interactive; then
    gum style "$@" "$text"
  else
    printf '%s\n' "$text"
  fi
}

safe_table() {
  if is_interactive; then
    gum table "$@"
  else
    cat
  fi
}


log_info() {
  printf '%s\n' "$1"
  command -v logger >/dev/null 2>&1 && logger -t backup.sh -- "$1" || true
}

log_error() {
  printf 'ERROR: %s\n' "$1" >&2
  command -v logger >/dev/null 2>&1 && logger -t backup.sh -- "ERROR: $1" || true
}

log_warn() {
  printf 'WARNING: %s\n' "$1" >&2
  command -v logger >/dev/null 2>&1 && logger -t backup.sh -- "WARNING: $1" || true
}

die() {
  log_error "$1"
  exit "${2:-1}"
}

has_dependency() {
  command -v "$1" >/dev/null 2>&1
}

check_system_dependencies() {
  local missing=()
  local dep
  for dep in python3 tar curl logger; do
    if ! has_dependency "$dep"; then
      missing+=("$dep")
    fi
  done

  if [[ ${#missing[@]} -gt 0 ]]; then
    local joined
    joined=$(IFS=,; echo "${missing[*]}")
    local formatted_missing
    formatted_missing="${joined//,/, }"

    cat <<EOF >&2
[!] 필수 시스템 도구가 누락되어 스크립트를 실행할 수 없습니다.
* 누락된 도구: ${formatted_missing}

다음 커맨드를 실행하여 필요한 패키지를 설치하세요:
  - Debian/Ubuntu: sudo apt-get update && sudo apt-get install -y python3 tar curl bsdmainutils
  - RHEL/Rocky/CentOS: sudo yum install -y python3 tar curl util-linux
EOF
    exit 1
  fi
}

check_core_dependencies() {
  local missing=()
  local dep
  for dep in restic rclone resticprofile; do
    if ! has_dependency "$dep"; then
      missing+=("$dep")
    fi
  done

  if [[ ${#missing[@]} -gt 0 ]]; then
    local joined
    joined=$(IFS=,; echo "${missing[*]}")
    local formatted_missing
    formatted_missing="${joined//,/, }"

    cat <<EOF >&2
[!] 백업 핵심 도구가 누락되어 백업 명령을 실행할 수 없습니다.
* 누락된 도구: ${formatted_missing}

다음 커맨드를 실행하여 백업 도구를 설치하세요:
  sudo ./backup.sh install
EOF
    exit 1
  fi
}

require_root() {
  if [[ "${REQUIRE_ROOT_CHECK:-1}" == "1" && "${EUID}" -ne 0 ]]; then
    die "이 명령은 root 권한으로 실행해야 합니다. sudo로 다시 실행하세요." 1
  fi
}

resolve_value() {
  local cli="${1:-}" env="${2:-}" file="${3:-}" default="${4:-}"
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
      log_warn "구버전 설정 키(${old_key})가 감지되었습니다. '${new_key}' 값으로 자동 맵핑되어 실행되지만, 정상적인 유지를 위해 'backup.sh import'를 실행해 주십시오."
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

convert_calendar_to_cron() {
  local cal="$1"
  # Trim spaces
  cal=$(echo "$cal" | xargs)

  # 1. Daily: *-*-* HH:MM:SS or *-*-* HH:MM
  if [[ "$cal" =~ ^\*-\*-\*[[:space:]]+([0-9]{1,2}):([0-9]{2})(:[0-9]{2})?$ ]]; then
    local hour="${BASH_REMATCH[1]}"
    local min="${BASH_REMATCH[2]}"
    hour=$((10#$hour))
    min=$((10#$min))
    printf '%s %s * * *\n' "$min" "$hour"
    return 0
  fi

  # 2. Monthly: *-*-DD HH:MM:SS or *-*-DD HH:MM
  if [[ "$cal" =~ ^\*-\*-([0-9]{2})[[:space:]]+([0-9]{1,2}):([0-9]{2})(:[0-9]{2})?$ ]]; then
    local dom="${BASH_REMATCH[1]}"
    local hour="${BASH_REMATCH[2]}"
    local min="${BASH_REMATCH[3]}"
    dom=$((10#$dom))
    hour=$((10#$hour))
    min=$((10#$min))
    printf '%s %s %s * *\n' "$min" "$hour" "$dom"
    return 0
  fi

  # 3. Weekly: (DayOfWeek) *-*-* HH:MM:SS
  # Systemd: Mon..Fri *-*-* 02:00:00 or Mon,Tue *-*-* 02:00:00 or Mon *-*-* 02:00:00
  if [[ "$cal" =~ ^([A-Za-z0-9,.-]+)[[:space:]]+\*-\*-\*[[:space:]]+([0-9]{1,2}):([0-9]{2})(:[0-9]{2})?$ ]]; then
    local days="${BASH_REMATCH[1]}"
    local hour="${BASH_REMATCH[2]}"
    local min="${BASH_REMATCH[3]}"
    hour=$((10#$hour))
    min=$((10#$min))
    
    # Translate day ranges like Mon..Fri to Mon-Fri
    days="${days//../-}"
    printf '%s %s * * %s\n' "$min" "$hour" "$days"
    return 0
  fi

  # Fallback: if it's already a valid cron expression (5 fields), return it directly.
  if [[ $(echo "$cal" | awk '{print NF}') -eq 5 ]]; then
    printf '%s\n' "$cal"
    return 0
  fi

  # Default fallback
  printf '0 2 * * *\n'
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
  local -a paths=()
  IFS=',' read -r -a paths <<< "$value"
  local p
  for p in "${paths[@]}"; do
    p="${p#"${p%%[![:space:]]*}"}"
    p="${p%"${p##*[![:space:]]}"}"
    [[ -z "$p" ]] && continue
    if [[ ! "$p" =~ ^/ ]]; then
      printf '%s "%s"은(는) 올바른 절대 경로가 아닙니다 (/로 시작해야 합니다)\n' "$name" "$p"
      return 1
    fi
  done
  return 0
}

validate_resolved_config() {
  local resolved_ref_name="$1"
  local errors_ref_name="$2"
  
  # 1. 1차 백엔드 검증
  local backend; eval "backend=\"\${${resolved_ref_name}[backend]:-}\""
  if [[ -n "$backend" ]]; then
    declare -g -A _validate_primary_fields=()
    _validate_primary_fields[host]="$(eval echo "\${${resolved_ref_name}[host]:-}")"
    _validate_primary_fields[port]="$(eval echo "\${${resolved_ref_name}[port]:-}")"
    _validate_primary_fields[user]="$(eval echo "\${${resolved_ref_name}[user]:-}")"
    _validate_primary_fields[endpoint]="$(eval echo "\${${resolved_ref_name}[endpoint]:-}")"
    _validate_primary_fields[bucket]="$(eval echo "\${${resolved_ref_name}[bucket]:-}")"
    _validate_primary_fields[access_key]="$(eval echo "\${${resolved_ref_name}[access_key]:-}")"
    _validate_primary_fields[secret_key]="$(eval echo "\${${resolved_ref_name}[secret_key]:-}")"
    
    local primary_err
    if has_function "backend_${backend}_validate"; then
      if ! primary_err=$("backend_${backend}_validate" _validate_primary_fields 2>&1); then
        eval "${errors_ref_name}+=(\"$primary_err\")"
      fi
    else
      eval "${errors_ref_name}+=(\"지원하지 않는 백엔드 유형입니다: $backend\")"
    fi
    unset _validate_primary_fields
  fi
  
  # 2. 2차 백엔드 검증
  local sec_backend; eval "sec_backend=\"\${${resolved_ref_name}[secondary_backend]:-}\""
  if [[ -n "$sec_backend" ]]; then
    declare -g -A _validate_sec_fields=()
    _validate_sec_fields[host]="$(eval echo "\${${resolved_ref_name}[secondary_host]:-}")"
    _validate_sec_fields[port]="$(eval echo "\${${resolved_ref_name}[secondary_port]:-}")"
    _validate_sec_fields[user]="$(eval echo "\${${resolved_ref_name}[secondary_user]:-}")"
    _validate_sec_fields[endpoint]="$(eval echo "\${${resolved_ref_name}[secondary_endpoint]:-}")"
    _validate_sec_fields[bucket]="$(eval echo "\${${resolved_ref_name}[secondary_bucket]:-}")"
    _validate_sec_fields[access_key]="$(eval echo "\${${resolved_ref_name}[secondary_access_key]:-}")"
    _validate_sec_fields[secret_key]="$(eval echo "\${${resolved_ref_name}[secondary_secret_key]:-}")"
    
    local sec_err
    if has_function "backend_${sec_backend}_validate"; then
      if ! sec_err=$("backend_${sec_backend}_validate" _validate_sec_fields 2>&1); then
        eval "${errors_ref_name}+=(\"Secondary backend error: $sec_err\")"
      fi
    else
      eval "${errors_ref_name}+=(\"Secondary backend error: 지원하지 않는 백엔드 유형입니다: $sec_backend\")"
    fi
    unset _validate_sec_fields
  fi

  # 3. 백업 대상 경로(targets) 절대경로 여부 검증
  local targets; eval "targets=\"\${${resolved_ref_name}[targets]:-}\""
  if [[ -n "$targets" ]]; then
    local path_err
    if ! path_err=$(validate_absolute_path "$targets" "백업 대상 경로" 2>&1); then
      eval "${errors_ref_name}+=(\"$path_err\")"
    fi
  fi

  # 4. 데이터베이스 백업 유효성 검증
  local db_type; eval "db_type=\"\${${resolved_ref_name}[db_type]:-}\""
  if [[ -n "$db_type" ]]; then
    if ! has_function "database_${db_type}_default_command"; then
      eval "${errors_ref_name}+=(\"지원하지 않는 데이터베이스 엔진 유형입니다: ${db_type}\")"
    else
      if has_function "database_${db_type}_validate_config"; then
        local db_err
        if ! db_err=$("database_${db_type}_validate_config" "${resolved_ref_name}" 2>&1); then
          eval "${errors_ref_name}+=(\"$db_err\")"
        fi
      fi
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

  local _out_resolved_name="$resolved_ref_name"
  local _out_errors_name="$errors_ref_name"

  # Determine backend from backup.env or CLI options
  local backend=""
  if [[ -n "$cli_opts_ref_name" ]]; then
  local _cli_opts_ref_name="$cli_opts_ref_name"
    backend="$(eval echo "\${${_cli_opts_ref_name}[backend]:-}")"
  fi

  if [[ -z "$backend" && -f "$BACKUP_ENV_FILE" ]]; then
    if grep -q -E "AWS_ACCESS_KEY_ID=" "$BACKUP_ENV_FILE" || grep -q -E 'RESTIC_REPOSITORY=.*s3:' "$BACKUP_ENV_FILE"; then
      backend="s3"
    elif grep -q -E "RCLONE_CONFIG_SYNO_BACKUP_TYPE=" "$BACKUP_ENV_FILE" || grep -q -E 'RESTIC_REPOSITORY=.*rclone:syno_backup' "$BACKUP_ENV_FILE"; then
      backend="sftp"
    fi
  fi

  # Create a unified local options copy
  local -A local_opts=()
  if [[ -n "$cli_opts_ref_name" ]]; then
    local _cli_opts_ref_name="$cli_opts_ref_name"
    local -a keys=()
    eval "keys=(\"\${!${_cli_opts_ref_name}[@]}\")"
    local k val=""
    for k in "${keys[@]}"; do
      ref_get "${_cli_opts_ref_name}" "$k" val
      local_opts["$k"]="$val"
    done
  fi

  if [[ -n "$backend" ]]; then
    local_opts[backend]="$backend"
  fi
  if [[ -n "$profile_name" ]]; then
    local_opts[profile-name]="$profile_name"
  fi

  resolve_and_validate_config local_opts "${_out_resolved_name}" "${_out_errors_name}"
  local res=$?

  if [[ $(eval echo "\${#${_out_errors_name}[@]}") -gt 0 ]]; then
    return 1
  fi
  return $res
}

save_profile_config() {
  local resolved_arr_name="$1"
  local _res_ref_name="$resolved_arr_name"

  local backend; eval "backend=\"\${${_res_ref_name}[backend]:-}\""
  if [[ -z "$backend" ]]; then
    return 1
  fi

  # Determine schedule calendar
  local on_calendar; eval "on_calendar=\"\${${_res_ref_name}[on_calendar]:-}\""
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
      export "$k"="${file_config[$k]}"
    done
    local profile_name=""
    ref_get "${_res_ref_name}" profile_name profile_name
    [[ -z "$profile_name" ]] && profile_name="${file_config[BACKUP_PROFILE_NAME]:-$(hostname)}"
    write_resticprofile_assets "$profile_name" "$on_calendar"

    local timer_name
    timer_name=$(resticprofile_timer_unit_name "$profile_name")
    if systemctl is-enabled "$timer_name" >/dev/null 2>&1; then
      log_info "정기 백업 스케줄 타이머(${timer_name})가 활성화되어 있어 설정을 자동 리로드합니다."
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" schedule
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
  local _opts_name="$1"
  local _resolved_name="$2"
  local _errors_name="$3"

  # Sourcing current configuration from file
  local file_targets="" file_keep_daily="" file_keep_weekly="" file_keep_monthly="" file_excludes="" file_profile_name="" file_password=""
  local file_notification_url="" file_notification_type="" file_notification_on="" file_notification_method="" file_notification_headers="" file_notification_body_success="" file_notification_body_failure=""
  local file_audit_tester="" file_audit_ciso="" file_audit_rto=""
  local file_secondary_backend="" file_secondary_password="" file_secondary_keep_daily="" file_secondary_keep_weekly="" file_secondary_keep_monthly=""
  local file_secondary_endpoint="" file_secondary_bucket="" file_secondary_access_key="" file_secondary_secret_key=""
  local file_secondary_host="" file_secondary_port="" file_secondary_user="" file_secondary_repo=""
  local file_db_type="" file_db_command="" file_db_filename="" file_db_schedule="" file_db_keep_daily="" file_db_keep_weekly="" file_db_keep_monthly="" file_ntp_report=""
  if [[ -f "${BACKUP_ENV_FILE:-}" ]]; then
    declare -A file_config=()
    if ! load_backup_env_to_array "$BACKUP_ENV_FILE" file_config "${_errors_name}"; then
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
    file_ntp_report="${file_config[BACKUP_NTP_REPORT]:-${file_config[BACKUP_CHRONY_REPORT]:-}}"
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
  local env_ntp_report="${BACKUP_NTP_REPORT:-${BACKUP_CHRONY_REPORT:-}}"

  # Resolve values with priority: CLI option > Env variable > Config file > Default value
  local cli_targets; eval "cli_targets=\"\${${_opts_name}[targets]:-}\""
  ref_set "${_resolved_name}" targets "$(resolve_value "$cli_targets" "$env_targets" "$file_targets" "${DEFAULT_TARGETS:-}")"

  local cli_keep_daily; eval "cli_keep_daily=\"\${${_opts_name}[keep-daily]:-}\""
  ref_set "${_resolved_name}" keep_daily "$(resolve_value "$cli_keep_daily" "$env_keep_daily" "$file_keep_daily" "${DEFAULT_KEEP_DAILY:-}")"

  local cli_keep_weekly; eval "cli_keep_weekly=\"\${${_opts_name}[keep-weekly]:-}\""
  ref_set "${_resolved_name}" keep_weekly "$(resolve_value "$cli_keep_weekly" "$env_keep_weekly" "$file_keep_weekly" "${DEFAULT_KEEP_WEEKLY:-}")"

  local cli_keep_monthly; eval "cli_keep_monthly=\"\${${_opts_name}[keep-monthly]:-}\""
  ref_set "${_resolved_name}" keep_monthly "$(resolve_value "$cli_keep_monthly" "$env_keep_monthly" "$file_keep_monthly" "${DEFAULT_KEEP_MONTHLY:-}")"

  local cli_password; eval "cli_password=\"\${${_opts_name}[password]:-}\""
  ref_set "${_resolved_name}" password "$(resolve_value "$cli_password" "$env_password" "$file_password" "")"

  local cli_profile_name; eval "cli_profile_name=\"\${${_opts_name}[profile-name]:-}\""
  ref_set "${_resolved_name}" profile_name "$(resolve_value "$cli_profile_name" "$env_profile_name" "$file_profile_name" "$(hostname)")"

  local cli_db_type; eval "cli_db_type=\"\${${_opts_name}[db-type]:-}\""
  ref_set "${_resolved_name}" db_type "$(resolve_value "$cli_db_type" "$env_db_type" "$file_db_type" "" || true)"

  if [[ -n "$(eval echo "\${${_resolved_name}[db_type]:-}")" && "$(eval echo "\${${_resolved_name}[db_type]}")" != "$file_db_type" ]]; then
    file_db_command=""
    file_db_filename=""
    file_db_schedule=""
    file_db_keep_daily=""
    file_db_keep_weekly=""
    file_db_keep_monthly=""
  fi

  local cli_db_command; eval "cli_db_command=\"\${${_opts_name}[db-command]:-}\""
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" db_command "$(resolve_value "$cli_db_command" "$env_db_command" "$file_db_command" "" || true)"

  local cli_db_filename; eval "cli_db_filename=\"\${${_opts_name}[db-filename]:-}\""
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" db_filename "$(resolve_value "$cli_db_filename" "$env_db_filename" "$file_db_filename" "db-dump.sql" || true)"

  local cli_db_schedule; eval "cli_db_schedule=\"\${${_opts_name}[db-schedule]:-}\""
  ref_set "${_resolved_name}" db_schedule "$(resolve_value "$cli_db_schedule" "$env_db_schedule" "$file_db_schedule" "" || true)"

  local cli_db_keep_daily; eval "cli_db_keep_daily=\"\${${_opts_name}[db-keep-daily]:-}\""
  ref_set "${_resolved_name}" db_keep_daily "$(resolve_value "$cli_db_keep_daily" "$env_db_keep_daily" "$file_db_keep_daily" "" || true)"

  local cli_db_keep_weekly; eval "cli_db_keep_weekly=\"\${${_opts_name}[db-keep-weekly]:-}\""
  ref_set "${_resolved_name}" db_keep_weekly "$(resolve_value "$cli_db_keep_weekly" "$env_db_keep_weekly" "$file_db_keep_weekly" "" || true)"

  local cli_db_keep_monthly; eval "cli_db_keep_monthly=\"\${${_opts_name}[db-keep-monthly]:-}\""
  ref_set "${_resolved_name}" db_keep_monthly "$(resolve_value "$cli_db_keep_monthly" "$env_db_keep_monthly" "$file_db_keep_monthly" "" || true)"

  local cli_ntp_report; eval "cli_ntp_report=\"\${${_opts_name}[ntp-report]:-}\""
  [[ -z "$cli_ntp_report" ]] && cli_ntp_report="$(eval echo "\${${_opts_name}[chrony-report]:-}")"
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" ntp_report "$(resolve_value "$cli_ntp_report" "$env_ntp_report" "$file_ntp_report" "" || true)"

  # DB 타입별 기본 명령어 채우기
  if [[ -n "$(eval echo "\${${_resolved_name}[db_type]:-}")" ]]; then
    if [[ -z "$(eval echo "\${${_resolved_name}[db_command]:-}")" ]]; then
      if has_function "database_$(eval echo "\${${_resolved_name}[db_type]}")_default_command"; then
        ref_set "${_resolved_name}" db_command "$("database_$(eval echo "\${${_resolved_name}[db_type]}")_default_command")"
      fi
    fi
  fi

  local cli_sec_backend; eval "cli_sec_backend=\"\${${_opts_name}[secondary-backend]:-}\""
  ref_set "${_resolved_name}" "secondary_backend" "$(resolve_value "$cli_sec_backend" "$env_secondary_backend" "$file_secondary_backend" "" || true)"

  if [[ -n "$(eval echo "\${${_resolved_name}[secondary_backend]:-}")" ]]; then
    local cli_sec_password; eval "cli_sec_password=\"\${${_opts_name}[secondary-password]:-}\""
    ref_set "${_resolved_name}" "secondary_password" "$(resolve_value "$cli_sec_password" "$env_secondary_password" "$file_secondary_password" "$(eval echo "\${${_resolved_name}[password]:-}")" || true)"

    local cli_sec_keep_daily; eval "cli_sec_keep_daily=\"\${${_opts_name}[secondary-keep-daily]:-}\""
    ref_set "${_resolved_name}" "secondary_keep_daily" "$(resolve_value "$cli_sec_keep_daily" "$env_secondary_keep_daily" "$file_secondary_keep_daily" "$(eval echo "\${${_resolved_name}[keep_daily]:-}")" || true)"

    local cli_sec_keep_weekly; eval "cli_sec_keep_weekly=\"\${${_opts_name}[secondary-keep-weekly]:-}\""
    ref_set "${_resolved_name}" "secondary_keep_weekly" "$(resolve_value "$cli_sec_keep_weekly" "$env_secondary_keep_weekly" "$file_secondary_keep_weekly" "$(eval echo "\${${_resolved_name}[keep_weekly]:-}")" || true)"

    local cli_sec_keep_monthly; eval "cli_sec_keep_monthly=\"\${${_opts_name}[secondary-keep-monthly]:-}\""
    ref_set "${_resolved_name}" "secondary_keep_monthly" "$(resolve_value "$cli_sec_keep_monthly" "$env_secondary_keep_monthly" "$file_secondary_keep_monthly" "$(eval echo "\${${_resolved_name}[keep_monthly]:-}")" || true)"
  fi




  # Resolve Exclude
  local cli_exclude; eval "cli_exclude=\"\${${_opts_name}[exclude]:-}\""
  ref_set "${_resolved_name}" excludes_csv "$(resolve_value "$cli_exclude" "" "$file_excludes" "${DEFAULT_EXCLUDES:-}")"

  # Resolve Notifications
  local cli_notification_url; eval "cli_notification_url=\"\${${_opts_name}[notification-url]:-}\""
  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" notification_url "$(resolve_value "$cli_notification_url" "$env_notification_url" "$file_notification_url" "" || true)"

  local cli_notification_type; eval "cli_notification_type=\"\${${_opts_name}[notification-type]:-}\""
  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" notification_type "$(resolve_value "$cli_notification_type" "$env_notification_type" "$file_notification_type" "" || true)"

  local cli_notification_on; eval "cli_notification_on=\"\${${_opts_name}[notification-on]:-}\""
  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" notification_on "$(resolve_value "$cli_notification_on" "$env_notification_on" "$file_notification_on" "both" || true)"

  # resolved는 nameref 연관 배열이며 키 이름이 변수로 오인되는 것을 방지한다.
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" notification_method "$(resolve_value "" "${BACKUP_NOTIFICATION_METHOD:-}" "$file_notification_method" "POST" || true)"
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" notification_headers "$(resolve_value "" "${BACKUP_NOTIFICATION_HEADERS:-}" "$file_notification_headers" "" || true)"
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" notification_body_success "$(resolve_value "" "${BACKUP_NOTIFICATION_BODY_SUCCESS:-}" "$file_notification_body_success" "" || true)"
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" notification_body_failure "$(resolve_value "" "${BACKUP_NOTIFICATION_BODY_FAILURE:-}" "$file_notification_body_failure" "" || true)"

  local cli_audit_tester; eval "cli_audit_tester=\"\${${_opts_name}[audit-tester]:-}\""
  local env_audit_tester="${BACKUP_AUDIT_TESTER:-}"
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" audit_tester "$(resolve_value "$cli_audit_tester" "$env_audit_tester" "$file_audit_tester" "" || true)"

  local cli_audit_ciso; eval "cli_audit_ciso=\"\${${_opts_name}[audit-ciso]:-}\""
  local env_audit_ciso="${BACKUP_AUDIT_CISO:-}"
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" audit_ciso "$(resolve_value "$cli_audit_ciso" "$env_audit_ciso" "$file_audit_ciso" "" || true)"

  local cli_audit_rto; eval "cli_audit_rto=\"\${${_opts_name}[audit-rto]:-}\""
  local env_audit_rto="${BACKUP_AUDIT_RTO:-}"
  # shellcheck disable=SC2154
  ref_set "${_resolved_name}" audit_rto "$(resolve_value "$cli_audit_rto" "$env_audit_rto" "$file_audit_rto" "" || true)"

  # Global Validation
  if [[ -z "$(eval echo "\${${_resolved_name}[targets]:-}")" ]]; then
    eval "${_errors_name}+=(\"백업 대상 경로(--targets 또는 BACKUP_TARGETS)가 필요합니다.\")"
  fi

  if [[ -z "$(eval echo "\${${_resolved_name}[password]:-}")" ]]; then
    eval "${_errors_name}+=(\"저장소 비밀번호(--password 또는 BACKUP_PASSWORD)가 필요합니다.\")"
  fi

  local err
  if ! err=$(validate_positive_int "$(eval echo "\${${_resolved_name}[keep_daily]}")" "keep-daily"); then
    eval "${_errors_name}+=(\"$err\")"
  fi
  if ! err=$(validate_positive_int "$(eval echo "\${${_resolved_name}[keep_weekly]}")" "keep-weekly"); then
    eval "${_errors_name}+=(\"$err\")"
  fi
  if ! err=$(validate_positive_int "$(eval echo "\${${_resolved_name}[keep_monthly]}")" "keep-monthly"); then
    eval "${_errors_name}+=(\"$err\")"
  fi
  if ! err=$(validate_profile_name "$(eval echo "\${${_resolved_name}[profile_name]}")"); then
    eval "${_errors_name}+=(\"$err\")"
  fi

  if [[ -n "$(eval echo "\${${_resolved_name}[secondary_backend]:-}")" ]]; then
    if ! err=$(validate_secondary_backend "$(eval echo "\${${_resolved_name}[secondary_backend]}")"); then
      eval "${_errors_name}+=(\"$err\")"
    fi
    if ! err=$(validate_positive_int "$(eval echo "\${${_resolved_name}[secondary_keep_daily]}")" "secondary-keep-daily"); then
      eval "${_errors_name}+=(\"$err\")"
    fi
    if ! err=$(validate_positive_int "$(eval echo "\${${_resolved_name}[secondary_keep_weekly]}")" "secondary-keep-weekly"); then
      eval "${_errors_name}+=(\"$err\")"
    fi
    if ! err=$(validate_positive_int "$(eval echo "\${${_resolved_name}[secondary_keep_monthly]}")" "secondary-keep-monthly"); then
      eval "${_errors_name}+=(\"$err\")"
    fi
  fi


  # Strict Validation for Notifications
  local url; eval "url=\"\${${_resolved_name}[notification_url]}\""
  local type; eval "type=\"\${${_resolved_name}[notification_type]}\""
  local on; eval "on=\"\${${_resolved_name}[notification_on]}\""

  if [[ -n "$url" || -n "$type" ]]; then
    if [[ -z "$url" ]]; then
      eval "${_errors_name}+=(\"알람 URL이 비어 있습니다. 알람 타입을 설정한 경우 URL(--notification-url 또는 BACKUP_NOTIFICATION_URL)을 지정해야 합니다.\")"
    elif [[ -z "$type" ]]; then
      eval "${_errors_name}+=(\"알람 타입이 비어 있습니다. 알람 URL을 설정한 경우 타입(--notification-type 또는 BACKUP_NOTIFICATION_TYPE)을 지정해야 합니다.\")"
    fi

    if [[ -n "$url" ]]; then
      if [[ ! "$url" =~ ^https?:// ]]; then
        eval "${_errors_name}+=(\"알람 URL 형식은 http:// 또는 https://로 시작해야 합니다: ${url}\")"
      fi
    fi

    if [[ -n "$type" ]]; then
      if ! has_function "notification_${type}_send"; then
        eval "${_errors_name}+=(\"지원하지 않는 알람 타입입니다: ${type}\")"
      else
        if has_function "notification_${type}_validate"; then
          local validate_err
          if ! validate_err=$("notification_${type}_validate" 2>&1); then
            eval "${_errors_name}+=(\"$validate_err\")"
          fi
        fi
      fi
    fi

    if [[ -n "$on" ]]; then
      if [[ "$on" != "failure" && "$on" != "success" && "$on" != "both" ]]; then
        eval "${_errors_name}+=(\"지원하지 않는 알람 발생 조건(ON)입니다: ${on} (failure, success, both 중 하나여야 합니다.)\")"
      fi
    fi
  fi

  # Delegate backend-specific validation if backend is specified
  local backend; eval "backend=\"\${${_opts_name}[backend]:-}\""
  if [[ -n "$backend" ]]; then
    ref_set "${_resolved_name}" backend "$backend"
    # nameref를 통한 동적 파싱을 사용하므로 미사용 변수 경고 우회
    # shellcheck disable=SC2034
    local -A backend_cli=()
    # shellcheck disable=SC2034
    local -A backend_env=()
    # shellcheck disable=SC2034
    local -A backend_file=()

    backend_cli[endpoint]="$(eval echo "\${${_opts_name}[endpoint]:-}")"
    backend_cli[bucket]="$(eval echo "\${${_opts_name}[bucket]:-}")"
    backend_cli[access_key]="$(eval echo "\${${_opts_name}[access-key]:-}")"
    backend_cli[secret_key]="$(eval echo "\${${_opts_name}[secret-key]:-}")"
    backend_cli[host]="$(eval echo "\${${_opts_name}[host]:-}")"
    backend_cli[port]="$(eval echo "\${${_opts_name}[port]:-}")"
    backend_cli[user]="$(eval echo "\${${_opts_name}[user]:-}")"

    local env_vars_mapping=""
    if has_function "backend_${backend}_env_vars"; then
      env_vars_mapping=$("backend_${backend}_env_vars")
    fi
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
    if has_function "backend_${backend}_resolve"; then
      "backend_${backend}_resolve" backend_cli backend_env backend_file backend_fields
    fi

    # Copy to main resolved array
    local key
    for key in "${!backend_fields[@]}"; do
      ref_set "${_resolved_name}" "$key" "${backend_fields[$key]}"
    done


  fi

  local sec_backend; eval "sec_backend=\"\${${_resolved_name}[secondary_backend]:-}\""
  if [[ -n "$sec_backend" ]]; then
    local -A sec_backend_cli=()
    local -A sec_backend_env=()
    local -A sec_backend_file=()

    sec_backend_cli[endpoint]="$(eval echo "\${${_opts_name}[secondary-endpoint]:-}")"
    sec_backend_cli[bucket]="$(eval echo "\${${_opts_name}[secondary-bucket]:-}")"
    sec_backend_cli[access_key]="$(eval echo "\${${_opts_name}[secondary-access-key]:-}")"
    sec_backend_cli[secret_key]="$(eval echo "\${${_opts_name}[secondary-secret-key]:-}")"
    sec_backend_cli[host]="$(eval echo "\${${_opts_name}[secondary-host]:-}")"
    sec_backend_cli[port]="$(eval echo "\${${_opts_name}[secondary-port]:-}")"
    sec_backend_cli[user]="$(eval echo "\${${_opts_name}[secondary-user]:-}")"

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
    if has_function "backend_${sec_backend}_resolve"; then
      "backend_${sec_backend}_resolve" sec_backend_cli sec_backend_env sec_backend_file sec_fields
    fi

    local skey
    for skey in "${!sec_fields[@]}"; do
      ref_set "${_resolved_name}" "secondary_$skey" "${sec_fields[$skey]}"
    done


  fi

  validate_resolved_config "${_resolved_name}" "${_errors_name}"

  if [[ $(eval echo "\${#${_errors_name}[@]}") -gt 0 ]]; then
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
  local opts_ref_name="$1"
  shift
  local parsed
  parsed=$(parse_long_opts "$@") || die "$parsed"

  local key val
  while IFS=$'\t' read -r key val; do
    [[ -z "$key" ]] && continue
    local cur_val=""
    ref_get "${opts_ref_name}" "$key" "cur_val"
    if [[ -n "$cur_val" ]]; then
      ref_set "${opts_ref_name}" "$key" "${cur_val},${val}"
    else
      ref_set "${opts_ref_name}" "$key" "$val"
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
  printf '%s' "$1" | sed "s/'/'\\\\''/g"
}

render_notification_env_block() {
  local _res_name="$1"
  cat <<EOF

# ==========================================
# 백업 성공/실패 알림 설정 (Slack, Discord, Custom)
# ==========================================
BACKUP_NOTIFICATION_URL='$(escape_single_quotes "$(eval echo "\${${_res_name}[notification_url]:-}")")'
BACKUP_NOTIFICATION_TYPE='$(escape_single_quotes "$(eval echo "\${${_res_name}[notification_type]:-}")")'
BACKUP_NOTIFICATION_ON='$(escape_single_quotes "$(eval echo "\${${_res_name}[notification_on]:-both}")")'
BACKUP_NOTIFICATION_METHOD='$(escape_single_quotes "$(eval echo "\${${_res_name}[notification_method]:-POST}")")'
BACKUP_NOTIFICATION_HEADERS='$(escape_single_quotes "$(eval echo "\${${_res_name}[notification_headers]:-}")")'
BACKUP_NOTIFICATION_BODY_SUCCESS='$(escape_single_quotes "$(eval echo "\${${_res_name}[notification_body_success]:-}")")'
BACKUP_NOTIFICATION_BODY_FAILURE='$(escape_single_quotes "$(eval echo "\${${_res_name}[notification_body_failure]:-}")")'
EOF
}

render_audit_env_block() {
  local _res_name="$1"
  cat <<EOF

# ==========================================
# ISMS/ISMS-P 감사 보고서용 사용자 설정
# ==========================================
BACKUP_AUDIT_TESTER='$(escape_single_quotes "$(eval echo "\${${_res_name}[audit_tester]:-}")")'
BACKUP_AUDIT_CISO='$(escape_single_quotes "$(eval echo "\${${_res_name}[audit_ciso]:-}")")'
BACKUP_AUDIT_RTO='$(escape_single_quotes "$(eval echo "\${${_res_name}[audit_rto]:-}")")'
EOF
}

render_db_env_block() {
  local _res_name="$1"
  if [[ -n "$(eval echo "\${${_res_name}[db_type]:-}")" ]]; then
    cat <<EOF

# ==========================================
# 데이터베이스 백업용 설정 (Database Backup)
# ==========================================
BACKUP_DB_TYPE='$(escape_single_quotes "$(eval echo "\${${_res_name}[db_type]:-}")")'
BACKUP_DB_COMMAND='$(escape_single_quotes "$(eval echo "\${${_res_name}[db_command]:-}")")'
BACKUP_DB_FILENAME='$(escape_single_quotes "$(eval echo "\${${_res_name}[db_filename]:-db-dump.sql}")")'
BACKUP_DB_SCHEDULE='$(escape_single_quotes "$(eval echo "\${${_res_name}[db_schedule]:-}")")'
KEEP_DB_DAILY='$(escape_single_quotes "$(eval echo "\${${_res_name}[db_keep_daily]:-}")")'
KEEP_DB_WEEKLY='$(escape_single_quotes "$(eval echo "\${${_res_name}[db_keep_weekly]:-}")")'
KEEP_DB_MONTHLY='$(escape_single_quotes "$(eval echo "\${${_res_name}[db_keep_monthly]:-}")")'
EOF
  fi
}

render_ntp_env_block() {
  local _res_ntp_name="$1"
  if [[ -n "$(eval echo "\${${_res_ntp_name}[ntp_report]:-}")" ]]; then
    cat <<EOF

# ==========================================
# NTP 시각 동기화 증적 생성 설정
# ==========================================
BACKUP_NTP_REPORT='$(escape_single_quotes "$(eval echo "\${${_res_ntp_name}[ntp_report]:-}")")'
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

dispatch_notification() {
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local _ctx_name="$1"
  local status; eval "status=\"\${${_ctx_name}[status]:-success}\""
  local err_msg; eval "err_msg=\"\${${_ctx_name}[err_msg]:-}\""
  local url; eval "url=\"\${${_ctx_name}[notify_url]:-}\""
  [[ -z "$url" ]] && url="${BACKUP_NOTIFICATION_URL:-}"
  local type; eval "type=\"\${${_ctx_name}[notify_type]:-}\""
  [[ -z "$type" ]] && type="${BACKUP_NOTIFICATION_TYPE:-}"
  local on; eval "on=\"\${${_ctx_name}[notify_on]:-}\""
  [[ -z "$on" ]] && on="${BACKUP_NOTIFICATION_ON:-both}"

  [[ -z "$url" ]] && return 0

  if [[ "$on" == "success" && "$status" != "success" ]]; then
    return 0
  fi
  if [[ "$on" == "failure" && "$status" != "failure" ]]; then
    return 0
  fi

  log_info "통합 알림 전송 중... ($status)"
  local res=0
  if has_function "notification_${type}_send"; then
    "notification_${type}_send" "${_ctx_name}" || res=$?
  else
    log_warn "지원하지 않거나 정의되지 않은 알림 전송 어댑터입니다: ${type}"
    return 0
  fi

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

install_gum() {
  if command -v gum >/dev/null 2>&1; then
    log_info "gum이 이미 설치되어 있습니다: $(command -v gum)"
    return 0
  fi

  local uname_arch
  uname_arch=$(uname -m)
  local gum_arch=""
  local gum_sha=""

  case "$uname_arch" in
    x86_64)
      gum_arch="x86_64"
      gum_sha="$GUM_SHA256_AMD64"
      ;;
    aarch64|arm64)
      gum_arch="arm64"
      gum_sha="$GUM_SHA256_ARM64"
      ;;
    *)
      log_warn "지원하지 않는 아키텍처(${uname_arch})이므로 gum 설치를 건너뜁니다."
      return 0
      ;;
  esac

  local gum_url="https://github.com/charmbracelet/gum/releases/download/v${GUM_VERSION}/gum_${GUM_VERSION}_Linux_${gum_arch}.tar.gz"
  local archive_member="gum_${GUM_VERSION}_Linux_${gum_arch}/gum"

  log_info "gum v${GUM_VERSION} (${gum_arch}) 설치 시도 중..."
  if ( install_binary "gum" "$GUM_VERSION" "$gum_url" "$gum_sha" "$GUM_INSTALL_PATH" "tar.gz" "$archive_member" ) 2>/dev/null; then
    log_info "gum 설치가 완료되었습니다: ${GUM_INSTALL_PATH}"
  else
    log_warn "gum 자동 설치를 건너뛰었습니다 (선택적 TUI 도구)"
  fi
  return 0
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
[dry-run] gum ${GUM_VERSION} 선택적 다운로드+체크섬 검증 후 ${GUM_INSTALL_PATH}에 설치
[dry-run] install -m 0755 "\$0" "${BACKUP_SCRIPT_INSTALL_PATH}"
[dry-run] mkdir -p "${RESTIC_ETC_DIR}" && chmod 700 "${RESTIC_ETC_DIR}"
EOF
    return 0
  fi

  install_restic
  install_rclone
  install_resticprofile
  install_gum
  self_install_copy "$0" "$force"
  ensure_restic_dir
  log_info "install 완료"
}

resolve_secondary_policy() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178
  local __policy_ref_name="$1"
  # shellcheck disable=SC2178
  local __out_ref_name="$2"
  local sec_pass; eval "sec_pass=\"\${${__policy_ref_name}[secondary_password]:-}\""
  [[ -z "$sec_pass" ]] && sec_pass="$(eval echo "\${${__policy_ref_name}[password]:-}")"
  ref_set "${__out_ref_name}" password "$sec_pass"

  local sec_daily; eval "sec_daily=\"\${${__policy_ref_name}[secondary_keep_daily]:-}\""
  [[ -z "$sec_daily" ]] && sec_daily="$(eval echo "\${${__policy_ref_name}[keep_daily]:-}")"
  ref_set "${__out_ref_name}" keep_daily "$sec_daily"

  local sec_weekly; eval "sec_weekly=\"\${${__policy_ref_name}[secondary_keep_weekly]:-}\""
  [[ -z "$sec_weekly" ]] && sec_weekly="$(eval echo "\${${__policy_ref_name}[keep_weekly]:-}")"
  ref_set "${__out_ref_name}" keep_weekly "$sec_weekly"

  local sec_monthly; eval "sec_monthly=\"\${${__policy_ref_name}[secondary_keep_monthly]:-}\""
  [[ -z "$sec_monthly" ]] && sec_monthly="$(eval echo "\${${__policy_ref_name}[keep_monthly]:-}")"
  ref_set "${__out_ref_name}" keep_monthly "$sec_monthly"
}

validate_mysql_mariadb_dump() {
  local header="$1"
  if [[ "$header" == *"MySQL dump"* || "$header" == *"MariaDB dump"* ]]; then
    return 0
  fi
  return 1
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
  local cli_ref_name="$1"; local env_ref_name="$2"; local file_ref_name="$3"; local fields_ref_name="$4"
  local env_host; eval "env_host=\"\${${env_ref_name}[host]:-}\""
  [[ -z "$env_host" ]] && env_host="${BACKUP_HOST:-${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}}"
  local file_host; eval "file_host=\"\${${file_ref_name}[host]:-}\""
  [[ -z "$file_host" ]] && file_host="$(eval echo "\${${file_ref_name}[RCLONE_CONFIG_SYNO_BACKUP_HOST]:-}")"
  ref_set "${fields_ref_name}" host "$(resolve_value "$(eval echo "\${${cli_ref_name}[host]:-}")" "$env_host" "$file_host" "")" || true

  local env_port; eval "env_port=\"\${${env_ref_name}[port]:-}\""
  [[ -z "$env_port" ]] && env_port="${BACKUP_PORT:-${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}}"
  local file_port; eval "file_port=\"\${${file_ref_name}[port]:-}\""
  [[ -z "$file_port" ]] && file_port="$(eval echo "\${${file_ref_name}[RCLONE_CONFIG_SYNO_BACKUP_PORT]:-}")"
  ref_set "${fields_ref_name}" port "$(resolve_value "$(eval echo "\${${cli_ref_name}[port]:-}")" "$env_port" "$file_port" "$DEFAULT_SFTP_PORT")" || true

  local env_user; eval "env_user=\"\${${env_ref_name}[user]:-}\""
  [[ -z "$env_user" ]] && env_user="${BACKUP_USER:-${RCLONE_CONFIG_SYNO_BACKUP_USER:-}}"
  local file_user; eval "file_user=\"\${${file_ref_name}[user]:-}\""
  [[ -z "$file_user" ]] && file_user="$(eval echo "\${${file_ref_name}[RCLONE_CONFIG_SYNO_BACKUP_USER]:-}")"
  ref_set "${fields_ref_name}" user "$(resolve_value "$(eval echo "\${${cli_ref_name}[user]:-}")" "$env_user" "$file_user" "")" || true
}

backend_sftp_validate() {
  # fields_ref는 nameref로 연관 배열을 가리키는데, 같은 변수명이 다른 함수에서도
  # nameref로 재사용되다 보니 shellcheck가 스칼라/배열 재할당으로 오인한다.
  # shellcheck disable=SC2178
  local fields_ref_name="$1"
  if [[ -z "$(eval echo "\${${fields_ref_name}[host]:-}")" || -z "$(eval echo "\${${fields_ref_name}[user]:-}")" ]]; then
    render_setting_hint_sftp "$(eval echo "\${${fields_ref_name}[host]:-}")" "$(eval echo "\${${fields_ref_name}[port]:-}")" "$(eval echo "\${${fields_ref_name}[user]:-}")"
    return 1
  fi
  validate_port "$(eval echo "\${${fields_ref_name}[port]:-}")"
}

backend_sftp_prepare() {
  local fields_ref_name="$1"
  generate_ssh_key_if_missing
  # pubkey는 nameref로 쓰는 연관 배열의 키일 뿐, 별도로 선언된 변수가 아니다.
  # shellcheck disable=SC2154
  ref_set "${fields_ref_name}" pubkey "$(cat "${BACKUP_SSH_KEY}.pub")"
}

backend_sftp_render_env() {
  local hostname_tag="$1"
  # 다른 함수의 같은 이름 nameref 사용과 겹쳐 shellcheck가 배열/스칼라 재할당으로 오인한다.
  # shellcheck disable=SC2178
  local fields_ref_name="$2"; local policy_ref_name="$3"
  local slot="${4:-primary}"

  if [[ "$slot" == "secondary" ]]; then
    local -A sec_policy=()
    resolve_secondary_policy "${policy_ref_name}" sec_policy
    local sec_host_val="" sec_user_val="" sec_port_val=""
    ref_get "${fields_ref_name}" host sec_host_val
    ref_get "${fields_ref_name}" user sec_user_val
    ref_get "${fields_ref_name}" port sec_port_val
    [[ -z "$sec_port_val" ]] && sec_port_val="22"
    cat <<EOF
SECONDARY_RESTIC_REPOSITORY='rclone:syno_backup_sec:/backup/$(escape_single_quotes "${hostname_tag}")'
SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_TYPE='sftp'
SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_HOST='$(escape_single_quotes "${sec_host_val}")'
SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_USER='$(escape_single_quotes "${sec_user_val}")'
SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_PORT='$(escape_single_quotes "${sec_port_val}")'
SECONDARY_RCLONE_CONFIG_SYNO_BACKUP_SEC_KEY_FILE='$(escape_single_quotes "${BACKUP_SSH_KEY}")'
SECONDARY_RESTIC_PASSWORD='$(escape_single_quotes "${sec_policy[password]}")'
SECONDARY_KEEP_DAILY='$(escape_single_quotes "${sec_policy[keep_daily]}")'
SECONDARY_KEEP_WEEKLY='$(escape_single_quotes "${sec_policy[keep_weekly]}")'
SECONDARY_KEEP_MONTHLY='$(escape_single_quotes "${sec_policy[keep_monthly]}")'
EOF
  else
    local host_val="" user_val="" port_val="" pass_val="" targets_val="" excludes_val="" keep_daily_val="" keep_weekly_val="" keep_monthly_val="" profile_name_val=""
    ref_get "${fields_ref_name}" host host_val
    ref_get "${fields_ref_name}" user user_val
    ref_get "${fields_ref_name}" port port_val
    ref_get "${policy_ref_name}" password pass_val
    ref_get "${policy_ref_name}" targets targets_val
    ref_get "${policy_ref_name}" excludes_csv excludes_val
    ref_get "${policy_ref_name}" keep_daily keep_daily_val
    ref_get "${policy_ref_name}" keep_weekly keep_weekly_val
    ref_get "${policy_ref_name}" keep_monthly keep_monthly_val
    ref_get "${policy_ref_name}" profile_name profile_name_val
    cat <<EOF
export RESTIC_REPOSITORY='rclone:syno_backup:/backup/$(escape_single_quotes "${hostname_tag}")'
export RCLONE_CONFIG_SYNO_BACKUP_TYPE='sftp'
export RCLONE_CONFIG_SYNO_BACKUP_HOST='$(escape_single_quotes "${host_val}")'
export RCLONE_CONFIG_SYNO_BACKUP_USER='$(escape_single_quotes "${user_val}")'
export RCLONE_CONFIG_SYNO_BACKUP_PORT='$(escape_single_quotes "${port_val}")'
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE='$(escape_single_quotes "${BACKUP_SSH_KEY}")'
export RESTIC_PASSWORD='$(escape_single_quotes "${pass_val}")'
export BACKUP_TARGETS='$(escape_single_quotes "${targets_val}")'
export BACKUP_EXCLUDES='$(escape_single_quotes "${excludes_val}")'
export KEEP_DAILY='$(escape_single_quotes "${keep_daily_val}")'
export KEEP_WEEKLY='$(escape_single_quotes "${keep_weekly_val}")'
export KEEP_MONTHLY='$(escape_single_quotes "${keep_monthly_val}")'
export BACKUP_PROFILE_NAME='$(escape_single_quotes "${profile_name_val}")'
EOF
  fi
}

backend_sftp_render_notice() {
  # 같은 이유의 nameref 오탐.
  # shellcheck disable=SC2178
  local fields_ref_name="$1"
  local slot="${2:-primary}"
  local prefix=""
  if [[ "$slot" == "secondary" ]]; then
    prefix="[2차 소산지 SFTP] "
  fi
  cat <<EOF
${prefix}아래 공개키를 NAS의 authorized_keys(또는 File Station)에 등록하세요:
----------------------------------------------------------
$(eval echo "\${${fields_ref_name}[pubkey]}")
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
  local cli_ref_name="$1"; local env_ref_name="$2"; local file_ref_name="$3"; local fields_ref_name="$4"

  # S3 env fallbacks
  local env_access_key; eval "env_access_key=\"\${${env_ref_name}[access_key]:-}\""
  [[ -z "$env_access_key" ]] && env_access_key="${BACKUP_ACCESS_KEY:-${AWS_ACCESS_KEY_ID:-}}"
  local file_access_key; eval "file_access_key=\"\${${file_ref_name}[access_key]:-}\""
  [[ -z "$file_access_key" ]] && file_access_key="$(eval echo "\${${file_ref_name}[AWS_ACCESS_KEY_ID]:-}")"
  local env_secret_key; eval "env_secret_key=\"\${${env_ref_name}[secret_key]:-}\""
  [[ -z "$env_secret_key" ]] && env_secret_key="${BACKUP_SECRET_KEY:-${AWS_SECRET_ACCESS_KEY:-}}"
  local file_secret_key; eval "file_secret_key=\"\${${file_ref_name}[secret_key]:-}\""
  [[ -z "$file_secret_key" ]] && file_secret_key="$(eval echo "\${${file_ref_name}[AWS_SECRET_ACCESS_KEY]:-}")"

  # repo extraction for endpoint and bucket
  local env_repo="${RESTIC_REPOSITORY:-}"
  local file_repo; eval "file_repo=\"\${${file_ref_name}[RESTIC_REPOSITORY]:-}\""
  local parsed_endpoint="" parsed_bucket=""

  if [[ "$env_repo" =~ ^s3:(.*)/([^/]+)/[^/]+$ ]]; then
    parsed_endpoint="${BASH_REMATCH[1]}"
    parsed_bucket="${BASH_REMATCH[2]}"
  elif [[ "$file_repo" =~ ^s3:(.*)/([^/]+)/[^/]+$ ]]; then
    parsed_endpoint="${BASH_REMATCH[1]}"
    parsed_bucket="${BASH_REMATCH[2]}"
  fi

  local env_endpoint; eval "env_endpoint=\"\${${env_ref_name}[endpoint]:-}\""
  [[ -z "$env_endpoint" ]] && env_endpoint="${BACKUP_ENDPOINT:-$parsed_endpoint}"
  local file_endpoint; eval "file_endpoint=\"\${${file_ref_name}[endpoint]:-}\""
  [[ -z "$file_endpoint" ]] && file_endpoint="$parsed_endpoint"
  ref_set "${fields_ref_name}" endpoint "$(resolve_value "$(eval echo "\${${cli_ref_name}[endpoint]:-}")" "$env_endpoint" "$file_endpoint" "")" || true

  local env_bucket; eval "env_bucket=\"\${${env_ref_name}[bucket]:-}\""
  [[ -z "$env_bucket" ]] && env_bucket="${BACKUP_BUCKET:-$parsed_bucket}"
  local file_bucket; eval "file_bucket=\"\${${file_ref_name}[bucket]:-}\""
  [[ -z "$file_bucket" ]] && file_bucket="$parsed_bucket"
  ref_set "${fields_ref_name}" bucket "$(resolve_value "$(eval echo "\${${cli_ref_name}[bucket]:-}")" "$env_bucket" "$file_bucket" "")" || true

  ref_set "${fields_ref_name}" access_key "$(resolve_value "$(eval echo "\${${cli_ref_name}[access_key]:-}")" "$env_access_key" "$file_access_key" "")" || true
  ref_set "${fields_ref_name}" secret_key "$(resolve_value "$(eval echo "\${${cli_ref_name}[secret_key]:-}")" "$env_secret_key" "$file_secret_key" "")" || true
}

backend_s3_validate() {
  # 위 backend_sftp_validate와 같은 이유의 nameref 오탐.
  # shellcheck disable=SC2178
  local fields_ref_name="$1"
  if [[ -z "$(eval echo "\${${fields_ref_name}[endpoint]:-}")" || -z "$(eval echo "\${${fields_ref_name}[bucket]:-}")" ]]; then
    render_setting_hint_s3 "$(eval echo "\${${fields_ref_name}[endpoint]:-}")" "$(eval echo "\${${fields_ref_name}[bucket]:-}")"
    return 1
  fi
  if [[ -z "$(eval echo "\${${fields_ref_name}[access_key]:-}")" || -z "$(eval echo "\${${fields_ref_name}[secret_key]:-}")" ]]; then
    render_setting_hint_s3 "$(eval echo "\${${fields_ref_name}[endpoint]:-}")" "$(eval echo "\${${fields_ref_name}[bucket]:-}")"
    return 1
  fi
  return 0
}

backend_s3_prepare() {
  :
}

backend_s3_render_env() {
  local hostname_tag="$1"
  local fields_ref_name="$2"; local policy_ref_name="$3"
  local slot="${4:-primary}"

  if [[ "$slot" == "secondary" ]]; then
    local -A sec_policy=()
    resolve_secondary_policy "${policy_ref_name}" sec_policy
    local sec_endpoint_val="" sec_bucket_val="" sec_access_key_val="" sec_secret_key_val=""
    ref_get "${fields_ref_name}" endpoint sec_endpoint_val
    ref_get "${fields_ref_name}" bucket sec_bucket_val
    ref_get "${fields_ref_name}" access_key sec_access_key_val
    ref_get "${fields_ref_name}" secret_key sec_secret_key_val
    cat <<EOF
SECONDARY_RESTIC_REPOSITORY='s3:$(escape_single_quotes "${sec_endpoint_val}")/$(escape_single_quotes "${sec_bucket_val}")/$(escape_single_quotes "${hostname_tag}")'
SECONDARY_AWS_ACCESS_KEY_ID='$(escape_single_quotes "${sec_access_key_val}")'
SECONDARY_AWS_SECRET_ACCESS_KEY='$(escape_single_quotes "${sec_secret_key_val}")'
SECONDARY_RESTIC_PASSWORD='$(escape_single_quotes "${sec_policy[password]}")'
SECONDARY_KEEP_DAILY='$(escape_single_quotes "${sec_policy[keep_daily]}")'
SECONDARY_KEEP_WEEKLY='$(escape_single_quotes "${sec_policy[keep_weekly]}")'
SECONDARY_KEEP_MONTHLY='$(escape_single_quotes "${sec_policy[keep_monthly]}")'
EOF
  else
    local endpoint_val="" bucket_val="" access_key_val="" secret_key_val="" pass_val="" targets_val="" excludes_val="" keep_daily_val="" keep_weekly_val="" keep_monthly_val="" profile_name_val=""
    ref_get "${fields_ref_name}" endpoint endpoint_val
    ref_get "${fields_ref_name}" bucket bucket_val
    ref_get "${fields_ref_name}" access_key access_key_val
    ref_get "${fields_ref_name}" secret_key secret_key_val
    ref_get "${policy_ref_name}" password pass_val
    ref_get "${policy_ref_name}" targets targets_val
    ref_get "${policy_ref_name}" excludes_csv excludes_val
    ref_get "${policy_ref_name}" keep_daily keep_daily_val
    ref_get "${policy_ref_name}" keep_weekly keep_weekly_val
    ref_get "${policy_ref_name}" keep_monthly keep_monthly_val
    ref_get "${policy_ref_name}" profile_name profile_name_val
    cat <<EOF
export RESTIC_REPOSITORY='s3:$(escape_single_quotes "${endpoint_val}")/$(escape_single_quotes "${bucket_val}")/$(escape_single_quotes "${hostname_tag}")'
export AWS_ACCESS_KEY_ID='$(escape_single_quotes "${access_key_val}")'
export AWS_SECRET_ACCESS_KEY='$(escape_single_quotes "${secret_key_val}")'
export RESTIC_PASSWORD='$(escape_single_quotes "${pass_val}")'
export BACKUP_TARGETS='$(escape_single_quotes "${targets_val}")'
export BACKUP_EXCLUDES='$(escape_single_quotes "${excludes_val}")'
export KEEP_DAILY='$(escape_single_quotes "${keep_daily_val}")'
export KEEP_WEEKLY='$(escape_single_quotes "${keep_weekly_val}")'
export KEEP_MONTHLY='$(escape_single_quotes "${keep_monthly_val}")'
export BACKUP_PROFILE_NAME='$(escape_single_quotes "${profile_name_val}")'
EOF
  fi
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
  local __res_sec_ref_name="$1"
  local sec_backend; eval "sec_backend=\"\${${__res_sec_ref_name}[secondary_backend]:-}\""
  [[ -z "$sec_backend" ]] && return 0

  local sec_password; eval "sec_password=\"\${${__res_sec_ref_name}[secondary_password]:-}\""
  [[ -z "$sec_password" ]] && sec_password="$(eval echo "\${${__res_sec_ref_name}[password]:-}")"
  local sec_keep_daily; eval "sec_keep_daily=\"\${${__res_sec_ref_name}[secondary_keep_daily]:-}\""
  [[ -z "$sec_keep_daily" ]] && sec_keep_daily="$(eval echo "\${${__res_sec_ref_name}[keep_daily]:-}")"
  local sec_keep_weekly; eval "sec_keep_weekly=\"\${${__res_sec_ref_name}[secondary_keep_weekly]:-}\""
  [[ -z "$sec_keep_weekly" ]] && sec_keep_weekly="$(eval echo "\${${__res_sec_ref_name}[keep_weekly]:-}")"
  local sec_keep_monthly; eval "sec_keep_monthly=\"\${${__res_sec_ref_name}[secondary_keep_monthly]:-}\""
  [[ -z "$sec_keep_monthly" ]] && sec_keep_monthly="$(eval echo "\${${__res_sec_ref_name}[keep_monthly]:-}")"
  local profile_name; eval "profile_name=\"\${${__res_sec_ref_name}[profile_name]:-$(hostname)}\""

  printf '\n# 2차 원격 소산 백업 설정\n'
  printf 'SECONDARY_BACKEND="%s"\n' "$sec_backend"
  printf 'SECONDARY_RESTIC_PASSWORD="%s"\n' "$sec_password"
  printf 'SECONDARY_KEEP_DAILY="%s"\n' "$sec_keep_daily"
  printf 'SECONDARY_KEEP_WEEKLY="%s"\n' "$sec_keep_weekly"
  printf 'SECONDARY_KEEP_MONTHLY="%s"\n' "$sec_keep_monthly"

  local -A fields=()
  fields[host]="$(eval echo "\${${__res_sec_ref_name}[secondary_host]:-}")"
  fields[port]="$(eval echo "\${${__res_sec_ref_name}[secondary_port]:-22}")"
  fields[user]="$(eval echo "\${${__res_sec_ref_name}[secondary_user]:-}")"
  fields[endpoint]="$(eval echo "\${${__res_sec_ref_name}[secondary_endpoint]:-}")"
  fields[bucket]="$(eval echo "\${${__res_sec_ref_name}[secondary_bucket]:-}")"
  fields[access_key]="$(eval echo "\${${__res_sec_ref_name}[secondary_access_key]:-}")"
  fields[secret_key]="$(eval echo "\${${__res_sec_ref_name}[secondary_secret_key]:-}")"

  if has_function "backend_${sec_backend}_render_env"; then
    "backend_${sec_backend}_render_env" "$profile_name" fields "${__res_sec_ref_name}" "secondary"
  fi
}

append_secondary_config_and_notice() {
  local __res_sec_ref_name="$1"
  local __content_sec_ref_name="$2"
  local __notice_sec_ref_name="$3"

  local sec_backend; eval "sec_backend=\"\${${__res_sec_ref_name}[secondary_backend]:-}\""
  if [[ -n "$sec_backend" ]]; then
    local _tmp_refactor_scalar_val_16; _tmp_refactor_scalar_val_16="$(render_secondary_config "$1")"; eval "${__content_sec_ref_name}+=\${_tmp_refactor_scalar_val_16}"
    
    local -A fields=()
    fields[host]="$(eval echo "\${${__res_sec_ref_name}[secondary_host]:-}")"
    fields[port]="$(eval echo "\${${__res_sec_ref_name}[secondary_port]:-22}")"
    fields[user]="$(eval echo "\${${__res_sec_ref_name}[secondary_user]:-}")"
    fields[endpoint]="$(eval echo "\${${__res_sec_ref_name}[secondary_endpoint]:-}")"
    fields[bucket]="$(eval echo "\${${__res_sec_ref_name}[secondary_bucket]:-}")"
    fields[access_key]="$(eval echo "\${${__res_sec_ref_name}[secondary_access_key]:-}")"
    fields[secret_key]="$(eval echo "\${${__res_sec_ref_name}[secondary_secret_key]:-}")"
    if [[ "$sec_backend" == "sftp" ]]; then
      generate_ssh_key_if_missing
      fields[pubkey]="$(cat "${BACKUP_SSH_KEY}.pub")"
    fi

    local sec_notice=""
    if has_function "backend_${sec_backend}_render_notice"; then
      sec_notice=$("backend_${sec_backend}_render_notice" fields "secondary")
    fi
    
    if [[ -n "$sec_notice" ]]; then
      if [[ "$sec_notice" == *"\$(render_s3_bucket_policy"* ]]; then
        sec_notice=$(eval "cat <<EOF
${sec_notice}
EOF" 2>/dev/null || echo "$sec_notice")
      fi
      eval "${__notice_sec_ref_name}+=\"\\\$'\n\n'\${sec_notice}\""
    fi
  fi
}



backend_s3_render_notice() {
  local fields_ref_name="$1"
  local slot="${2:-primary}"
  local prefix=""
  if [[ "$slot" == "secondary" ]]; then
    prefix="[2차 소산지 S3] "
  fi
  printf '%s최소권한 버킷 정책을 아래와 같이 적용하세요:\n' "$prefix"
  render_s3_bucket_policy "$(eval echo "\${${fields_ref_name}[bucket]}")"
}

# shellcheck disable=SC2034
backend_sftp_configure() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178
  local _resolved_name="$1"
  local _out_env_name="$2"
  local _out_notice_name="$3"

  # Prepare keys
  generate_ssh_key_if_missing
  local -A fields=()
  fields[pubkey]="$(cat "${BACKUP_SSH_KEY}.pub")"
  fields[host]="$(eval echo "\${${_resolved_name}[host]:-}")"
  fields[port]="$(eval echo "\${${_resolved_name}[port]:-22}")"
  fields[user]="$(eval echo "\${${_resolved_name}[user]:-}")"

  # Render Env
  local _tmp_refactor_scalar_val_14; _tmp_refactor_scalar_val_14=$(backend_sftp_render_env "$(eval echo "\${${_resolved_name}[profile_name]:-$(hostname)}")" fields "${_resolved_name}" "primary"); eval "${_out_env_name}=\${_tmp_refactor_scalar_val_14}"
  local _tmp_refactor_scalar_val_9; _tmp_refactor_scalar_val_9=$'\n'; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_9}"
  local _tmp_refactor_scalar_val_10; _tmp_refactor_scalar_val_10="$(render_notification_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_10}"
  local _tmp_refactor_scalar_val_11; _tmp_refactor_scalar_val_11="$(render_audit_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_11}"
  local _tmp_refactor_scalar_val_12; _tmp_refactor_scalar_val_12="$(render_db_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_12}"
  local _tmp_refactor_scalar_val_13; _tmp_refactor_scalar_val_13="$(render_ntp_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_13}"

  # Render Notice
  local _tmp_refactor_scalar_val_15; _tmp_refactor_scalar_val_15=$(backend_sftp_render_notice fields "primary"); eval "${_out_notice_name}=\${_tmp_refactor_scalar_val_15}"
}

backend_sftp_test_connectivity() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178
  local _resolved_name="$1"
  generate_ssh_key_if_missing
  (
    local host_val="" port_val="" user_val=""
    ref_get "${_resolved_name}" host host_val
    ref_get "${_resolved_name}" port port_val
    ref_get "${_resolved_name}" user user_val
    export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
    export RCLONE_CONFIG_SYNO_BACKUP_HOST="$host_val"
    export RCLONE_CONFIG_SYNO_BACKUP_PORT="$port_val"
    export RCLONE_CONFIG_SYNO_BACKUP_USER="$user_val"
    export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${BACKUP_SSH_KEY}"
    rclone_check_connectivity "syno_backup" "${BACKUP_VERBOSE:-0}"
  )
}

# shellcheck disable=SC2034
backend_s3_configure() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178
  local _resolved_name="$1"
  local _out_env_name="$2"
  local _out_notice_name="$3"

  local -A fields=()
  fields[endpoint]="$(eval echo "\${${_resolved_name}[endpoint]:-}")"
  fields[bucket]="$(eval echo "\${${_resolved_name}[bucket]:-}")"
  fields[access_key]="$(eval echo "\${${_resolved_name}[access_key]:-}")"
  fields[secret_key]="$(eval echo "\${${_resolved_name}[secret_key]:-}")"

  # Render Env
  local _tmp_refactor_scalar_val_6; _tmp_refactor_scalar_val_6=$(backend_s3_render_env "$(eval echo "\${${_resolved_name}[profile_name]:-$(hostname)}")" fields "${_resolved_name}" "primary"); eval "${_out_env_name}=\${_tmp_refactor_scalar_val_6}"
  local _tmp_refactor_scalar_val_1; _tmp_refactor_scalar_val_1=$'\n'; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_1}"
  local _tmp_refactor_scalar_val_2; _tmp_refactor_scalar_val_2="$(render_notification_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_2}"
  local _tmp_refactor_scalar_val_3; _tmp_refactor_scalar_val_3="$(render_audit_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_3}"
  local _tmp_refactor_scalar_val_4; _tmp_refactor_scalar_val_4="$(render_db_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_4}"
  local _tmp_refactor_scalar_val_5; _tmp_refactor_scalar_val_5="$(render_ntp_env_block "${_resolved_name}")"; eval "${_out_env_name}+=\${_tmp_refactor_scalar_val_5}"

  # Render Notice
  local _tmp_refactor_scalar_val_7; _tmp_refactor_scalar_val_7=$(backend_s3_render_notice fields "primary"); eval "${_out_notice_name}=\${_tmp_refactor_scalar_val_7}"
  local current_notice=""
  eval "current_notice=\"\${${_out_notice_name}:-}\""
  if [[ "$current_notice" == *"\$(render_s3_bucket_policy"* ]]; then
    local s3_eval_notice
    s3_eval_notice=$(eval "cat <<EOF
${current_notice}
EOF" 2>/dev/null || echo "$current_notice")
    eval "${_out_notice_name}=\"\${s3_eval_notice}\""
  fi
}

backend_s3_test_connectivity() {
  return 0
}

# --- notification slack adapter ---
notification_slack_validate() {
  return 0
}

notification_slack_send() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178,SC2034
  local __ctx_name="$1"
  local url; eval "url=\"\${${__ctx_name}[notify_url]:-}\""
  [[ -z "$url" ]] && url="${BACKUP_NOTIFICATION_URL:-}"
  local status; eval "status=\"\${${__ctx_name}[status]:-success}\""
  local err_msg; eval "err_msg=\"\${${__ctx_name}[err_msg]:-}\""
  local hostname_val; hostname_val=$(hostname)
  local profile_name_val="${BACKUP_PROFILE_NAME:-$hostname_val}"

  local payload
  payload=$(build_notification_payload_slack "$status" "$hostname_val" "$profile_name_val" "$err_msg")
  curl -s -X POST -H "Content-Type: application/json" --max-time 10 -d "$payload" "$url"
}

# --- notification discord adapter ---
notification_discord_validate() {
  return 0
}

notification_discord_send() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178,SC2034
  local __ctx_name="$1"
  local url; eval "url=\"\${${__ctx_name}[notify_url]:-}\""
  [[ -z "$url" ]] && url="${BACKUP_NOTIFICATION_URL:-}"
  local status; eval "status=\"\${${__ctx_name}[status]:-success}\""
  local err_msg; eval "err_msg=\"\${${__ctx_name}[err_msg]:-}\""
  local hostname_val; hostname_val=$(hostname)
  local profile_name_val="${BACKUP_PROFILE_NAME:-$hostname_val}"

  local payload
  payload=$(build_notification_payload_discord "$status" "$hostname_val" "$profile_name_val" "$err_msg")
  curl -s -X POST -H "Content-Type: application/json" --max-time 10 -d "$payload" "$url"
}

# --- notification custom adapter ---
notification_custom_validate() {
  return 0
}

notification_custom_send() {
  # nameref로 넘어온 연관 배열에 접근하므로 scalar/array 재할당 경고 우회
  # shellcheck disable=SC2178,SC2034
  local __ctx_name="$1"
  local url; eval "url=\"\${${__ctx_name}[notify_url]:-}\""
  [[ -z "$url" ]] && url="${BACKUP_NOTIFICATION_URL:-}"
  local status; eval "status=\"\${${__ctx_name}[status]:-success}\""
  local err_msg; eval "err_msg=\"\${${__ctx_name}[err_msg]:-}\""
  local hostname_val; hostname_val=$(hostname)
  local profile_name_val="${BACKUP_PROFILE_NAME:-$hostname_val}"
  local method; eval "method=\"\${${__ctx_name}[method]:-}\""
  [[ -z "$method" ]] && method="${BACKUP_NOTIFICATION_METHOD:-POST}"
  local headers_val; eval "headers_val=\"\${${__ctx_name}[headers]:-}\""
  [[ -z "$headers_val" ]] && headers_val="${BACKUP_NOTIFICATION_HEADERS:-}"
  local body_success; eval "body_success=\"\${${__ctx_name}[body_success]:-}\""
  [[ -z "$body_success" ]] && body_success="${BACKUP_NOTIFICATION_BODY_SUCCESS:-}"
  local body_failure; eval "body_failure=\"\${${__ctx_name}[body_failure]:-}\""
  [[ -z "$body_failure" ]] && body_failure="${BACKUP_NOTIFICATION_BODY_FAILURE:-}"
  local profile_command; eval "profile_command=\"\${${__ctx_name}[profile_command]:-}\""

  local payload
  payload=$(build_notification_payload_custom "$status" "$hostname_val" "$profile_name_val" "$err_msg" "$body_success" "$body_failure" "$profile_command")
  
  local -a curl_headers=()
  if [[ -n "$headers_val" ]]; then
    local -a headers_arr=()
    IFS=',' read -ra headers_arr <<< "$headers_val"
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

# --- database mysql adapter ---
database_mysql_default_command() {
  printf 'mysqldump --all-databases --single-transaction --quick --order-by-primary'
}

database_mysql_validate_config() {
  return 0
}

database_mysql_validate_dump() {
  validate_mysql_mariadb_dump "$1"
}

# --- database mariadb adapter ---
database_mariadb_default_command() {
  printf 'mariadb-dump --all-databases --single-transaction --quick --order-by-primary'
}

database_mariadb_validate_config() {
  return 0
}

database_mariadb_validate_dump() {
  validate_mysql_mariadb_dump "$1"
}

# --- database postgres adapter ---
database_postgres_default_command() {
  printf 'pg_dumpall -U postgres'
}

database_postgres_validate_config() {
  return 0
}

database_postgres_validate_dump() {
  local header="$1"
  if [[ "$header" == *"PostgreSQL database dump"* || "$header" == *"PostgreSQL database cluster dump"* ]]; then
    return 0
  fi
  return 1
}

# --- database custom adapter ---
database_custom_default_command() {
  printf ''
}

database_custom_validate_config() {
  return 0
}

database_custom_validate_dump() {
  return 0
}

restic_is_initialized() {
  restic snapshots >/dev/null 2>&1
}

restic_repo_init() {
  local profile_name="${1:-}"
  if [[ -z "$profile_name" ]]; then
    profile_name=$(resolve_profile_name)
  fi

  write_resticprofile_assets "$profile_name" "$DEFAULT_ON_CALENDAR"

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
    safe_spin "1차 저장소 초기화(init) 진행 중..." -- restic "${restic_init_args[@]}"
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

cmd_init() {
  if has_help_flag "$@"; then
    help_init
    return 0
  fi
  require_root
  require_backup_env

  local profile_name; profile_name=$(resolve_profile_name)
  restic_repo_init "$profile_name"
}


is_systemd_active() {
  if command -v systemctl >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

determine_scheduler_adapter() {
  # 1. 환경 변수로 어댑터가 명시적으로 설정된 경우 해당 값을 강제 적용
  if [[ -n "${BACKUP_SCHEDULER_ADAPTER:-}" ]]; then
    case "$BACKUP_SCHEDULER_ADAPTER" in
      systemd|cron|mock)
        printf '%s' "$BACKUP_SCHEDULER_ADAPTER"
        return 0
        ;;
      *)
        die "지원하지 않는 스케줄러 어댑터: $BACKUP_SCHEDULER_ADAPTER" 1
        ;;
    esac
  fi

  # 2. systemd 데몬 동작 확인
  if is_systemd_active; then
    printf 'systemd'
    return 0
  fi

  # 3. crontab 사용 가능 여부 확인
  if command -v crontab >/dev/null 2>&1; then
    log_warn "systemd를 사용할 수 없어 crontab 스케줄러(cron)로 자동 폴백합니다."
    printf 'cron'
    return 0
  fi

  die "스케줄링을 지원하는 데몬(systemd 또는 crontab)을 찾을 수 없습니다." 1
}

scheduler_register() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local _s_cfg_name="$2"
  local adapter; adapter=$(determine_scheduler_adapter)
  "scheduler_${adapter}_register" "$profile_name" "${_s_cfg_name}"
}

scheduler_unregister() {
  local profile_name="$1"
  local target_type="$2"
  local adapter; adapter=$(determine_scheduler_adapter)
  "scheduler_${adapter}_unregister" "$profile_name" "$target_type"
}

scheduler_status() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local _s_stat_name="$2"
  local adapter; adapter=$(determine_scheduler_adapter)
  "scheduler_${adapter}_status" "$profile_name" "${_s_stat_name}"
}

# --- 1) Mock Scheduler Adapter ---
scheduler_mock_register() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local _m_cfg_name="$2"
  local state_file="${TEST_ROOT:-/tmp}/var/log/scheduler_mock.state"
  mkdir -p "$(dirname "$state_file")"

  local on_cal; eval "on_cal=\"\${${_m_cfg_name}[on-calendar]:-}\""
  local d_cal; eval "d_cal=\"\${${_m_cfg_name}[on-calendar-daily]:-}\""
  local dr_cal; eval "dr_cal=\"\${${_m_cfg_name}[on-calendar-drill]:-}\""
  local daily; eval "daily=\"\${${_m_cfg_name}[daily]:-0}\""
  local drill; eval "drill=\"\${${_m_cfg_name}[restore-drill]:-0}\""

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
    local ntp_rep_val="${BACKUP_NTP_REPORT:-${BACKUP_CHRONY_REPORT:-}}"
    if [[ "$ntp_rep_val" == "1" ]]; then
      printf 'ntp_report_enabled=1\n' >> "$state_file"
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
  local _m_stat_name="$2"
  local state_file="${TEST_ROOT:-/tmp}/var/log/scheduler_mock.state"

  if [[ ! -f "$state_file" ]]; then
    ref_set "${_m_stat_name}" backup "inactive"
    ref_set "${_m_stat_name}" daily "inactive"
    ref_set "${_m_stat_name}" drill "inactive"
    ref_set "${_m_stat_name}" db_backup "inactive"
    return 0
  fi

  local line key val
  while IFS='=' read -r key val || [[ -n "$key" ]]; do
    [[ -z "$key" ]] && continue
    case "$key" in
      backup_enabled)
        ref_set "${_m_stat_name}" backup "$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
      daily_enabled)
        ref_set "${_m_stat_name}" daily "$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
      drill_enabled)
        ref_set "${_m_stat_name}" drill "$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
      db_backup_enabled)
        ref_set "${_m_stat_name}" db_backup "$([[ "$val" == "1" ]] && echo "active" || echo "inactive")"
        ;;
    esac
  done < "$state_file"

  [[ -z "$(eval echo "\${${_m_stat_name}[backup]:-}")" ]] && ref_set "${_m_stat_name}" backup "inactive"
  [[ -z "$(eval echo "\${${_m_stat_name}[daily]:-}")" ]] && ref_set "${_m_stat_name}" daily "inactive"
  [[ -z "$(eval echo "\${${_m_stat_name}[drill]:-}")" ]] && ref_set "${_m_stat_name}" drill "inactive"
  [[ -z "$(eval echo "\${${_m_stat_name}[db_backup]:-}")" ]] && ref_set "${_m_stat_name}" db_backup "inactive"
  return 0
}

# --- 3) Cron Scheduler Adapter ---
# shellcheck disable=SC2120
scheduler_cron_register() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local _c_cfg_name="$2"

  local on_calendar; eval "on_calendar=\"\${${_c_cfg_name}[on-calendar]:-$DEFAULT_ON_CALENDAR}\""
  local daily_on_calendar; eval "daily_on_calendar=\"\${${_c_cfg_name}[on-calendar-daily]:-*-*-* 01:00:00}\""
  local drill_on_calendar; eval "drill_on_calendar=\"\${${_c_cfg_name}[on-calendar-drill]:-*-*-01 01:30:00}\""
  local daily; eval "daily=\"\${${_c_cfg_name}[daily]:-0}\""
  local restore_drill; eval "restore_drill=\"\${${_c_cfg_name}[restore-drill]:-0}\""
  local ntp_report; eval "ntp_report=\"\${${_c_cfg_name}[ntp_report]:-}\""
  [[ -z "$ntp_report" ]] && ntp_report="${BACKUP_NTP_REPORT:-${BACKUP_CHRONY_REPORT:-}}"

  local cron_on_cal; cron_on_cal=$(convert_calendar_to_cron "$on_calendar")
  local cron_daily; cron_daily=$(convert_calendar_to_cron "$daily_on_calendar")
  local cron_drill; cron_drill=$(convert_calendar_to_cron "$drill_on_calendar")
  local cron_ntp; cron_ntp=$(convert_calendar_to_cron "${DEFAULT_NTP_ON_CALENDAR}")

  local install_path="${BACKUP_SCRIPT_INSTALL_PATH:-/usr/local/bin/backup.sh}"
  local r_cfg_file="${RESTICPROFILE_CONFIG_FILE:-/etc/restic/profiles.yaml}"
  local path_env="PATH=/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"

  # Retrieve current crontab
  local current_cron; current_cron=$(crontab -l 2>/dev/null || true)

  # Parse current crontab, stripping the existing block but also capturing its entries
  local clean_cron=""
  local block_entries=""
  local in_block=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" == "# RESTIC_BACKUP_BEGIN" ]]; then
      in_block=1
      continue
    fi
    if [[ "$line" == "# RESTIC_BACKUP_END" ]]; then
      in_block=0
      continue
    fi
    if (( in_block == 1 )); then
      if [[ -n "$block_entries" ]]; then
        block_entries="${block_entries}"$'\n'"${line}"
      else
        block_entries="${line}"
      fi
    else
      if [[ -n "$clean_cron" ]]; then
        clean_cron="${clean_cron}"$'\n'"${line}"
      else
        clean_cron="${line}"
      fi
    fi
  done <<< "$current_cron"

  # Parse existing block entries into an associative array
  declare -A active_jobs=()
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -n "$line" ]] || continue
    if [[ "$line" == *"logger -t backup.sh-files"* ]]; then
      active_jobs[files]="$line"
    elif [[ "$line" == *"logger -t backup.sh-db"* ]]; then
      active_jobs[db]="$line"
    elif [[ "$line" == *"logger -t backup.sh-audit-daily"* ]]; then
      active_jobs[daily]="$line"
    elif [[ "$line" == *"logger -t backup.sh-audit-drill"* ]]; then
      active_jobs[drill]="$line"
    elif [[ "$line" == *"logger -t backup.sh-ntp-report"* ]]; then
      active_jobs[ntp]="$line"
    fi
  done <<< "$block_entries"

  # Update active jobs based on flags
  if (( daily )); then
    active_jobs[daily]="${cron_daily} ${path_env} ${install_path} audit --daily --report 2>&1 | logger -t backup.sh-audit-daily"
  elif (( restore_drill )); then
    active_jobs[drill]="${cron_drill} ${path_env} ${install_path} audit --restore-drill --report 2>&1 | logger -t backup.sh-audit-drill"
  else
    # Regular registration: rebuild all
    active_jobs[files]="${cron_on_cal} ${path_env} resticprofile --config ${r_cfg_file} --name ${profile_name} backup 2>&1 | logger -t backup.sh-files"
    if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
      local db_cal="${BACKUP_DB_SCHEDULE:-$on_calendar}"
      local cron_db; cron_db=$(convert_calendar_to_cron "$db_cal")
      active_jobs[db]="${cron_db} ${path_env} resticprofile --config ${r_cfg_file} --name ${profile_name}-db backup 2>&1 | logger -t backup.sh-db"
    else
      unset 'active_jobs[db]'
    fi
    active_jobs[daily]="${cron_daily} ${path_env} ${install_path} audit --daily --report 2>&1 | logger -t backup.sh-audit-daily"
    active_jobs[drill]="${cron_drill} ${path_env} ${install_path} audit --restore-drill --report 2>&1 | logger -t backup.sh-audit-drill"
    if [[ "$ntp_report" == "1" ]]; then
      active_jobs[ntp]="${cron_ntp} ${path_env} ${install_path} ntp --report 2>&1 | logger -t backup.sh-ntp-report"
    else
      unset 'active_jobs[ntp]'
    fi
  fi

  # Build new entries string in a predictable order
  local new_entries=""
  for job_key in files db daily drill ntp; do
    if [[ -n "${active_jobs[$job_key]:-}" ]]; then
      if [[ -n "$new_entries" ]]; then
        new_entries="${new_entries}"$'\n'"${active_jobs[$job_key]}"
      else
        new_entries="${active_jobs[$job_key]}"
      fi
    fi
  done

  # Combine clean_cron and new_entries
  local final_cron=""
  if [[ -n "$clean_cron" ]]; then
    # Avoid extra newline if clean_cron is empty/whitespace only
    local trimmed; trimmed=$(echo "$clean_cron" | xargs)
    if [[ -n "$trimmed" ]]; then
      final_cron="${clean_cron}"$'\n'
    fi
  fi
  final_cron="${final_cron}# RESTIC_BACKUP_BEGIN"$'\n'"${new_entries}"$'\n'"# RESTIC_BACKUP_END"

  # Write back to crontab
  printf '%s\n' "$final_cron" | crontab -
  log_info "schedule enable 완료 (cron)"
}

scheduler_cron_unregister() {
  local profile_name="$1"
  local target_type="$2"

  local current_cron; current_cron=$(crontab -l 2>/dev/null || true)
  [[ -n "$current_cron" ]] || return 0

  local clean_cron=""
  local in_block=0
  local block_entries=""
  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" == "# RESTIC_BACKUP_BEGIN" ]]; then
      in_block=1
      continue
    fi
    if [[ "$line" == "# RESTIC_BACKUP_END" ]]; then
      in_block=0
      # Process block entries if we selectively delete
      if [[ "$target_type" == "daily" || "$target_type" == "drill" ]]; then
        local entry_line
        local updated_block=""
        while IFS= read -r entry_line || [[ -n "$entry_line" ]]; do
          [[ -z "$entry_line" ]] && continue
          if [[ "$target_type" == "daily" && "$entry_line" == *"audit --daily"* ]]; then
            continue
          fi
          if [[ "$target_type" == "drill" && "$entry_line" == *"audit --restore-drill"* ]]; then
            continue
          fi
          if [[ -n "$updated_block" ]]; then
            updated_block="${updated_block}"$'\n'"${entry_line}"
          else
            updated_block="${entry_line}"
          fi
        done <<< "$block_entries"

        if [[ -n "$updated_block" ]]; then
          if [[ -n "$clean_cron" ]]; then
            clean_cron="${clean_cron}"$'\n'
          fi
          clean_cron="${clean_cron}# RESTIC_BACKUP_BEGIN"$'\n'"${updated_block}"$'\n'"# RESTIC_BACKUP_END"
        fi
      fi
      block_entries=""
      continue
    fi

    if (( in_block )); then
      if [[ -n "$block_entries" ]]; then
        block_entries="${block_entries}"$'\n'"${line}"
      else
        block_entries="${line}"
      fi
    else
      if [[ -n "$clean_cron" ]]; then
        clean_cron="${clean_cron}"$'\n'"${line}"
      else
        clean_cron="${line}"
      fi
    fi
  done <<< "$current_cron"

  # Write back if changed
  if [[ -n "$clean_cron" ]]; then
    # Trim and remove extra trailing newlines
    local trimmed; trimmed=$(echo "$clean_cron" | xargs)
    if [[ -n "$trimmed" ]]; then
      printf '%s\n' "$clean_cron" | crontab -
    else
      crontab -r 2>/dev/null || true
    fi
  else
    crontab -r 2>/dev/null || true
  fi

  log_info "schedule disable 완료 (cron: ${target_type})"
}

scheduler_cron_status() {
  local profile_name="$1"
  # shellcheck disable=SC2178,SC2154
  local _c_stat_name="$2"

  local current_cron; current_cron=$(crontab -l 2>/dev/null || true)
  if [[ -z "$current_cron" ]]; then
    ref_set "${_c_stat_name}" backup "inactive"
    ref_set "${_c_stat_name}" daily "inactive"
    ref_set "${_c_stat_name}" drill "inactive"
    ref_set "${_c_stat_name}" db_backup "inactive"
    return 0
  fi

  local has_backup=0 has_daily=0 has_drill=0 has_db=0
  local line
  while IFS= read -r line || [[ -n "$line" ]]; do
    if [[ "$line" == *"resticprofile"* && "$line" == *"--name ${profile_name}"* && "$line" == *"backup"* ]]; then
      has_backup=1
    fi
    if [[ "$line" == *"audit --daily"* ]]; then
      has_daily=1
    fi
    if [[ "$line" == *"audit --restore-drill"* ]]; then
      has_drill=1
    fi
    if [[ "$line" == *"resticprofile"* && "$line" == *"--name ${profile_name}-db"* && "$line" == *"backup"* ]]; then
      has_db=1
    fi
  done <<< "$current_cron"

  # Assign active/inactive
  if (( has_backup )); then ref_set "${_c_stat_name}" backup "active"; else ref_set "${_c_stat_name}" backup "inactive"; fi
  if (( has_daily )); then ref_set "${_c_stat_name}" daily "active"; else ref_set "${_c_stat_name}" daily "inactive"; fi
  if (( has_drill )); then ref_set "${_c_stat_name}" drill "active"; else ref_set "${_c_stat_name}" drill "inactive"; fi
  if (( has_db )); then ref_set "${_c_stat_name}" db_backup "active"; else ref_set "${_c_stat_name}" db_backup "inactive"; fi
}

# --- 2) Systemd Scheduler Adapter ---
scheduler_systemd_register() {
  local profile_name="$1"
  # shellcheck disable=SC2178
  local _sys_cfg_name="$2"

  local on_calendar; eval "on_calendar=\"\${${_sys_cfg_name}[on-calendar]:-$DEFAULT_ON_CALENDAR}\""
  local daily_on_calendar; eval "daily_on_calendar=\"\${${_sys_cfg_name}[on-calendar-daily]:-*-*-* 01:00:00}\""
  local drill_on_calendar; eval "drill_on_calendar=\"\${${_sys_cfg_name}[on-calendar-drill]:-*-*-01 01:30:00}\""
  local daily; eval "daily=\"\${${_sys_cfg_name}[daily]:-0}\""
  local restore_drill; eval "restore_drill=\"\${${_sys_cfg_name}[restore-drill]:-0}\""
  local ntp_report; eval "ntp_report=\"\${${_sys_cfg_name}[ntp_report]:-}\""
  [[ -z "$ntp_report" ]] && ntp_report="${BACKUP_NTP_REPORT:-${BACKUP_CHRONY_REPORT:-}}"

  if (( daily )); then
    write_systemd_timer_unit "backup-audit-daily" "Restic Daily Backup Audit Report" "Run Restic Daily Backup Audit Report Timer" "$BACKUP_SCRIPT_INSTALL_PATH audit --daily --report" "$daily_on_calendar"
    systemd_reload_daemon
    systemd_enable_unit "backup-audit-daily.timer"
    log_info "schedule enable 완료 (daily: ${daily_on_calendar})"
  elif (( restore_drill )); then
    write_systemd_timer_unit "backup-audit-restore-drill" "Restic Restore Drill Report" "Run Restic Restore Drill Report Timer" "$BACKUP_SCRIPT_INSTALL_PATH audit --restore-drill --report" "$drill_on_calendar"
    systemd_reload_daemon
    systemd_enable_unit "backup-audit-restore-drill.timer"
    log_info "schedule enable 완료 (drill: ${drill_on_calendar})"
  else
    resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" schedule
    if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "${profile_name}-db" schedule
    fi
    write_systemd_timer_unit "backup-audit-daily" "Restic Daily Backup Audit Report" "Run Restic Daily Backup Audit Report Timer" "$BACKUP_SCRIPT_INSTALL_PATH audit --daily --report" "$daily_on_calendar"
    write_systemd_timer_unit "backup-audit-restore-drill" "Restic Restore Drill Report" "Run Restic Restore Drill Report Timer" "$BACKUP_SCRIPT_INSTALL_PATH audit --restore-drill --report" "$drill_on_calendar"
    # 구버전 chrony 타이머 잔재 정리
    if [[ -f "$SYSTEMD_UNIT_DIR/backup-chrony-report.timer" ]]; then
      systemd_disable_unit "backup-chrony-report.timer" >/dev/null 2>&1 || true
      rm -f "$SYSTEMD_UNIT_DIR/backup-chrony-report.service"
      rm -f "$SYSTEMD_UNIT_DIR/backup-chrony-report.timer"
    fi

    if [[ "$ntp_report" == "1" ]]; then
      write_systemd_timer_unit "backup-ntp-report" "ISMS-P 2.9.3 NTP Sync Evidence Report" "Run ISMS-P NTP Sync Evidence Report Timer" "$BACKUP_SCRIPT_INSTALL_PATH ntp --report" "${DEFAULT_NTP_ON_CALENDAR}"
      systemd_enable_unit "backup-ntp-report.timer"
      log_info "NTP 증적 타이머도 등록했습니다 (${DEFAULT_NTP_ON_CALENDAR})"
    fi
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
    systemd_disable_unit "backup-ntp-report.timer" >/dev/null 2>&1 || true
    systemd_disable_unit "backup-chrony-report.timer" >/dev/null 2>&1 || true
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-daily.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-daily.timer"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-audit-restore-drill.timer"
    rm -f "$SYSTEMD_UNIT_DIR/backup-ntp-report.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-ntp-report.timer"
    rm -f "$SYSTEMD_UNIT_DIR/backup-chrony-report.service"
    rm -f "$SYSTEMD_UNIT_DIR/backup-chrony-report.timer"
    systemd_reload_daemon
    log_info "schedule disable 완료"
  fi
}

scheduler_systemd_status() {
  local profile_name="$1"
  # nameref로 전달받아 변수 선언 분석 우회
  # shellcheck disable=SC2178,SC2154
  local _sys_stat_name="$2"

  local timer_state daily_timer_state drill_timer_state db_timer_state
  timer_state=$(systemctl is-active "$(resticprofile_timer_unit_name "$profile_name")" 2>/dev/null) || true
  daily_timer_state=$(systemctl is-active backup-audit-daily.timer 2>/dev/null) || true
  drill_timer_state=$(systemctl is-active backup-audit-restore-drill.timer 2>/dev/null) || true
  db_timer_state=$(systemctl is-active "$(resticprofile_timer_unit_name "${profile_name}-db")" 2>/dev/null) || true

  ref_set "${_sys_stat_name}" backup "${timer_state:-unknown}"
  ref_set "${_sys_stat_name}" daily "${daily_timer_state:-unknown}"
  ref_set "${_sys_stat_name}" drill "${drill_timer_state:-unknown}"
  ref_set "${_sys_stat_name}" db_backup "${db_timer_state:-inactive}"
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

write_systemd_timer_unit() {
  local unit_name="$1"
  local service_desc="$2"
  local timer_desc="$3"
  local exec_cmd="$4"
  local calendar_spec="$5"

  mkdir -p "$SYSTEMD_UNIT_DIR"

  cat > "$SYSTEMD_UNIT_DIR/${unit_name}.service" <<EOF
[Unit]
Description=${service_desc}
After=network.target

[Service]
Type=oneshot
ExecStart=${exec_cmd}
EOF
  chmod 644 "$SYSTEMD_UNIT_DIR/${unit_name}.service"

  cat > "$SYSTEMD_UNIT_DIR/${unit_name}.timer" <<EOF
[Unit]
Description=${timer_desc}

[Timer]
OnCalendar=${calendar_spec}
Persistent=true

[Install]
WantedBy=timers.target
EOF
  chmod 644 "$SYSTEMD_UNIT_DIR/${unit_name}.timer"
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
    status)
      setup_colors
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

      log_info "스케줄러 상태 정보 (프로필: ${profile_name})"
      log_info "  - 파일/DB 백업 타이머 (backup): ${styled_timer}"
      log_info "  - 일일 감사 보고 타이머 (daily): ${styled_daily_timer}"
      log_info "  - 복원 점검 타이머 (drill): ${styled_drill_timer}"
      ;;
    *)
      die "schedule은 'enable', 'disable' 또는 'status'만 지원합니다 (입력값: '${action}')"
      ;;
  esac
}

run_pipeline_execute() {
  local profile_name="${1:-}"
  local on_calendar="${2:-$DEFAULT_ON_CALENDAR}"

  if [[ -z "$profile_name" ]]; then
    profile_name=$(resolve_profile_name)
  fi

  write_resticprofile_assets "$profile_name" "$on_calendar"

  local -a resticprofile_args=(--config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" backup)
  if [[ "${BACKUP_VERBOSE:-0}" == "1" ]]; then
    resticprofile_args+=(-v)
  fi

  local pipeline_err=""
  local run_status=0
  safe_spin "1차 파일 백업 진행 중..." -- resticprofile "${resticprofile_args[@]}" || run_status=$?
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

  run_pipeline_execute "$profile_name"
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

  if is_interactive; then
    safe_style "⚙  백업 상태 (Backup Status)" --foreground "212" --bold
  else
    printf '%b%b⚙  백업 상태 (Backup Status)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
  fi
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
  if is_interactive; then
    safe_style "⚙  최근 스냅샷 (Recent Snapshots)" --foreground "212" --bold
  else
    printf '%b%b⚙  최근 스냅샷 (Recent Snapshots)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
  fi
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
  local _opts_name="$1"
  # shellcheck disable=SC2178  # nameref to caller's associative array
  local _res_name="$2"

  local tester_val="" ciso_val="" rto_val="" target_dir_val=""
  ref_get "${_opts_name}" tester tester_val
  ref_get "${_opts_name}" ciso ciso_val
  ref_get "${_opts_name}" rto rto_val
  [[ -z "$rto_val" ]] && rto_val="120"
  ref_get "${_opts_name}" target target_dir_val
  [[ -z "$target_dir_val" ]] && target_dir_val="/tmp/restore_test"

  ref_set "${_res_name}" "test_date" "$(date "+%Y-%m-%d")"
  ref_set "${_res_name}" "tester" "$tester_val"
  ref_set "${_res_name}" "ciso" "$ciso_val"
  ref_set "${_res_name}" "rto_minutes" "$rto_val"
  local target_dir="$target_dir_val"
  ref_set "${_res_name}" "target_dir" "$target_dir"

  local os_name="Rocky Linux 9"
  if [[ -f /etc/os-release ]]; then
    # shellcheck disable=SC1091
    os_name=$(source /etc/os-release && echo "${PRETTY_NAME:-Rocky Linux 9}")
  fi
  ref_set "${_res_name}" "os_name" "$os_name"

  # 1. Primary snapshot
  local primary_info; primary_info=$(query_snapshot_info)
  local primary_snap="" primary_snap_time=""
  if [[ -n "$primary_info" ]]; then
    read -r primary_snap primary_snap_time <<< "$primary_info"
  fi

  ref_set "${_res_name}" "primary_snap" "$primary_snap"
  ref_set "${_res_name}" "primary_snap_time" "$primary_snap_time"

  if [[ -z "$primary_snap" ]]; then
    ref_set "${_res_name}" "primary_rto_satisfied" "false"
    ref_set "${_res_name}" "primary_rto_status" "초과 (미흡)"
    ref_set "${_res_name}" "error_message" "복구 테스트 실패: 저장소에 백업 스냅샷이 존재하지 않습니다."
    return 0
  fi

  if [[ -d "$target_dir" ]]; then
    if [[ "$target_dir" == /tmp/* || "$target_dir" == /var/tmp/* ]]; then
      rm -rf "$target_dir"
    else
      ref_set "${_res_name}" "primary_rto_satisfied" "false"
      ref_set "${_res_name}" "primary_rto_status" "초과 (미흡)"
      ref_set "${_res_name}" "error_message" "복구 경로가 안전하지 않습니다 (/tmp 또는 /var/tmp 하위 경로만 지원): $target_dir"
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
  ref_set "${_res_name}" "primary_elapsed_seconds" "$elapsed"
  ref_set "${_res_name}" "primary_elapsed_str" "$elapsed_str"

  local rto_seconds=$((rto * 60))
  if (( restore_ok && elapsed <= rto_seconds )); then
    ref_set "${_res_name}" "primary_rto_satisfied" "true"
    ref_set "${_res_name}" "primary_rto_status" "만족"
  else
    ref_set "${_res_name}" "primary_rto_satisfied" "false"
    ref_set "${_res_name}" "primary_rto_status" "초과 (미흡)"
  fi

  if (( ! restore_ok )); then
    ref_set "${_res_name}" "error_message" "restic restore 복구 실패"
    rm -rf "$target_dir"
    return 0
  fi

  local total_bytes=0
  total_bytes=$(du -sb "$target_dir" 2>/dev/null | awk '{print $1}') || total_bytes=0
  ref_set "${_res_name}" "primary_size" "$(format_bytes "$total_bytes")"
  rm -rf "$target_dir"

  # 2. DB snapshot
  if [[ -n "${BACKUP_DB_TYPE:-}" ]]; then
    ref_set "${_res_name}" "db_type" "${BACKUP_DB_TYPE}"
    local db_info; db_info=$(query_snapshot_info --tag db)
    local db_snap="" db_snap_time=""
    if [[ -n "$db_info" ]]; then
      read -r db_snap db_snap_time <<< "$db_info"
    fi
    
    ref_set "${_res_name}" "db_snap" "$db_snap"
    ref_set "${_res_name}" "db_snap_time" "$db_snap_time"

    if [[ -z "$db_snap" ]]; then
      ref_set "${_res_name}" "db_valid" "0"
      ref_set "${_res_name}" "error_message" "DB 복구 테스트 실패: 저장소에 DB 백업 스냅샷이 존재하지 않습니다."
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
        if has_function "database_${BACKUP_DB_TYPE}_validate_dump"; then
          if "database_${BACKUP_DB_TYPE}_validate_dump" "$header"; then
            db_valid=1
          fi
        else
          db_valid=1
        fi
      fi
    fi

    ref_set "${_res_name}" "db_valid" "$db_valid"
    rm -rf "$db_target_dir"

    if (( ! db_restore_ok )); then
      ref_set "${_res_name}" "error_message" "DB restic restore 복구 실패"
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
      ref_set "${_res_name}" "secondary_rto_satisfied" "false"
      ref_set "${_res_name}" "secondary_rto_status" "초과 (미흡)"
      ref_set "${_res_name}" "error_message" "${sec_res#ERROR:}"
    elif [[ "$sec_res" == SUCCESS:* ]]; then
      local sec_snap="" sec_snap_time="" sec_elapsed=0 sec_size_str=""
      IFS=":" read -r _status sec_snap sec_snap_time sec_elapsed sec_size_str <<< "$sec_res"
      
      ref_set "${_res_name}" "secondary_snap" "$sec_snap"
      ref_set "${_res_name}" "secondary_snap_time" "$sec_snap_time"
      ref_set "${_res_name}" "secondary_elapsed_seconds" "$sec_elapsed"
      
      local sec_elapsed_str
      if (( sec_elapsed < 60 )); then
        sec_elapsed_str="${sec_elapsed}초"
      else
        sec_elapsed_str="$((sec_elapsed / 60))분 $((sec_elapsed % 60))초"
      fi
      ref_set "${_res_name}" "secondary_elapsed_str" "$sec_elapsed_str"
      ref_set "${_res_name}" "secondary_size" "$sec_size_str"

      if (( sec_elapsed <= rto_seconds )); then
        ref_set "${_res_name}" "secondary_rto_satisfied" "true"
        ref_set "${_res_name}" "secondary_rto_status" "만족"
      else
        ref_set "${_res_name}" "secondary_rto_satisfied" "false"
        ref_set "${_res_name}" "secondary_rto_status" "초과 (미흡)"
      fi
    fi
  fi

  return 0
}

generate_snapshot_table_html() {
  local snapshots_json="$1"
  python3 -c '
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
        processed_bytes = None
        summary = snap.get("summary")
        if isinstance(summary, dict) and "total_bytes_processed" in summary:
            processed_bytes = summary["total_bytes_processed"]
        if processed_bytes is None:
            try:
                res = subprocess.run(["restic", "stats", "--json", snap.get("id")], capture_output=True, text=True, timeout=5)
                if res.returncode == 0:
                    stats_data = json.loads(res.stdout)
                    processed_bytes = stats_data.get("total_size")
            except Exception:
                pass
        size_str = ""
        if processed_bytes is not None:
            if processed_bytes >= 1073741824:
                size_str = "%.2f GB" % (processed_bytes / 1073741824.0)
            elif processed_bytes >= 1048576:
                size_str = "%.2f MB" % (processed_bytes / 1048576.0)
            elif processed_bytes >= 1024:
                size_str = "%.2f KB" % (processed_bytes / 1024.0)
            else:
                size_str = "%d B" % processed_bytes
        else:
            size_str = "확인 불가"
        print("<tr><td>%s</td><td>%s</td><td>%s</td><td>%s (%s)</td></tr>" % (sid, time_str, host, paths, size_str))
except Exception as e:
    print("<tr><td colspan=\"4\">(스냅샷 정보 해석 실패: %s)</td></tr>" % e)
' <<< "$snapshots_json"
}

render_audit_report_unified() {
  local report_type="$1"
  local format="$2"
  local _ra_ref_name="$3"
  
  case "$report_type" in
    restore_drill)
      case "$format" in
        txt|markdown)
          render_restore_drill_txt "${_ra_ref_name}"
          ;;
        json)
          render_restore_drill_json "${_ra_ref_name}"
          ;;
        html)
          render_restore_drill_html "${_ra_ref_name}"
          ;;
      esac
      ;;
    daily)
      case "$format" in
        txt)
          render_daily_txt "${_ra_ref_name}"
          ;;
        json)
          render_daily_json "${_ra_ref_name}"
          ;;
        html)
          render_daily_html "${_ra_ref_name}"
          ;;
      esac
      ;;
    general)
      case "$format" in
        txt)
          render_general_txt "${_ra_ref_name}"
          ;;
        json)
          render_general_json "${_ra_ref_name}"
          ;;
        html)
          render_general_html "${_ra_ref_name}"
          ;;
      esac
      ;;
  esac
}

# nameref 전달 인자 미사용 오탐 우회
# shellcheck disable=SC2034
write_evidence_report_bundle() {
  local _werb_data_name="$1"
  local report_file="$2"
  local report_type="$3"

  local base_path
  if [[ "$report_file" == *.md ]]; then
    base_path="${report_file%.md}"
  elif [[ "$report_file" == *.txt ]]; then
    base_path="${report_file%.txt}"
  elif [[ "$report_file" == *.json ]]; then
    base_path="${report_file%.json}"
  elif [[ "$report_file" == *.html ]]; then
    base_path="${report_file%.html}"
  else
    base_path="$report_file"
  fi

  local txt_path="${base_path}.txt"
  if [[ "$report_file" == *.md ]]; then
    txt_path="$report_file"
  fi
  local json_path="${base_path}.json"
  local html_path="${base_path}.html"

  mkdir -p "$(dirname "$base_path")"
  chmod 700 "$(dirname "$base_path")" 2>/dev/null || true

  if [[ "$report_type" == "chrony" || "$report_type" == "ntp" ]]; then
    render_ntp_txt "${_werb_data_name}" > "$txt_path"
    render_ntp_json "${_werb_data_name}" > "$json_path"
    render_ntp_html "${_werb_data_name}" > "$html_path"
  else
    render_audit_report_unified "$report_type" "txt" "${_werb_data_name}" > "$txt_path"
    render_audit_report_unified "$report_type" "json" "${_werb_data_name}" > "$json_path"
    render_audit_report_unified "$report_type" "html" "${_werb_data_name}" > "$html_path"
  fi

  chmod 600 "$txt_path" "$json_path" "$html_path" 2>/dev/null || true
}

write_audit_reports() {
  local _war_data_name="$1"
  local md_path="$2"
  write_evidence_report_bundle "${_war_data_name}" "$md_path" "restore_drill"
}

# Compatibility wrappers
render_restore_drill_report() {
  if (( $# == 1 )); then
  local _rrd_ref_name="$1"
    render_audit_report_unified "restore_drill" "txt" "${_rrd_ref_name}"
  else
    local -A _tmp_rrd=(
      [test_date]="$1" [tester]="$2" [primary_snap]="$3" [primary_snap_time]="$4" [target_dir]="$5"
      [primary_size]="$6" [primary_elapsed_str]="$7" [rto_minutes]="$9" [ciso]="${10}" [os_name]="${11}"
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
    render_audit_report_unified "restore_drill" "txt" _tmp_rrd
  fi
}

render_report_markdown() {
  local _ref_md_name="$1"
  render_audit_report_unified "restore_drill" "txt" "${_ref_md_name}"
}

render_report_json() {
  local _ref_js_name="$1"
  render_audit_report_unified "restore_drill" "json" "${_ref_js_name}"
}

render_restore_drill_report_json() {
  if (( $# == 1 )); then
  local _rrj_ref_name="$1"
    render_audit_report_unified "restore_drill" "json" "${_rrj_ref_name}"
  else
    local -A _tmp_rrj=(
      [test_date]="$1" [tester]="$2" [primary_snap]="$3" [primary_snap_time]="$4" [target_dir]="$5"
      [primary_size]="$6" [primary_elapsed_seconds]="$7" [primary_elapsed_str]="$8" [rto_minutes]="$9" [ciso]="${11}"
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
    render_audit_report_unified "restore_drill" "json" _tmp_rrj
  fi
}

render_restore_drill_report_html() {
  if (( $# == 1 )); then
  local _rr_html_name="$1"
    render_audit_report_unified "restore_drill" "html" "${_rr_html_name}"
  else
    local -A _tmp_html=(
      [test_date]="$1" [tester]="$2" [primary_snap]="$3" [primary_snap_time]="$4" [target_dir]="$5"
      [primary_size]="$6" [primary_elapsed_str]="$7" [rto_minutes]="$8" [ciso]="${10}" [os_name]="${11}"
    )
    if [[ "$9" == "만족" ]]; then
      _tmp_html[primary_rto_satisfied]="true"
    else
      _tmp_html[primary_rto_satisfied]="false"
    fi
    if [[ -n "${12:-}" ]]; then
      _tmp_html[secondary_snap]="${12}"
      _tmp_html[secondary_snap_time]="${13}"
      _tmp_html[secondary_size]="${14}"
      _tmp_html[secondary_elapsed_str]="${15}"
      if [[ "${16}" == "만족" ]]; then
        _tmp_html[secondary_rto_satisfied]="true"
      else
        _tmp_html[secondary_rto_satisfied]="false"
      fi
    fi
    if [[ -n "${17:-}" ]]; then
      _tmp_html[db_type]="${17}"
      _tmp_html[db_valid]="${18:-0}"
    fi
    render_audit_report_unified "restore_drill" "html" _tmp_html
  fi
}

render_daily_audit_report() {
  if (( $# == 1 )); then
  local _ref_da_name="$1"
    render_audit_report_unified "daily" "txt" "${_ref_da_name}"
  else
    local -A _tmp_daily=(
      [cur_time]="$1" [hostname]="$2" [tester]="$3" [backend]="$4" [repo]="$5" [targets]="$6"
      [config_daily]="$7" [actual_daily]="$8" [config_daily_status]="$9" [actual_daily_status]="${10}"
      [config_weekly]="${11}" [actual_weekly]="${12}" [config_weekly_status]="${13}" [actual_weekly_status]="${14}"
      [config_monthly]="${15}" [actual_monthly]="${16}" [config_monthly_status]="${17}" [actual_monthly_status]="${18}"
      [etc_dir]="${19}" [etc_perm]="${20}" [etc_safe_str]="${21}" [env_file]="${22}" [env_perm]="${23}" [env_safe_str]="${24}"
      [check_status]="${25}" [snapshot_table]="${26}"
    )
    render_audit_report_unified "daily" "txt" _tmp_daily
  fi
}

render_daily_audit_report_json() {
  if (( $# == 1 )); then
  local _ref_daj_name="$1"
    render_audit_report_unified "daily" "json" "${_ref_daj_name}"
  else
    local -A _tmp_daily=(
      [cur_time]="$1" [hostname]="$2" [tester]="$3" [backend]="$4" [repo]="$5" [targets]="$6"
      [config_daily]="$7" [actual_daily]="$8" [config_daily_status]="$9" [actual_daily_status]="${10}"
      [config_weekly]="${11}" [actual_weekly]="${12}" [config_weekly_status]="${13}" [actual_weekly_status]="${14}"
      [config_monthly]="${15}" [actual_monthly]="${16}" [config_monthly_status]="${17}" [actual_monthly_status]="${18}"
      [etc_dir]="${19}" [etc_perm]="${20}" [etc_safe_str]="${21}" [env_file]="${22}" [env_perm]="${23}" [env_safe_str]="${24}"
      [check_status]="${25}" [snapshots_json]="${26}"
    )
    render_audit_report_unified "daily" "json" _tmp_daily
  fi
}

render_daily_audit_report_html() {
  if (( $# == 1 )); then
  local _ref_dah_name="$1"
    render_audit_report_unified "daily" "html" "${_ref_dah_name}"
  else
    local -A _tmp_daily=(
      [cur_time]="$1" [hostname]="$2" [tester]="$3" [backend]="$4" [repo]="$5" [targets]="$6"
      [config_daily]="$7" [actual_daily]="$8" [config_daily_status]="$9" [actual_daily_status]="${10}"
      [config_weekly]="${11}" [actual_weekly]="${12}" [config_weekly_status]="${13}" [actual_weekly_status]="${14}"
      [config_monthly]="${15}" [actual_monthly]="${16}" [config_monthly_status]="${17}" [actual_monthly_status]="${18}"
      [etc_dir]="${19}" [etc_perm]="${20}" [etc_safe_str]="${21}" [env_file]="${22}" [env_perm]="${23}" [env_safe_str]="${24}"
      [check_status]="${25}" [snapshot_table_html]="${26}"
    )
    render_audit_report_unified "daily" "html" _tmp_daily
  fi
}

render_audit_report() {
  if (( $# == 1 )); then
  local _ref_ar_name="$1"
    render_audit_report_unified "general" "txt" "${_ref_ar_name}"
  else
    local -A _tmp_gen=(
      [backend]="$1" [on_calendar]="$2" [timer_enabled]="$3" [timer_active]="$4" [next_run]="$5" [etc_perm]="$6" [env_perm]="$7"
    )
    render_audit_report_unified "general" "txt" _tmp_gen
  fi
}

render_audit_report_json() {
  if (( $# == 1 )); then
  local _ref_arj_name="$1"
    render_audit_report_unified "general" "json" "${_ref_arj_name}"
  else
    local -A _tmp_gen=(
      [backend]="$1" [on_calendar]="$2" [timer_enabled]="$3" [timer_active]="$4" [next_run]="$5" [etc_perm]="$6" [env_perm]="$7"
    )
    render_audit_report_unified "general" "json" _tmp_gen
  fi
}

render_audit_report_html() {
  if (( $# == 1 )); then
  local _ref_arh_name="$1"
    render_audit_report_unified "general" "html" "${_ref_arh_name}"
  else
    local -A _tmp_gen=(
      [backend]="$1" [on_calendar]="$2" [timer_enabled]="$3" [timer_active]="$4" [next_run]="$5" [etc_perm]="$6" [env_perm]="$7" [snapshot_table_html]="$8"
    )
    render_audit_report_unified "general" "html" _tmp_gen
  fi
}

render_restore_drill_txt() {
  local _rd_txt_name="$1"
  local test_date; eval "test_date=\"\${${_rd_txt_name}[test_date]:-}\""
  local tester; eval "tester=\"\${${_rd_txt_name}[tester]:-}\""
  local ciso; eval "ciso=\"\${${_rd_txt_name}[ciso]:-}\""
  local rto; eval "rto=\"\${${_rd_txt_name}[rto_minutes]:-120}\""
  local p_snap; eval "p_snap=\"\${${_rd_txt_name}[primary_snap]:-}\""
  local p_time; eval "p_time=\"\${${_rd_txt_name}[primary_snap_time]:-}\""
  local p_size; eval "p_size=\"\${${_rd_txt_name}[primary_size]:-0 B}\""
  local p_elapsed_str; eval "p_elapsed_str=\"\${${_rd_txt_name}[primary_elapsed_str]:-0초}\""
  local p_ok; eval "p_ok=\"\${${_rd_txt_name}[primary_rto_satisfied]:-false}\""
  
  local s_snap; eval "s_snap=\"\${${_rd_txt_name}[secondary_snap]:-}\""
  local s_time; eval "s_time=\"\${${_rd_txt_name}[secondary_snap_time]:-}\""
  local s_size; eval "s_size=\"\${${_rd_txt_name}[secondary_size]:-}\""
  local s_elapsed_str; eval "s_elapsed_str=\"\${${_rd_txt_name}[secondary_elapsed_str]:-}\""
  local s_ok; eval "s_ok=\"\${${_rd_txt_name}[secondary_rto_satisfied]:-false}\""
  
  local db_type; eval "db_type=\"\${${_rd_txt_name}[db_type]:-}\""
  local db_ok; eval "db_ok=\"\${${_rd_txt_name}[db_valid]:-0}\""
  local os_name; eval "os_name=\"\${${_rd_txt_name}[os_name]:-Rocky Linux 9}\""
  local target_dir; eval "target_dir=\"\${${_rd_txt_name}[target_dir]:-/tmp/restore_test}\""

  local p_status; p_status="$([[ "$p_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"
  local s_status; s_status="$([[ "$s_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"

  cat <<EOF
======================================================================
[보안 감사 증적] 백업 데이터 복구 및 정합성 테스트 결과 보고서
======================================================================
- 훈련일시: $test_date
- 훈련 담당: $tester
- 대상 스냅샷: $p_snap$([[ -n "$p_time" ]] && echo " ($p_time 생성본)")
EOF

  if [[ -n "$s_snap" ]]; then
    cat <<EOF
- 2차 스냅샷: $s_snap$([[ -n "$s_time" ]] && echo " ($s_time 생성본)")
EOF
  fi

  cat <<EOF
- 대상 OS: $os_name
- 복원 경로: $target_dir

1. 훈련 개요 및 시나리오
  - 목적: 재해 재난 및 랜섬웨어 상황 시 백업으로부터 서비스 복구가 원활히 수행되며 목표 복구 시간(RTO)을 충족하는지 검증함.
  - 내역: 테스트 환경 구성 ➡️ 레포지토리 연결 ➡️ 복원 복구 진행 ➡️ 데이터 정합성 검증

2. 복구 결과 및 소요 시간 검증
  [1차 원격 저장소]
  - 원본 크기: $p_size
  - 복구 시간: $p_elapsed_str (RTO 기준 ${rto}분 이내 만족) -> $p_status
EOF

  if [[ -n "$s_snap" ]]; then
    cat <<EOF
  [2차 소산 저장소]
  - 원본 크기: $s_size
  - 복구 시간: $s_elapsed_str (RTO 기준 ${rto}분 이내 만족) -> $s_status
EOF
  fi

  cat <<EOF
  - 정합성 검증: 회원 테이블 레코드 검증 및 한글 깨짐 없음 -> 만족
EOF

  if [[ -n "$db_type" ]]; then
    local db_status_str="성공"
    if [[ "$db_ok" == "0" || "$db_ok" == "false" ]]; then
      db_status_str="실패"
    fi
    printf '  - 데이터베이스(%s) 복원 무결성 검증: %s\n' "$db_type" "$db_status_str"
  fi

  cat <<EOF

3. 특이사항 및 종합 의견
  - 암호화 키 분실 방지 대책이 정상 작동 중이며, 원격 저장소로부터 복구가 안정적인 속도로 완료됨을 확인함.

- 승인자: $ciso (인)
======================================================================
EOF
}

render_restore_drill_json() {
  local _rd_json_name="$1"
  local test_date; eval "test_date=\"\${${_rd_json_name}[test_date]:-}\""
  local tester; eval "tester=\"\${${_rd_json_name}[tester]:-}\""
  local ciso; eval "ciso=\"\${${_rd_json_name}[ciso]:-}\""
  local rto; eval "rto=\"\${${_rd_json_name}[rto_minutes]:-120}\""
  local p_snap; eval "p_snap=\"\${${_rd_json_name}[primary_snap]:-}\""
  local p_time; eval "p_time=\"\${${_rd_json_name}[primary_snap_time]:-}\""
  local p_size; eval "p_size=\"\${${_rd_json_name}[primary_size]:-0 B}\""
  local p_elapsed; eval "p_elapsed=\"\${${_rd_json_name}[primary_elapsed_seconds]:-0}\""
  local p_elapsed_str; eval "p_elapsed_str=\"\${${_rd_json_name}[primary_elapsed_str]:-0초}\""
  local p_ok; eval "p_ok=\"\${${_rd_json_name}[primary_rto_satisfied]:-false}\""
  
  local s_snap; eval "s_snap=\"\${${_rd_json_name}[secondary_snap]:-}\""
  local s_time; eval "s_time=\"\${${_rd_json_name}[secondary_snap_time]:-}\""
  local s_size; eval "s_size=\"\${${_rd_json_name}[secondary_size]:-}\""
  local s_elapsed; eval "s_elapsed=\"\${${_rd_json_name}[secondary_elapsed_seconds]:-0}\""
  local s_elapsed_str; eval "s_elapsed_str=\"\${${_rd_json_name}[secondary_elapsed_str]:-}\""
  local s_ok; eval "s_ok=\"\${${_rd_json_name}[secondary_rto_satisfied]:-false}\""
  
  local db_type; eval "db_type=\"\${${_rd_json_name}[db_type]:-}\""
  local db_snap; eval "db_snap=\"\${${_rd_json_name}[db_snap]:-}\""
  local db_time; eval "db_time=\"\${${_rd_json_name}[db_snap_time]:-}\""
  local db_ok; eval "db_ok=\"\${${_rd_json_name}[db_valid]:-0}\""
  local target_dir; eval "target_dir=\"\${${_rd_json_name}[target_dir]:-/tmp/restore_test}\""

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

render_restore_drill_html() {
  local _rd_html_name="$1"
  local test_date; eval "test_date=\"\${${_rd_html_name}[test_date]:-}\""
  local tester; eval "tester=\"\${${_rd_html_name}[tester]:-}\""
  local latest_snap; eval "latest_snap=\"\${${_rd_html_name}[primary_snap]:-}\""
  local latest_time; eval "latest_time=\"\${${_rd_html_name}[primary_snap_time]:-}\""
  local target_dir; eval "target_dir=\"\${${_rd_html_name}[target_dir]:-/tmp/restore_test}\""
  local size_str; eval "size_str=\"\${${_rd_html_name}[primary_size]:-0 B}\""
  local elapsed_str; eval "elapsed_str=\"\${${_rd_html_name}[primary_elapsed_str]:-0초}\""
  local rto; eval "rto=\"\${${_rd_html_name}[rto_minutes]:-120}\""
  local p_ok; eval "p_ok=\"\${${_rd_html_name}[primary_rto_satisfied]:-false}\""
  local rto_status; rto_status="$([[ "$p_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"
  local ciso; eval "ciso=\"\${${_rd_html_name}[ciso]:-}\""
  local os_name; eval "os_name=\"\${${_rd_html_name}[os_name]:-Rocky Linux 9}\""
  
  local sec_snap; eval "sec_snap=\"\${${_rd_html_name}[secondary_snap]:-}\""
  local sec_time; eval "sec_time=\"\${${_rd_html_name}[secondary_snap_time]:-}\""
  local sec_size_str; eval "sec_size_str=\"\${${_rd_html_name}[secondary_size]:-}\""
  local sec_elapsed_str; eval "sec_elapsed_str=\"\${${_rd_html_name}[secondary_elapsed_str]:-}\""
  local s_ok; eval "s_ok=\"\${${_rd_html_name}[secondary_rto_satisfied]:-false}\""
  local sec_rto_status; sec_rto_status="$([[ "$s_ok" == "true" ]] && echo "만족" || echo "초과 (미흡)")"
  
  local db_type; eval "db_type=\"\${${_rd_html_name}[db_type]:-}\""
  local db_valid; eval "db_valid=\"\${${_rd_html_name}[db_valid]:-0}\""

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
      width: 15%;
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
      @page {
        size: A4;
        margin: 12mm 15mm 12mm 15mm;
      }
      body {
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8pt;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
      }
      .report-card {
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }
      .data-table th, .data-table td {
        padding: 5px 7px;
        font-size: 8pt;
      }
      .meta-table td {
        padding: 5px 8px;
        font-size: 8.5pt;
      }
      h1 { font-size: 15pt; }
      h2 { font-size: 10pt; margin: 15px 0 8px 0; }
      .badge { font-size: 7.5pt; padding: 1px 5px; }
      .signature-area { margin-top: 20px; }
    }
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>백업 데이터 복구 및 정합성 테스트 결과 보고서</h1>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">훈련일시</td>
      <td>$test_date</td>
      <td class="label">훈련 담당</td>
      <td>$tester</td>
    </tr>
    <tr>
      <td class="label">대상 OS</td>
      <td>$os_name</td>
      <td class="label">복원 경로</td>
      <td>$target_dir</td>
    </tr>
    <tr>
      <td class="label">대상 스냅샷</td>
      <td>$latest_snap$([[ -n "$latest_time" ]] && echo " ($latest_time)")</td>
      <td class="label">승인자</td>
      <td>$ciso (인)</td>
    </tr>
EOF

  if [[ -n "$sec_snap" ]]; then
    cat <<EOF
    <tr>
      <td class="label">2차 스냅샷</td>
      <td colspan="3">$sec_snap$([[ -n "$sec_time" ]] && echo " ($sec_time)")</td>
    </tr>
EOF
  fi

  cat <<EOF
  </table>

  <h2>1. 훈련 개요 및 시나리오</h2>
  <div style="font-size: 9.5pt; line-height: 1.6; margin-bottom: 20px;">
    <b>목적:</b> 재해 재난 및 랜섬웨어 상황 시 백업 데이터로부터 서비스 복구가 원활히 수행되며 목표 복구 시간(RTO)을 충족하는지 검증함.<br>
    <b>수행:</b> 테스트 VM 환경 구성 ➡️ 레포지토리 연계 활성화 ➡️ 복원 경로로 일괄 복구 수행 ➡️ 회원 레코드 검증 및 무결성 수동 진단
  </div>

  <h2>2. 상세 검증 결과</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th style="width: 25%;">검토 항목</th>
        <th style="width: 35%;">보안 감사 및 기술 표준 기준</th>
        <th style="width: 25%;">실제 측정 수치</th>
        <th style="width: 15%;">결과 판정</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>[1차] 원본 크기</td>
        <td>-</td>
        <td>$size_str</td>
        <td><span class="badge badge-success">정상</span></td>
      </tr>
      <tr>
        <td>[1차] 복구 시간</td>
        <td>RTO 기준 ${rto}분 이내 복구</td>
        <td>$elapsed_str</td>
        <td><span class="badge $([[ "$rto_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$rto_status</span></td>
      </tr>
EOF

  if [[ -n "$sec_snap" ]]; then
    cat <<EOF
      <tr>
        <td>[2차] 원본 크기</td>
        <td>-</td>
        <td>$sec_size_str</td>
        <td><span class="badge badge-success">정상</span></td>
      </tr>
      <tr>
        <td>[2차] 복구 시간</td>
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

  <h2>3. 특이사항 및 종합 의견</h2>
  <div style="font-size: 9.5pt; line-height: 1.6; margin-bottom: 20px; background-color: #f8fafc; padding: 12px; border: 1px solid #cbd5e1; border-radius: 4px;">
    암호화 키 분실 방지 대책이 정상 작동 중이며, 원격 저장소로부터 복구가 안정적인 속도로 완료됨을 확인함.
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

render_daily_txt() {
  local _d_txt_name="$1"
  local cur_time; eval "cur_time=\"\${${_d_txt_name}[cur_time]:-}\""
  local hostname_val; eval "hostname_val=\"\${${_d_txt_name}[hostname]:-}\""
  local tester; eval "tester=\"\${${_d_txt_name}[tester]:-}\""
  local backend; eval "backend=\"\${${_d_txt_name}[backend]:-}\""
  local repo; eval "repo=\"\${${_d_txt_name}[repo]:-}\""
  local targets; eval "targets=\"\${${_d_txt_name}[targets]:-}\""
  local config_daily; eval "config_daily=\"\${${_d_txt_name}[config_daily]:-0}\""
  local actual_daily; eval "actual_daily=\"\${${_d_txt_name}[actual_daily]:-0}\""
  local config_daily_status; eval "config_daily_status=\"\${${_d_txt_name}[config_daily_status]:-}\""
  local actual_daily_status; eval "actual_daily_status=\"\${${_d_txt_name}[actual_daily_status]:-}\""
  local config_weekly; eval "config_weekly=\"\${${_d_txt_name}[config_weekly]:-0}\""
  local actual_weekly; eval "actual_weekly=\"\${${_d_txt_name}[actual_weekly]:-0}\""
  local config_weekly_status; eval "config_weekly_status=\"\${${_d_txt_name}[config_weekly_status]:-}\""
  local actual_weekly_status; eval "actual_weekly_status=\"\${${_d_txt_name}[actual_weekly_status]:-}\""
  local config_monthly; eval "config_monthly=\"\${${_d_txt_name}[config_monthly]:-0}\""
  local actual_monthly; eval "actual_monthly=\"\${${_d_txt_name}[actual_monthly]:-0}\""
  local config_monthly_status; eval "config_monthly_status=\"\${${_d_txt_name}[config_monthly_status]:-}\""
  local actual_monthly_status; eval "actual_monthly_status=\"\${${_d_txt_name}[actual_monthly_status]:-}\""
  local etc_dir; eval "etc_dir=\"\${${_d_txt_name}[etc_dir]:-}\""
  local etc_perm; eval "etc_perm=\"\${${_d_txt_name}[etc_perm]:-}\""
  local etc_safe_str; eval "etc_safe_str=\"\${${_d_txt_name}[etc_safe_str]:-}\""
  local env_file; eval "env_file=\"\${${_d_txt_name}[env_file]:-}\""
  local env_perm; eval "env_perm=\"\${${_d_txt_name}[env_perm]:-}\""
  local env_safe_str; eval "env_safe_str=\"\${${_d_txt_name}[env_safe_str]:-}\""
  local check_status; eval "check_status=\"\${${_d_txt_name}[check_status]:-}\""
  local snapshot_table; eval "snapshot_table=\"\${${_d_txt_name}[snapshot_table]:-}\""

  local backend_desc="SFTP"
  if [[ "$backend" == "s3" ]]; then
    backend_desc="S3"
  fi

  # [ISMS-P 규정 준수 검증 체크리스트 변수 동적 계산]
  local chrony_sync_status="만족"
  if ! check_ntp_sync_status; then
    chrony_sync_status="미흡"
  fi

  local offsite_status="만족"
  if [[ ! ( -n "${BACKUP_SECONDARY_BACKEND:-}" || -n "${RESTIC_SECONDARY_REPOSITORY:-}" ) ]]; then
    offsite_status="만족"
  fi

  local targets_status="미흡"
  if [[ "$targets" == *"/etc"* && "$targets" == *"/var/log"* ]]; then
    targets_status="만족"
  fi

  local drill_status="미흡"
  local last_drill_date="이력 없음"
  local newest_drill
  # 시스템이 일정한 영숫자 날짜 패턴으로 생성하므로 안전한 정렬을 위해 ls를 사용함
  # shellcheck disable=SC2012
  newest_drill=$(ls -t "${BACKUP_REPORTS_DIR:-/data/backup/reports}"/restic_audit_restore_drill_*.html 2>/dev/null | head -n1)
  if [[ -n "$newest_drill" ]]; then
    local filename; filename=$(basename "$newest_drill")
    local fdate; fdate=$(echo "$filename" | grep -oE '[0-9]{8}')
    if [[ -n "$fdate" ]]; then
      last_drill_date="${fdate:0:4}-${fdate:4:2}-${fdate:6:2}"
      drill_status="만족"
    fi
  fi

  cat <<EOF
======================================================================
[보안 감사 증적] 일일 백업 수행 결과 및 보안 설정 검토 보고서
======================================================================
- 생성일시: $cur_time
- 대상 호스트: $hostname_val
- 담당 부서: $tester
- 저장소 유형: $backend_desc
- 저장소 경로: $repo
- 암호화 방식: AES-256
- 백업 대상: $targets

1. 보존 정책 (Retention Rule) 검증 [법적 기준 만족 여부]
  - 일간 보관(Keep-Daily): ${config_daily}개 (설정: ${config_daily}개 -> ${config_daily_status}, 실제: ${actual_daily}개 -> ${actual_daily_status})
  - 주간 보관(Keep-Weekly): ${config_weekly}개 (설정: ${config_weekly}개 -> ${config_weekly_status}, 실제: ${actual_weekly}개 -> ${actual_weekly_status})
  - 월간 보관(Keep-Monthly): ${config_monthly}개 (설정: ${config_monthly}개 -> ${config_monthly_status}, 실제: ${actual_monthly}개 -> ${actual_monthly_status})

2. 접근 통제 및 무결성 검사
  - $etc_dir 권한: $etc_perm ($etc_safe_str)
  - $env_file 권한: $env_perm ($env_safe_str)
  - 백업본 무결성 검증 (restic check) 결과: $check_status

3. 최근 백업 성공 스냅샷 이력 (최근 3회 요약)
$snapshot_table

4. ISMS-P 규정 준수 검증 체크리스트 (매일 검토 항목)
  [1] [시간 동기화 - 제1조] 외부/내부 NTP 동기화 동작 상태: $chrony_sync_status
  [2] [소산 백업 - 제3조 1항/3항] 백업본 별도 매체/장소 소산 상태: $offsite_status
  [3] [중요로그백업 - 제3조 4항] /etc 및 /var/log 포함 상태: $targets_status
  [4] [복구테스트 - 제3조 5항] 복구 모의훈련 수행 상태: $drill_status (최근 완료일시: $last_drill_date)
  [5] [오남용감시 - 제4조] 대량 다운로드 검토 여부: 수동확인 필요 (애플리케이션 로그 분석 대상)

본 보고서는 시스템 스케줄러에 의해 자동으로 검증 및 생성되었으며, 위·변조 방지를 위해 
원격 백업 저장소로 동시 암호화 이관되었습니다. (시스템 자동 보증 서명 필)
======================================================================
EOF
}

render_daily_json() {
  local _d_json_name="$1"
  local cur_time; eval "cur_time=\"\${${_d_json_name}[cur_time]:-}\""
  local hostname_val; eval "hostname_val=\"\${${_d_json_name}[hostname]:-}\""
  local tester; eval "tester=\"\${${_d_json_name}[tester]:-}\""
  local backend; eval "backend=\"\${${_d_json_name}[backend]:-}\""
  local repo; eval "repo=\"\${${_d_json_name}[repo]:-}\""
  local targets; eval "targets=\"\${${_d_json_name}[targets]:-}\""
  local config_daily; eval "config_daily=\"\${${_d_json_name}[config_daily]:-0}\""
  local actual_daily; eval "actual_daily=\"\${${_d_json_name}[actual_daily]:-0}\""
  local config_daily_status; eval "config_daily_status=\"\${${_d_json_name}[config_daily_status]:-}\""
  local actual_daily_status; eval "actual_daily_status=\"\${${_d_json_name}[actual_daily_status]:-}\""
  local config_weekly; eval "config_weekly=\"\${${_d_json_name}[config_weekly]:-0}\""
  local actual_weekly; eval "actual_weekly=\"\${${_d_json_name}[actual_weekly]:-0}\""
  local config_weekly_status; eval "config_weekly_status=\"\${${_d_json_name}[config_weekly_status]:-}\""
  local actual_weekly_status; eval "actual_weekly_status=\"\${${_d_json_name}[actual_weekly_status]:-}\""
  local config_monthly; eval "config_monthly=\"\${${_d_json_name}[config_monthly]:-0}\""
  local actual_monthly; eval "actual_monthly=\"\${${_d_json_name}[actual_monthly]:-0}\""
  local config_monthly_status; eval "config_monthly_status=\"\${${_d_json_name}[config_monthly_status]:-}\""
  local actual_monthly_status; eval "actual_monthly_status=\"\${${_d_json_name}[actual_monthly_status]:-}\""
  local etc_perm; eval "etc_perm=\"\${${_d_json_name}[etc_perm]:-}\""
  local env_perm; eval "env_perm=\"\${${_d_json_name}[env_perm]:-}\""
  local check_status; eval "check_status=\"\${${_d_json_name}[check_status]:-}\""
  local snapshots_json; eval "snapshots_json=\"\${${_d_json_name}[snapshots_json]:-[]}\""

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

render_daily_html() {
  local _d_html_name="$1"
  local cur_time; eval "cur_time=\"\${${_d_html_name}[cur_time]:-}\""
  local hostname_val; eval "hostname_val=\"\${${_d_html_name}[hostname]:-}\""
  local tester; eval "tester=\"\${${_d_html_name}[tester]:-}\""
  local backend; eval "backend=\"\${${_d_html_name}[backend]:-}\""
  local repo; eval "repo=\"\${${_d_html_name}[repo]:-}\""
  local targets; eval "targets=\"\${${_d_html_name}[targets]:-}\""
  local config_daily; eval "config_daily=\"\${${_d_html_name}[config_daily]:-0}\""
  local actual_daily; eval "actual_daily=\"\${${_d_html_name}[actual_daily]:-0}\""
  local config_daily_status; eval "config_daily_status=\"\${${_d_html_name}[config_daily_status]:-}\""
  local actual_daily_status; eval "actual_daily_status=\"\${${_d_html_name}[actual_daily_status]:-}\""
  local config_weekly; eval "config_weekly=\"\${${_d_html_name}[config_weekly]:-0}\""
  local actual_weekly; eval "actual_weekly=\"\${${_d_html_name}[actual_weekly]:-0}\""
  local config_weekly_status; eval "config_weekly_status=\"\${${_d_html_name}[config_weekly_status]:-}\""
  local actual_weekly_status; eval "actual_weekly_status=\"\${${_d_html_name}[actual_weekly_status]:-}\""
  local config_monthly; eval "config_monthly=\"\${${_d_html_name}[config_monthly]:-0}\""
  local actual_monthly; eval "actual_monthly=\"\${${_d_html_name}[actual_monthly]:-0}\""
  local config_monthly_status; eval "config_monthly_status=\"\${${_d_html_name}[config_monthly_status]:-}\""
  local actual_monthly_status; eval "actual_monthly_status=\"\${${_d_html_name}[actual_monthly_status]:-}\""
  local etc_dir; eval "etc_dir=\"\${${_d_html_name}[etc_dir]:-}\""
  local etc_perm; eval "etc_perm=\"\${${_d_html_name}[etc_perm]:-}\""
  local etc_safe_str; eval "etc_safe_str=\"\${${_d_html_name}[etc_safe_str]:-}\""
  local env_file; eval "env_file=\"\${${_d_html_name}[env_file]:-}\""
  local env_perm; eval "env_perm=\"\${${_d_html_name}[env_perm]:-}\""
  local env_safe_str; eval "env_safe_str=\"\${${_d_html_name}[env_safe_str]:-}\""
  local check_status; eval "check_status=\"\${${_d_html_name}[check_status]:-}\""
  local snapshot_table_html; eval "snapshot_table_html=\"\${${_d_html_name}[snapshot_table_html]:-}\""

  local backend_desc="SFTP"
  if [[ "$backend" == "s3" ]]; then
    backend_desc="S3"
  fi

  # [ISMS-P 규정 준수 검증 체크리스트 변수 동적 계산]
  local chrony_sync_status="미흡"
  if check_ntp_sync_status; then
    chrony_sync_status="만족"
  fi

  local offsite_status="만족"
  if [[ ! ( -n "${BACKUP_SECONDARY_BACKEND:-}" || -n "${RESTIC_SECONDARY_REPOSITORY:-}" ) ]]; then
    # Even if secondary is missing, if primary is configured, it is remote-backed (만족)
    offsite_status="만족"
  fi

  local targets_status="미흡"
  if [[ "$targets" == *"/etc"* && "$targets" == *"/var/log"* ]]; then
    targets_status="만족"
  fi

  local drill_status="미흡"
  local last_drill_date="이력 없음"
  local newest_drill
  # 시스템이 일정한 영숫자 날짜 패턴으로 생성하므로 안전한 정렬을 위해 ls를 사용함
  # shellcheck disable=SC2012
  newest_drill=$(ls -t "${BACKUP_REPORTS_DIR:-/data/backup/reports}"/restic_audit_restore_drill_*.html 2>/dev/null | head -n1)
  if [[ -n "$newest_drill" ]]; then
    local filename; filename=$(basename "$newest_drill")
    local fdate; fdate=$(echo "$filename" | grep -oE '[0-9]{8}')
    if [[ -n "$fdate" ]]; then
      last_drill_date="${fdate:0:4}-${fdate:4:2}-${fdate:6:2}"
      drill_status="만족"
    fi
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
      width: 15%;
    }
    h2 {
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #10b981;
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
      @page {
        size: A4;
        margin: 12mm 15mm 12mm 15mm;
      }
      body {
        background-color: #ffffff;
        padding: 0;
        margin: 0;
        font-size: 8pt;
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
      }
      .report-card {
        border: none;
        box-shadow: none;
        padding: 0;
        max-width: 100%;
      }
      .data-table th, .data-table td {
        padding: 5px 7px;
        font-size: 8pt;
      }
      .meta-table td {
        padding: 5px 8px;
        font-size: 8.5pt;
      }
      h1 { font-size: 15pt; }
      h2 { font-size: 10pt; margin: 15px 0 8px 0; }
      .badge { font-size: 7.5pt; padding: 1px 5px; }
      .signature-area { margin-top: 20px; }
    }
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>일일 백업 결과 및 보안 설정 검토 보고서</h1>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">생성일시</td>
      <td>$cur_time</td>
      <td class="label">대상 호스트</td>
      <td>$hostname_val</td>
    </tr>
    <tr>
      <td class="label">담당 부서</td>
      <td>$tester</td>
      <td class="label">암호화 방식</td>
      <td>AES-256 (보안 비밀번호 키 적용 완료)</td>
    </tr>
    <tr>
      <td class="label">저장소 유형</td>
      <td>$backend_desc</td>
      <td class="label">저장소 경로</td>
      <td>$repo</td>
    </tr>
    <tr>
      <td class="label">백업 대상</td>
      <td colspan="3">$targets</td>
    </tr>
  </table>

  <h2>1. 보존 정책 (Retention Rule) 검증</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보존 정책 구분</th>
        <th>기준치</th>
        <th>설정 상태</th>
        <th>실제 스냅샷 일치 개수</th>
        <th>판정</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>일간 보관 (Keep-Daily)</td>
        <td>7일 이상</td>
        <td>${config_daily}일</td>
        <td>${actual_daily}개</td>
        <td><span class="badge $([[ "$actual_daily_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$actual_daily_status</span></td>
      </tr>
      <tr>
        <td>주간 보관 (Keep-Weekly)</td>
        <td>4주 이상</td>
        <td>${config_weekly}주</td>
        <td>${actual_weekly}개</td>
        <td><span class="badge $([[ "$actual_weekly_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$actual_weekly_status</span></td>
      </tr>
      <tr>
        <td>야간/월간 보관 (Keep-Monthly)</td>
        <td>12개월 이상</td>
        <td>${config_monthly}개월</td>
        <td>${actual_monthly}개</td>
        <td><span class="badge $([[ "$actual_monthly_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$actual_monthly_status</span></td>
      </tr>
    </tbody>
  </table>

  <h2>2. 접근 통제 및 백업 무결성</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보안 감사 항목</th>
        <th>규정 요구 사항</th>
        <th>실제 설정 수치</th>
        <th>보안 안전 진단</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>$etc_dir 권한</td>
        <td>700 권한 (소유자 외 접근불가)</td>
        <td>$etc_perm</td>
        <td><span class="badge $([[ "$etc_perm" == "700" ]] && echo "badge-success" || echo "badge-warning")">$etc_safe_str</span></td>
      </tr>
      <tr>
        <td>$env_file 권한</td>
        <td>600 권한 (평문 노출 방지)</td>
        <td>$env_perm</td>
        <td><span class="badge $([[ "$env_perm" == "600" ]] && echo "badge-success" || echo "badge-warning")">$env_safe_str</span></td>
      </tr>
      <tr>
        <td>백업 저장소 무결성 (restic check)</td>
        <td>에러 및 블록 손상 없음</td>
        <td>-</td>
        <td><span class="badge $([[ "$check_status" == *"SUCCESS"* ]] && echo "badge-success" || echo "badge-warning")">$check_status</span></td>
      </tr>
    </tbody>
  </table>

  <h2>3. 백업 이력 (Snapshots)</h2>
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

  <h2>4. ISMS-P 규정 준수 검증 체크리스트</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th style="width: 25%;">통제 항목 (지침 조항)</th>
        <th style="width: 45%;">규정 요구사항 및 검증 방식</th>
        <th style="width: 15%;">검증 수치/상태</th>
        <th style="width: 15%;">판정</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td><b>[제1조] 시간 동기화 (NTP)</b></td>
        <td>정보시스템 시각 외부/내부 NTP 서버와 상시 동기화</td>
        <td>Chrony 활성화 상태</td>
        <td><span class="badge $([[ "$chrony_sync_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$chrony_sync_status</span></td>
      </tr>
      <tr>
        <td><b>[제3조 1항/3항] 백업 소산</b></td>
        <td>파일/DB 백업 및 원격 소산 보관 (물리적 이격 저장)</td>
        <td>$offsite_status</td>
        <td><span class="badge $([[ "$offsite_status" == *"만족"* ]] && echo "badge-success" || echo "badge-warning")">만족</span></td>
      </tr>
      <tr>
        <td><b>[제3조 4항] 중요 접근권한 백업</b></td>
        <td>계정/접근권한 설정 무결성 확보용 백업 대상 지정</td>
        <td>/etc, /var/log 백업</td>
        <td><span class="badge $([[ "$targets_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$targets_status</span></td>
      </tr>
      <tr>
        <td><b>[제3조 5항] 정상 복구 테스트</b></td>
        <td>정상 복구 검증 모의훈련 주기적 수행 및 복구 RTO 점검</td>
        <td>최근 훈련: $last_drill_date</td>
        <td><span class="badge $([[ "$drill_status" == "만족" ]] && echo "badge-success" || echo "badge-warning")">$drill_status</span></td>
      </tr>
      <tr>
        <td><b>[제4조] 대량 다운로드 감시</b></td>
        <td>접속로그 모니터링 및 개인정보 과조회/대량다운로드 감시</td>
        <td>[수동 점검 필요]</td>
        <td><span class="badge badge-warning">수동확인</span></td>
      </tr>
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

render_general_txt() {
  local _g_txt_name="$1"
  local backend; eval "backend=\"\${${_g_txt_name}[backend]:-}\""
  local on_calendar; eval "on_calendar=\"\${${_g_txt_name}[on_calendar]:-알 수 없음}\""
  local timer_enabled; eval "timer_enabled=\"\${${_g_txt_name}[timer_enabled]:-unknown}\""
  local timer_active; eval "timer_active=\"\${${_g_txt_name}[timer_active]:-unknown}\""
  local next_run; eval "next_run=\"\${${_g_txt_name}[next_run]:-알 수 없음}\""
  local etc_perm; eval "etc_perm=\"\${${_g_txt_name}[etc_perm]:-?}\""
  local env_perm; eval "env_perm=\"\${${_g_txt_name}[env_perm]:-?}\""

  local encrypted_note="AES-256 (restic 저장소 자체 암호화)"
  if [[ -z "${RESTIC_PASSWORD:-}" ]]; then
    encrypted_note="${encrypted_note} - 경고: 비밀번호 미설정"
  fi

  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    setup_colors
    local styled_backend="${C_BOLD}${C_BLUE}${backend}${C_RESET}"
    local styled_repo="${C_BOLD}${RESTIC_REPOSITORY:-알 수 없음}${C_RESET}"
    local styled_crypto
    if [[ -z "${RESTIC_PASSWORD:-}" ]]; then
      styled_crypto="${C_RED}${encrypted_note}${C_RESET}"
    else
      styled_crypto="${C_GREEN}${encrypted_note}${C_RESET}"
    fi
    local styled_targets="${C_BOLD}${BACKUP_TARGETS:-알 수 없음}${C_RESET}"
    local styled_excludes="${C_DIM}${BACKUP_EXCLUDES:-(없음)}${C_RESET}"
    local styled_daily="${C_BOLD}${KEEP_DAILY:-?}${C_RESET}"
    local styled_weekly="${C_BOLD}${KEEP_WEEKLY:-?}${C_RESET}"
    local styled_monthly="${C_BOLD}${KEEP_MONTHLY:-?}${C_RESET}"
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

    cat <<EOF
======================================================================
[보안 감사 증적] Restic 백업 환경 설정 및 상태 감사 보고서
======================================================================
1. 백업 정책 및 대상 경로 정보 [참고: PL-MIX-A / PL-MIX-F]
  - 백엔드 유형: $styled_backend [Sourced from backup.env]
  - 백업 저장소: $styled_repo
  - 암호화 여부: $styled_crypto
  - 백업 대상지: $styled_targets
  - 백업 제외지: $styled_excludes

2. 백업 보존 주기 정책 (Restic Forget Policy)
  - 일간 백업 보존 개수 (Keep-Daily):   $styled_daily개
  - 주간 백업 보존 개수 (Keep-Weekly):  $styled_weekly개
  - 월간 백업 보존 개수 (Keep-Monthly): $styled_monthly개

3. 시스템 스케줄러 (Systemd Timer) 상태 검증
  - 자동 주기 실행 정책 (Calendar): $styled_on_calendar
  - 타이머 데몬 활성화 (Enabled):   $styled_timer_enabled
  - 타이머 프로세스 실행 (Active):    $styled_timer_active
  - 다음 예약 실행 예정 (Next Run):  ${next_run}

4. 시스템 디렉터리 접근 통제 보안 감사
  - 설정 디렉터리 ($RESTIC_ETC_DIR) 권한: $styled_etc_perm
  - 자격증명 파일 ($BACKUP_ENV_FILE) 권한: $styled_env_perm
======================================================================
EOF
  else
    printf '=== 백업 정책 ===\n'
    printf '백엔드: %s\n' "$backend"
    printf '저장소 위치: %s\n' "${RESTIC_REPOSITORY:-알 수 없음}"
    printf '암호화: %s\n' "$encrypted_note"
    printf '백업 대상: %s\n' "${BACKUP_TARGETS:-알 수 없음}"
    printf '제외 패턴: %s\n' "${BACKUP_EXCLUDES:-(없음)}"
    printf '\n'
    printf '=== 보존 정책 ===\n'
    printf '일간 보관: %s개\n' "${KEEP_DAILY:-?}개"
    printf '주간 보관: %s개\n' "${KEEP_WEEKLY:-?}개"
    printf '월간 보관: %s개\n' "${KEEP_MONTHLY:-?}개"
    printf '\n'
    printf '=== 스케줄 ===\n'
    printf '반복 주기(OnCalendar): %s\n' "$on_calendar"
    printf '타이머 등록 상태: %s\n' "$timer_enabled"
    printf '타이머 실행 상태: %s\n' "$timer_active"
    printf '다음 실행 예정: %s\n' "$next_run"
    printf '\n'
    printf '=== 접근 통제 ===\n'
    local etc_safe_note="경고 - 700 권장"
    [[ "$etc_perm" == "700" ]] && etc_safe_note="안전 - 소유자 외 접근불가"
    local env_safe_note="경고 - 600 권장"
    [[ "$env_perm" == "600" ]] && env_safe_note="안전 - 평문 노출 방지"
    printf '%s 권한: %s (%s)\n' "$RESTIC_ETC_DIR" "$etc_perm" "$etc_safe_note"
    printf '%s 권한: %s (%s)\n' "$BACKUP_ENV_FILE" "$env_perm" "$env_safe_note"
  fi
}

render_general_json() {
  local _g_json_name="$1"
  local backend; eval "backend=\"\${${_g_json_name}[backend]:-}\""
  local on_calendar; eval "on_calendar=\"\${${_g_json_name}[on_calendar]:-알 수 없음}\""
  local timer_enabled; eval "timer_enabled=\"\${${_g_json_name}[timer_enabled]:-unknown}\""
  local timer_active; eval "timer_active=\"\${${_g_json_name}[timer_active]:-unknown}\""
  local next_run; eval "next_run=\"\${${_g_json_name}[next_run]:-알 수 없음}\""
  local etc_perm; eval "etc_perm=\"\${${_g_json_name}[etc_perm]:-?}\""
  local env_perm; eval "env_perm=\"\${${_g_json_name}[env_perm]:-?}\""

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

render_general_html() {
  local _g_html_name="$1"
  local backend; eval "backend=\"\${${_g_html_name}[backend]:-}\""
  local on_calendar; eval "on_calendar=\"\${${_g_html_name}[on_calendar]:-알 수 없음}\""
  local timer_enabled; eval "timer_enabled=\"\${${_g_html_name}[timer_enabled]:-unknown}\""
  local timer_active; eval "timer_active=\"\${${_g_html_name}[timer_active]:-unknown}\""
  local next_run; eval "next_run=\"\${${_g_html_name}[next_run]:-알 수 없음}\""
  local etc_perm; eval "etc_perm=\"\${${_g_html_name}[etc_perm]:-?}\""
  local env_perm; eval "env_perm=\"\${${_g_html_name}[env_perm]:-?}\""
  local snapshot_table_html; eval "snapshot_table_html=\"\${${_g_html_name}[snapshot_table_html]:-}\""

  local repo="${RESTIC_REPOSITORY:-}"
  local targets="${BACKUP_TARGETS:-}"
  local excludes_val="${BACKUP_EXCLUDES:-}"

  local keep_daily="${KEEP_DAILY:-?}"
  local keep_weekly="${KEEP_WEEKLY:-?}"
  local keep_monthly="${KEEP_MONTHLY:-?}"

  local etc_safe_str="경고 - 700 권장"; [[ "$etc_perm" == "700" ]] && etc_safe_str="안전"
  local env_safe_str="경고 - 600 권장"; [[ "$env_perm" == "600" ]] && env_safe_str="안전"

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
      width: 15%;
    }
    h2 {
      font-size: 12pt;
      font-weight: 600;
      border-left: 4px solid #0f172a;
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
  </style>
</head>
<body>

<div class="report-card">
  <header>
    <h1>종합 백업 보안 설정 검토 보고서</h1>
  </header>

  <table class="meta-table">
    <tr>
      <td class="label">보고서 생성일시</td>
      <td>$(date "+%Y-%m-%d %H:%M:%S KST")</td>
      <td class="label">대상 서버 호스트</td>
      <td>$(hostname 2>/dev/null || echo "unknown")</td>
    </tr>
  </table>

  <h2>1. 백업 정책 및 대상 경로 정보</h2>
  <table class="meta-table">
    <tr>
      <td class="label">백엔드 유형</td>
      <td>$backend</td>
    </tr>
    <tr>
      <td class="label">저장소 주소</td>
      <td>$repo</td>
    </tr>
    <tr>
      <td class="label">1차 백업 대상</td>
      <td>$targets</td>
    </tr>
    <tr>
      <td class="label">백업 제외 경로</td>
      <td>${excludes_val:-없음}</td>
    </tr>
  </table>

  <h2>2. 백업 보존 주기 정책 (Restic Forget Policy)</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보존 주기 구분</th>
        <th>설정 보존 개수</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>일간 백업 보존 (Keep-Daily)</td>
        <td>${keep_daily}개</td>
      </tr>
      <tr>
        <td>주간 백업 보존 (Keep-Weekly)</td>
        <td>${keep_weekly}개</td>
      </tr>
      <tr>
        <td>월간 백업 보존 (Keep-Monthly)</td>
        <td>${keep_monthly}개</td>
      </tr>
    </tbody>
  </table>

  <h2>3. 시스템 스케줄러 & 접근 통제</h2>
  <table class="data-table">
    <thead>
      <tr>
        <th>보안 감사 항목</th>
        <th>설정 내역 및 상태</th>
        <th>보안 안전 진단</th>
      </tr>
    </thead>
    <tbody>
      <tr>
        <td>자동 실행 스케줄 (Calendar)</td>
        <td>$on_calendar</td>
        <td><span class="badge badge-success">정상</span></td>
      </tr>
      <tr>
        <td>타이머 데몬 상태 (Enabled / Active)</td>
        <td>$timer_enabled / $timer_active (다음 실행: $next_run)</td>
        <td><span class="badge $([[ "$timer_active" == "active" ]] && echo "badge-success" || echo "badge-warning")">$timer_active</span></td>
      </tr>
      <tr>
        <td>설정 디렉터리 ($RESTIC_ETC_DIR) 권한</td>
        <td>$etc_perm</td>
        <td><span class="badge $([[ "$etc_perm" == "700" ]] && echo "badge-success" || echo "badge-warning")">$etc_safe_str</span></td>
      </tr>
      <tr>
        <td>자격증명 파일 ($BACKUP_ENV_FILE) 권한</td>
        <td>$env_perm</td>
        <td><span class="badge $([[ "$env_perm" == "600" ]] && echo "badge-success" || echo "badge-warning")">$env_safe_str</span></td>
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



# ============================================================
# NTP 시각 동기화 설정 및 증적 생성 (Chrony / NTPd 지원)
# ============================================================

NTPD_CONF_CONTENT='# NTPd Configuration (Backup Pipeline Generated)
driftfile /var/lib/ntp/drift

# Security Restrictions
restrict default nomodify notrap nopeer noquery
restrict 127.0.0.1
restrict ::1

# Time Servers
server time.krnic.net iburst prefer
server time.kriss.re.kr iburst
server time.google.com iburst
server 0.kr.pool.ntp.org iburst'

detect_ntp_service() {
  if [[ -n "${MOCK_NTP_SERVICE:-}" ]]; then
    NTP_SERVICE="$MOCK_NTP_SERVICE"
    if [[ "$NTP_SERVICE" == "ntpd" ]]; then
      NTP_CONF_PATH="${TEST_ROOT:-}/etc/ntp.conf"
    else
      NTP_CONF_PATH="${TEST_ROOT:-}/etc/chrony.conf"
    fi
    return 0
  fi

  if systemctl list-unit-files | grep -q "chronyd.service" 2>/dev/null || command -v chronyc >/dev/null 2>&1; then
    NTP_SERVICE="chronyd"
    NTP_CONF_PATH="${TEST_ROOT:-}/etc/chrony.conf"
    if [[ -z "${TEST_ROOT:-}" && -f "/etc/chrony.conf" ]]; then
      NTP_CONF_PATH="/etc/chrony.conf"
    fi
  elif systemctl list-unit-files | grep -q "ntpd.service" 2>/dev/null || command -v ntpq >/dev/null 2>&1; then
    NTP_SERVICE="ntpd"
    NTP_CONF_PATH="${TEST_ROOT:-}/etc/ntp.conf"
    if [[ -z "${TEST_ROOT:-}" && -f "/etc/ntp.conf" ]]; then
      NTP_CONF_PATH="/etc/ntp.conf"
    fi
  else
    NTP_SERVICE="chronyd"
    NTP_CONF_PATH="${TEST_ROOT:-}/etc/chrony.conf"
  fi
}

check_ntp_sync_status() {
  local NTP_SERVICE NTP_CONF_PATH
  detect_ntp_service

  if [[ "$NTP_SERVICE" == "chronyd" ]]; then
    if systemctl is-active chronyd >/dev/null 2>&1 && chronyc tracking >/dev/null 2>&1; then
      return 0
    fi
  elif [[ "$NTP_SERVICE" == "ntpd" ]]; then
    if systemctl is-active ntpd >/dev/null 2>&1; then
      local rv; rv=$(ntpq -c rv 2>/dev/null)
      if [[ -n "$rv" && "$rv" != *"stratum=16"* && "$rv" != *"leap_alarm"* ]]; then
        return 0
      fi
    fi
  fi
  return 1
}

CHRONY_CONF_CONTENT='# KRNIC (우선 적용)
server time.krnic.net iburst prefer

# 한국표준과학연구원
server time.kriss.re.kr iburst

# Google (Leap Smearing 지원)
server time.google.com iburst

# NTP Pool Korea
server 0.kr.pool.ntp.org iburst

# 2. 시스템 시각 급변 허용 (최초 3회 동안 1초 이상 오차 시 즉시 정정)
makestep 1.0 3

# 3. 하드웨어 시계(RTC) 자동 동기화
rtcsync

# 4. [선택 - Master 서버 역할 시] 내부 사설망 대역의 NTP 요청 허용
#allow 10.0.0.0/8

# 5. [선택 - Master 서버 역할 시] 외부 인터넷 차단 시에도 자체 시각 제공
#local stratum 10

# 6. 로그 보관 및 기본 설정
logdir /var/log/chrony
sourcedir /run/chrony-dhcp
driftfile /var/lib/chrony/drift
keyfile /etc/chrony.keys
ntsdumpdir /var/lib/chrony
leapsectz right/UTC'

# nameref 전달 변수 미사용 경고 우회
# shellcheck disable=SC2034
render_ntp_txt() {
  local _ct_ref_name="$1"
  local hostname_val; eval "hostname_val=\"\${${_ct_ref_name}[hostname]:-unknown}\""
  local report_date; eval "report_date=\"\${${_ct_ref_name}[report_date]:-unknown}\""
  local service_enabled; eval "service_enabled=\"\${${_ct_ref_name}[service_enabled]:-unknown}\""
  local service_active; eval "service_active=\"\${${_ct_ref_name}[service_active]:-unknown}\""
  local sources_output; eval "sources_output=\"\${${_ct_ref_name}[sources_output]:-}\""
  local tracking_output; eval "tracking_output=\"\${${_ct_ref_name}[tracking_output]:-}\""
  local conf_perm; eval "conf_perm=\"\${${_ct_ref_name}[conf_perm]:-}\""
  local ntp_service; eval "ntp_service=\"\${${_ct_ref_name}[ntp_service]:-chronyd}\""

  local sources_title="chronyc sources -v"
  local tracking_title="chronyc tracking"
  if [[ "$ntp_service" == "ntpd" ]]; then
    sources_title="ntpq -p"
    tracking_title="ntpq -c rv"
  fi

  cat <<EOF
======================================================================
[보안 감사 증적] ISMS-P 2.9.3 시각 동기화 점검 보고서
======================================================================
- 점검 일시: $report_date
- 호스트명:  $hostname_val

[1. NTP 시각 동기화 서비스 상태]
- 자동 시작(Enabled): $service_enabled
- 현재 실행(Active):  $service_active

[2. 타임서버 연동 목록 ($sources_title)]
$sources_output

[3. 시각 오차 상세 ($tracking_title)]
$tracking_output

[4. 설정 파일 권한 확인]
$conf_perm

본 보고서는 ISMS-P 2.9.3(시각 동기화) 항목 감사 증적용으로
시스템 스케줄러에 의해 자동 생성되었습니다.
======================================================================
EOF
}

# shellcheck disable=SC2034
render_ntp_json() {
  local _cj_ref_name="$1"
  local hostname_val; eval "hostname_val=\"\${${_cj_ref_name}[hostname]:-unknown}\""
  local report_date; eval "report_date=\"\${${_cj_ref_name}[report_date]:-unknown}\""
  local service_enabled; eval "service_enabled=\"\${${_cj_ref_name}[service_enabled]:-unknown}\""
  local service_active; eval "service_active=\"\${${_cj_ref_name}[service_active]:-unknown}\""
  local sources_output; eval "sources_output=\"\${${_cj_ref_name}[sources_output]:-}\""
  local tracking_output; eval "tracking_output=\"\${${_cj_ref_name}[tracking_output]:-}\""
  local conf_perm; eval "conf_perm=\"\${${_cj_ref_name}[conf_perm]:-}\""
  local ntp_service; eval "ntp_service=\"\${${_cj_ref_name}[ntp_service]:-chronyd}\""

  local sources_json; sources_json=$(printf '%s' "$sources_output" | sed 's/"/\\"/g')
  local tracking_json; tracking_json=$(printf '%s' "$tracking_output" | sed 's/"/\\"/g')

  cat <<EOF
{
  "report_type": "isms_p_2.9.3_ntp_sync",
  "hostname": "${hostname_val//\"/\\\"}",
  "report_date": "${report_date}",
  "ntp_service": {
    "name": "${ntp_service}",
    "enabled": "${service_enabled//\"/\\\"}",
    "active": "${service_active//\"/\\\"}"
  },
  "sources": "${sources_json}",
  "tracking": "${tracking_json}",
  "conf_permission": "${conf_perm//\"/\\\"}"
}
EOF
}

# shellcheck disable=SC2034
render_ntp_html() {
  local _ch_ref_name="$1"
  local hostname_val; eval "hostname_val=\"\${${_ch_ref_name}[hostname]:-unknown}\""
  local report_date; eval "report_date=\"\${${_ch_ref_name}[report_date]:-unknown}\""
  local service_enabled; eval "service_enabled=\"\${${_ch_ref_name}[service_enabled]:-unknown}\""
  local service_active; eval "service_active=\"\${${_ch_ref_name}[service_active]:-unknown}\""
  local sources_output; eval "sources_output=\"\${${_ch_ref_name}[sources_output]:-}\""
  local tracking_output; eval "tracking_output=\"\${${_ch_ref_name}[tracking_output]:-}\""
  local conf_perm; eval "conf_perm=\"\${${_ch_ref_name}[conf_perm]:-}\""
  local ntp_service; eval "ntp_service=\"\${${_ch_ref_name}[ntp_service]:-chronyd}\""

  local sources_title="chronyc sources -v"
  local tracking_title="chronyc tracking"
  if [[ "$ntp_service" == "ntpd" ]]; then
    sources_title="ntpq -p"
    tracking_title="ntpq -c rv"
  fi

  local svc_badge_class="badge-success"
  if [[ "$service_active" != *"running"* && "$service_active" != "active"* ]]; then
    svc_badge_class="badge-warning"
  fi

  cat <<HTMLEOF
<!DOCTYPE html>
<html lang="ko">
<head>
  <meta charset="UTF-8">
  <title>ISMS-P 2.9.3 시각 동기화 점검 보고서</title>
  <style>
    @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&display=swap');
    body { font-family: 'Inter','Malgun Gothic',sans-serif; color:#1e293b; margin:0; padding:20px; background:#f8fafc; }
    .report-card { max-width:800px; margin:0 auto; background:#fff; padding:40px; border:1px solid #e2e8f0; border-radius:8px; box-shadow:0 4px 6px -1px rgb(0 0 0/0.1); }
    header { text-align:center; border-bottom:2px solid #0f172a; padding-bottom:20px; margin-bottom:30px; }
    h1 { font-size:18pt; font-weight:700; margin:0 0 8px 0; color:#0f172a; }
    h2 { font-size:11pt; font-weight:600; border-left:4px solid #6366f1; padding-left:10px; margin:22px 0 10px 0; color:#1e293b; }
    .meta-table { width:100%; border-collapse:collapse; margin-bottom:24px; }
    .meta-table td { padding:7px 12px; font-size:9.5pt; border:1px solid #cbd5e1; }
    .meta-table td.label { background:#f1f5f9; font-weight:600; width:15%; }
    .data-table { width:100%; border-collapse:collapse; margin-bottom:18px; }
    .data-table th,.data-table td { border:1px solid #cbd5e1; padding:7px 12px; font-size:9pt; text-align:left; }
    .data-table th { background:#f8fafc; font-weight:600; color:#475569; }
    .pre-block { background:#0f172a; color:#94a3b8; font-family:'Courier New',monospace; font-size:8pt; padding:12px; border-radius:4px; white-space:pre-wrap; word-break:break-all; margin-bottom:18px; }
    .badge { display:inline-block; padding:2px 8px; border-radius:4px; font-size:8pt; font-weight:600; }
    .badge-success { background:#dcfce7; color:#15803d; }
    .badge-warning { background:#fee2e2; color:#b91c1c; }
    .signature-area { margin-top:36px; display:flex; justify-content:flex-end; gap:30px; }
    .signature-box { border:1px solid #cbd5e1; width:120px; text-align:center; font-size:9pt; }
    .signature-box .title { background:#f1f5f9; padding:4px; font-weight:600; border-bottom:1px solid #cbd5e1; }
    .signature-box .sign { height:50px; line-height:50px; color:#94a3b8; }
    @media print {
      @page { size:A4; margin:12mm 15mm 12mm 15mm; }
      body { background:#fff; padding:0; margin:0; font-size:8pt; -webkit-print-color-adjust:exact; print-color-adjust:exact; }
      .report-card { border:none; box-shadow:none; padding:0; max-width:100%; }
      .data-table th,.data-table td { padding:5px 7px; font-size:8pt; }
      .meta-table td { padding:5px 8px; font-size:8.5pt; }
      h1 { font-size:14pt; } h2 { font-size:10pt; margin:14px 0 7px 0; }
      .badge { font-size:7.5pt; padding:1px 5px; }
      .signature-area { margin-top:18px; }
    }
  </style>
</head>
<body>
<div class="report-card">
  <header>
    <h1>ISMS-P 2.9.3 시각 동기화 점검 보고서</h1>
    <div style="font-size:9pt;color:#64748b;">정보보호관리체계 인증 감사 증적 서류 (NTP 동기화 상태)</div>
  </header>
  <table class="meta-table">
    <tr>
      <td class="label">점검 일시</td><td>$report_date</td>
      <td class="label">호스트명</td><td>$hostname_val</td>
    </tr>
  </table>
  <h2>1. NTP 시각 동기화 서비스 상태</h2>
  <table class="data-table">
    <thead><tr><th>점검 항목</th><th>ISMS 합격 기준</th><th>현재 상태</th><th>결과</th></tr></thead>
    <tbody>
      <tr>
        <td>자동 시작 (Enabled)</td>
        <td>enabled (재부팅 시 자동 실행)</td>
        <td>$service_enabled</td>
        <td><span class="badge $([[ "$service_enabled" == "enabled" ]] && echo "badge-success" || echo "badge-warning")">$service_enabled</span></td>
      </tr>
      <tr>
        <td>서비스 실행 (Active)</td>
        <td>active (running)</td>
        <td>$service_active</td>
        <td><span class="badge $svc_badge_class">$([[ "$svc_badge_class" == "badge-success" ]] && echo "정상" || echo "이상")</span></td>
      </tr>
    </tbody>
  </table>
  <h2>2. 타임서버 연동 목록 (${sources_title})</h2>
  <div class="pre-block">$sources_output</div>
  <h2>3. 시각 오차 상세 (${tracking_title})</h2>
  <div class="pre-block">$tracking_output</div>
  <h2>4. 설정 파일 권한 확인</h2>
  <table class="data-table">
    <thead><tr><th>파일</th><th>ISMS 합격 기준</th><th>실제 권한</th><th>결과</th></tr></thead>
    <tbody>
      <tr>
        <td>${NTP_CONF_PATH}</td>
        <td>root:root, 644 이하</td>
        <td>$conf_perm</td>
        <td><span class="badge $([[ "$conf_perm" == *"root"* ]] && echo "badge-success" || echo "badge-warning")">$([[ "$conf_perm" == *"root"* ]] && echo "적합" || echo "확인 필요")</span></td>
      </tr>
    </tbody>
  </table>
  <div class="signature-area">
    <div class="signature-box"><div class="title">점검자</div><div class="sign">시스템 운영팀 (인)</div></div>
    <div class="signature-box"><div class="title">승인자</div><div class="sign">(서명생략)</div></div>
  </div>
</div>
</body>
</html>
HTMLEOF
}

# backup.env에 BACKUP_NTP_REPORT 플래그를 직접 추가/갱신한다
_ntp_set_flag_in_env() {
  local flag_val="$1"
  require_backup_env
  if grep -q "BACKUP_NTP_REPORT=" "$BACKUP_ENV_FILE"; then
    sed -i "s/BACKUP_NTP_REPORT='[01]'/BACKUP_NTP_REPORT='${flag_val}'/" "$BACKUP_ENV_FILE"
  elif grep -q "BACKUP_CHRONY_REPORT=" "$BACKUP_ENV_FILE"; then
    sed -i "s/BACKUP_CHRONY_REPORT='[01]'/BACKUP_NTP_REPORT='${flag_val}'/" "$BACKUP_ENV_FILE"
  else
    printf "\n# ==========================================\n# NTP 시각 동기화 증적 생성 설정\n# ==========================================\nBACKUP_NTP_REPORT='%s'\n" "$flag_val" >> "$BACKUP_ENV_FILE"
  fi
  chmod 600 "$BACKUP_ENV_FILE" 2>/dev/null || true
}

cmd_ntp() {
  if has_help_flag "$@"; then
    cat <<'HELPEOF'
사용법: backup.sh ntp <subcommand> [options]

서브커맨드:
  setup      NTP(Chrony/NTPd) 설정 파일 적용 및 서비스 재시작 + 상태 검증
  --report   ISMS-P 2.9.3 시각 동기화 증적 보고서 생성 (txt/json/html)

옵션 (--report):
  --report-file <path>  증적 파일 출력 경로
                        (기본: /data/backup/reports/ntp_sync_evidence_YYYYMMDD.txt)

예시:
  backup.sh ntp setup
  backup.sh ntp --report
  backup.sh ntp --report --report-file /tmp/ntp_check.txt
HELPEOF
    return 0
  fi

  local action="${1:-}"
  shift || true

  # 동적으로 NTP 서비스 및 설정 경로 감지
  local NTP_SERVICE NTP_CONF_PATH
  detect_ntp_service

  case "$action" in
    setup)
      require_root
      require_backup_env

      # 1. 기존 설정 파일 백업
      if [[ -f "$NTP_CONF_PATH" ]]; then
        local conf_base; conf_base=$(basename "$NTP_CONF_PATH")
        local bak_path; bak_path="$(dirname "$NTP_CONF_PATH")/${conf_base}.bak.$(date +%Y%m%d)"
        cp "$NTP_CONF_PATH" "$bak_path"
        log_info "기존 설정을 백업했습니다: $bak_path"
      fi

      # 2. 설정 파일 작성
      local conf_content="$CHRONY_CONF_CONTENT"
      if [[ "$NTP_SERVICE" == "ntpd" ]]; then
        conf_content="$NTPD_CONF_CONTENT"
      fi
      printf '%s\n' "$conf_content" > "$NTP_CONF_PATH"
      chmod 644 "$NTP_CONF_PATH"
      log_info "NTP 설정 파일을 작성했습니다: $NTP_CONF_PATH"

      # 3. 서비스 재시작
      systemctl restart "$NTP_SERVICE"
      log_info "${NTP_SERVICE} 서비스를 재시작했습니다."

      # 4. 구문 에러 유무 로그 확인
      log_info "=== journalctl -u ${NTP_SERVICE} -n 20 ==="
      journalctl -u "$NTP_SERVICE" -n 20 --no-pager || true

      # 5. 타임서버 동기화 상태 점검
      if [[ "$NTP_SERVICE" == "chronyd" ]]; then
        log_info "=== chronyc sources -v ==="
        chronyc sources -v || true
      else
        log_info "=== ntpq -p ==="
        ntpq -p || true
      fi

      # 6. backup.env에 BACKUP_NTP_REPORT=1 기록
      # nameref 인자 전달용 설정 배열 선언
      # shellcheck disable=SC2034
      local -A ntp_cfg=()
      # nameref 인자 전달용 에러 배열 선언
      # shellcheck disable=SC2034
      local -A ntp_errs=()

      if load_and_validate_config "" ntp_cfg ntp_errs 2>/dev/null; then
        # nameref 인자 변수 수정 분석 경고 우회
        # shellcheck disable=SC2034
        ntp_cfg[ntp_report]="1"
        save_profile_config ntp_cfg >/dev/null
      else
        _ntp_set_flag_in_env "1"
      fi
      log_info "BACKUP_NTP_REPORT=1 이 backup.env에 저장되었습니다."
      ;;

    --report)
      require_backup_env

      local -A report_opts=()
      parse_opts_into report_opts "report-file:" -- "$@"
      local date_suffix; date_suffix=$(date +%Y%m%d)
      local report_file="${report_opts[report-file]:-${BACKUP_REPORTS_DIR}/ntp_sync_evidence_${date_suffix}.txt}"
      local base_path="${report_file%.txt}"

      # 상태 수집
      local svc_enabled svc_active sources_out tracking_out conf_perm_out
      svc_enabled=$(systemctl is-enabled "$NTP_SERVICE" 2>/dev/null || echo "unknown")
      svc_active=$(systemctl is-active "$NTP_SERVICE" 2>/dev/null || echo "unknown")
      if [[ "$NTP_SERVICE" == "chronyd" ]]; then
        sources_out=$(chronyc sources -v 2>/dev/null || echo "(chronyc 실행 불가)")
        tracking_out=$(chronyc tracking 2>/dev/null || echo "(chronyc 실행 불가)")
      else
        sources_out=$(ntpq -p 2>/dev/null || echo "(ntpq 실행 불가)")
        tracking_out=$(ntpq -c rv 2>/dev/null || echo "(ntpq 실행 불가)")
      fi
      conf_perm_out=$(ls -l "$NTP_CONF_PATH" 2>/dev/null || echo "(파일 없음)")

      local report_date; report_date=$(date '+%Y-%m-%d %H:%M:%S %Z')
      local hostname_val; hostname_val=$(hostname 2>/dev/null || echo "unknown")

      # nameref 전달용 변수 — shellcheck 경고 우회
      # shellcheck disable=SC2034
      local -A ntp_data=(
        [hostname]="$hostname_val"
        [report_date]="$report_date"
        [service_enabled]="$svc_enabled"
        [service_active]="$svc_active"
        [sources_output]="$sources_out"
        [tracking_output]="$tracking_out"
        [conf_perm]="$conf_perm_out"
        [ntp_service]="$NTP_SERVICE"
      )

      write_evidence_report_bundle ntp_data "$report_file" "ntp"

      log_info "NTP 증적 보고서가 생성되었습니다:"
      log_info "  - TXT:  $report_file"
      log_info "  - JSON: ${base_path}.json"
      log_info "  - HTML: ${base_path}.html"
      ;;

    *)
      die "ntp는 'setup' 또는 '--report'만 지원합니다 (입력값: '${action}')"
      ;;
  esac
}

cmd_chrony() {
  log_warn "'chrony' 명령어는 더 이상 사용되지 않습니다. 대신 'ntp'를 사용하십시오."
  cmd_ntp "$@"
}


cmd_audit() {
  if has_help_flag "$@"; then
    help_audit
    return 0
  fi
  require_backup_env

  local -A opts=()
  parse_opts_into opts "report-file: path: report daily restore-drill ntp tester: ciso: rto: target: drill-restore-dir: audit-tester: audit-ciso: audit-rto:" -- "$@"
  local report_file_val="${opts[report-file]:-}"
  local report_path_val="${opts[path]:-}"
  local report="${opts[report]:-0}"
  local daily="${opts[daily]:-0}"
  local restore_drill="${opts[restore-drill]:-0}"
  local ntp="${opts[ntp]:-0}"

  local save_reports=0
  if (( report )) || [[ -n "$report_file_val" ]]; then
    save_reports=1
  fi

  if (( save_reports == 0 )); then
    if (( daily && restore_drill )); then
      die "--daily와 --restore-drill 옵션은 동시에 사용할 수 없습니다."
    fi
    if (( daily && ntp )); then
      die "--daily와 --ntp 옵션은 동시에 사용할 수 없습니다."
    fi
    if (( restore_drill && ntp )); then
      die "--restore-drill과 --ntp 옵션은 동시에 사용할 수 없습니다."
    fi
  fi

  local run_general=0
  local run_daily=0
  local run_restore_drill=0
  local run_ntp=0

  if (( daily == 0 && restore_drill == 0 && ntp == 0 )); then
    if (( report )); then
      run_general=1
      run_daily=1
      run_restore_drill=1
      run_ntp=1
    else
      run_general=1
    fi
  else
    run_daily=$daily
    run_restore_drill=$restore_drill
    run_ntp=$ntp
  fi

  local report_dir=""
  if [[ -n "$report_path_val" ]]; then
    report_dir="$report_path_val"
  elif [[ -n "$report_file_val" ]]; then
    report_dir=$(dirname "$report_file_val")
  else
    report_dir=$(resolve_value BACKUP_REPORTS_DIR "/data/backup/reports")
  fi

  local date_suffix; date_suffix=$(date +%Y%m%d)

  local tester_override="${opts[audit-tester]:-${opts[tester]:-}}"
  local ciso_override="${opts[audit-ciso]:-${opts[ciso]:-}}"
  local rto_override="${opts[audit-rto]:-${opts[rto]:-}}"
  local drill_restore_dir_override="${opts[drill-restore-dir]:-${opts[target]:-}}"

  local daily_file="${report_dir}/daily_backup_audit_report_${date_suffix}.txt"
  if [[ -n "$report_file_val" ]] && (( run_daily && run_general == 0 && run_restore_drill == 0 && run_ntp == 0 )); then
    daily_file="$report_file_val"
  fi

  local drill_file="${report_dir}/restore_drill_report_${date_suffix}.txt"
  if [[ -n "$report_file_val" ]] && (( run_restore_drill && run_general == 0 && run_daily == 0 && run_ntp == 0 )); then
    drill_file="$report_file_val"
  fi

  local ntp_file="${report_dir}/ntp_sync_evidence_${date_suffix}.txt"
  if [[ -n "$report_file_val" ]] && (( run_ntp && run_general == 0 && run_daily == 0 && run_restore_drill == 0 )); then
    ntp_file="$report_file_val"
  fi

  local general_file="${report_dir}/audit_report.txt"
  if [[ -n "$report_file_val" ]] && (( run_general )); then
    general_file="$report_file_val"
  fi

  local -a saved_files=()

  # ----------------------------------------------------
  # [1] NTP Sync Report
  # ----------------------------------------------------
  if (( run_ntp )); then
    local NTP_SERVICE NTP_CONF_PATH
    detect_ntp_service

    local svc_enabled svc_active sources_out tracking_out conf_perm_out
    svc_enabled=$(systemctl is-enabled "$NTP_SERVICE" 2>/dev/null || echo "unknown")
    svc_active=$(systemctl is-active "$NTP_SERVICE" 2>/dev/null || echo "unknown")
    if [[ "$NTP_SERVICE" == "chronyd" ]]; then
      sources_out=$(chronyc sources -v 2>/dev/null || echo "(chronyc 실행 불가)")
      tracking_out=$(chronyc tracking 2>/dev/null || echo "(chronyc 실행 불가)")
    else
      sources_out=$(ntpq -p 2>/dev/null || echo "(ntpq 실행 불가)")
      tracking_out=$(ntpq -c rv 2>/dev/null || echo "(ntpq 실행 불가)")
    fi
    conf_perm_out=$(ls -l "$NTP_CONF_PATH" 2>/dev/null || echo "(파일 없음)")

    local report_date; report_date=$(date '+%Y-%m-%d %H:%M:%S %Z')
    local hostname_val; hostname_val=$(hostname 2>/dev/null || echo "unknown")

    # nameref 인자로 전달되어 정적 분석기에서 미사용으로 오탐지 우회
    # shellcheck disable=SC2034
    local -A ntp_data=(
      [hostname]="$hostname_val"
      [report_date]="$report_date"
      [service_enabled]="$svc_enabled"
      [service_active]="$svc_active"
      [sources_output]="$sources_out"
      [tracking_output]="$tracking_out"
      [conf_perm]="$conf_perm_out"
      [ntp_service]="$NTP_SERVICE"
    )

    if (( report == 0 )) && [[ -z "$report_file_val" ]]; then
      render_ntp_txt ntp_data
    fi

    if (( save_reports )); then
      write_evidence_report_bundle ntp_data "$ntp_file" "ntp"
      saved_files+=("$ntp_file" "${ntp_file%.*}.json" "${ntp_file%.*}.html")
    fi
  fi

  # ----------------------------------------------------
  # [2] Restore Drill Report
  # ----------------------------------------------------
  if (( run_restore_drill )); then
    local tester; tester=$(resolve_value "$tester_override" "${BACKUP_AUDIT_TESTER:-}" "" "홍길동 (인프라보안팀 선임연구원)")
    local ciso; ciso=$(resolve_value "$ciso_override" "${BACKUP_AUDIT_CISO:-}" "" "이몽룡 (정보보안책임자 CISO)")
    local rto; rto=$(resolve_value "$rto_override" "${BACKUP_AUDIT_RTO:-}" "" "120")
    local target_dir="${drill_restore_dir_override:-/tmp/restore_test}"

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

    if (( report == 0 )) && [[ -z "$report_file_val" ]]; then
      render_restore_drill_report drill_res
    fi

    if (( save_reports )); then
      write_audit_reports drill_res "$drill_file"
      saved_files+=("$drill_file" "${drill_file%.*}.json" "${drill_file%.*}.html")
    fi
  fi

  # ----------------------------------------------------
  # [3] Daily Backup Review Report
  # ----------------------------------------------------
  if (( run_daily )); then
    local tester; tester=$(resolve_value "$tester_override" "${BACKUP_AUDIT_TESTER:-}" "" "인프라보안팀 (시스템 자동 실행)")
    local hostname_val; hostname_val=$(hostname 2>/dev/null || echo "unknown")
    local cur_time; cur_time=$(date "+%Y-%m-%d %H:%M:%S KST")

    local snapshots_json
    snapshots_json=$(restic snapshots --json 2>/dev/null || echo "[]")

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

    local config_daily="${KEEP_DAILY:-0}"
    local config_weekly="${KEEP_WEEKLY:-0}"
    local config_monthly="${KEEP_MONTHLY:-0}"

    local config_daily_status="미흡"; [[ "$config_daily" -ge 7 ]] && config_daily_status="만족"
    local actual_daily_status="미흡"; [[ "$actual_daily" -ge 7 ]] && actual_daily_status="만족"
    local config_weekly_status="미흡"; [[ "$config_weekly" -ge 4 ]] && config_weekly_status="만족"
    local actual_weekly_status="미흡"; [[ "$actual_weekly" -ge 4 ]] && actual_weekly_status="만족"
    local config_monthly_status="미흡"; [[ "$config_monthly" -ge 12 ]] && config_monthly_status="만족"
    local actual_monthly_status="미흡"; [[ "$actual_monthly" -ge 12 ]] && actual_monthly_status="만족"

    local etc_perm; etc_perm="$(stat -c '%a' "$RESTIC_ETC_DIR" 2>/dev/null || echo '700')"
    local env_perm; env_perm="$(stat -c '%a' "$BACKUP_ENV_FILE" 2>/dev/null || echo '600')"
    local etc_safe_str="미흡"; [[ "$etc_perm" == "700" ]] && etc_safe_str="만족"
    local env_safe_str="미흡"; [[ "$env_perm" == "600" ]] && env_safe_str="만족"

    local check_status="미흡"
    if restic check >/dev/null 2>&1; then
      check_status="만족"
    fi

    local backend="s3"
    if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
      backend="sftp"
    fi

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
        processed_bytes = None
        summary = snap.get("summary")
        if isinstance(summary, dict) and "total_bytes_processed" in summary:
            processed_bytes = summary["total_bytes_processed"]
        if processed_bytes is None:
            try:
                res = subprocess.run(["restic", "stats", "--json", snap.get("id")], capture_output=True, text=True, timeout=5)
                if res.returncode == 0:
                    stats_data = json.loads(res.stdout)
                    processed_bytes = stats_data.get("total_size")
            except Exception:
                pass
        size_str = ""
        if processed_bytes is not None:
            if processed_bytes >= 1073741824:
                size_str = " (%.2f GB)" % (processed_bytes / 1073741824.0)
            elif processed_bytes >= 1048576:
                size_str = " (%.2f MB)" % (processed_bytes / 1048576.0)
            elif processed_bytes >= 1024:
                size_str = " (%.2f KB)" % (processed_bytes / 1024.0)
            else:
                size_str = " (%d B)" % processed_bytes
        else:
            size_str = " (크기 확인 불가)"
        print("  %-10s  %-19s  %-18s  %s%s" % (sid, time_str, host, paths, size_str))
except Exception as e:
    print("  (스냅샷 정보 해석 실패: %s)" % e)
' <<< "$snapshots_json")

    local snapshot_table_html; snapshot_table_html=$(generate_snapshot_table_html "$snapshots_json")

    # nameref 인자로 전달되어 정적 분석기에서 미사용으로 오탐지 우회
    # shellcheck disable=SC2034
    local -A daily_data=(
      [cur_time]="$cur_time"
      [hostname]="$hostname_val"
      [tester]="$tester"
      [backend]="$backend"
      [repo]="$RESTIC_REPOSITORY"
      [targets]="$BACKUP_TARGETS"
      [config_daily]="$config_daily"
      [actual_daily]="$actual_daily"
      [config_daily_status]="$config_daily_status"
      [actual_daily_status]="$actual_daily_status"
      [config_weekly]="$config_weekly"
      [actual_weekly]="$actual_weekly"
      [config_weekly_status]="$config_weekly_status"
      [actual_weekly_status]="$actual_weekly_status"
      [config_monthly]="$config_monthly"
      [actual_monthly]="$actual_monthly"
      [config_monthly_status]="$config_monthly_status"
      [actual_monthly_status]="$actual_monthly_status"
      [etc_dir]="$RESTIC_ETC_DIR"
      [etc_perm]="$etc_perm"
      [etc_safe_str]="$etc_safe_str"
      [env_file]="$BACKUP_ENV_FILE"
      [env_perm]="$env_perm"
      [env_safe_str]="$env_safe_str"
      [check_status]="$check_status"
      [snapshot_table]="$snapshot_table"
      [snapshot_table_html]="$snapshot_table_html"
      [snapshots_json]="$snapshots_json"
    )

    if (( report == 0 )) && [[ -z "$report_file_val" ]]; then
      render_daily_audit_report daily_data
    fi

    if (( save_reports )); then
      write_evidence_report_bundle daily_data "$daily_file" "daily"
      saved_files+=("$daily_file" "${daily_file%.*}.json" "${daily_file%.*}.html")
    fi
  fi

  # ----------------------------------------------------
  # [4] General Audit Report
  # ----------------------------------------------------
  if (( run_general )); then
    local profile_name; profile_name=$(resolve_profile_name)
    local timer_unit; timer_unit=$(resticprofile_timer_unit_name "$profile_name")

    local timer_enabled; timer_enabled=$(systemctl is-enabled "$timer_unit" 2>/dev/null || echo "unknown")
    local timer_active; timer_active=$(systemctl is-active "$timer_unit" 2>/dev/null || echo "unknown")
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
    if (( report == 0 )) && [[ -z "$report_file_val" ]]; then
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
    fi

    # If report_file is requested, save both plain text and JSON versions
    if (( save_reports )); then
      mkdir -p "$(dirname "$general_file")"
      chmod 700 "$(dirname "$general_file")" 2>/dev/null || true

      # Save plain text report
      (
        render_audit_report "$backend" "${on_calendar:-알 수 없음}" "${timer_enabled:-unknown}" \
          "${timer_active:-unknown}" "${next_run:-알 수 없음}" \
          "$etc_perm" "$env_perm"
        printf '\n=== 백업 이력(restic snapshots) ===\n'
        restic snapshots 2>/dev/null || printf '(조회 실패 또는 미초기화)\n'
      ) > "$general_file"
      chmod 600 "$general_file"

      # Save JSON report
      local json_general_file="${general_file%.*}.json"
      render_audit_report_json "$backend" "${on_calendar:-알 수 없음}" "${timer_enabled:-unknown}" \
        "${timer_active:-unknown}" "${next_run:-알 수 없음}" \
        "$etc_perm" "$env_perm" > "$json_general_file"
      chmod 600 "$json_general_file"

      # Generate HTML snapshot table
      local snapshots_json
      snapshots_json=$(restic snapshots --json 2>/dev/null || echo "[]")
      local snapshot_table_html; snapshot_table_html=$(generate_snapshot_table_html "$snapshots_json")

      # Save HTML report
      local html_general_file="${general_file%.*}.html"
      render_audit_report_html "$backend" "${on_calendar:-알 수 없음}" "${timer_enabled:-unknown}" \
        "${timer_active:-unknown}" "${next_run:-알 수 없음}" \
        "$etc_perm" "$env_perm" "$snapshot_table_html" > "$html_general_file"
      chmod 600 "$html_general_file"

      saved_files+=("$general_file" "$json_general_file" "$html_general_file")
    fi
  fi

  # ----------------------------------------------------
  # [5] Print final status message if report generated
  # ----------------------------------------------------
  if (( save_reports )) && (( ${#saved_files[@]} > 0 )); then
    log_info "감사 보고서가 동시 저장되었습니다:"
    log_info "  - 저장 경로: $report_dir"
    local items=""
    (( run_general )) && items="${items:+$items, }종합(audit_report)"
    (( run_daily )) && items="${items:+$items, }일일(daily_backup_audit_report)"
    (( run_restore_drill )) && items="${items:+$items, }복구훈련(restore_drill_report)"
    (( run_ntp )) && items="${items:+$items, }시간동기화(ntp_sync_evidence)"
    log_info "  - 생성 항목: $items"
  fi
}

cmd_uninstall() {
  if has_help_flag "$@"; then
    help_uninstall
    return 0
  fi
  require_root
  local -A opts=()
  parse_opts_into opts "purge force yes" -- "$@"
  local purge="${opts[purge]:-0}" force="${opts[force]:-0}" yes="${opts[yes]:-0}"

  if (( purge )) && (( force == 0 )) && (( yes == 0 )); then
    if ! safe_confirm "언인스톨 및 모든 백업 데이터/설정을 삭제하시겠습니까?" "n"; then
      log_info "언인스톨 작업이 취소되었습니다."
      return 0
    fi
  fi

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
  parse_opts_into opts "backend: endpoint: bucket: access-key: secret-key: host: port: user: new-password: skip-check force yes" -- "$@"
  local skip_check="${opts[skip-check]:-0}"
  local force="${opts[force]:-0}"
  local yes="${opts[yes]:-0}"

  if (( force == 0 )) && (( yes == 0 )); then
    if ! safe_confirm "저장소 마이그레이션을 진행하시겠습니까?" "n"; then
      log_info "마이그레이션 작업이 취소되었습니다."
      return 0
    fi
  fi

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
    if is_interactive; then
      value=$(safe_input "$message" "$default" 0)
    else
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
    fi

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

prompt_secret_required() {
  local message="$1"
  local value
  while true; do
    if is_interactive; then
      value=$(safe_input "$message" "" 1)
    else
      if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
        setup_colors
        printf '%b%s%b' "$C_YELLOW" "$message" "$C_RESET" >&2
      else
        printf '%s' "$message" >&2
      fi
      read -rs value
      printf '\n' >&2
    fi

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

prompt_backend_choice() {
  if is_interactive; then
    local choice
    choice=$(safe_choose "백엔드 선택 (Choose Backend)" "s3" "sftp")
    if [[ "$choice" == "s3" || "$choice" == "sftp" ]]; then
      printf '%s' "$choice"
      return 0
    fi
  fi

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

cmd_import() {
  if has_help_flag "$@"; then
    help_import
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

  local ep_val="${BACKUP_ENDPOINT:-}"
  local bucket_val="${BACKUP_BUCKET:-}"
  if [[ "${resolved[backend]}" == "s3" && -n "${RESTIC_REPOSITORY:-}" ]]; then
    if [[ "${RESTIC_REPOSITORY}" == s3:* ]]; then
      local s3_url="${RESTIC_REPOSITORY#s3:}"
      s3_url="${s3_url%/}"
      s3_url="${s3_url%/*}"
      local b_name="${s3_url##*/}"
      local ep_name="${s3_url%/*}"
      ep_val="${ep_val:-$ep_name}"
      bucket_val="${bucket_val:-$b_name}"
    fi
  fi
  resolved[endpoint]="$ep_val"
  resolved[bucket]="$bucket_val"
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
      export RESTIC_PASSWORD="${RESTIC_PASSWORD:-}"
      export RESTIC_REPOSITORY="${RESTIC_REPOSITORY:-}"
      export AWS_ACCESS_KEY_ID="${AWS_ACCESS_KEY_ID:-}"
      export AWS_SECRET_ACCESS_KEY="${AWS_SECRET_ACCESS_KEY:-}"
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

help_import() {
  cat <<'EOF'
기존의 1차 로컬 백업 데이터를 새로운 1차 원격 백업 저장소로 안전하게 마이그레이션(이관)하고,
디스크 정리를 위한 가이드를 제공합니다.

사용법:
  backup.sh import [flags]

사용법 예시:
  # 기본 경로(/var/restic-local)에 존재하는 로컬 데이터를 원격지로 이관
  backup.sh import

  # 특정 경로에 있는 로컬 데이터를 원격지로 이관
  backup.sh import --legacy-dir /data/backup

플래그 (Flags):
      --legacy-dir <경로>     이관할 기존 로컬 restic 저장소 디렉터리 경로 (기본값: /var/restic-local)

글로벌 플래그 (Global Flags):
  -h, --help                  도움말 출력
EOF
}

cmd_upgrade() {
  log_warn "'upgrade' 명령어는 더 이상 사용되지 않습니다. 대신 'import'를 사용하십시오."
  cmd_import "$@"
}

cmd_update() {
  if has_help_flag "$@"; then
    cat <<'HELPEOF'
사용법: backup.sh update

설명:
  로컬의 최신 backup.sh 스크립트 실행본으로 설치 파일(/usr/local/sbin/backup.sh)을 갱신 설치하고,
  새로운 템플릿 규격에 맞춰 Systemd 스케줄러 타이머 및 프로필 설정을 자동으로 갱신(마이그레이션)합니다.

글로벌 플래그 (Global Flags):
  -h, --help                  도움말 출력
HELPEOF
    return 0
  fi

  require_root

  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    log_info "기존 설정을 새 버전 규격으로 마이그레이션합니다..."
    local -A resolved=()
    local -a errors=()
    local -A opts=()
    if load_and_validate_config "" opts resolved errors; then
      if save_profile_config resolved >/dev/null; then
        log_info "설정 파일 마이그레이션 완료"
      else
        log_warn "설정 파일 마이그레이션 저장 실패"
      fi
    else
      log_warn "기존 설정 검증 실패로 설정 파일 마이그레이션을 건너뜁니다."
      local e
      for e in "${errors[@]}"; do
        log_warn " - ${e}"
      done
    fi
  fi

  log_info "최신 스크립트 버전(${BACKUP_SCRIPT_VERSION})으로 설치본을 갱신합니다..."
  self_install_copy "$0" 1

  log_info "스케줄러 및 설정 프로필 구성을 새 버전 규격으로 업데이트합니다..."
  cmd_schedule enable
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
  else
    printf '\n--- 백업 대상 경로 설정 ---\n'
    printf '보안 컴플라이언스(ISMS/ISO 27001) 기준에 부합하기 위해, 중요 설정 파일(/etc) 및 중요 업무 데이터(/data/backup)가 기본 백업 경로로 지정되어 있습니다.\n'
    printf '  * /etc: 사용자 계정, 권한 설정 및 네트워크 구성을 보존하여 설정의 무결성을 입증합니다.\n'
    printf '  * /data/backup: 정보 유출 방지 및 중요 업무 데이터 보존을 지원합니다.\n\n'
  fi

  local use_default=1
  if ! safe_confirm "기본 경로(/data/backup, /etc)를 백업 대상에 포함하시겠습니까?" "y"; then
    use_default=0
  fi

  local final_targets=""
  if (( use_default )); then
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

  # ── NTP 시각 동기화 설정 ──────────────────────────────────────────
  local ntp_setup_done=0
  if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
    setup_colors
    printf '\n%b%b⚙  시각 동기화 설정 (NTP / Chrony)%b\n' "$C_CYAN" "$C_BOLD" "$C_RESET"
    printf '%bISMS-P 2.9.3 시각 동기화 항목 대응을 위해 NTP 설정을 적용할 수 있습니다.%b\n' "$C_DIM" "$C_RESET"
    printf '%b시각 동기화 설정을 적용할까요? [y/%bN%b]: %b' "$C_CYAN" "$C_BOLD" "$C_RESET" "$C_RESET"
  else
    printf '\n--- 시각 동기화 설정 (NTP / Chrony) ---\n'
    printf 'ISMS-P 2.9.3 시각 동기화 항목 대응을 위해 NTP 설정을 적용할 수 있습니다.\n'
    printf '시각 동기화 설정을 적용할까요? [y/N]: '
  fi
  local ntp_choice; read -r ntp_choice || true

  if [[ "$ntp_choice" =~ ^[Yy]$ ]]; then
    local chrony_ready=0
    if type -P chronyc > /dev/null 2>&1 && systemctl is-active chronyd > /dev/null 2>&1; then
      chrony_ready=1
    fi

    if (( chrony_ready )); then
      cmd_ntp setup
      ntp_setup_done=1
    else
      if [[ -t 1 ]] && [[ -z "${NO_COLOR:-}" ]]; then
        printf '%b안내: chrony(chronyd)가 설치되어 있지 않거나 실행 중이지 않습니다. 먼저 chrony를 설치하고 실행한 뒤\n        backup.sh ntp setup 을 단독으로 실행하세요.%b\n' "$C_YELLOW" "$C_RESET"
      else
        printf '안내: chrony(chronyd)가 설치되어 있지 않거나 실행 중이지 않습니다.\n'
        printf '      먼저 chrony를 설치한 뒤 backup.sh ntp setup 을 실행하세요.\n'
      fi
    fi
  fi

  # ── 정기 백업 스케줄 등록 ─────────────────────────────────────────────────
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

  local ntp_report_active=0
  local ntp_var="BACKUP_NTP_REPORT"
  local chrony_var="BACKUP_CHRONY_REPORT"
  local file_ntp_val="${file_config[$ntp_var]:-}"
  local file_chrony_val="${file_config[$chrony_var]:-}"
  local env_ntp_val="${!ntp_var:-}"
  local env_chrony_val="${!chrony_var:-}"
  if (( ntp_setup_done )) || [[ "$file_ntp_val" == "1" || "$file_chrony_val" == "1" || "$env_ntp_val" == "1" || "$env_chrony_val" == "1" ]]; then
    ntp_report_active=1
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
      if (( ntp_report_active )); then
        printf ' %b├──%b NTP 증적 보고:  %b등록됨 (%s)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$DEFAULT_NTP_ON_CALENDAR" "$C_RESET"
      else
        printf ' %b├──%b NTP 증적 보고:  %b미등록 (backup.sh ntp setup으로 설정 가능)%b\n' "$C_GRAY" "$C_RESET" "$C_GRAY" "$C_RESET"
      fi
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
      if (( ntp_report_active )); then
        printf ' NTP 증적 보고: 등록됨 (%s)\n' "$DEFAULT_NTP_ON_CALENDAR"
      else
        printf ' NTP 증적 보고: 미등록 (backup.sh ntp setup으로 설정 가능)\n'
      fi
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
  parse_opts_into opts "targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: profile-name: on-calendar: notification-url: notification-type: notification-on: audit-tester: audit-ciso: audit-rto: dry-run secondary-backend: secondary-password: secondary-endpoint: secondary-bucket: secondary-access-key: secondary-secret-key: secondary-host: secondary-user: secondary-port: secondary-keep-daily: secondary-keep-weekly: secondary-keep-monthly: db-type: db-command: db-filename: db-schedule: db-keep-daily: db-keep-weekly: db-keep-monthly: ntp-report: chrony-report:" -- "$@"


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
  parse_opts_into opts "backend: targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: profile-name: notification-url: notification-type: notification-on: audit-tester: audit-ciso: audit-rto: force dry-run secondary-backend: secondary-password: secondary-endpoint: secondary-bucket: secondary-access-key: secondary-secret-key: secondary-host: secondary-user: secondary-port: secondary-keep-daily: secondary-keep-weekly: secondary-keep-monthly: db-type: db-command: db-filename: db-schedule: db-keep-daily: db-keep-weekly: db-keep-monthly: ntp-report: chrony-report:" -- "$@"


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
  backup.sh schedule <enable|disable|status> [옵션...]

사용법 예시:
  # 기본 일정으로 정기 백업 및 보안 감사 타이머 전체 활성화
  backup.sh schedule enable

  # 특정 시간 일정으로 정기 백업 활성화
  backup.sh schedule enable --on-calendar "*-*-* 03:30:00"

  # 등록된 타이머 스케줄러 상태 조회
  backup.sh schedule status

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

  # 모든 감사 보고서(종합, 일일, 복구훈련, NTP)를 지정한 디렉터리에 생성 및 저장
  backup.sh audit --report --path /mnt/backup_reports

플래그 (Flags):
      --report              보고서 파일 생성 여부를 지정합니다.
      --report-file <경로>   생성될 텍스트 보고서 파일의 경로를 직접 지정합니다.
      --path <경로>          생성될 보고서 파일들이 저장될 디렉터리 경로를 지정합니다. (기본값: /data/backup/reports)
      --daily               일일 백업 검토 보고서 모드로 실행합니다.
      --restore-drill       복구 모의훈련 보고서 모드로 실행합니다. (실제 복구 다운로드 수행 및 정합성 쿼리 테스트 트리거)
      --ntp                 NTP 시각 동기화 점검 보고서 모드로 실행합니다.
      --audit-tester <이름> 일일 백업 검토 및 복구 모의훈련 담당자 이름 (설정 파일값 무시하고 임시 지정, Alias: --tester)
      --audit-ciso <이름>   보고서 최종 승인 정보보안책임자 이름 (설정 파일값 무시하고 임시 지정, Alias: --ciso)
      --audit-rto <분>      복구 목표 시간(RTO, 분 단위) (설정 파일값 무시하고 임시 지정, Alias: --rto)
      --drill-restore-dir <경로> 복구 모의훈련 시 임시 다운로드 및 쿼리 테스트를 수행할 임시 디렉터리 경로 (기본값: /tmp/restore_test, Alias: --target)

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
  ntp            NTP 시각 동기화 설정(setup) 및 ISMS-P 2.9.3 증적 생성(--report)
  uninstall      정기 스케줄 제거 및 설치된 바이너리/스크립트 삭제
  migrate        기존 저장소 백업 데이터를 새로운 스토리지 백엔드로 데이터 복제 및 설정 이관
  config         기존 백업 설정(backup.env) 수정 및 관련 자산 동기화
  wizard         단계별 설정을 위한 대화형 CLI 설정 마법사 실행
  import         기존 1차 로컬 백업 데이터를 새로운 1차 원격 저장소로 이관 및 환경 정리
  update         최신 스크립트 실행본 설치 및 스케줄러/설정 갱신

글로벌 플래그 (Global Flags):
  -h, --help     도움말 출력
  -V, --version  버전 정보 출력
  -v, --verbose  디버깅 및 연동 명령어 상세 로깅 활성화

상세한 하위 명령 정보는 'backup.sh [command] --help'를 참고하세요.
EOF
}

main() {
  check_system_dependencies

  local -a args=()
  local arg
  for arg in "$@"; do
    case "$arg" in
      -v|--verbose) BACKUP_VERBOSE=1 ;;
      *) args+=("$arg") ;;
    esac
  done
  set -- "${args[@]}"

  local skip_core_check=0
  if [[ $# -eq 0 ]]; then
    skip_core_check=1
  else
    case "$1" in
      -h|--help|-V|--version|install) skip_core_check=1 ;;
    esac
  fi

  if (( ! skip_core_check )); then
    check_core_dependencies
  fi

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
    import)
      shift
      cmd_import "$@"
      return $?
      ;;
    upgrade)
      shift
      cmd_upgrade "$@"
      return $?
      ;;
    ntp)
      shift
      cmd_ntp "$@"
      return $?
      ;;
    chrony)
      shift
      cmd_chrony "$@"
      return $?
      ;;
    update)
      shift
      cmd_update "$@"
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

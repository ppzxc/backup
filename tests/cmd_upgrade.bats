#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
}

@test "cmd_upgrade fails when backup.env is missing" {
  run main upgrade
  [ "$status" -eq 1 ]
  [[ "$output" == *"설정 파일이 존재하지 않습니다"* ]]
}

@test "cmd_upgrade completes successfully when there is no legacy local repository" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-bucket/host"
export RESTIC_PASSWORD="secret"
ENV

  run main upgrade
  [ "$status" -eq 0 ]
  [[ "$output" == *"이관할 로컬 데이터가 없습니다"* ]]
}

@test "cmd_upgrade performs migration from local repo to remote repo when legacy local repo exists" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-bucket/host"
export RESTIC_PASSWORD="secret"
ENV
  
  # 레거시 로컬 저장소 경로 생성 및 스냅샷 목업
  local legacy_local_dir="${TEST_ROOT}/var/restic-local"
  mkdir -p "$legacy_local_dir"
  echo "fake-local-repo" > "${legacy_local_dir}/config"

  # restic copy 및 snapshots 스텁 처리
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$*" in
      *copy*) exit 0 ;;
      *) exit 0 ;;
    esac
  '

  # --legacy-dir 옵션을 명시적으로 주거나 환경변수를 활용하여 마이그레이션 기동
  run main upgrade --legacy-dir "$legacy_local_dir"
  [ "$status" -eq 0 ]
  
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" == *"copy"* ]]
  [[ "$output" == *"--from-repo ${legacy_local_dir}"* ]]
}

@test "cmd_upgrade handles backup.env with multiline notification body" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-bucket/host"
export RESTIC_PASSWORD="secret"
export BACKUP_NOTIFICATION_BODY_SUCCESS='[백업 성공] Restic 백업 정상 완료 보고
----------------------------------------
- 대상 호스트: ${HOSTNAME}
- 실행 프로파일: ${PROFILE_NAME}
----------------------------------------'
ENV

  run main upgrade
  [ "$status" -eq 0 ]

  run config_get "BACKUP_NOTIFICATION_BODY_SUCCESS" "$BACKUP_ENV_FILE"
  [[ "$output" == *"[백업 성공]"* ]]
  [[ "$output" == *"대상 호스트"* ]]
}

@test "cmd_upgrade creates a backup copy of backup.env and migrates legacy variables" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export BACKUP_EXCLUDE_PATHS="/tmp/*"
export RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-bucket/host"
export RESTIC_PASSWORD="secret"
ENV

  source "${BATS_TEST_DIRNAME}/../backup.sh"
  
  run main upgrade
  [ "$status" -eq 0 ]

  # 백업본 생성 확인
  local backups
  backups=$(find "$RESTIC_ETC_DIR" -name "backup.env.*.bak")
  [ -n "$backups" ]
  
  # 백업본 권한 확인
  local backup_file
  for backup_file in $backups; do
    local perm
    perm=$(stat -c "%a" "$backup_file")
    [ "$perm" = "600" ]
  done

  # 신규 포맷으로 변환 확인 (BACKUP_EXCLUDE_PATHS -> BACKUP_EXCLUDES)
  run config_get "BACKUP_EXCLUDES" "$BACKUP_ENV_FILE"
  [ "$output" = "/tmp/*" ]
}

@test "/data/backup is guaranteed with 700 permissions upon script initialization" {
  local target_dir="${TEST_ROOT}/data/backup"
  [ ! -d "$target_dir" ]

  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-bucket/host"
export RESTIC_PASSWORD="secret"
ENV

  source "${BATS_TEST_DIRNAME}/../backup.sh"
  run require_backup_env
  [ "$status" -eq 0 ]

  [ -d "$target_dir" ]
  local perm
  perm=$(stat -c "%a" "$target_dir")
  [ "$perm" = "700" ]
}

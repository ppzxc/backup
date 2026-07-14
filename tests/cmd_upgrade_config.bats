#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
}

@test "cmd_upgrade_config fails when backup.env is missing" {
  run main upgrade-config
  [ "$status" -eq 1 ]
  [[ "$output" == *"설정 파일이 존재하지 않습니다"* ]]
}

@test "cmd_upgrade_config completes successfully when there is no legacy local repository" {
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-bucket/host"
export RESTIC_PASSWORD="secret"
ENV

  run main upgrade-config
  [ "$status" -eq 0 ]
  [[ "$output" == *"이관할 로컬 데이터가 없습니다"* ]]
}

@test "cmd_upgrade_config performs migration from local repo to remote repo when legacy local repo exists" {
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
  run main upgrade-config --legacy-dir "$legacy_local_dir"
  [ "$status" -eq 0 ]
  
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" == *"copy"* ]]
  [[ "$output" == *"--from-repo ${legacy_local_dir}"* ]]
}

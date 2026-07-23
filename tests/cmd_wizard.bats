#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  # Simulate an already-installed package so cmd_wizard's install-check branch
  # is skipped. self_install_copy() copies "$0", but the `bash -c '...'`
  # invocation idiom below has no argv[0], so $0 is literally "bash" — not a
  # real file to copy. Exercising that path isn't this test's job (it only
  # asserts on the setting/init/schedule orchestration), so we pre-create the
  # install path the same way Task 10 pre-created $RESTIC_ETC_DIR locally.
  : > "$BACKUP_SCRIPT_INSTALL_PATH"
  # cmd_wizard's install-check now looks at whether restic/rclone/resticprofile
  # actually exist, not just this marker file (see backup.sh cmd_wizard) — so
  # resticprofile's own install path must look "already installed" too,
  # otherwise install_resticprofile would try a real curl download here.
  export RESTICPROFILE_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/resticprofile"
  mkdir -p "$(dirname "$RESTICPROFILE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\ntrue\n' > "$RESTICPROFILE_INSTALL_PATH"
  chmod +x "$RESTICPROFILE_INSTALL_PATH"
  stub_command "install" 'cp "${@: -2:1}" "${@: -1}"'
  stub_command "ssh-keygen" '
    keyfile=""
    while [[ $# -gt 0 ]]; do
      if [[ "$1" == "-f" ]]; then keyfile="$2"; fi
      shift
    done
    echo "fake-private-key" > "$keyfile"
    echo "ssh-ed25519 AAAAFAKEKEY test@stub" > "${keyfile}.pub"
  '
  stub_command "restic" 'case "$1" in snapshots) exit 1 ;; init) exit 0 ;; esac'
  stub_command "rclone" 'exit 0'
  stub_command "systemctl" 'true'
  stub_command "resticprofile" 'true'
}

@test "wizard walks through sftp setup end to end and writes backup.env" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\ny\n\n\n" | cmd_wizard
  '
  echo "WIZARD_STATUS: $status"
  echo "WIZARD_OUTPUT: $output"
  [ "$status" -eq 0 ]
  [[ "$output" == *"ssh-ed25519"* ]]
  [[ "$output" == *"저장소 위치:"* ]]
  [ -f "$BACKUP_ENV_FILE" ]
  run config_get "host" "$BACKUP_ENV_FILE"
  [ "$output" = "1.2.3.4" ]
}






@test "wizard's sftp prompts stay in sync with backend_sftp_resolve's field set" {
  local -A cli=() env=() file=() fields=()
  backend_sftp_resolve cli env file fields
  local expected actual
  expected=$(printf '%s\n' "${!fields[@]}" | sort)
  actual=$(declare -f cmd_wizard | grep 'setting_args+=(--host' \
    | grep -oE -- '--[a-z-]+' | sed 's/^--//; s/-/_/g' | sort)
  [ "$actual" = "$expected" ]
}

@test "wizard's s3 prompts stay in sync with backend_s3_resolve's field set" {
  local -A cli=() env=() file=() fields=()
  backend_s3_resolve cli env file fields
  local expected actual
  expected=$(printf '%s\n' "${!fields[@]}" | sort)
  actual=$(declare -f cmd_wizard | grep 'setting_args+=(--endpoint' \
    | grep -oE -- '--[a-z-]+' | sed 's/^--//; s/-/_/g' | sort)
  [ "$actual" = "$expected" ]
}

@test "wizard answering no at the confirm step cancels without applying settings" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\nn\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [ ! -f "$BACKUP_ENV_FILE" ]
  [[ "$output" == *"설정을 취소했습니다"* ]]
}

@test "wizard re-prompts instead of dying on an invalid backend choice" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "9\n2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  [[ "$output" == *"1 또는 2를 입력하세요"* ]]
}

@test "wizard re-prompts on an empty required field instead of accepting it" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [[ "$output" == *"값을 입력해야 합니다"* ]]
  run config_get "host" "$BACKUP_ENV_FILE"
  [ "$output" = "1.2.3.4" ]
}

@test "wizard re-prompts on an invalid port and accepts the default on retry" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\nabc\n\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [[ "$output" == *"port must be numeric"* ]]
  run config_get "port" "$BACKUP_ENV_FILE"
  [ "$output" = "22" ]
}

@test "wizard shows the port default inline instead of a separate sentence" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [[ "$output" == *"[22]"* ]]
}

@test "wizard prompts don't leak raw internal field names" {
  local body
  body=$(declare -f cmd_wizard)
  [[ "$body" != *"NAS_IP:"* ]]
  [[ "$body" != *"PORT:"* ]]
  [[ "$body" != *"USER:"* ]]
  [[ "$body" != *"S3_ENDPOINT:"* ]]
  [[ "$body" != *"BUCKET:"* ]]
  [[ "$body" != *"ACCESS_KEY:"* ]]
  [[ "$body" != *"SECRET_KEY:"* ]]
}

@test "wizard re-installs packages when restic/rclone are missing, even though the script copy marker already exists" {
  # 실사용에서 발견된 버그: 이전 실행이 스크립트만 복사해두고 실제
  # restic/rclone 설치는 안 됐거나 이후 지워진 경우, $BACKUP_SCRIPT_INSTALL_PATH
  # 마커만 보고 넘어가면 cmd_install이 다시 실행되지 않는다.
  rm -f "${STUB_BIN}/restic" "${STUB_BIN}/rclone"
  stub_command "curl" 'echo "curl $*" >> "'"${STUB_BIN}"'/curl.calls"; exit 1'

  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [[ "$output" == *"패키지를 설치합니다"* ]]
  run cat "${STUB_BIN}/curl.calls"
  [[ "$output" == *"restic"* ]]
}

@test "wizard skips the install step when restic/rclone/resticprofile are all already present" {
  stub_command "curl" 'echo "curl should not run" >> "'"${STUB_BIN}"'/curl.calls"; exit 1'

  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [[ "$output" != *"패키지를 설치합니다"* ]]
  [ ! -f "${STUB_BIN}/curl.calls" ]
}

@test "wizard configures custom targets and excludes default targets if requested" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\nn\n/var/www\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  run config_get "targets" "$BACKUP_ENV_FILE"
  [ "$output" = "/var/www" ]
}

@test "wizard prompts for a custom folder name after sftp connection info" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\nmy-nas-box\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  run config_get "profile_name" "$BACKUP_ENV_FILE"
  [ "$output" = "my-nas-box" ]
  run config_get "RESTIC_REPOSITORY" "$BACKUP_ENV_FILE"
  [ "$output" = "rclone:syno_backup:/backup/my-nas-box" ]
}

@test "wizard uses backup as default folder name when Enter is pressed at the profile-name prompt" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  run config_get "RESTIC_REPOSITORY" "$BACKUP_ENV_FILE"
  [ "$output" = "rclone:syno_backup:/backup/backup" ]
}

@test "wizard shows folder name in confirm summary" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\nmy-nas-box\nrepo-pass\n\n\n\n\n\n\nn\ny\n\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [[ "$output" == *"my-nas-box"* ]]
}

@test "wizard configures audit report settings when requested" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\ny\n임꺽정\n정보보안부장 CISO\n60\nn\ny\n\nn\ny\n" | cmd_wizard
  '
  echo "STATUS: $status"
  echo "OUTPUT: $output"
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  run config_get "audit_tester" "$BACKUP_ENV_FILE"
  [ "$output" = "임꺽정" ]
  run config_get "audit_ciso" "$BACKUP_ENV_FILE"
  [ "$output" = "정보보안부장 CISO" ]
  run config_get "audit_rto" "$BACKUP_ENV_FILE"
  [ "$output" = "60" ]
}

@test "wizard configures DB backup settings when requested" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\n\nrepo-pass\n\n\nn\ny\nmariadb\n\n7\n4\n12\n\ny\n\nn\ny\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  run config_get "db_type" "$BACKUP_ENV_FILE"
  [ "$output" = "mariadb" ]
  run config_get "db_command" "$BACKUP_ENV_FILE"
  [ "$output" = "mariadb-dump --all-databases --single-transaction --quick --order-by-primary" ]
  run config_get "keep_db_daily" "$BACKUP_ENV_FILE"
  [ "$output" = "7" ]
  run config_get "db_schedule" "$BACKUP_ENV_FILE"
  [[ "$output" == *"03:00:00" ]]
}

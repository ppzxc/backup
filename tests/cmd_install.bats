#!/usr/bin/env bats

load test_helper.bash

setup() {
  export RESTICPROFILE_SHA256
  RESTICPROFILE_SHA256=$(sha256sum "${BATS_TEST_DIRNAME}/fixtures/resticprofile-fake.tar.gz" | awk '{print $1}')
  export RESTICPROFILE_URL="http://fixture.invalid/resticprofile.tar.gz"
  export RESTIC_SHA256
  RESTIC_SHA256=$(sha256sum "${BATS_TEST_DIRNAME}/fixtures/restic-fake.bz2" | awk '{print $1}')
  export RESTIC_URL="http://fixture.invalid/restic.bz2"
  export RCLONE_SHA256
  RCLONE_SHA256=$(sha256sum "${BATS_TEST_DIRNAME}/fixtures/rclone-fake.zip" | awk '{print $1}')
  export RCLONE_URL="http://fixture.invalid/rclone.zip"
  export RCLONE_VERSION="1.74.3"
  setup_backup_sh_env
  export RESTICPROFILE_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/resticprofile"
  export RESTIC_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/restic"
  export RCLONE_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/rclone"
  # 실제 install(1)은 -m 모드를 무조건 강제 적용한다. bz2/zip에서 python3로
  # 풀어낸 파일은 원본에 실행권한 메타데이터가 없을 수 있으므로, 스텁도 -m을
  # 반영해야 실제 동작과 같아진다.
  stub_command "install" '
    mode="" args=("$@")
    for ((i=0; i<${#args[@]}; i++)); do
      if [[ "${args[$i]}" == "-m" ]]; then mode="${args[$((i+1))]}"; fi
    done
    cp "${@: -2:1}" "${@: -1}"
    [[ -n "$mode" ]] && chmod "$mode" "${@: -1}"
    echo "install $*" >> "'"${STUB_BIN}"'/install.calls"
  '
  # 목적지가 아니라 요청 URL(마지막 인자)로 어떤 픽스처를 내려줄지 구분한다.
  stub_command "curl" '
    dest="" url=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    url="${*: -1}"
    case "$url" in
      *resticprofile*) cp "'"${BATS_TEST_DIRNAME}"'/fixtures/resticprofile-fake.tar.gz" "$dest" ;;
      *restic.bz2*) cp "'"${BATS_TEST_DIRNAME}"'/fixtures/restic-fake.bz2" "$dest" ;;
      *rclone.zip*) cp "'"${BATS_TEST_DIRNAME}"'/fixtures/rclone-fake.zip" "$dest" ;;
    esac
  '
}

@test "cmd_install installs restic/rclone/resticprofile, self-copies, and creates the restic dir" {
  run cmd_install
  [ "$status" -eq 0 ]
  [ -x "$RESTIC_INSTALL_PATH" ]
  [ -x "$RCLONE_INSTALL_PATH" ]
  [ -x "$RESTICPROFILE_INSTALL_PATH" ]
  run "$RESTIC_INSTALL_PATH"
  [[ "$output" == *"fake-restic"* ]]
  run "$RCLONE_INSTALL_PATH"
  [[ "$output" == *"fake-rclone"* ]]
  [ -d "$RESTIC_ETC_DIR" ]
  perm=$(stat -c '%a' "$RESTIC_ETC_DIR")
  [ "$perm" = "700" ]
  [ -f "$BACKUP_SCRIPT_INSTALL_PATH" ]
}

@test "cmd_install --dry-run makes no changes" {
  run cmd_install --dry-run
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/install.calls" ]
  [ ! -e "$RESTIC_INSTALL_PATH" ]
  [ ! -e "$RCLONE_INSTALL_PATH" ]
  [ ! -d "$RESTIC_ETC_DIR" ]
  [ ! -e "$BACKUP_SCRIPT_INSTALL_PATH" ]
  [[ "$output" == *"dry-run"* ]]
}

@test "cmd_install does not overwrite an existing install without --force" {
  mkdir -p "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"
  echo "old-content" > "$BACKUP_SCRIPT_INSTALL_PATH"
  run cmd_install
  [ "$status" -eq 0 ]
  run cat "$BACKUP_SCRIPT_INSTALL_PATH"
  [ "$output" = "old-content" ]
}

@test "cmd_install --force overwrites an existing install" {
  mkdir -p "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"
  echo "old-content" > "$BACKUP_SCRIPT_INSTALL_PATH"
  run cmd_install --force
  [ "$status" -eq 0 ]
  run cat "$BACKUP_SCRIPT_INSTALL_PATH"
  [ "$output" != "old-content" ]
}

@test "cmd_install downloads, verifies checksum, and installs resticprofile" {
  run cmd_install
  [ "$status" -eq 0 ]
  [ -x "$RESTICPROFILE_INSTALL_PATH" ]
  run "$RESTICPROFILE_INSTALL_PATH"
  [[ "$output" == *"fake-resticprofile"* ]]
}

@test "cmd_install dies when the downloaded resticprofile checksum does not match" {
  stub_command "curl" '
    dest=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    echo "corrupted content, not the real tarball" > "$dest"
  '
  run cmd_install
  [ "$status" -eq 1 ]
  [[ "$output" == *"체크섬"* ]]
  [ ! -e "$RESTICPROFILE_INSTALL_PATH" ]
}

@test "cmd_install skips the resticprofile download when already installed" {
  mkdir -p "$(dirname "$RESTICPROFILE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\necho already-here\n' > "$RESTICPROFILE_INSTALL_PATH"
  chmod +x "$RESTICPROFILE_INSTALL_PATH"
  mkdir -p "$(dirname "$RESTIC_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\ntrue\n' > "$RESTIC_INSTALL_PATH"
  chmod +x "$RESTIC_INSTALL_PATH"
  mkdir -p "$(dirname "$RCLONE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\ntrue\n' > "$RCLONE_INSTALL_PATH"
  chmod +x "$RCLONE_INSTALL_PATH"
  stub_command "curl" 'echo "curl should not run" >&2; exit 1'
  run cmd_install
  [ "$status" -eq 0 ]
  run "$RESTICPROFILE_INSTALL_PATH"
  [[ "$output" == "already-here" ]]
}

@test "cmd_install downloads, verifies checksum, and installs restic (bz2)" {
  run cmd_install
  [ "$status" -eq 0 ]
  [ -x "$RESTIC_INSTALL_PATH" ]
  run "$RESTIC_INSTALL_PATH"
  [[ "$output" == *"fake-restic"* ]]
}

@test "cmd_install dies when the downloaded restic checksum does not match" {
  stub_command "curl" '
    dest="" url=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    url="${*: -1}"
    case "$url" in
      *resticprofile*) cp "'"${BATS_TEST_DIRNAME}"'/fixtures/resticprofile-fake.tar.gz" "$dest" ;;
      *restic.bz2*) echo "corrupted" > "$dest" ;;
      *rclone.zip*) cp "'"${BATS_TEST_DIRNAME}"'/fixtures/rclone-fake.zip" "$dest" ;;
    esac
  '
  run cmd_install
  [ "$status" -eq 1 ]
  [[ "$output" == *"restic 체크섬 불일치"* ]]
  [ ! -e "$RESTIC_INSTALL_PATH" ]
}

@test "cmd_install skips the restic download when already installed" {
  mkdir -p "$(dirname "$RESTIC_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\necho already-here\n' > "$RESTIC_INSTALL_PATH"
  chmod +x "$RESTIC_INSTALL_PATH"
  mkdir -p "$(dirname "$RCLONE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\ntrue\n' > "$RCLONE_INSTALL_PATH"
  chmod +x "$RCLONE_INSTALL_PATH"
  mkdir -p "$(dirname "$RESTICPROFILE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\ntrue\n' > "$RESTICPROFILE_INSTALL_PATH"
  chmod +x "$RESTICPROFILE_INSTALL_PATH"
  stub_command "curl" 'echo "curl should not run for restic" >&2; exit 1'
  run cmd_install
  [ "$status" -eq 0 ]
  run "$RESTIC_INSTALL_PATH"
  [[ "$output" == "already-here" ]]
}

@test "cmd_install downloads, verifies checksum, and installs rclone (zip)" {
  run cmd_install
  [ "$status" -eq 0 ]
  [ -x "$RCLONE_INSTALL_PATH" ]
  run "$RCLONE_INSTALL_PATH"
  [[ "$output" == *"fake-rclone"* ]]
}

@test "cmd_install dies when the downloaded rclone checksum does not match" {
  stub_command "curl" '
    dest="" url=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    url="${*: -1}"
    case "$url" in
      *resticprofile*) cp "'"${BATS_TEST_DIRNAME}"'/fixtures/resticprofile-fake.tar.gz" "$dest" ;;
      *restic.bz2*) cp "'"${BATS_TEST_DIRNAME}"'/fixtures/restic-fake.bz2" "$dest" ;;
      *rclone.zip*) echo "corrupted" > "$dest" ;;
    esac
  '
  run cmd_install
  [ "$status" -eq 1 ]
  [[ "$output" == *"rclone 체크섬 불일치"* ]]
  [ ! -e "$RCLONE_INSTALL_PATH" ]
}

@test "cmd_install skips the rclone download when already installed" {
  mkdir -p "$(dirname "$RCLONE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\necho already-here\n' > "$RCLONE_INSTALL_PATH"
  chmod +x "$RCLONE_INSTALL_PATH"
  mkdir -p "$(dirname "$RESTIC_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\ntrue\n' > "$RESTIC_INSTALL_PATH"
  chmod +x "$RESTIC_INSTALL_PATH"
  mkdir -p "$(dirname "$RESTICPROFILE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\ntrue\n' > "$RESTICPROFILE_INSTALL_PATH"
  chmod +x "$RESTICPROFILE_INSTALL_PATH"
  stub_command "curl" 'echo "curl should not run for rclone" >&2; exit 1'
  run cmd_install
  [ "$status" -eq 0 ]
  run "$RCLONE_INSTALL_PATH"
  [[ "$output" == "already-here" ]]
}

@test "install_binary fails with checksum mismatch" {
  run install_binary "testtool" "1.0.0" "http://fixture.invalid/restic.bz2" "INCORRECTSHA" "${TEST_ROOT}/bin/testtool" "bz2"
  [ "$status" -eq 1 ]
  [[ "$output" == *"체크섬 불일치"* ]]
}

@test "install_binary fails when curl download fails" {
  stub_command "curl" 'echo "curl failed" >&2; exit 22'
  run install_binary "testtool" "1.0.0" "http://fixture.invalid/failed" "EXPECTEDSHA" "${TEST_ROOT}/bin/testtool" "bz2"
  [ "$status" -eq 1 ]
  [[ "$output" == *"다운로드 실패"* ]]
}


#!/usr/bin/env bats

load test_helper.bash

setup() {
  export RESTICPROFILE_SHA256
  RESTICPROFILE_SHA256=$(sha256sum "${BATS_TEST_DIRNAME}/fixtures/resticprofile-fake.tar.gz" | awk '{print $1}')
  export RESTICPROFILE_URL="http://fixture.invalid/resticprofile.tar.gz"
  setup_backup_sh_env
  export RESTICPROFILE_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/resticprofile"
  stub_command "dnf" 'echo "dnf $*" >> "'"${STUB_BIN}"'/dnf.calls"'
  stub_command "install" 'echo "install $*" >> "'"${STUB_BIN}"'/install.calls"; cp "${@: -2:1}" "${@: -1}"'
  stub_command "curl" '
    dest=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    cp "'"${BATS_TEST_DIRNAME}"'/fixtures/resticprofile-fake.tar.gz" "$dest"
  '
}

@test "cmd_install installs packages, self-copies, and creates the restic dir" {
  run cmd_install
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/dnf.calls"
  [[ "$output" == *"install -y epel-release"* ]]
  [[ "$output" == *"install -y restic rclone"* ]]
  [ -d "$RESTIC_ETC_DIR" ]
  perm=$(stat -c '%a' "$RESTIC_ETC_DIR")
  [ "$perm" = "700" ]
  [ -f "$BACKUP_SCRIPT_INSTALL_PATH" ]
}

@test "cmd_install --dry-run makes no changes" {
  run cmd_install --dry-run
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/dnf.calls" ]
  [ ! -f "${STUB_BIN}/install.calls" ]
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
  stub_command "curl" '
    # 마지막 인자가 -o 뒤에 오는 목적지 경로
    dest=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    cp "'"${BATS_TEST_DIRNAME}"'/fixtures/resticprofile-fake.tar.gz" "$dest"
  '
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
  stub_command "curl" 'echo "curl should not run" >&2; exit 1'
  run cmd_install
  [ "$status" -eq 0 ]
  run "$RESTICPROFILE_INSTALL_PATH"
  [[ "$output" == "already-here" ]]
}

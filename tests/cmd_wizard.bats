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
  stub_command "dnf" 'true'
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
  stub_command "systemctl" 'true'
}

@test "wizard walks through sftp setup end to end and writes backup.env" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\nrepo-pass\n\ny\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  grep -q 'RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"' "$BACKUP_ENV_FILE"
  [[ "$output" == *"ssh-ed25519"* ]]
  [[ "$output" == *"저장소 위치:"* ]]
}

@test "wizard answering no at the confirm step cancels without applying settings" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\nrepo-pass\nn\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [ ! -f "$BACKUP_ENV_FILE" ]
  [[ "$output" == *"설정을 취소했습니다"* ]]
}

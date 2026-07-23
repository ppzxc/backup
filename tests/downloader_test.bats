#!/usr/bin/env bats

setup() {
  TEST_DIR="$(mktemp -d)"
  export BACKUP_BIN="${TEST_DIR}/bin/backup"
}

teardown() {
  rm -rf "$TEST_DIR"
}

@test "backup.sh installs binary from target/debug/backup if target bin missing" {
  mkdir -p "${TEST_DIR}/target/debug"
  cat <<'EOF' > "${TEST_DIR}/target/debug/backup"
#!/usr/bin/env bash
echo "MOCK_RUST_BIN args: $@"
EOF
  chmod +x "${TEST_DIR}/target/debug/backup"

  cd "$TEST_DIR"
  cp "$BATS_TEST_DIRNAME/../backup.sh" ./backup.sh
  chmod +x ./backup.sh

  run ./backup.sh run --profile myprof
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_BIN" ]
  [[ "$output" =~ "Installing backup Rust binary" ]]
  [[ "$output" =~ "MOCK_RUST_BIN args: run --profile myprof" ]]
}

@test "backup.sh directly executes BACKUP_BIN if already installed" {
  mkdir -p "$(dirname "$BACKUP_BIN")"
  cat <<'EOF' > "$BACKUP_BIN"
#!/usr/bin/env bash
echo "EXISTING_RUST_BIN args: $@"
EOF
  chmod +x "$BACKUP_BIN"

  run "$BATS_TEST_DIRNAME/../backup.sh" status
  [ "$status" -eq 0 ]
  [[ "$output" =~ "EXISTING_RUST_BIN args: status" ]]
  [[ ! "$output" =~ "Installing backup Rust binary" ]]
}

@test "backup.sh fails gracefully when Rust binary is not found" {
  mkdir -p "${TEST_DIR}/empty"
  cd "${TEST_DIR}/empty"
  cp "$BATS_TEST_DIRNAME/../backup.sh" ./backup.sh
  chmod +x ./backup.sh

  run ./backup.sh run
  [ "$status" -eq 1 ]
  [[ "$output" =~ "Rust binary not found" ]]
}

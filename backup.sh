#!/usr/bin/env bash
set -euo pipefail

# shellcheck disable=SC2034
BACKUP_SCRIPT_VERSION="0.0.68"

BACKUP_BIN="${BACKUP_BIN:-/usr/local/sbin/backup}"

install_backup_binary() {
  if [[ -f "$BACKUP_BIN" ]]; then
    return 0
  fi
  echo "Installing backup Rust binary to $BACKUP_BIN..."
  mkdir -p "$(dirname "$BACKUP_BIN")"
  if [[ -f "./target/debug/backup" ]]; then
    cp "./target/debug/backup" "$BACKUP_BIN"
  elif [[ -f "./target/release/backup" ]]; then
    cp "./target/release/backup" "$BACKUP_BIN"
  else
    echo "Rust binary not found. Please run 'cargo build' first." >&2
    exit 1
  fi
  chmod 755 "$BACKUP_BIN"
}

install_backup_binary
exec "$BACKUP_BIN" "$@"

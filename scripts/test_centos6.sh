#!/bin/sh
set -eu

ROOT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
IMAGE=${BACKUP_CENTOS6_IMAGE:-backup-centos6-compat:test}
BINARY="$ROOT_DIR/target/x86_64-unknown-linux-musl/release/backup"
ARTIFACT_DIR=$(mktemp -d "${TMPDIR:-/tmp}/backup-centos6-artifacts.XXXXXX")

cleanup() {
  rm -rf "$ARTIFACT_DIR"
}
trap cleanup EXIT HUP INT TERM

if ! rustup target list --installed | grep -qx 'x86_64-unknown-linux-musl'; then
  rustup target add x86_64-unknown-linux-musl
fi

cargo build --release --target x86_64-unknown-linux-musl --bin backup
docker build --tag "$IMAGE" --file "$ROOT_DIR/docker/Dockerfile.centos6" "$ROOT_DIR"

printf '%s\n' \
  '#!/bin/sh' \
  'target=' \
  'previous=' \
  'for arg in "$@"; do' \
  '  if [ "$previous" = "--target" ]; then target=$arg; fi' \
  '  previous=$arg' \
  'done' \
  'if [ -n "$target" ]; then mkdir -p "$target"; printf "centos6 smoke payload\\n" >"$target/restored.txt"; fi' \
  'printf "%s version\\n" "$0"' >"$ARTIFACT_DIR/restic"
chmod 700 "$ARTIFACT_DIR/restic"

printf '%s\n' \
  '#!/bin/sh' \
  'printf "resticprofile version\\n"' >"$ARTIFACT_DIR/resticprofile"
chmod 700 "$ARTIFACT_DIR/resticprofile"

printf '%s\n' \
  '#!/bin/sh' \
  'printf "rclone version\\n"' >"$ARTIFACT_DIR/rclone"
chmod 700 "$ARTIFACT_DIR/rclone"

for binary in restic rclone resticprofile; do
  chmod 700 "$ARTIFACT_DIR/$binary"
done
(cd "$ARTIFACT_DIR" && sha256sum restic rclone resticprofile >SHA256SUMS)

docker run --rm --platform linux/amd64 \
  -v "$BINARY:/usr/local/bin/backup:ro" \
  -v "$ARTIFACT_DIR:/work/artifacts:ro" \
  "$IMAGE" sh -ceu '
    export PATH=/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/bin
    test "$(uname -m)" = x86_64
    test ! -e /usr/bin/systemctl
    test -x /usr/sbin/crond
    test -x /usr/sbin/ntpd
    test -x /usr/sbin/ntpq
    /usr/local/bin/backup --version | grep -q "backup "

    mkdir -p /tmp/backup-centos6
    mkdir -p /tmp/backup-centos6/source /tmp/backup-centos6/repository
    printf "centos6 smoke payload\\n" >/tmp/backup-centos6/source/input.txt
    printf "centos6-smoke-password\\n" >/tmp/backup-centos6/password
    chmod 600 /tmp/backup-centos6/password
    cat >/tmp/backup-centos6/profiles.yaml <<EOF
version: "2"
profiles:
  default:
    insecure-tls: true
  primary:
    repository: /tmp/backup-centos6/repository
    password-file: /tmp/backup-centos6/password
  daily:
    inherit: primary
    initialize: true
    backup:
      source: [/tmp/backup-centos6/source]
      tag: [daily, backup-profile:daily]
EOF
    chmod 600 /tmp/backup-centos6/profiles.yaml

    /usr/local/bin/backup setup dependencies \
      --dependency-archive-dir /work/artifacts >/tmp/backup-centos6/dependencies.out
    grep -q "verified and installed" /tmp/backup-centos6/dependencies.out
    test -x /usr/local/bin/restic
    test -x /usr/local/bin/rclone
    test -x /usr/local/bin/resticprofile

    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml run
    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml restore \
      --snapshot centos6-smoke --target /tmp/backup-centos6/restore
    test -s /tmp/backup-centos6/restore/restored.txt

    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml doctor \
      >/tmp/backup-centos6/doctor.out 2>/tmp/backup-centos6/doctor.err || test $? -eq 1
    grep -q "time_sync_method" /tmp/backup-centos6/doctor.out
    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml report time-sync \
      --file /tmp/backup-centos6/report --format json
    test -s /tmp/backup-centos6/report.json

    ssh-keyscan -T 1 -p 22 localhost >/tmp/backup-centos6/hostkeys 2>/dev/null || true
    ssh-keygen -t rsa -N "" -f /tmp/backup-centos6/id_rsa >/dev/null
    : >/tmp/backup-centos6/known_hosts
    cat >/tmp/backup-centos6/sftp-profiles.yaml <<EOF
version: "2"
profiles:
  primary:
    repository: sftp:backup@localhost:/backup
    password-file: /tmp/backup-centos6/password
    option:
      sftp.args: "-i /tmp/backup-centos6/id_rsa -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new"
EOF
    chmod 600 /tmp/backup-centos6/sftp-profiles.yaml
    /usr/local/bin/backup --profiles /tmp/backup-centos6/sftp-profiles.yaml snapshots
    grep -q "StrictHostKeyChecking=yes" /tmp/backup-centos6/sftp-profiles.yaml

    /usr/sbin/crond
    sleep 1
    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml schedule status \
      | grep -q "inactive (cron)"
    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml schedule enable \
      | grep -q "with cron"
    crontab -l | grep -q "# backup-pipeline"
    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml schedule disable \
      | grep -q "with cron"
    ! crontab -l 2>/dev/null | grep -q "# backup-pipeline"

    test -s /tmp/backup-centos6/id_rsa
    test -s /tmp/backup-centos6/id_rsa.pub
    ntpq --version 2>&1 | grep -q "ntpq"
  '

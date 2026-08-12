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

for binary in restic rclone resticprofile; do
  printf '#!/bin/sh\nprintf "%s version\\n" "$0"\n' "$binary" >"$ARTIFACT_DIR/$binary"
  chmod 700 "$ARTIFACT_DIR/$binary"
done
(cd "$ARTIFACT_DIR" && sha256sum restic rclone resticprofile >SHA256SUMS)

docker run --rm --platform linux/amd64 \
  -v "$BINARY:/usr/local/bin/backup:ro" \
  -v "$ARTIFACT_DIR:/work/artifacts:ro" \
  "$IMAGE" sh -ceu '
    test "$(uname -m)" = x86_64
    test ! -e /usr/bin/systemctl
    test -x /usr/sbin/crond
    test -x /usr/sbin/ntpd
    test -x /usr/sbin/ntpq
    /usr/local/bin/backup --version | grep -q "backup "

    mkdir -p /tmp/backup-centos6
    printf "%s\n" "version: \"2\"" "profiles: {}" >/tmp/backup-centos6/profiles.yaml
    chmod 600 /tmp/backup-centos6/profiles.yaml

    /usr/local/bin/backup setup dependencies \
      --dependency-archive-dir /work/artifacts >/tmp/backup-centos6/dependencies.out
    grep -q "verified and installed" /tmp/backup-centos6/dependencies.out
    test -x /usr/local/bin/restic
    test -x /usr/local/bin/rclone
    test -x /usr/local/bin/resticprofile

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

    ssh-keygen -t rsa -N "" -f /tmp/backup-centos6/id_rsa >/dev/null
    test -s /tmp/backup-centos6/id_rsa
    test -s /tmp/backup-centos6/id_rsa.pub
    ntpq --version 2>&1 | grep -q "ntpq"
  '

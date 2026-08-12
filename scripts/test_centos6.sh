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
  'exec /work/artifacts/restic.real "$@"' \
  'target=' \
  'printf "%s version\\n" "$0"' >"$ARTIFACT_DIR/restic"
chmod 700 "$ARTIFACT_DIR/restic"

if command -v restic >/dev/null 2>&1; then
  cp "$(command -v restic)" "$ARTIFACT_DIR/restic.real"
else
  curl -fsSL \
    https://github.com/restic/restic/releases/download/v0.16.4/restic_0.16.4_linux_amd64.bz2 \
    | bunzip2 >"$ARTIFACT_DIR/restic.real"
fi
chmod 700 "$ARTIFACT_DIR/restic.real"

printf '%s\n' \
  '#!/bin/sh' \
  'config=' \
  'previous=' \
  'for arg in "$@"; do' \
  '  if [ "$previous" = "--config" ]; then config=$arg; fi' \
  '  previous=$arg' \
  'done' \
  'if [ "$config" = /tmp/backup-centos6/profiles.yaml ]; then' \
  '  restic_args="-r /tmp/backup-centos6/repository --password-file /tmp/backup-centos6/password"' \
  '  case " $* " in' \
  '    *" backup "*)' \
  '      /usr/local/bin/restic $restic_args init >/dev/null 2>&1 || true' \
  '      /usr/local/bin/restic $restic_args backup --tag daily --tag backup-profile:daily /tmp/backup-centos6/source >/tmp/backup-centos6/restic-backup.out' \
  '      status=$?' \
  '      cat /tmp/backup-centos6/restic-backup.out' \
  '      printf "snapshot centos6-smoke saved\\n"' \
  '      exit $status' \
  '      ;;' \
  '  esac' \
  'fi' \
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
    test -x /usr/sbin/sshd
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
      --snapshot latest --target /tmp/backup-centos6/restore
    test -s "$(find /tmp/backup-centos6/restore -type f -name input.txt -print -quit)"

    /usr/sbin/crond
    sleep 1
    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml doctor \
      >/tmp/backup-centos6/doctor.out 2>/tmp/backup-centos6/doctor.err || test $? -eq 1
    grep -q "time_sync_method" /tmp/backup-centos6/doctor.out
    grep -q "scheduler_backend" /tmp/backup-centos6/doctor.out
    grep -q "NotApplicable" /tmp/backup-centos6/doctor.out
    /usr/local/bin/backup --profiles /tmp/backup-centos6/profiles.yaml report time-sync \
      --file /tmp/backup-centos6/report --format json
    test -s /tmp/backup-centos6/report.json
    grep -q 'time_sync_method.*ntpd' /tmp/backup-centos6/report.json
    grep -q 'scheduler_backend.*cron' /tmp/backup-centos6/report.json
    grep -q 'conf_permission.*not-applicable' /tmp/backup-centos6/report.json

    ssh-keygen -t rsa -N "" -f /tmp/backup-centos6/id_rsa >/dev/null
    useradd -m -s /bin/bash backup
    mkdir -p /home/backup/.ssh /backup
    cp /tmp/backup-centos6/id_rsa.pub /home/backup/.ssh/authorized_keys
    chown -R backup:backup /home/backup/.ssh /backup
    chmod 700 /home/backup/.ssh
    chmod 600 /home/backup/.ssh/authorized_keys
    mkdir -p /var/run/sshd
    test -f /etc/ssh/ssh_host_rsa_key || ssh-keygen -t rsa -N "" -f /etc/ssh/ssh_host_rsa_key >/dev/null
    test -f /etc/ssh/ssh_host_dsa_key || ssh-keygen -t dsa -N "" -f /etc/ssh/ssh_host_dsa_key >/dev/null
    test -f /etc/ssh/ssh_host_ecdsa_key || ssh-keygen -t ecdsa -N "" -f /etc/ssh/ssh_host_ecdsa_key >/dev/null
    /usr/sbin/sshd
    sleep 1
    ssh-keyscan -T 5 -p 22 localhost >/tmp/backup-centos6/hostkeys 2>/dev/null
    test -s /tmp/backup-centos6/hostkeys
    cp /tmp/backup-centos6/hostkeys /tmp/backup-centos6/known_hosts
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
    strict_sftp_args="-i /tmp/backup-centos6/id_rsa -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=yes -o UserKnownHostsFile=/tmp/backup-centos6/known_hosts"
    /usr/local/bin/restic -r sftp:backup@localhost:/backup \
      --password-file /tmp/backup-centos6/password \
      --option "sftp.args=$strict_sftp_args" init >/tmp/backup-centos6/sftp-init.out
    /usr/local/bin/backup --profiles /tmp/backup-centos6/sftp-profiles.yaml snapshots
    grep -q "StrictHostKeyChecking=yes" /tmp/backup-centos6/sftp-profiles.yaml

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

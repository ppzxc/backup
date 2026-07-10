#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

cleanup() {
  docker compose down -v >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "=== 컨테이너 기동 ==="
docker compose up -d
docker compose exec -T minio sh -c 'until mc alias set local http://localhost:9000 AKIAIOSFODNN7EXAMPLE wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY 2>/dev/null; do sleep 1; done'
docker compose exec -T minio mc mb -p local/restic-test

echo "=== install ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh install
  # atmoz/sftp 클라이언트 접속을 위한 ssh-keygen은 실제 RHEL에는 기본 포함되지만
  # 이 최소 rockylinux:9 이미지에는 없으므로 테스트 환경에서만 별도 설치한다.
  dnf install -y openssh-clients
'

echo "=== S3 시나리오: setting -> init -> run ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  export RESTIC_ETC_DIR=/etc/restic
  bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force
  bash backup.sh init
  bash backup.sh run
'

echo "=== S3 스냅샷 확인 ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  source /etc/restic/backup.env
  restic snapshots --json | grep -q "\"hostname\""
'

echo "=== SFTP 시나리오: setting -> 키 등록 -> init -> run ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password --force
'

# atmoz/sftp는 컨테이너 기동 시점에만 .ssh/keys/*.pub를 authorized_keys로 수집하므로,
# 기동 후에 생성된 키는 authorized_keys를 직접 만들어 등록해야 한다.
docker compose exec -T app cat /etc/restic/backup_key.pub | \
  docker compose exec -T sftp sh -c '
    mkdir -p /home/backup_restic/.ssh
    cat > /home/backup_restic/.ssh/authorized_keys
    chown -R backup_restic:users /home/backup_restic/.ssh
    chmod 700 /home/backup_restic/.ssh
    chmod 600 /home/backup_restic/.ssh/authorized_keys
  '

# backup.sh는 시놀로지 NAS의 "backup" 공유폴더를 전제로 저장소 경로를
# rclone:syno_backup:/backup/<hostname> 로 고정한다(backup.sh:282 참고).
# atmoz/sftp 컨테이너에서 이 경로에 대응하는 쓰기 가능 디렉터리를 만들어준다.
docker compose exec -T sftp sh -c '
  mkdir -p /home/backup_restic/backup
  chown backup_restic:users /home/backup_restic/backup
'

docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh init
  bash backup.sh run
'

echo "=== SFTP 스냅샷 확인 ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  source /etc/restic/backup.env
  restic snapshots --json | grep -q "\"hostname\""
'

echo "=== 모든 통합 테스트 통과 ==="

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
  bash backup.sh install --force
  # atmoz/sftp 클라이언트 접속을 위한 ssh-keygen은 실제 RHEL에는 기본 포함되지만
  # 이 최소 rockylinux:9 이미지에는 없으므로 테스트 환경에서만 별도 설치한다.
  # systemd는 resticprofile schedule/unschedule이 만든 유닛 파일을 조회하기 위해 필요하다.
  # 실제 DB 덤프 동작 검증을 위해 mariadb, postgresql 클라이언트를 함께 설치한다.
  dnf install -y openssh-clients systemd mariadb postgresql
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

echo "=== S3: schedule enable -> 생성된 유닛에 비밀값이 없는지 확인 -> disable ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh schedule enable || true
  # 이 컨테이너는 systemd가 PID 1이 아니라(entrypoint: sleep infinity) resticprofile의
  # systemctl daemon-reload/enable 호출이 "Failed to connect to bus"로 실패한다.
  # 하지만 유닛 파일 자체는 그 실패 이전에 이미 쓰여지므로(실측 확인됨), 여기서는
  # "타이머가 활성화됐는가"가 아니라 "유닛 파일에 비밀값이 새지 않는가"만 검증한다.
  unit_file=$(find /etc/systemd/system -maxdepth 1 -name "resticprofile-backup@profile-*.service")
  test -n "$unit_file"
  grep -q "ISMS Compliance" "$unit_file"
  ! grep -q "RESTIC_PASSWORD" "$unit_file"
  ! grep -q "AWS_SECRET_ACCESS_KEY" "$unit_file"
  bash backup.sh schedule disable || true
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

echo "=== SFTP: schedule enable -> 생성된 유닛에 비밀값이 없는지 확인 -> disable ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh schedule enable || true
  unit_file=$(find /etc/systemd/system -maxdepth 1 -name "resticprofile-backup@profile-*.service")
  test -n "$unit_file"
  grep -q "ISMS Compliance" "$unit_file"
  ! grep -q "RESTIC_PASSWORD" "$unit_file"
  bash backup.sh schedule disable || true
'

echo "=== 마이그레이션 시나리오: SFTP to S3 ==="
# Reset env to original SFTP setting (source of migration)
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password --force
'
# Create target bucket
docker compose exec -T minio mc mb -p local/restic-test-sftp-to-s3 || true

# Run migration
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh migrate --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-sftp-to-s3 \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --new-password pass-sftp-to-s3 --force
'
# Verify migration
docker compose exec -T app bash -c '
  set -euo pipefail
  source /etc/restic/backup.env
  restic snapshots --json | grep -q "\"hostname\""
'


echo "=== 마이그레이션 시나리오: S3 to SFTP ==="
# Reset env to original S3 setting (source of migration)
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force
'
# Setup key and directories for backup_migrate SFTP user
docker compose exec -T app cat /etc/restic/backup_key.pub | \
  docker compose exec -T sftp sh -c '
    mkdir -p /home/backup_migrate/.ssh
    cat > /home/backup_migrate/.ssh/authorized_keys
    chown -R backup_migrate:users /home/backup_migrate/.ssh
    chmod 700 /home/backup_migrate/.ssh
    chmod 600 /home/backup_migrate/.ssh/authorized_keys
    mkdir -p /home/backup_migrate/backup
    chown backup_migrate:users /home/backup_migrate/backup
  '

# Clean up destination directory
docker compose exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

# Run migration
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh migrate --backend sftp \
    --host sftp --port 22 --user backup_migrate \
    --new-password pass-s3-to-sftp --force
'
# Verify migration
docker compose exec -T app bash -c '
  set -euo pipefail
  source /etc/restic/backup.env
  restic snapshots --json | grep -q "\"hostname\""
'


echo "=== 마이그레이션 시나리오: SFTP to SFTP ==="
# Reset env to original SFTP setting (source of migration)
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password --force
'
# Clean up destination directory
docker compose exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

# Run migration to another user (backup_migrate)
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh migrate --backend sftp \
    --host sftp --port 22 --user backup_migrate \
    --new-password pass-sftp-to-sftp --force
'
# Verify migration
docker compose exec -T app bash -c '
  set -euo pipefail
  source /etc/restic/backup.env
  restic snapshots --json | grep -q "\"hostname\""
'


echo "=== 마이그레이션 시나리오: S3 to S3 ==="
# Reset env to original S3 setting (source of migration)
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force
'
# Create target bucket
docker compose exec -T minio mc mb -p local/restic-test-s3-to-s3 || true

# Run migration
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh migrate --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-s3-to-s3 \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --new-password pass-s3-to-s3 --force
'
# Verify migration
docker compose exec -T app bash -c '
  set -euo pipefail
  source /etc/restic/backup.env
  restic snapshots --json | grep -q "\"hostname\""
'

echo "=== 실제 데이터베이스 더미 데이터 주입 ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  mysql -h mariadb-5-5 -u root -ptestpass testdb -e "CREATE TABLE IF NOT EXISTS users (id INT, name VARCHAR(50)); INSERT INTO users VALUES (1, '\''user_5_5'\'');"
  mariadb -h mariadb-latest -u root -ptestpass testdb -e "CREATE TABLE IF NOT EXISTS users (id INT, name VARCHAR(50)); INSERT INTO users VALUES (2, '\''user_latest'\'');"
  PGPASSWORD=testpass psql -h postgres-13 -U postgres -d testdb -c "CREATE TABLE IF NOT EXISTS users (id INT, name VARCHAR(50)); INSERT INTO users VALUES (3, '\''user_pg'\'');"
'

echo "=== 1. 실제 MariaDB 5.5 백업 & 복구 실증 ==="
# 1차 S3 및 2차 SFTP 소산지 동시 설정
docker compose exec -T minio mc mb -p local/restic-test-primary-55 || true
docker compose exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-55 \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force \
    --db-type mysql \
    --db-command "mysqldump -h mariadb-5-5 -u root -ptestpass --all-databases --single-transaction --quick --order-by-primary" \
    --db-keep-daily 7 --db-keep-weekly 4 --db-keep-monthly 12 \
    --db-schedule "*-*-* 03:00:00"
'

docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh init
  bash backup.sh run
'

# 복구 훈련 수행 및 보고서 동시 저장 검증
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh audit --restore-drill --report-file /tmp/audit_report_55.md
  
  test -f /tmp/audit_report_55.md
  test -f /tmp/audit_report_55.json
  test -f /tmp/audit_report_55.html
  
  # 보고서에 2차 복구 결과 및 DB 검증 상태가 정상 렌더링되었는지 검증
  grep -q "2차 소산 저장소" /tmp/audit_report_55.md
  grep -q "\"secondary_recovery_results\":" /tmp/audit_report_55.json
  grep -q "데이터베이스(mysql) 복원 무결성 검증: 성공" /tmp/audit_report_55.md
  grep -q "\"db_integrity_verified\": true" /tmp/audit_report_55.json
'


echo "=== 2. 실제 MariaDB Latest 백업 & 복구 실증 ==="
# 1차 S3 및 2차 SFTP 소산지 동시 설정
docker compose exec -T minio mc mb -p local/restic-test-primary-latest || true
docker compose exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-latest \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force \
    --db-type mariadb \
    --db-command "mariadb-dump -h mariadb-latest -u root -ptestpass --all-databases --single-transaction --quick --order-by-primary" \
    --db-keep-daily 7 --db-keep-weekly 4 --db-keep-monthly 12 \
    --db-schedule "*-*-* 03:00:00"
'

docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh init
  bash backup.sh run
'

# 복구 훈련 수행 및 보고서 동시 저장 검증
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh audit --restore-drill --report-file /tmp/audit_report_latest.md
  
  test -f /tmp/audit_report_latest.md
  test -f /tmp/audit_report_latest.json
  test -f /tmp/audit_report_latest.html
  
  # 보고서에 2차 복구 결과 및 DB 검증 상태가 정상 렌더링되었는지 검증
  grep -q "2차 소산 저장소" /tmp/audit_report_latest.md
  grep -q "\"secondary_recovery_results\":" /tmp/audit_report_latest.json
  grep -q "데이터베이스(mariadb) 복원 무결성 검증: 성공" /tmp/audit_report_latest.md
  grep -q "\"db_integrity_verified\": true" /tmp/audit_report_latest.json
'


echo "=== 3. 실제 PostgreSQL Latest 백업 & 복구 실증 ==="
# 1차 S3 및 2차 SFTP 소산지 동시 설정
docker compose exec -T minio mc mb -p local/restic-test-primary-postgres || true
docker compose exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-postgres \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force \
    --db-type postgres \
    --db-command "env PGPASSWORD=testpass pg_dumpall -h postgres-13 -U postgres" \
    --db-keep-daily 7 --db-keep-weekly 4 --db-keep-monthly 12 \
    --db-schedule "*-*-* 03:00:00"
'

docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh init
  bash backup.sh run
'

# 복구 훈련 수행 및 보고서 동시 저장 검증
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh audit --restore-drill --report-file /tmp/audit_report_postgres.md
  
  test -f /tmp/audit_report_postgres.md
  test -f /tmp/audit_report_postgres.json
  test -f /tmp/audit_report_postgres.html
  
  # 보고서에 2차 복구 결과 및 DB 검증 상태가 정상 렌더링되었는지 검증
  grep -q "2차 소산 저장소" /tmp/audit_report_postgres.md
  grep -q "\"secondary_recovery_results\":" /tmp/audit_report_postgres.json
  grep -q "데이터베이스(postgres) 복원 무결성 검증: 성공" /tmp/audit_report_postgres.md
  grep -q "\"db_integrity_verified\": true" /tmp/audit_report_postgres.json
'


# 3. 기존 로컬 데이터 이관 검증 (upgrade-config)
docker compose exec -T app bash -c '
  set -euo pipefail
  
  # 임시 레거시 로컬 저장소 생성 및 스냅샷 생성
  local_repo="/tmp/legacy-local"
  rm -rf "$local_repo"
  restic init -r "$local_repo" --password-file <(echo -n "test-repo-password")
  
  # 임시 백업 데이터 생성 후 로컬 저장소에 백업 실행
  echo "legacy-data" > /tmp/legacy_file.txt
  restic -r "$local_repo" --password-file <(echo -n "test-repo-password") backup /tmp/legacy_file.txt

  
  # upgrade-config 로 이관
  bash backup.sh upgrade-config --legacy-dir "$local_repo"
  
  # 1차 원격 저장소에 로컬에서 마이그레이션된 스냅샷이 존재하는지 검증
  source /etc/restic/backup.env
  restic snapshots --json | grep -q "/tmp/legacy_file.txt"
'

echo "=== 모든 통합 테스트 통과 ==="



#!/usr/bin/env bats

# Setup helper for executing commands in the 'app' container
dexec() {
  local cmd="$1"
  if [[ "$cmd" == *"source /etc/backup/backup.env"* ]]; then
    local subcmd="${cmd#*source /etc/backup/backup.env && }"
    docker compose -f "$BATS_TEST_DIRNAME/docker-compose.yml" exec -T app bash -c "
      if [[ -f /etc/backup/backup.env ]]; then
        while IFS= read -r line || [[ -n \"\$line\" ]]; do
          [[ \"\$line\" =~ ^[[:space:]]*# ]] && continue
          [[ \"\$line\" =~ ^[[:space:]]*\$ ]] && continue
          line=\"\${line#export }\"
          if [[ \"\$line\" =~ ^([A-Za-z_][A-Za-z0-9_]*)=\'(.*)\'\$ ]]; then
            k=\"\${BASH_REMATCH[1]}\"
            v=\"\${BASH_REMATCH[2]}\"
            v=\"\${v//\\'\\'/\\'}\"
            export \"\$k\"=\"\$v\"
          elif [[ \"\$line\" =~ ^([A-Za-z_][A-Za-z0-9_]*)=\"(.*)\"\$ ]]; then
            k=\"\${BASH_REMATCH[1]}\"
            v=\"\${BASH_REMATCH[2]}\"
            v=\"\${v//\\\"\\\"/\\\"}\"
            export \"\$k\"=\"\$v\"
          elif [[ \"\$line\" =~ ^([A-Za-z_][A-Za-z0-9_]*)=(.*)\$ ]]; then
            k=\"\${BASH_REMATCH[1]}\"
            v=\"\${BASH_REMATCH[2]}\"
            export \"\$k\"=\"\$v\"
          fi
        done < /etc/backup/backup.env
      fi
      $subcmd
    "
  else
    docker compose -f "$BATS_TEST_DIRNAME/docker-compose.yml" exec -T app bash -c "$cmd"
  fi
}

# General helper for running docker compose commands in the integration folder
dc_exec() {
  docker compose -f "$BATS_TEST_DIRNAME/docker-compose.yml" "$@"
}

# Helper to register public SSH keys and configure backup folder permissions inside SFTP container
register_sftp_keys() {
  local user="$1"
  dc_exec exec -T app cat /etc/backup/backup_key.pub | \
    dc_exec exec -T sftp sh -c "
      mkdir -p /home/${user}/.ssh
      cat > /home/${user}/.ssh/authorized_keys
      chown -R ${user}:users /home/${user}/.ssh
      chmod 700 /home/${user}/.ssh
      chmod 600 /home/${user}/.ssh/authorized_keys
      mkdir -p /home/${user}/backup
      chown ${user}:users /home/${user}/backup
    "
}

setup_file() {
  # Start containers
  dc_exec up -d --build

  # Wait for MinIO and set up mc alias
  dc_exec exec -T minio sh -c \
    'until mc alias set local http://localhost:9000 AKIAIOSFODNN7EXAMPLE wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY 2>/dev/null; do sleep 1; done'

  # Run install once for the entire test suite execution
  dc_exec exec -T app bash -c "bash backup.sh install --force"
}

teardown_file() {
  dc_exec down -v
}

setup() {
  # Clean only the generated settings, keys, and profiles (preserving templates)
  dexec "rm -f /etc/backup/backup.env /etc/backup/profiles.yaml /etc/backup/backup_key /etc/backup/backup_key.pub"

  # Clean leftover systemd unit files from previous runs
  dexec "rm -f /etc/systemd/system/resticprofile-backup@*.service /etc/systemd/system/resticprofile-backup@*.timer"

  # Clean audit reports and legacy files
  dexec "rm -rf /tmp/audit_report* /tmp/legacy* /tmp/legacy_file.txt"

  # Reset SFTP home directory backup contents
  dc_exec exec -T sftp sh -c \
    "rm -rf /home/backup_restic/backup/* /home/backup_migrate/backup/* /home/backup_restic/.ssh/* /home/backup_migrate/.ssh/*"

  # Re-create MinIO buckets (clean previous state)
  dc_exec exec -T minio sh -c \
    "mc rm --recursive --force local/restic-test || true
     mc rm --recursive --force local/restic-test-sftp-to-s3 || true
     mc rm --recursive --force local/restic-test-s3-to-s3 || true
     mc rm --recursive --force local/restic-test-primary-55 || true
     mc rm --recursive --force local/restic-test-primary-latest || true
     mc rm --recursive --force local/restic-test-primary-postgres || true
     mc mb -p local/restic-test || true"
}

# 1. S3 scenario
@test "S3 scenario: setting -> init -> run -> schedule" {
  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # Verify snapshots
  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]]

  # Schedule check
  dexec "bash backup.sh schedule enable || true"

  # Find unit file and check no secrets leaked
  run dexec "find /etc/systemd/system -maxdepth 1 -name 'resticprofile-backup@*.service'"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
  local unit_file="$output"

  dexec "grep -q 'ISMS Compliance' '$unit_file'"

  run dexec "grep -q 'RESTIC_PASSWORD' '$unit_file'"
  [ "$status" -ne 0 ]

  run dexec "grep -q 'AWS_SECRET_ACCESS_KEY' '$unit_file'"
  [ "$status" -ne 0 ]

  dexec "bash backup.sh schedule disable || true"
}

# 2. SFTP scenario
@test "SFTP scenario: setting -> key setup -> init -> run -> schedule" {
  dexec "bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password --force"

  # Register public keys on sftp container
  register_sftp_keys "backup_restic"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # Verify snapshots
  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]]

  # Schedule check
  dexec "bash backup.sh schedule enable || true"

  run dexec "find /etc/systemd/system -maxdepth 1 -name 'resticprofile-backup@*.service'"
  [ "$status" -eq 0 ]
  [ -n "$output" ]
  local unit_file="$output"

  dexec "grep -q 'ISMS Compliance' '$unit_file'"

  run dexec "grep -q 'RESTIC_PASSWORD' '$unit_file'"
  [ "$status" -ne 0 ]

  dexec "bash backup.sh schedule disable || true"
}

# 3. Migration SFTP to S3
@test "Migration: SFTP to S3" {
  # 1. Setup SFTP source
  dexec "bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password --force"

  register_sftp_keys "backup_restic"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # 2. Setup target S3 bucket
  dc_exec exec -T minio mc mb -p local/restic-test-sftp-to-s3 || true

  # 3. Run migration
  dexec "bash backup.sh migrate --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-sftp-to-s3 \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --new-password pass-sftp-to-s3 --force"

  # 4. Verify migrated S3 snapshots
  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]]
}

# 4. Migration S3 to SFTP
@test "Migration: S3 to SFTP" {
  # 1. Setup S3 source
  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # 2. Setup SFTP target key and directory
  # Generate SSH key pair in app container first (since S3 source doesn't use/generate it)
  dexec "ssh-keygen -t ed25519 -N '' -f /etc/backup/backup_key"

  register_sftp_keys "backup_migrate"

  # Clean target directory
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  # 3. Run migration
  dexec "bash backup.sh migrate --backend sftp \
    --host sftp --port 22 --user backup_migrate \
    --new-password pass-s3-to-sftp --force"

  # 4. Verify migrated SFTP snapshots
  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]]
}

# 5. Migration SFTP to SFTP
@test "Migration: SFTP to SFTP" {
  # 1. Setup SFTP source
  dexec "bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password --force"

  register_sftp_keys "backup_restic"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # 2. Setup SFTP target (backup_migrate)
  register_sftp_keys "backup_migrate"

  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  # 3. Run migration
  dexec "bash backup.sh migrate --backend sftp \
    --host sftp --port 22 --user backup_migrate \
    --new-password pass-sftp-to-sftp --force"

  # 4. Verify migrated SFTP snapshots
  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]]
}

# 6. Migration S3 to S3
@test "Migration: S3 to S3" {
  # 1. Setup S3 source
  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # 2. Setup target S3 bucket
  dc_exec exec -T minio mc mb -p local/restic-test-s3-to-s3 || true

  # 3. Run migration
  dexec "bash backup.sh migrate --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-s3-to-s3 \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --new-password pass-s3-to-s3 --force"

  # 4. Verify migrated S3 snapshots
  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]]
}

# Seed DB Helper
seed_databases() {
  dexec "mysql -h mariadb-5-5 -u root -ptestpass testdb -e \"CREATE TABLE IF NOT EXISTS users (id INT, name VARCHAR(50)); INSERT INTO users VALUES (1, 'user_5_5');\""
  dexec "mariadb -h mariadb-latest -u root -ptestpass testdb -e \"CREATE TABLE IF NOT EXISTS users (id INT, name VARCHAR(50)); INSERT INTO users VALUES (2, 'user_latest');\""
  dexec "env PGPASSWORD=testpass psql -h postgres-13 -U postgres -d testdb -c \"CREATE TABLE IF NOT EXISTS users (id INT, name VARCHAR(50)); INSERT INTO users VALUES (3, 'user_pg');\""
}

# 7. MariaDB 5.5 Backup & Restore Audit
@test "Database integration: MariaDB 5.5 Backup & Dual Audit" {
  seed_databases

  # Setup S3 primary and SFTP secondary
  dc_exec exec -T minio mc mb -p local/restic-test-primary-55 || true
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-55 \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force \
    --db-type mysql \
    --db-command \"mysqldump -h mariadb-5-5 -u root -ptestpass --all-databases --single-transaction --quick --order-by-primary\" \
    --db-keep-daily 7 --db-keep-weekly 4 --db-keep-monthly 12 \
    --db-schedule \"*-*-* 03:00:00\""

  # Setup keys for SFTP (must be done after setting generates the keys)
  register_sftp_keys "backup_migrate"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # Restore Drill and Audit
  dexec "bash backup.sh audit --restore-drill --report-file /tmp/audit_report_55.md"

  # Check reports exist
  dexec "test -f /tmp/audit_report_55.md && test -f /tmp/audit_report_55.json && test -f /tmp/audit_report_55.html"

  # Verify report content
  dexec "grep -q '2차 소산 저장소' /tmp/audit_report_55.md"

  dexec "grep -q '\"secondary_recovery_results\":' /tmp/audit_report_55.json"

  dexec "grep -q '데이터베이스(mysql) 복원 무결성 검증: 성공' /tmp/audit_report_55.md"

  dexec "grep -q '\"db_integrity_verified\": true' /tmp/audit_report_55.json"
}

# 8. MariaDB Latest Backup & Restore Audit
@test "Database integration: MariaDB Latest Backup & Dual Audit" {
  seed_databases

  dc_exec exec -T minio mc mb -p local/restic-test-primary-latest || true
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-latest \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force \
    --db-type mariadb \
    --db-command \"mariadb-dump -h mariadb-latest -u root -ptestpass --all-databases --single-transaction --quick --order-by-primary\" \
    --db-keep-daily 7 --db-keep-weekly 4 --db-keep-monthly 12 \
    --db-schedule \"*-*-* 03:00:00\""

  # Setup keys for SFTP (must be done after setting generates the keys)
  register_sftp_keys "backup_migrate"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # Restore Drill and Audit
  dexec "bash backup.sh audit --restore-drill --report-file /tmp/audit_report_latest.md"

  # Check reports exist
  dexec "test -f /tmp/audit_report_latest.md && test -f /tmp/audit_report_latest.json && test -f /tmp/audit_report_latest.html"

  # Verify report content
  dexec "grep -q '2차 소산 저장소' /tmp/audit_report_latest.md"

  dexec "grep -q '\"secondary_recovery_results\":' /tmp/audit_report_latest.json"

  dexec "grep -q '데이터베이스(mariadb) 복원 무결성 검증: 성공' /tmp/audit_report_latest.md"

  dexec "grep -q '\"db_integrity_verified\": true' /tmp/audit_report_latest.json"
}

# 9. PostgreSQL Latest Backup & Restore Audit
@test "Database integration: PostgreSQL Backup & Dual Audit" {
  seed_databases

  dc_exec exec -T minio mc mb -p local/restic-test-primary-postgres || true
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-postgres \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force \
    --db-type postgres \
    --db-command \"env PGPASSWORD=testpass pg_dumpall -h postgres-13 -U postgres\" \
    --db-keep-daily 7 --db-keep-weekly 4 --db-keep-monthly 12 \
    --db-schedule \"*-*-* 03:00:00\""

  # Setup keys for SFTP (must be done after setting generates the keys)
  register_sftp_keys "backup_migrate"

  dexec "bash backup.sh init"

  dexec "bash backup.sh run"

  # Restore Drill and Audit
  dexec "bash backup.sh audit --restore-drill --report-file /tmp/audit_report_postgres.md"

  # Check reports exist
  dexec "test -f /tmp/audit_report_postgres.md && test -f /tmp/audit_report_postgres.json && test -f /tmp/audit_report_postgres.html"

  # Verify report content
  dexec "grep -q '2차 소산 저장소' /tmp/audit_report_postgres.md"

  dexec "grep -q '\"secondary_recovery_results\":' /tmp/audit_report_postgres.json"

  dexec "grep -q '데이터베이스(postgres) 복원 무결성 검증: 성공' /tmp/audit_report_postgres.md"

  dexec "grep -q '\"db_integrity_verified\": true' /tmp/audit_report_postgres.json"
}

# 10. Config migration: import
@test "Config migration: import" {
  # Setup original S3 setting to migrate legacy to S3
  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force"

  dexec "bash backup.sh init"

  # Create temporary legacy repo and snapshot
  dexec "
    rm -rf /tmp/legacy-local
    restic init -r /tmp/legacy-local --password-file <(echo -n 'test-repo-password')
    echo 'legacy-data' > /tmp/legacy_file.txt
    restic -r /tmp/legacy-local --password-file <(echo -n 'test-repo-password') backup /tmp/legacy_file.txt
  "

  # Run import
  dexec "bash backup.sh import --legacy-dir '/tmp/legacy-local'"

  # Verify migrated snapshot in destination S3 repository
  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"/tmp/legacy_file.txt"* ]]
}

# 11. Migration: SFTP to S3 preserving secondary S3 backend
@test "Migration: SFTP to S3 preserving secondary S3 backend" {
  dc_exec exec -T minio mc mb -p local/restic-test-primary-migrate || true
  dc_exec exec -T minio mc mb -p local/restic-test-secondary-migrate || true
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_restic/backup/*'

  dexec "bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password \
    --secondary-backend s3 \
    --secondary-endpoint http://minio:9000 --secondary-bucket restic-test-secondary-migrate \
    --secondary-access-key AKIAIOSFODNN7EXAMPLE --secondary-secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --secondary-password test-sec-password --force"

  register_sftp_keys "backup_restic"

  dexec "bash backup.sh init"
  dexec "bash backup.sh run"

  dexec "bash backup.sh migrate --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-migrate \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --new-password pass-migrated-primary --force"

  run dexec "source /etc/backup/backup.env && restic snapshots --json"
  [ "$status" -eq 0 ]
  [[ "$output" == *"hostname"* ]]

  run dexec "grep -q 'SECONDARY_BACKEND=.*s3' /etc/backup/backup.env"
  [ "$status" -eq 0 ]
  run dexec "grep -q 'SECONDARY_RESTIC_REPOSITORY=.*s3:http://minio:9000/restic-test-secondary-migrate' /etc/backup/backup.env"
  [ "$status" -eq 0 ]
  run dexec "grep -q 'SECONDARY_AWS_ACCESS_KEY_ID=.*AKIAIOSFODNN7EXAMPLE' /etc/backup/backup.env"
  [ "$status" -eq 0 ]
}

# 12. Failover: Dual Audit fallback to Secondary when Primary is unreachable
@test "Failover: Dual Audit fallback to Secondary when Primary is unreachable" {
  dc_exec exec -T minio mc mb -p local/restic-test-primary-failover || true
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-primary-failover \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force"

  register_sftp_keys "backup_migrate"
  dexec "bash backup.sh init"
  dexec "bash backup.sh run"

  # Invalidate primary password to simulate primary failure without HTTP retry timeouts
  dexec "sed -i 's|RESTIC_PASSWORD=.*|RESTIC_PASSWORD=\"invalid_wrong_pass\"|g' /etc/backup/backup.env"

  run dexec "bash backup.sh audit --restore-drill --report-file /tmp/audit_failover.md"
  dexec "test -f /tmp/audit_failover.md"
  dexec "grep -q '2차 소산 저장소' /tmp/audit_failover.md"
}

# 13. Pipeline Resilience: Execution pipeline completion
@test "Pipeline Resilience: Execution pipeline completion" {
  dc_exec exec -T minio mc mb -p local/restic-test-status3 || true
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-status3 \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force"

  register_sftp_keys "backup_migrate"
  dexec "bash backup.sh init"

  run dexec "bash backup.sh run"
  [ "$status" -eq 0 ]
  [[ "$output" == *"백업"* ]]
}

# 14. Scheduler E2E: resticprofile asset and unit generation
@test "Scheduler E2E: resticprofile asset and unit generation" {
  dc_exec exec -T minio mc mb -p local/restic-test-systemd || true

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-systemd \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password --force"

  dexec "bash backup.sh init"
  dexec "bash backup.sh schedule enable || true"

  run dexec "find /etc/systemd/system -maxdepth 1 -name 'resticprofile-backup@*.service'"
  [ "$status" -eq 0 ]
  [ -n "$output" ]

  dexec "bash backup.sh schedule disable || true"
}

# 15. Database integration: Custom DB Backup & Dual Audit
@test "Database integration: Custom DB Backup & Dual Audit" {
  dc_exec exec -T minio mc mb -p local/restic-test-custom-db || true
  dc_exec exec -T sftp sh -c 'rm -rf /home/backup_migrate/backup/*'

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-custom-db \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --secondary-backend sftp \
    --secondary-host sftp --secondary-port 22 --secondary-user backup_migrate \
    --secondary-password test-sec-password --force \
    --db-type custom \
    --db-command \"echo '-- Custom DB Dump'\" \
    --db-keep-daily 7 --db-keep-weekly 4 --db-keep-monthly 12 \
    --db-schedule \"*-*-* 03:00:00\""

  register_sftp_keys "backup_migrate"
  dexec "bash backup.sh init"
  dexec "bash backup.sh run"

  dexec "bash backup.sh audit --restore-drill --report-file /tmp/audit_report_custom_db.md"
  dexec "test -f /tmp/audit_report_custom_db.md"
  dexec "grep -q '데이터베이스(custom) 복원 무결성 검증: 성공' /tmp/audit_report_custom_db.md"
}

# 16. Notification: Custom webhook payload generation on run
@test "Notification: Custom webhook payload generation on run" {
  dc_exec exec -T minio mc mb -p local/restic-test-notify || true

  dexec "bash backup.sh setting --backend s3 \
    --endpoint http://minio:9000 --bucket restic-test-notify \
    --access-key AKIAIOSFODNN7EXAMPLE --secret-key wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY \
    --password test-repo-password \
    --notification-url 'http://127.0.0.1:9999/webhook' \
    --notification-type 'slack' \
    --notification-on 'both' --force"

  dexec "bash backup.sh init"

  run dexec "bash backup.sh run"
  [ "$status" -eq 0 ]
}




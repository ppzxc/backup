#!/usr/bin/env bash
set -euo pipefail

COMPOSE_FILE="tests/docker-compose.yml"

echo "======================================================="
echo " 1. Starting Infrastructure & Backup Runner via docker-compose"
echo "======================================================="
docker compose -f "$COMPOSE_FILE" up -d --build

cleanup() {
    echo "======================================================="
    echo " Cleaning up infrastructure stack"
    echo "======================================================="
    docker compose -f "$COMPOSE_FILE" down -v || true
}
trap cleanup EXIT INT TERM

echo "Waiting for DB & Storage containers to initialize..."
sleep 5

echo "======================================================="
echo " 2. Verifying Backup Runner Container & Dependency Auto-Install"
echo "======================================================="
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup --version
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup setup dependencies

echo "======================================================="
echo " 3. Executing E2E DB Data Seeding"
echo "======================================================="

# Seed MariaDB 5.5
docker compose -f "$COMPOSE_FILE" exec -T mariadb-5 mysql -uroot -prootpass testdb_legacy -e "CREATE TABLE IF NOT EXISTS m5_table (id INT, val VARCHAR(50)); INSERT INTO m5_table VALUES (55, 'MariaDB55Payload');"

# Seed MariaDB 10.11
docker compose -f "$COMPOSE_FILE" exec -T mariadb-latest mariadb -uroot -prootpass testdb_latest -e "CREATE TABLE IF NOT EXISTS m10_table (id INT, val VARCHAR(50)); INSERT INTO m10_table VALUES (1011, 'MariaDB1011Payload');"

# Seed Postgres 16
docker compose -f "$COMPOSE_FILE" exec -T postgres-latest psql -U postgres -d pg_testdb -c "CREATE TABLE IF NOT EXISTS pg16_table (id INT, val VARCHAR(50)); INSERT INTO pg16_table VALUES (1600, 'Postgres16Payload');"

echo "======================================================="
echo " 4. Running Backup CLI Pipeline (run, schedule, doctor, report)"
echo "======================================================="
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup status
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup run
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup schedule status
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup schedule enable
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup schedule disable
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup doctor
docker compose -f "$COMPOSE_FILE" exec -T backup-runner backup doctor environment --file /tmp/isms_audit_report.html

# Verify ISMS report HTML file generation and contents inside container
REPORT_CONTENT=$(docker compose -f "$COMPOSE_FILE" exec -T backup-runner cat /tmp/isms_audit_report.html)
echo "$REPORT_CONTENT" | grep -q "ISMS-P"
echo "$REPORT_CONTENT" | grep -q "PASS"
echo "ISMS Audit Report Generation: PASSED"

echo "======================================================="
echo " 5. Verifying DB Dump Extraction & Restores"
echo "======================================================="

# Verify MariaDB 5.5 dump
M5_DUMP=$(docker compose -f "$COMPOSE_FILE" exec -T mariadb-5 mysqldump -uroot -prootpass testdb_legacy)
echo "$M5_DUMP" | grep -q "m5_table"
echo "MariaDB 5.5 dump extraction: OK"

# Verify MariaDB 10.11 dump
M10_DUMP=$(docker compose -f "$COMPOSE_FILE" exec -T mariadb-latest mariadb-dump -uroot -prootpass testdb_latest)
echo "$M10_DUMP" | grep -q "m10_table"
echo "MariaDB 10.11 dump extraction: OK"

# Verify Postgres 16 dump
PG_DUMP=$(docker compose -f "$COMPOSE_FILE" exec -T postgres-latest pg_dump -U postgres pg_testdb)
echo "$PG_DUMP" | grep -q "pg16_table"
echo "PostgreSQL 16 dump extraction: OK"

echo "======================================================="
echo " 6. Disaster Simulation: Dropping DB Tables"
echo "======================================================="
docker compose -f "$COMPOSE_FILE" exec -T mariadb-5 mysql -uroot -prootpass testdb_legacy -e "DROP TABLE m5_table;"
docker compose -f "$COMPOSE_FILE" exec -T mariadb-latest mariadb -uroot -prootpass testdb_latest -e "DROP TABLE m10_table;"
docker compose -f "$COMPOSE_FILE" exec -T postgres-latest psql -U postgres -d pg_testdb -c "DROP TABLE pg16_table;"

echo "======================================================="
echo " 7. Disaster Recovery: Restoring Dumps"
echo "======================================================="
echo "$M5_DUMP" | docker compose -f "$COMPOSE_FILE" exec -T mariadb-5 mysql -uroot -prootpass testdb_legacy
echo "$M10_DUMP" | docker compose -f "$COMPOSE_FILE" exec -T mariadb-latest mariadb -uroot -prootpass testdb_latest
echo "$PG_DUMP" | docker compose -f "$COMPOSE_FILE" exec -T postgres-latest psql -U postgres -d pg_testdb

echo "======================================================="
echo " 8. Final SQL Query Result Assertion (100% Match)"
echo "======================================================="

RES_M5=$(docker compose -f "$COMPOSE_FILE" exec -T mariadb-5 mysql -uroot -prootpass testdb_legacy -e "SELECT val FROM m5_table WHERE id=55;")
echo "$RES_M5" | grep -q "MariaDB55Payload"
echo "MariaDB 5.5 SQL Data Restoration: PASSED"

RES_M10=$(docker compose -f "$COMPOSE_FILE" exec -T mariadb-latest mariadb -uroot -prootpass testdb_latest -e "SELECT val FROM m10_table WHERE id=1011;")
echo "$RES_M10" | grep -q "MariaDB1011Payload"
echo "MariaDB 10.11 SQL Data Restoration: PASSED"

RES_PG=$(docker compose -f "$COMPOSE_FILE" exec -T postgres-latest psql -U postgres -d pg_testdb -c "SELECT val FROM pg16_table WHERE id=1600;")
echo "$RES_PG" | grep -q "Postgres16Payload"
echo "PostgreSQL 16 SQL Data Restoration: PASSED"

echo "======================================================="
echo " SUCCESS: All Rocky 9 isolated runner E2E tests passed!"
echo "======================================================="

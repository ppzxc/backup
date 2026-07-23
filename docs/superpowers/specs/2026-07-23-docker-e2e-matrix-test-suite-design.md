# Docker Testcontainers 기반 E2E 매트릭스 통합 테스트 설계서

- **작성일자**: 2026-07-23
- **목적**: 불명확했던 기존 `integration_*.rs` 모듈들을 정리하고, 실제 Docker Testcontainers 환경에서 `backup` CLI의 모든 백업/복구/DB 스트리밍/1차-2차 백업 카피 및 마이그레이션 매트릭스 조합을 완결성 있게 검증하는 E2E 통합 테스트 체계를 구축합니다.

---

## 1. 테스트 구조 재편 및 파일 정리 (File Renaming & Restructuring)

단순 컨테이너 구동 여부만 체크하던 기존 `integration_*.rs` 파일들을 삭제 및 재구조화하여, 실제 기능 단위의 E2E 테스트 모듈로 전환합니다.

- **삭제/대체 대상**:
  - `tests/integration_s3.rs` (삭제)
  - `tests/integration_sftp.rs` (삭제)
  - `tests/integration_db.rs` (삭제)
  - `tests/integration_scenario.rs` (삭제)
- **신규 E2E 테스트 모듈**:
  1. `tests/e2e_storage_matrix_test.rs`: S3(MinIO) & SFTP 저장소 1차 백업, 2차 카피, 1차↔2차 마이그레이션, 원복(Restore) 검증
  2. `tests/e2e_db_streaming_matrix_test.rs`: MariaDB(5.5 / 최신), PostgreSQL(최신) DB 스트리밍 덤프 ➡️ 1차/2차 저장소(S3, SFTP) 백업 및 복원 검증
  3. `tests/e2e_cli_lifecycle_test.rs`: `backup` CLI의 전체 서브커맨드(`doctor`, `schedule`, `setup`, `config`, `uninstall` 등) 실체 동작 검증

---

## 2. 테스트 매트릭스 명세 (Test Matrix Specification)

### A. Storage & Migration Matrix (`tests/e2e_storage_matrix_test.rs`)
| 테스트 케이스 | 1차 저장소 (Primary) | 2차 저장소 (Secondary) | 백업/복사/복구 대상 | 검증 내용 |
|---|---|---|---|---|
| **Case 1: S3 1차 백업 & SFTP 2차 카피** | MinIO (S3) | atmoz/SFTP | 파일 시스템 (임시 데이터) | 1차 S3 백업 ➡️ 2차 SFTP 카피 ➡️ 1차/2차 각각 원복 후 SHA256 100% 일치 검증 |
| **Case 2: SFTP 1차 백업 & S3 2차 카피** | atmoz/SFTP | MinIO (S3) | 파일 시스템 (임시 데이터) | 1차 SFTP 백업 ➡️ 2차 S3 카피 ➡️ 1차/2차 각각 원복 후 SHA256 100% 일치 검증 |
| **Case 3: 1차 ↔ 2차 마이그레이션 백업** | MinIO (S3) | atmoz/SFTP | 백업 스냅샷 전체 | Primary 저장소 데이터 전체를 Secondary 저장소로 이관/마이그레이션 후 무결성 검증 |

### B. DB Streaming Backup Matrix (`tests/e2e_db_streaming_matrix_test.rs`)
| DB 엔진 및 버전 | 스트리밍 도구 | 대상 저장소 (1차 / 2차) | 검증 시나리오 |
|---|---|---|---|
| **MariaDB 10.x (최신)** | `mysqldump` | S3(Primary) + SFTP(Secondary) | DB 덤프 ➡️ restic stdin 백업 ➡️ DB 테이블 drop ➡️ restic restore ➡️ DB 복원 데이터 일치 검증 |
| **MariaDB 5.5 / 10.6 (레거시/호환)** | `mysqldump` | SFTP(Primary) + S3(Secondary) | DB 덤프 ➡️ restic stdin 백업 ➡️ DB 테이블 drop ➡️ restic restore ➡️ DB 복원 데이터 일치 검증 |
| **PostgreSQL 16 (최신)** | `pg_dump` | S3(Primary) + SFTP(Secondary) | DB 덤프 ➡️ restic stdin 백업 ➡️ DB 테이블 drop ➡️ restic restore ➡️ DB 복원 데이터 일치 검증 |

### C. CLI Subcommand Lifecycle (`tests/e2e_cli_lifecycle_test.rs`)
- `backup doctor`: NTP 동기화 점검, 컨테이너 저장소 무결성 진단, HTML/Text 리포트 생성 검증
- `backup schedule`: Systemd timer / Cron 스케줄 생성, 등록, 해제, 상태 조회 검증
- `backup setup` & `config`: 실제 YAML 설정 파일 생성, 암호화 마스킹, 내보내기 검증

---

## 3. 격리된 컨테이너 환경 및 실행기 (Isolated Harness)

- `testcontainers-rs`를 통해 Docker 대역 안에서 MinIO, SFTP, MariaDB(10.x / 5.5), PostgreSQL 컨테이너를 구동합니다.
- Rust `CommandRunner` / `SystemExecutor`를 사용하여 실제 시스템 바이너리(`restic`, `rclone`, `mysqldump`, `pg_dump`)를 호스트/컨테이너 컨텍스트에서 실행합니다.

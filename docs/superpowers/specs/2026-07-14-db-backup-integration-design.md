# Design Spec: Database Backup Integration using Stdin Streaming

## 1. 개요 (Overview)
본 문서는 Restic 백업 자동화 파이프라인(`backup.sh`)에서 데이터베이스(MySQL, MariaDB, PostgreSQL) 백업을 통합 지원하기 위한 상세 설계 사양을 정의합니다.

기존 파일 백업 파이프라인의 인증 정보 및 인프라 설정을 공유하되, **스트리밍 파이프 방식(`stdin-command` 및 `stdin-filename` 활용)**을 채택하여 로컬 임시 디스크 쓰기 없이 안전하고 신속하게 DB 덤프를 원격 저장소로 직접 백업하도록 구성합니다.

---

## 2. 요구사항 및 제약 조건 (Requirements & Constraints)
* **단일 원천(Single Source of Truth) 준수**: 모든 설정은 `/etc/restic/backup.env`를 유일한 원천으로 사용하며, 호스트 백업과 DB 백업은 자격 증명 정보를 완벽히 공유합니다.
* **보존 정책 격리**: 데이터베이스 백업 스냅샷은 파일 백업 스냅샷과 별도의 보관 주기(Retention Policy)를 지정할 수 있어야 합니다.
* **스케줄러 자율화**: systemd timer를 활용해 파일 백업과 DB 백업이 서로 다른 주기에 독립적으로 기동될 수 있어야 합니다.
* **컴플라이언스 준수**: ISMS/ISMS-P 요건에 따라 백업 상태 관리 및 복원 훈련(Restore Drill) 검증 항목에 데이터베이스 복원 정합성 검사가 포함되어야 합니다.

---

## 3. 세부 설계 사양 (Detailed Specification)

### 3.1. 환경 변수 및 CLI 플래그 설계 (Section 1)
데이터베이스 백업을 정의하기 위해 `/etc/restic/backup.env`에 기입되는 환경 변수군을 확장하고, `backup.sh setting` 및 `config` 서브커맨드에 이를 지정하기 위한 플래그를 추가합니다.

#### 환경 변수 명세 (`backup.env`)
* `BACKUP_DB_TYPE`: DB 엔진 종류 (`mysql`, `mariadb`, `postgres`, `custom`). 지정되지 않거나 비어 있으면 DB 백업을 수행하지 않음.
* `BACKUP_DB_COMMAND`: DB dump를 stdout으로 출력하는 쉘 커맨드.
* `BACKUP_DB_FILENAME`: Restic 저장소에 기록될 가상 파일명. (기본값: `db-dump.sql`)
* `BACKUP_DB_SCHEDULE`: DB 백업 전용 스케줄 주기 (`systemd calendar` 포맷). (기본값: 기존 파일 백업 스케줄 계승)
* `KEEP_DB_DAILY`, `KEEP_DB_WEEKLY`, `KEEP_DB_MONTHLY`: DB 스냅샷 개별 보존 개수. (지정하지 않으면 파일 백업 보존 정책 계승)

#### CLI 플래그 매핑
* `--db-type <mysql|mariadb|postgres|custom>`
* `--db-command <command>`
* `--db-filename <filename>`
* `--db-schedule <schedule>`
* `--db-keep-daily <count>`, `--db-keep-weekly <count>`, `--db-keep-monthly <count>`

#### DB 엔진별 기본 `BACKUP_DB_COMMAND` 자동 완성
사용자가 `--db-command`를 지정하지 않고 `--db-type`만 명시한 경우, 아래 기본 명령을 주입합니다.
* **`mysql`**: `mysqldump --all-databases --single-transaction --quick --order-by-primary`
* **`mariadb`**: `mariadb-dump --all-databases --single-transaction --quick --order-by-primary`
* **`postgres`**: `pg_dumpall -U postgres`

---

### 3.2. `profiles.yaml` 렌더링 설계 (Section 2)
`backup.sh` 내부의 `render_resticprofile_config` 함수를 수정하여, DB 백업 설정이 존재할 경우 기존 파일 백업 프로필(`${profile_name}`, 예: `default`)을 상속(`inherit`)받는 DB 백업용 프로필(`${profile_name}-db`, 예: `default-db`)을 `profiles.yaml` 파일 하단에 렌더링합니다.

#### 렌더링 예시 (`/etc/restic/profiles.yaml`)
```yaml
version: "1"

global:
  restic-lock-retry-after: 1m
  restic-stale-lock-age: 2h
  systemd-unit-template: "/etc/restic/resticprofile-unit.template"
  systemd-timer-template: "/etc/restic/resticprofile-timer.template"

# 파일 백업 프로필
default:
  repository: "s3:https://s3.amazonaws.com/my-backup-bucket"
  force-inactive-lock: true
  env:
    RESTIC_PASSWORD: "secure-repo-password"
    AWS_ACCESS_KEY_ID: "ACCESS_KEY"
    AWS_SECRET_ACCESS_KEY: "SECRET_KEY"
    HOSTNAME: "server-01"
  retention:
    after-backup: true
    prune: true
    group-by: host
    keep-daily: 7
    keep-weekly: 4
    keep-monthly: 12
  backup:
    schedule: "*-*-* 02:00:00"
    schedule-permission: system
    source:
      - "/etc"
      - "/var/log"

# DB 백업 프로필 (DB 설정 존재 시 추가 렌더링)
default-db:
  inherit: default
  # DB 개별 보존 정책이 정의된 경우에만 출력 (미정의 시 default retention 상속)
  retention:
    after-backup: true
    prune: true
    group-by: host
    keep-daily: 7
    keep-weekly: 4
    keep-monthly: 3
  backup:
    schedule: "*-*-* 03:00:00"
    schedule-permission: system
    stdin: true
    stdin-command: "mysqldump --all-databases --single-transaction --quick --order-by-primary"
    stdin-filename: "db-dump.sql"
    tag:
      - db
```

---

### 3.3. systemd 스케줄러 라이프사이클 관리 (Section 3)
* **스케줄 활성화 (`scheduler_systemd_register`)**:
  `backup.sh schedule enable` 실행 시, 기존 `$profile_name` 타이머 외에도 DB 백업이 활성화되어 있다면 `resticprofile --config <config> --name ${profile_name}-db schedule` 명령을 순차적으로 호출하여 DB 백업 전용 systemd 타이머(`resticprofile-backup@default-db.timer`)를 등록합니다.
* **스케줄 비활성화 (`scheduler_systemd_unregister`)**:
  `backup.sh schedule disable` 실행 시, `${profile_name}-db` 프로필에 대해서도 `unschedule`을 명시적으로 실행하여 systemd 타이머 및 서비스 유닛을 해제합니다.
* **상태 모니터링 (`scheduler_systemd_status`)**:
  `systemctl is-active resticprofile-backup@default-db.timer` 조회를 통해 DB 백업 타이머 작동 여부를 식별하고 `_sys_stat[db_backup]` 변수에 매핑합니다.
* **BATS 단위 테스트 Mocking**:
  `scheduler_mock_register`, `unregister`, `status` 함수도 `db_backup_schedule` 및 `db_backup_enabled` 상태를 추적할 수 있도록 개선합니다.

---

### 3.4. 상태 조회 및 감사 보고서(Restore Drill) 통합 (Section 4)

#### `backup.sh status` 연동
* 스케줄 상태 출력란에 `DB 백업 타이머` 상태와 주기를 추가로 노출합니다.
* Restic 스냅샷 목록 조회 시, `--tag db` 필터링을 사용하여 파일 백업 스냅샷 리스트와 DB 백업 스냅샷 리스트를 시각적으로 분리하여 표 형태로 출력합니다.

#### `backup.sh audit` 연동 및 복원 드릴
* **감사 보고서 연동**: 
  `audit_report.json` 및 `audit_report.txt` 결과 구조에 `db_backup_enabled`, `last_db_backup_time`, `db_snapshot_count` 메타데이터를 통합합니다.
* **복원 드릴 (`--restore-drill`) 시나리오 확장**:
  1. 기존 파일 복구 모의 훈련 수행 완료 후, DB 백업 최신 스냅샷에서 `db-dump.sql` 파일을 복원 디렉토리로 다운로드합니다.
  2. 다운로드된 파일의 크기(> 0 bytes)를 확인합니다.
  3. `db-dump.sql` 파일의 첫 몇 줄을 확인하여 덤프 파일 포맷 헤더가 정상적인지 확인합니다.
     * MySQL/MariaDB: `MySQL dump` 또는 `MariaDB dump` 패턴
     * PostgreSQL: `PostgreSQL database dump` 패턴
  4. 검증 성공/실패 여부를 최종 리포트(`restore_drill_report_YYYYMMDD.html`, `txt`, `json`)에 기록합니다.

---

## 4. 테스트 전략 (Testing Strategy)
1. **단위 테스트 (`bats tests/`)**:
   - `resolve_value` 및 `validate` 헬퍼 함수가 DB 백업 관련 플래그를 정상적으로 처리하는지 단위 테스트를 추가합니다.
   - `render_resticprofile_config` 함수가 DB 백업 정보가 주어졌을 때 상속 구조의 YAML을 의도한 대로 생성하는지 검증합니다.
2. **통합 테스트 (`tests/integration/run.sh`)**:
   - Docker Compose를 통해 구동되는 데이터베이스 컨테이너에 대해 수동 및 systemd 스케줄러 기반의 DB 백업 및 복원 드릴 파이프라인이 정상 동작하는지 테스트 시나리오를 추가하여 통합 검증합니다.

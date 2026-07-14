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

### 3.1.2. 마법사(wizard) 및 설정(setting/config) 편의성 설계
초보 관리자와 파워 유저 모두가 DB 백업 구성을 쉽게 제어할 수 있도록 대화형 프롬프트와 점진적 업데이트 기능을 설계합니다.

#### 1. 대화형 설정 마법사 (`backup.sh wizard`) 흐름 추가
파일 백업 기본 구성 완료 후, 다음과 같은 질문 시퀀스를 순차적으로 수행합니다.
1. **DB 백업 연동 여부**: `데이터베이스 백업을 함께 설정하시겠습니까? [y/N]`
2. **DB 엔진 유형 선택**: `y`를 입력한 경우, 연동할 데이터베이스 엔진을 선택합니다.
   * `1) mysql`, `2) mariadb`, `3) postgres`, `4) custom (사용자 정의 커맨드)`
   * `4) custom`을 선택한 경우, 덤프에 사용할 실제 쉘 명령어(`--db-command`)를 입력받습니다.
3. **가상 파일명 지정**: Restic 저장소에 저장될 백업 덤프 파일명(`--db-filename`, 기본값: `db-dump.sql`)을 입력받습니다.
4. **DB 개별 스케줄링**: `DB 백업 전용 스케줄 주기(systemd calendar 포맷)를 지정하시겠습니까? (미지정 시 기본 백업 주기 상속) [y/N]`
   * `y`를 입력한 경우 calendar 주기를 입력받으며, `N`을 입력하거나 생략하면 기본 백업 주기(예: 매일 새벽 2시)를 공유하여 기동하도록 설정합니다.

#### 2. `setting` 및 `config` 부분 업데이트(Upsert) 기능
* **`setting` (일괄 설정)**: 백업 초기 구축 시 디렉토리 대상과 DB 백업 대상을 한 번에 인자로 전달하여 통합 환경을 구축할 수 있습니다.
* **`config` (부분 갱신)**: 이미 파일 백업이 활성화되어 운영 중인 서버에서, 파일 백업 설정을 보존하면서 DB 백업 옵션만 덧붙여 활성화/수정할 수 있도록 유연한 병합 메커니즘을 적용합니다.
  * 예: `backup.sh config --db-type mariadb --db-schedule "*-*-* 04:00:00"`
  * 이 명령은 기존 `/etc/restic/backup.env` 파일 내의 다른 설정(S3 자격증명 등)을 일절 파괴하지 않고, `BACKUP_DB_TYPE` 및 `BACKUP_DB_SCHEDULE` 값만 안전하게 삽입/치환한 뒤 `profiles.yaml`을 재생성하여 반영합니다.

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
* **복원 드릴 (`--restore-drill`) 시나리오 확장 (안전한 파일 기반 검증)**:
  > [!IMPORTANT]
  > 복원 모의 훈련 시 실제 구동 중인 데이터베이스 엔진(MySQL, MariaDB, PostgreSQL 데몬)에 덤프 파일을 임포트하거나 데이터를 덮어쓰는 작업은 **절대 수행하지 않습니다.** 오직 임시 디렉토리 수준에서 파일 복구 상태만 검증합니다.
  1. 기존 파일 복구 모의 훈련 수행 완료 후, DB 백업 최신 스냅샷에서 `db-dump.sql` 파일을 임시 격리 디렉토리(예: `/tmp/restic-restore-drill.XXXXXX/`)로 다운로드합니다.
  2. 다운로드된 파일의 크기(> 0 bytes)를 확인합니다.
  3. `db-dump.sql` 파일의 첫 몇 줄을 읽어 데이터베이스 엔진별 덤프 파일 포맷 헤더가 정상적인지 무결성을 검사합니다.
     * MySQL/MariaDB: `MySQL dump` 또는 `MariaDB dump` 패턴
     * PostgreSQL: `PostgreSQL database dump` 패턴
  4. 검증이 완료되면 다운로드된 `db-dump.sql` 파일을 포함한 임시 디렉토리를 **즉시 소거(Clean-up)**합니다.
  5. 검증 성공/실패 여부를 최종 리포트(`restore_drill_report_YYYYMMDD.html`, `txt`, `json`)에 기록합니다.

---

### 3.5. 도움말 명세 및 Cobra 스타일 CLI 인터페이스 (Section 5)
`backup.sh` 내에 정의된 도움말 함수군(`help_setting` 및 `help_config`)을 확장하여, 새로 도입된 DB 백업 관련 플래그를 정렬된 Cobra 스타일 포맷에 맞추어 표기합니다.

#### 도움말 추가 대상 플래그 예시 (`help_setting` 및 `help_config` 하단)
```text
  데이터베이스 백업 옵션 (Database Backup Flags):
      --db-type <mysql|mariadb|postgres|custom>  통합할 DB 엔진 유형 (지정 시 DB 백업 활성화)
      --db-command <명령어>          DB 백업에 사용할 덤프 커맨드 (생략 시 기본 덤프 명령어 자동 주입)
      --db-filename <파일명>         Restic 내에 저장될 가상 덤프 파일명 (기본값: db-dump.sql)
      --db-schedule <스케줄>         DB 백업 전용 스케줄 주기 (기본값: 기본 파일 백업 스케줄 계승)
      --db-keep-daily <N>            DB 스냅샷 일별 보관 개수
      --db-keep-weekly <N>           DB 스냅샷 주별 보관 개수
      --db-keep-monthly <N>          DB 스냅샷 월별 보관 개수
```

---

## 4. 테스트 전략 (Testing Strategy)

### 4.1. 단위 테스트 (Unit Tests)
`bats tests/` 기반의 유닛 테스트 세트에 아래 테스트 커버리지를 반드시 추가하여 통과시킵니다.
1. **DB 설정 해석 테스트**: `setting` 커맨드 호출 시 `--db-type` 및 다양한 DB 플래그가 주어졌을 때 `/etc/restic/backup.env`에 환경변수가 규칙대로 올바르게 작성되는지 검증
2. **기본 커맨드 주입 테스트**: `--db-type`만 주어졌을 때 `mysql`, `mariadb`, `postgres` 각각에 해당하는 표준 안전 덤프 명령어(`BACKUP_DB_COMMAND`)가 자동으로 완성 및 세팅되는지 검증
3. **YAML 렌더링 정합성 테스트**: DB 백업이 활성화되었을 때 `profiles.yaml` 내에 상속 구조(`inherit: default`)를 가지는 `${profile_name}-db` 섹션이 사양에 맞게 올바르게 출력되는지 검증
4. **스케줄러 Mocking 동작 검증**: `scheduler_mock_register`/`unregister`/`status` 함수가 DB 스케줄 여부에 따라 mock 상태 파일에 덤프 스케줄 유무를 오차 없이 쓰고 조회하는지 검증

### 4.2. 도커 통합 테스트 (Docker Integration Tests)
`tests/integration/` 디렉토리 하위의 Docker Compose 환경 및 통합 테스트 자동화 스크립트(`run.sh`)에 아래 e2e 시나리오를 추가합니다.
1. **데이터베이스 컨테이너 가동**: 기존 MinIO, SFTP 컨테이너 외에 테스트용 MariaDB(또는 MySQL) 및 PostgreSQL 컨테이너를 Docker Compose 스택으로 함께 띄웁니다.
2. **백업 실행 검증**: `backup.sh run` 및 `backup.sh run --profile <profile>-db`를 수동 호출하여, 실시간 DB 스트림이 restic 저장소로 무사히 백업(Dedup 처리 포함)되는지 이력을 대조합니다.
3. **스케줄러 기동 검증**: systemd 타이머와 유사하게 등록된 스케줄에 의해 DB 백업이 작동하는지 이관 및 가동 여부를 검사합니다.
4. **복원 모의 훈련(Restore Drill) 검증**: `backup.sh audit --restore-drill`을 구동하여 실제 DB 컨테이너에는 영향을 주지 않으면서 다운로드된 `db-dump.sql` 파일의 무결성을 완벽하게 확인하고, 임시 파일이 디스크에서 누수 없이 지워지는지 통합 점검합니다.


# Context Glossary: Restic Backup Pipeline

이 문서는 Restic 백업 파이프라인 프로젝트에서 공통적으로 사용하는 핵심 도메인 용어를 정의합니다. 모든 코드 작성 및 테스트 시 다음 단어를 엄격히 준수합니다.

## 핵심 도메인 용어 (Core Domain Terms)

### 1. Backup Profile (백엔드 프로필)
* **설명**: 특정 데이터 대상(예: DB, 로그 파일 등)을 백업하기 위한 보관 주기, 저장 대상, 암호, 스케줄링 등의 구성을 갖춘 독립적인 백업 동작 단위.
* **비고**: `/etc/backup/profiles.yaml`의 resticprofile v2 `profiles` 키 아래 프로필 키로 표현되며, `backup run`은 명시적인 프로필 필터가 없으면 실행 가능한 모든 Backup Profile을 순회합니다.

### 2. Unified Backup Configuration (통합 백업 설정)
* **설명**: 호스트별 설정의 유일한 단일 원천(Source of Truth)으로 작동하는 환경설정 파일.
* **비고**: 기본 경로는 `/etc/backup/profiles.yaml`이며, 권한은 반드시 `600`이어야 합니다. resticprofile v2 설정은 표준 최상위 키(`version`, `global`, `groups`, `profiles`)를 사용하며, 백업 실행 설정의 유일한 정본이다. `application`은 충돌하지 않는 backup CLI 전용 메타데이터인 보고서, 선택적인 Database Stream, 감사 정보를 보관한다. 구성 형식의 버전은 최상위 `version: "2"`가 유일한 권위이며 `application`은 별도 형식 버전을 갖지 않습니다. 비밀값은 같은 디렉터리의 `600` 파일로 분리합니다.
* **전환 정책**: 이전 `application`에 백업 실행 설정이 남은 Unified Backup Configuration은 유효하지 않으며, 운영자가 표준 프로필로 수동 전환해야 한다.

### 2-1. Setup Wizard (설정 위자드)
* **설명**: 사용자가 Backup Profile과 Unified Backup Configuration을 대화형으로 생성하는 `backup setup` 진입점.
* **비고**: Setup Wizard는 1차·2차 Backend Adapter의 연결을 검증하고 비어 있는 저장소를 초기화하며, 서버 로컬 시간 기준 기본 03:00의 일별 스케줄러와 백업 실행 리포트의 파일 보관을 자동 설정합니다. 백엔드 연결 및 초기화 실행 시 사용자에게 명확한 상태 안내(Progress UX)를 제공하고 타임아웃(기본 15초)을 강제하여 무한 대기를 방지합니다. 초기화 실패 시 자격 증명을 마스킹하여 사유를 보고하고, 사용자의 선택에 따라 설정을 보존하거나 롤백합니다. 재실행 시 새 구성이 완전히 준비되기 전에는 기존 설정과 스케줄을 유지합니다. Container E2E Matrix의 저장소 조합은 Setup Wizard가 생성한 설정을 유일한 입력으로 사용하며, 테스트가 설정 파일을 직접 작성해 이를 대체하지 않습니다.

### 2-2. Application Configuration (애플리케이션 메타데이터 설정)
* **설명**: Unified Backup Configuration의 `application` 아래에 있는 backup CLI 전용 메타데이터이다.
* **비고**: 보고서, 감사, 선택적 Database Stream만 보관하며 Backup Profile, Backend Adapter, 보존 정책 등 백업 실행 설정을 소유하지 않는다. 이전 `BackupConfig`는 운영 설정 모델로 사용하지 않는다.

### 3. Configuration Registry (설정 레지스트리)
* **설명**: 백업 설정을 관리하는 심층 아키텍처 모듈.
* **비고**: 메모리에 설정을 로드하고 유효성 검증을 거치는 행위(`load_and_validate_config`), 설정을 파일에 쓰고 파생 산출물(profiles.yaml, systemd 타이머 등)을 동기화하는 행위(`save_profile_config`)를 제공하여 호출자와 시스템 간의 세임(Seam) 역할을 수행합니다.

### 3-1. Profile Resolver (프로필 해결기 / ProfileManager)
* **설명**: 3계층 백업 프로필 상속 체인(`inherit`) 순회, `primary`/`default` 폴백 처리, 백엔드 프로토콜 타입 추론을 전담하는 심층 도메인 모듈.
* **비고**: `ResolvedProfile` 구조체를 반환하며, CLI 명령어 층이 복잡한 프로필 상속 트리나 내열성(fallback) 규칙을 직접 처리하지 않도록 도메인 세임(Seam)을 제공합니다.


### 4. Backend Adapter (백엔드 어댑터)
* **설명**: 다양한 저장 대상(S3, SFTP 등)에 따라 다르게 요구되는 필드 검증, 환경 변수 렌더링, 공지 사항 생성, 연결 테스트 등의 행위를 추상화한 다형성 모듈.
* **비고**: `primary`와 `secondary`는 예약된 Backend Profile이며, 일반 Backup Profile이 상속 및 복사 설정으로 이를 참조한다. `backend_${backend}_${action}` 형태로 함수가 명명되며, 1차 및 2차 저장소 여부에 따른 동적 접두사 처리를 내부에서 캡슐화합니다.

### 5. Notification Adapter (알림 어댑터)
* **설명**: Slack, Discord, Custom 등 다양한 알림 채널에 맞추어 페이로드 포맷을 정하고 웹훅 디스패치 및 필수 값 검증을 추상화한 다형성 모듈.
* **비고**: `notification_${type}_${action}` 형태로 함수가 명명되며, 메인 디스패처 `dispatch_notification`는 각 어댑터의 세부 전송 방식에 의존하지 않고 다형적으로 호출합니다.

### 6. Database Backup Adapter (데이터베이스 백업 어댑터)
* **설명**: MySQL/MariaDB 또는 PostgreSQL 엔진에 알맞은 기본 백업(dump) 명령어 제공, 설정 검증, 복원 시 백업본의 무결성(헤더 검사 등)을 추상화한 다형성 모듈.
* **비고**: `database_${db_type}_${action}` 형태로 함수가 명명되며, 백업 실행기 및 복원 훈련 단계의 핵심 세임(Seam) 역할을 수행합니다. 임의 셸 명령은 Database Backup Adapter의 입력이 아닙니다.

### 6-1. Database Stream (데이터베이스 스트림)
* **설명**: Database Backup Adapter가 생성한 데이터베이스 덤프를 평문 임시 파일로 남기지 않고 BackupEngine Interface에 직접 전달하여 백업 스냅샷으로 보관하는 데이터 흐름.
* **비고**: Database Stream은 독립적인 데이터베이스 백업과 전체 Backup Pipeline 모두에서 같은 도메인 동작으로 사용됩니다.

### 7. BackupEngine Interface (통합 백업 엔진 실행기)
* **설명**: 외부 바이너리(`restic`, `resticprofile`, `rclone`) 프로세스 호출, 명령줄 플래그 생성, 임시 파일 관리, 출력 파싱을 모두 심하게 은닉하는 Trait 세임(Seam).
* **비고**: `SystemBackupEngine` 및 `MockBackupEngine`으로 구성되며, CLI 명령어가 문자열 옵션이나 raw JSON이 아닌 고수준 도메인 객체(`ResolvedProfile`, `SnapshotInfo` 등)만을 주고받도록 보장합니다.


### 8. Security Permission Enforcer (보안 권한 강제)
* **설명**: `/etc/backup` 디렉터리(`700`) 및 설정 파일(`600`)의 POSIX 권한을 생성/수정 시 명시적이고 엄격하게 강제하는 도메인 정책.
* **비고**: 권한 설정 중 오류 발생 시 경고에 그치지 않고 즉시 반환 에러(`Result::Err`)로 처리하여 권한이 보장되지 않은 상태에서의 프로세스 진행을 차단합니다.

### 9. Container E2E Matrix (컨테이너 E2E 매트릭스)
* **설명**: 격리된 Docker 환경에서 Backup Pipeline과 Database Stream의 실제 저장·복사·복원 결과를 검증하는 테스트 계약.
* **비고**: 기본 `cargo test`에 포함하며, Docker runner 안에서 필요한 외부 도구와 저장소를 격리해 단일 모듈에서 순차 실행합니다. 스케줄러 검증은 privileged systemd runner에서 수행합니다.

### 10. Test Configuration Override (테스트 설정 경로 오버라이드)
* **설명**: CLI가 기본 통합 설정 파일 대신 호출자가 명시한 `profiles.yaml`을 사용하는 실행 범위.
* **비고**: 모든 서브커맨드에서 공통 `--profiles` 옵션으로 지정합니다.

### 11. Restore Verification (복원 검증)
* **설명**: BackupEngine이 스냅샷을 지정한 대상에 복원한 뒤, 원본 데이터와 복원 산출물의 무결성을 확인하는 행위. Secondary Backend Adapter가 설정된 경우 1차와 2차 저장소 각각에서 독립적으로 수행한다.
* **비고**: 파일 복원은 상대 경로·내용 해시·빈 파일·중첩 디렉터리·유니코드 파일명·실행 권한의 동등성을 검증합니다. Database Stream의 SQL import와 행 검증은 Container E2E Matrix의 검증 단계이며 `backup restore`의 자동 동작이 아닙니다.

### 11-1. Backup Execution Report (백업 실행 리포트)
* **설명**: 하나의 `backup run` 또는 스케줄러 실행의 결과를 파일로 보존하는 운영 증적.
* **비고**: 성공과 실패 모두 생성하며, 실행 시각·Backup Profile·저장소별 결과·스냅샷 식별자를 기록합니다. 실패 시 실패 단계와 마스킹된 오류를 기록하고 실행 결과는 실패로 유지합니다.

### 11-2. Scheduled Backup Run (예약 백업 실행)
* **설명**: Setup Wizard가 등록한 스케줄러가 전체 Backup Pipeline을 호출하는 일별 자동 실행.
* **비고**: 실행 단위는 `backup run`이며, resticprofile의 단일 `backup` 작업이 아닙니다. 따라서 각 Backup Profile의 1차 백업·2차 복제·Retention과 Backup Execution Report가 같은 실행 결과에 포함됩니다. Setup Wizard는 systemd timer를 우선 선택하고, 사용할 수 없을 때만 Cron fallback을 자동 선택합니다.

### 12. Database E2E Support Matrix (데이터베이스 E2E 지원 매트릭스)
* **설명**: Database Stream의 실제 백업·복원을 계속 검증하는 프로덕션 데이터베이스 버전 집합.
* **비고**: MariaDB 12 LTS, MariaDB 5.5.56, PostgreSQL 16으로 고정합니다. Container E2E Matrix의 S3/SFTP 파일 복원 조합과 분리해 검증합니다.

### 13. Production Execution Integrity (프로덕션 실행 무결성)
* **설명**: 운영 CLI 경로는 선언하거나 완료로 보고하는 Backup Pipeline, Backend Adapter, Restore Verification, 감사 진단을 실제로 실행해야 한다는 정책.
* **비고**: `Mock*` 구현과 고정된 더미 결과는 테스트 격리에서만 허용됩니다. 일반 프로덕션 빌드와 public API에는 `Mock*` 구현을 포함하지 않습니다. 운영 경로는 실제 호출 없이 성공·완료·검증됨을 출력해서는 안 됩니다. 활성화된 Secondary Backend Adapter는 `backup run`에서 실제 스냅샷 복사를 마친 뒤에만 완료로 보고합니다. Database Stream은 하나의 `backup run`에서 Backup Profile 반복 전에 정확히 한 번 실제 실행합니다. ISMS 증적과 보고서는 실제 진단·복구 결과만 포함하며, 고정된 성공값 및 더미 스냅샷 메타데이터를 포함하지 않습니다. `backup doctor`는 Restic 실행 가능 여부와 지정된 통합 설정 파일의 실제 경로·권한을 검사한 결과로만 상태를 보고합니다. 필수 설정을 읽지 못하거나 선언한 작업이 실패한 운영 명령은 성공으로 반환하지 않고 오류 종료합니다. Restore Verification은 복원 산출물의 존재·비어 있지 않음을 확인하고, Database Stream에는 덤프 형식 검증과 실제 측정 RTO를 포함합니다. 비대화형 Setup은 명시 입력된 실제 대상·저장소·자격 증명만 저장하며, 예시값·기본 비밀값 또는 기본 구성으로 대체하지 않습니다. Update의 릴리스 조회·다운로드·자가 교체 실패는 오류 종료로 전파합니다.

### 14. System Diagnostic Logging (시스템 진단 로깅)
* **설명**: `tracing` 및 `tracing-subscriber` Layered 아키텍처 기반으로 터미널 UX 보호, 비밀값 마스킹, 3단계 시스템 폴백(Journald -> Syslog -> Rotating File)을 보장하는 구조화된 진단 및 운영 로그 수집 체계.
* **비고**: `stdout`은 데이터/리포트 전용으로 격리하고 `stderr`로 터미널 진단 메시지를 출력합니다. 3단계 폴백 파이프라인(1. Journald socket `/run/systemd/journal/socket`, 2. Syslog socket `/dev/log`, 3. 회전 로그 파일 `/var/log/backup/backup.log` 순차 적용)을 통해 어떤 실행 환경에서도 진단 로그를 안전하게 수집하며, 비밀값(`SecretString`, `password`, `access_key`, `secret_key`, `token` 등)은 `***MASKED***`로 자동 은닉합니다. 대화형 설정 위자드(`backup setup`) 실행 시 터미널 `stderr` 출력을 차단하여 TUI UI 깨짐을 방지합니다.


## CLI 서브커맨드 구조 명세 (Command Architecture Spec)

유비쿼터스 언어에 맞춰 설계된 Rust CLI 커맨드 구조 및 역할 정의입니다.

### 1. `backup setup` (환경 및 프로필 초기화)
* **`backup setup`**: `inquire` TUI 마법사로 통합 설정과 **Backup Profile** 대화형 생성
* **`backup setup --non-interactive`**: 대화 없이 설정 파일 기반으로 환경 설정 및 초기화 일괄 수행
* **`backup setup dependencies`**: 필수 바이너리 의존성(`restic`, `rclone`, `resticprofile`) 검증 및 자동 설치
* **`backup setup backend-init`**: 1차/2차 **Backend Adapter** 저장소(`restic init`) 연결 점검 및 초기화

### 3. `backup copy` (저장소 간 스냅샷 동기화 및 복사)
* **`backup copy [--profile <profile_name>] [--dry-run]`**: 1차 **Backend Adapter** 저장소의 스냅샷 데이터를 2차 **Backend Adapter** 저장소로 동기화/복사 (별칭: `backup sync`)

### 4. `backup run` (백업 파이프라인 실행)
* **`backup run`**: 전체 백업 파이프라인 수동 즉시 실행 (Database Backup Adapter -> Primary Backend Adapter -> Secondary Backend Adapter -> Retention Rule -> Notification Adapter)
* **`--skip-database`**: **Database Backup Adapter** 덤프 단계 건너뛰기
* **`--skip-secondary-sync`**: 2차 **Backend Adapter** 복제 건너뛰기
* **`--skip-retention`**: Retention/Prune 정리 단계 건너뛰기
* **`--dry-run`**: 실제 실행 없는 명령어 및 대상 시뮬레이션

### 4-1. `backup database` (데이터베이스 백업 실행)
* **`backup database`**: 현재 통합 설정의 Database Backup Adapter를 실행하여 Database Stream 스냅샷을 생성
* **관계**: 독립 실행 진입점이지만, Database Backup Adapter가 설정된 경우 `backup run`도 동일한 동작을 파이프라인의 데이터베이스 단계로 실행. Adapter가 없으면 `backup database`는 설정 오류로 종료하고, `backup run`은 파일 백업 파이프라인을 계속 실행합니다.

### 5. `backup doctor` (시스템 및 백업 종합 진단)
* **`backup doctor`**: 백업 바이너리, 설정파일/보안 권한(`700`/`600`), 저장소 네트워크 연결성, NTP 시각 동기화, 타이머 스케줄러 헬스체크 종합 진단 및 문제 조치 가이드 제공

### 6. `backup report` (ISMS-P 감사 증적 및 레포트 생성)
* **`backup report [subcommand] [--file <path>] [--format <html|json>]`**: 서브 커맨드 미지정 시 전체 검사 항목(`environment`, `time-sync`, `restore-drill`)을 순환 실행하여 통합 보고서를 생성하며, `--file` 미지정 시 `BackupConfig.reports.output_dir` (기본값: `/data/backup/reports`) 내 타임스탬프 파일로 기록됨. `--format` 미지정 시 HTML 및 JSON 포맷 보고서 2종을 모두 생성하며, 지정 시 해당 포맷 파일만 단독 생성됨.
* **`backup report environment [--file <path>] [--format <html|json>]`**: 통합 설정 파일의 권한 및 보안 규정 검증 보고서 생성
* **`backup report time-sync [--file <path>] [--format <html|json>]`**: NTP/Chrony 시각 동기화 검증 및 ISMS 증적 보고서 생성
* **`backup report restore-drill [--file <path>] [--format <html|json>]`**: 스냅샷을 임시 복구 대상에 실제로 복원하는 비파괴 모의훈련을 실행하고, RTO 및 DB 덤프 무결성 검증 보고서를 생성. 운영 데이터베이스에는 dump를 import하지 않습니다.

### 7. `backup schedule` (스케줄러 관리)
* **`backup schedule enable`**: Systemd Timer (또는 Cron Fallback) 자동 백업 스케줄 임의 등록
* **`backup schedule disable`**: 자동 백업 스케줄 해제
* **`backup schedule status`**: 타이머/스케줄러 현재 동작 상태 조회

### 7. 기타 운영 커맨드
* **`backup restore --target <path> [--storage <primary|secondary>]`**: 백업 데이터 및 DB dump를 지정한 복구 대상으로 실행. `storage`를 생략하면 primary를 사용하며, 활성화된 2차 저장소의 독립 복원은 `--storage secondary`로 명시합니다. 대상은 기본값 없이 명시해야 하며, 비어 있지 않은 대상에 대한 덮어쓰기는 명시적인 강제 확인이 필요합니다.
* **`backup snapshots`**: 1차/2차 저장소 스냅샷 목록을 저장소별로 구분해 조회. Primary Backend Adapter 조회 실패는 명령 실패이며, Secondary Backend Adapter 조회 실패는 primary 결과와 경고를 함께 출력합니다.
* **`backup status [--profile <name>]`**: 백업 프로필별 저장소 위치, 최신 스냅샷(ID/시각/용량) 및 파이프라인 상태 종합 동적 조회 (조회 실패 시 Graceful Fallback 경고 표기)
* **`backup update`**: 자기 자신(Rust 바이너리) 및 설정 갱신
* **`backup uninstall [--purge]`**: 스케줄 해제 및 바이너리 삭제 (`--purge` 시 설정/캐시 완전 제거)

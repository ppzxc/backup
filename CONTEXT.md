# Context Glossary: Restic Backup Pipeline

이 문서는 Restic 백업 파이프라인 프로젝트에서 공통적으로 사용하는 핵심 도메인 용어를 정의합니다. 모든 코드 작성 및 테스트 시 다음 단어를 엄격히 준수합니다.

## 핵심 도메인 용어 (Core Domain Terms)

### 1. Backup Profile (백엔드 프로필)
* **설명**: 특정 데이터 대상(예: DB, 로그 파일 등)을 백업하기 위한 보관 주기, 저장 대상, 암호, 스케줄링 등의 구성을 갖춘 독립적인 백업 동작 단위.
* **비고**: 설정 파일(`backup.env`) 내의 `BACKUP_PROFILE_NAME`으로 표현되며, 호스트의 호스트명을 기본값으로 갖습니다. `profiles.yaml`에서 프로필 키로 렌더링됩니다.

### 2. Backup Environment (백업 설정파일)
* **설명**: 호스트별 설정의 유일한 단일 원천(Source of Truth)으로 작동하는 환경설정 파일.
* **비고**: 기본 경로는 `/etc/restic/backup.env`이며, 권한은 반드시 `600`이어야 합니다. Restic 저장소 접속용 자격 증명(비밀번호, 액세스 키 등)과 백업 대상, 웹훅 정보 등을 환경 변수(`export VAR=val`) 형태로 가집니다.

### 3. Configuration Registry (설정 레지스트리)
* **설명**: 백업 설정을 관리하는 심층 아키텍처 모듈.
* **비고**: 메모리에 설정을 로드하고 유효성 검증을 거치는 행위(`load_and_validate_config`), 설정을 파일에 쓰고 파생 산출물(profiles.yaml, systemd 타이머 등)을 동기화하는 행위(`save_profile_config`)를 제공하여 호출자와 시스템 간의 세임(Seam) 역할을 수행합니다.

### 4. Backend Adapter (백엔드 어댑터)
* **설명**: 다양한 저장 대상(S3, SFTP 등)에 따라 다르게 요구되는 필드 검증, 환경 변수 렌더링, 공지 사항 생성, 연결 테스트 등의 행위를 추상화한 다형성 모듈.
* **비고**: `backend_${backend}_${action}` 형태로 함수가 명명되며, 1차 및 2차 저장소 여부에 따른 동적 접두사 처리를 내부에서 캡슐화합니다.

### 5. Notification Adapter (알림 어댑터)
* **설명**: Slack, Discord, Custom 등 다양한 알림 채널에 맞추어 페이로드 포맷을 정하고 웹훅 디스패치 및 필수 값 검증을 추상화한 다형성 모듈.
* **비고**: `notification_${type}_${action}` 형태로 함수가 명명되며, 메인 디스패처 `dispatch_notification`는 각 어댑터의 세부 전송 방식에 의존하지 않고 다형적으로 호출합니다.

### 6. Database Backup Adapter (데이터베이스 백업 어댑터)
* **설명**: MySQL, MariaDB, PostgreSQL, Custom 등 각 데이터베이스 엔진에 알맞은 기본 백업(dump) 명령어 제공, 설정 검증, 복원 시 백업본의 무결성(헤더 검사 등)을 추상화한 다형성 모듈.
* **비고**: `database_${db_type}_${action}` 형태로 함수가 명명되며, 백업 실행기 및 복원 훈련 단계의 핵심 세임(Seam) 역할을 수행합니다.

## CLI 서브커맨드 구조 명세 (Command Architecture Spec)

유비쿼터스 언어에 맞춰 설계된 Rust CLI 커맨드 구조 및 역할 정의입니다.

### 1. `backup setup` (환경 및 프로필 초기화)
* **`backup setup`**: `inquire` TUI 마법사로 **Backup Environment** 및 **Backup Profile** 대화형 생성
* **`backup setup --non-interactive`**: 대화 없이 설정 파일 기반으로 환경 설정 및 초기화 일괄 수행
* **`backup setup dependencies`**: 필수 바이너리 의존성(`restic`, `rclone`, `resticprofile`) 검증 및 자동 설치
* **`backup setup backend-init`**: 1차/2차 **Backend Adapter** 저장소(`restic init`) 연결 점검 및 초기화

### 2. `backup config` (백업 설정 관리)
* **`backup config show`**: **Backup Environment** 및 **Backup Profile** 설정값 출력 (SecretString 마스킹)
* **`backup config edit`**: 설정 파일 직접 편집 및 **Configuration Registry** 유효성 검증
* **`backup config import-legacy [--file <path>]`**: 구버전 Bash `backup.env` 파일을 현재 규격으로 이관

### 3. `backup backend` (저장소 백엔드 이관)
* **`backup backend migrate`**: 스냅샷 데이터 2차 복사(`restic copy`), 정합성 검증 및 신규 **Backend Adapter** 저장소로 이관

### 4. `backup run` (백업 파이프라인 실행)
* **`backup run`**: 전체 백업 파이프라인 수동 즉시 실행 (Database Backup Adapter -> Primary Backend Adapter -> Secondary Backend Adapter -> Retention Rule -> Notification Adapter)
* **`--skip-database`**: **Database Backup Adapter** 덤프 단계 건너뛰기
* **`--skip-secondary-sync`**: 2차 **Backend Adapter** 복제 건너뛰기
* **`--skip-retention`**: Retention/Prune 정리 단계 건너뛰기
* **`--dry-run`**: 실제 실행 없는 명령어 및 대상 시뮬레이션

### 5. `backup doctor` (진단, 검증 및 감사 증적 생성)
* **`backup doctor`**: 전체 시스템, 바이너리, 저장소 종합 진단
* **`backup doctor environment [--file <path>]`**: **Backup Environment** 권한(`700`/`600`) 및 보안 규정 검증 보고서 생성
* **`backup doctor time-sync [--file <path>]`**: NTP/Chrony 시각 동기화 검증 및 ISMS 증적 보고서 생성
* **`backup doctor restore-drill [--file <path>]`**: 스냅샷 복구 모의훈련 실행, RTO 측정 및 DB 무결성 검증 보고서 생성

### 6. `backup schedule` (스케줄러 관리)
* **`backup schedule enable`**: Systemd Timer (또는 Cron Fallback) 자동 백업 스케줄 등록
* **`backup schedule disable`**: 자동 백업 스케줄 해제
* **`backup schedule status`**: 타이머/스케줄러 현재 동작 상태 조회

### 7. 기타 운영 커맨드
* **`backup restore`**: 백업 데이터 및 DB 복원 실행
* **`backup snapshots`**: 1차/2차 저장소 스냅샷 목록 조회
* **`backup status`**: 저장소 위치 및 백업 상태 종합 조회
* **`backup update`**: 자기 자신(Rust 바이너리) 및 설정 갱신
* **`backup uninstall [--purge]`**: 스케줄 해제 및 바이너리 삭제 (`--purge` 시 설정/캐시 완전 제거)



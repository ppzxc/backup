# backup.sh 업데이트 및 설정 마이그레이션 구현 계획서 (TIDY FIRST)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `backup.sh` 버전 업데이트 시 구버전 설정을 안전하게 보존하고, 특정 기술 단어(/etc/restic, restic-audit 등)를 표준 도메인 단어인 `backup`으로 표준화하는 설정 마이그레이션 기능을 TDD/Tidy First 방식으로 구현합니다.

**Architecture:** 
1. **Tidy First (리팩토링)**: 동작 변경 없이 변수명 및 파일명 등의 명칭을 리팩토링하고 테스트 헬퍼를 갱신합니다.
2. **Behavior Change (동작 구현)**: 설정 디렉토리 자동 이관 및 레거시 서비스 정리(`ensure_backup_dir_migration`), 런타임 호환성 맵핑, `cmd_upgrade` 기능 확장 및 bats 단위 테스트 확장을 순차적으로 수행합니다.

**Tech Stack:** Bash, bats-core, systemd, python3 (for json/html reports mapping)

## Global Constraints
* `/etc/backup` 디렉토리는 권한 `700`, `backup.env` 및 `profiles.yaml` 파일은 권한 `600` 생성/이관 시 명시적으로 강제 적용.
* 민감정보(비밀번호, Secret) 노출 차단 및 화면 출력 시 마스킹 유지.
* 모든 변경사항은 Shellcheck 경고가 0건이어야 하며, `bats tests/` 테스트 스위트가 모두 통과해야 함.
* 스크립트 수정 시 최상단 `BACKUP_SCRIPT_VERSION` 변수 버전을 반드시 상승시킬 것.

---

## 1단계: TIDY FIRST (리팩토링 - 동작 변경 없음)

### Task 1: 헬프 텍스트 및 로그 포맷 경로명 리팩토링
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)

- [ ] **Step 1: backup.sh 내부 주석 및 도움말 텍스트 내 `/etc/restic`을 `/etc/backup`으로 교체**
  * 도움말 내용 및 예시 명령어에 적힌 경로명들 위주로 변경 (동작에 영향 없음)
- [ ] **Step 2: 린트(Shellcheck) 실행**
  * Run: `shellcheck backup.sh`
  * Expected: PASS (경고 0건)
- [ ] **Step 3: 전체 단위 테스트 실행**
  * Run: `bats tests/`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "refactor: rename configuration paths in help text and docs to /etc/backup"`

---

### Task 2: 글로벌 변수명 및 템플릿 기본값 리팩토링
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh:65-90)

- [ ] **Step 1: 글로벌 경로 변수 명칭을 `BACKUP_` 접두사로 리팩토링하되, 하위 호환성을 위해 `RESTIC_ETC_DIR`도 바인딩 지원**
  ```bash
  BACKUP_ETC_DIR="${BACKUP_ETC_DIR:-${RESTIC_ETC_DIR:-/etc/backup}}"
  BACKUP_ENV_FILE="${BACKUP_ENV_FILE:-${BACKUP_ETC_DIR}/backup.env}"
  BACKUP_SSH_KEY="${BACKUP_SSH_KEY:-${BACKUP_ETC_DIR}/backup_key}"
  BACKUP_UNIT_TEMPLATE="${BACKUP_UNIT_TEMPLATE:-${BACKUP_ETC_DIR}/backup-service.tmpl}"
  BACKUP_TIMER_TEMPLATE="${BACKUP_TIMER_TEMPLATE:-${BACKUP_ETC_DIR}/backup-timer.tmpl}"
  BACKUP_PROFILE_CONFIG_FILE="${BACKUP_PROFILE_CONFIG_FILE:-${BACKUP_ETC_DIR}/profiles.yaml}"
  ```
  기존 스크립트 전반에 걸쳐 사용 중인 `RESTIC_ETC_DIR` 참조 자리를 `BACKUP_ETC_DIR`로 변경.
- [ ] **Step 2: 린트 실행**
  * Run: `shellcheck backup.sh`
  * Expected: PASS
- [ ] **Step 3: 전체 단위 테스트 실행**
  * Run: `bats tests/`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "refactor: standardize config dir variables to BACKUP_ETC_DIR and update references"`

---

### Task 3: syslog 전송 태그 리팩토링
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh:420-438)

- [ ] **Step 1: logger 명령어 호출 시 사용되는 식별 태그 변경**
  * `logger -t restic-backup` -> `logger -t backup`으로 변경하여 기술 단어 제거
- [ ] **Step 2: 린트 실행**
  * Run: `shellcheck backup.sh`
  * Expected: PASS
- [ ] **Step 3: 전체 단위 테스트 실행**
  * Run: `bats tests/`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "refactor: change syslog tag from restic-backup to backup"`

---

### Task 4: 테스트 헬퍼 리팩토링 (Tidy)
**Files:**
- Modify: [tests/test_helper.bash](file:///home/ppzxc/projects/backup/tests/test_helper.bash)

- [ ] **Step 1: 테스트 헬퍼 내 경로 덮어쓰기 로직 보완**
  * `export BACKUP_ETC_DIR="${TEST_ROOT}/etc/backup"` 설정 추가
- [ ] **Step 2: 전체 단위 테스트 실행**
  * Run: `bats tests/`
  * Expected: PASS
- [ ] **Step 3: Commit**
  * Run: `git commit -am "test: update test helper to export BACKUP_ETC_DIR"`

---

## 2단계: BEHAVIOR CHANGE (동작 구현 및 기능 추가)

### Task 5: 호환성 맵핑 테이블 및 런타임 폴백 구현
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)

- [ ] **Step 1: 구버전 환경변수 -> 신버전 환경변수 호환성 매핑 선언 추가**
  ```bash
  declare -A COMPATIBILITY_MAP=(
    ["BACKUP_EXCLUDE_PATHS"]="BACKUP_EXCLUDES"
    ["BACKUP_SFTP_HOST"]="RCLONE_CONFIG_SYNO_BACKUP_HOST"
    ["BACKUP_SFTP_PORT"]="RCLONE_CONFIG_SYNO_BACKUP_PORT"
    ["BACKUP_SFTP_USER"]="RCLONE_CONFIG_SYNO_BACKUP_USER"
  )
  ```
- [ ] **Step 2: `require_backup_env` 내에 구버전 변수 자동 맵핑 바인딩 로직 구현**
  * `COMPATIBILITY_MAP`을 루프 돌며 신규 키가 없고 구버전 키가 있을 경우 바인딩하고 경고 로그(`log_warn`) 출력
- [ ] **Step 3: 린트 실행 및 테스트 검증**
  * Run: `shellcheck backup.sh && bats tests/`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "feat: implement compatibility mapping in require_backup_env"`

---

### Task 6: 설정 디렉토리 자동 이관 및 레거시 타이머 자동 정리 구현
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)

- [ ] **Step 1: `ensure_backup_dir_migration` 함수 추가**
  * 기존 `/etc/restic` 경로 하위의 설정을 `/etc/backup`으로 안전하게 복사 및 권한(`700`/`600`) 강제 적용
  * 기존 `restic-audit-daily.timer` 및 `restic-audit-restore-drill.timer` 서비스 파일 감지 시 `disable --now` 정지 및 유닛 파일 삭제(daemon-reload 수행)
- [ ] **Step 2: `require_backup_env` 시작 부분에 `ensure_backup_dir_migration` 주입**
- [ ] **Step 3: 린트 실행 및 테스트 검증**
  * Run: `shellcheck backup.sh && bats tests/`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "feat: add ensure_backup_dir_migration for config files and systemd service cleanup"`

---

### Task 7: 감사 보고서 생성 기본 경로 수정
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh:5025-5345)
- Modify: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)

- [ ] **Step 1: `cmd_audit` 내부의 보고서 파일 기본 생성 경로 변경**
  * `/var/log/restic-backup/...` -> `/data/backup/reports/...`
  * 생성 디렉토리에 대한 700 권한 셋업 및 리포트 파일 600 권한 준수
- [ ] **Step 2: `tests/cmd_audit.bats` 및 관련 테스트에서 `/var/log/restic-backup` 검증 경로 수정**
- [ ] **Step 3: 린트 실행 및 관련 테스트 통과 확인**
  * Run: `bats tests/cmd_audit.bats`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "feat: change default audit report directory to /data/backup/reports"`

---

### Task 8: systemd scheduler 유닛 명칭 표준화
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh:2619-2775)
- Modify: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)
- Modify: [tests/scheduler.bats](file:///home/ppzxc/projects/backup/tests/scheduler.bats)

- [ ] **Step 1: 감사 관련 systemd timer/service 파일명 표준화**
  * `restic-audit-daily` -> `backup-audit-daily`
  * `restic-audit-restore-drill` -> `backup-audit-restore-drill`
- [ ] **Step 2: 테스트 코드 내 하드코딩된 `restic-audit-*` 검증 문자열을 `backup-audit-*`으로 일괄 수정**
- [ ] **Step 3: 린트 실행 및 scheduler 테스트 통과 확인**
  * Run: `bats tests/scheduler.bats`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "feat: standardize audit scheduler systemd service and timer names"`

---

### Task 9: `cmd_upgrade` 기능 확장 및 버전 펌프
**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)

- [ ] **Step 1: `cmd_upgrade` 내에 백업본 생성 및 설정 마이그레이션 처리 추가**
  * 기존 `backup.env`가 있으면 `${BACKUP_ENV_FILE}.YYYYMMDD_HHMMSS.bak` 형태로 백업 파일 자동 복사 (권한 600)
  * 누락된 필수값이 있고 대화형 셸일 경우 위저드 입력 유도, 비대화형일 경우 기본값 주입
  * 최신화된 설정을 기반으로 `write_resticprofile_assets` 호출하여 프로필 갱신
- [ ] **Step 2: `BACKUP_SCRIPT_VERSION` 버전 상승**
  * 버전 값 `0.0.38` -> `0.0.39`로 범프
- [ ] **Step 3: 전체 통합 테스트 및 단위 테스트 완료 검증**
  * Run: `bats tests/`
  * Expected: PASS
- [ ] **Step 4: Commit**
  * Run: `git commit -am "feat: extend cmd_upgrade for safety settings migration and bump script version"`

# 이중 원격 소산 백업 파이프라인 구현 계획서

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `backup.sh`를 확장하여 1차 원격 백업 완료 후 2차 원격 소산지로 `restic copy`를 수행하는 이중 원격 백업 파이프라인을 구축하고, 도움말 문서(Cobra CLI 스타일)를 함께 업데이트합니다.

**Architecture:** 
1. `backup.env`에 `SECONDARY_` 접두사를 가진 2차 접속 변수군을 추가합니다.
2. `profiles.yaml` 생성 시 1차 원격 프로필과 2차 소산 프로필을 동시에 렌더링합니다.
3. `cmd_run`에서 1차 백업 후 `restic copy`로 2차 소산을 진행하고 최종 통합 1회 알림을 발송합니다.
4. `cmd_init`, `cmd_status`, `cmd_audit`(--restore-drill 포함), `cmd_uninstall`을 2차 원격 저장소 대응으로 연계 확장합니다.

**Tech Stack:** Bash shell, Restic, Rclone, Resticprofile, Bats test framework

## Global Constraints
- `BACKUP_SCRIPT_VERSION` 변수 버전을 변경 사항 적용 시 범프(상승)해야 함.
- `/etc/restic` 디렉터리는 권한 `700`, `backup.env` 및 `profiles.yaml` 파일은 권한 `600`을 명시적으로 강제 적용.
- 비밀번호 등 민감 자격 증명은 화면 출력이나 로그, systemd 유닛 파일에 평문 노출 금지 및 마스킹 처리.

---

### Task 1: CLI 옵션 파싱 확장 및 유효성 검증 추가

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/validators.bats`

**Interfaces:**
- Consumes: `parse_opts_into`, `validate_backend`
- Produces: `secondary-` 옵션들의 바인딩 값 및 `validate_secondary_config` 유효성 검증기

- [ ] **Step 1: `validators.bats`에 실패하는 테스트 추가**
  ```bash
  # tests/validators.bats 최하단에 추가
  @test "validate_secondary_config fails when secondary-backend is invalid" {
    run validate_secondary_config "invalid_backend"
    [ "$status" -ne 0 ]
  }
  ```
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/validators.bats`
  Expected: FAIL (validate_secondary_config command not found)
- [ ] **Step 3: `backup.sh`에 `validate_secondary_config` 구현 및 `cmd_setting` 옵션 리스트 확장**
  ```bash
  # backup.sh의 validate_backend 근처에 추가
  validate_secondary_config() {
    local backend="$1"
    if [[ "$backend" != "s3" && "$backend" != "sftp" ]]; then
      return 1
    fi
    return 0
  }
  ```
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/validators.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  ```bash
  git add backup.sh tests/validators.bats
  git commit -m "feat: add secondary option parser and validation helper"
  ```

---

### Task 2: 2차 백업 설정 및 환경 변수 저장 구현 (`cmd_setting`)

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/cmd_setting_s3.bats`

**Interfaces:**
- Consumes: `cmd_setting`
- Produces: `SECONDARY_` 자격증명 변수를 파싱하고 `backup.env`에 통합 보관하는 로직

- [ ] **Step 1: `cmd_setting_s3.bats`에 실패하는 테스트 추가**
  ```bash
  # tests/cmd_setting_s3.bats에 추가
  @test "cmd_setting registers secondary-backend parameters in backup.env" {
    # 2차 S3 설정과 함께 setting 기동 테스트
    run cmd_setting --backend sftp --host 127.0.0.1 --user backup --password 'pw123' --targets '/data' --secondary-backend s3 --secondary-endpoint 'https://s3.com' --secondary-bucket 'sec-bucket'
    [ "$status" -eq 0 ]
    grep -q "SECONDARY_BACKEND=\"s3\"" "$BACKUP_ENV_FILE"
    grep -q "SECONDARY_RESTIC_REPOSITORY=\"s3:https://s3.com/sec-bucket/" "$BACKUP_ENV_FILE"
  }
  ```
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/cmd_setting_s3.bats`
  Expected: FAIL
- [ ] **Step 3: `backup.sh`의 `cmd_setting` 및 렌더러 함수 수정**
  - `parse_opts_into` 부분에 `secondary-` 계열 변수들 모두 추가 바인딩.
  - S3 및 SFTP 렌더러가 호출될 때 `SECONDARY_` 계열 환경변수가 `backup.env`에 누적되어 출력되도록 렌더러 내부를 보완.
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/cmd_setting_s3.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  ```bash
  git add backup.sh tests/cmd_setting_s3.bats
  git commit -m "feat: support storing secondary-backend settings in backup.env"
  ```

---

### Task 3: Cobra 스타일 도움말 문서 업데이트 (`help_xxx`)

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/help.bats`

- [ ] **Step 1: `help.bats`에 실패하는 테스트 추가**
  ```bash
  # tests/help.bats에 추가
  @test "help_setting output contains secondary option descriptions" {
    run help_setting
    echo "$output" | grep -q "secondary-backend"
  }
  ```
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/help.bats`
  Expected: FAIL
- [ ] **Step 3: `backup.sh` 내 `help_setting` 함수에 Cobra 스타일 플래그 가이드 추가**
  - `--secondary-backend`, `--secondary-password`, `--secondary-endpoint` 등 신규 플래그 및 설명 추가.
  - 예제 세션에 2차 원격 소산 설정에 대한 예시 CLI 커맨드라인 표기.
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/help.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  ```bash
  git add backup.sh tests/help.bats
  git commit -m "docs: update help menus in Cobra style for secondary options"
  ```

---

### Task 4: 다중 프로필 `profiles.yaml` 렌더링 확장

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/resticprofile_config.bats`

**Interfaces:**
- Consumes: `render_resticprofile_config`
- Produces: 1차 원격 프로필 블록 및 2차 `[profile-name]-secondary` 프로필 블록이 합성된 yaml 내용

- [ ] **Step 1: `resticprofile_config.bats`에 실패하는 테스트 추가**
  ```bash
  # tests/resticprofile_config.bats에 추가
  @test "render_resticprofile_config prints dual profiles when secondary backup is configured" {
    export SECONDARY_BACKEND="s3"
    export SECONDARY_RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/sec-bucket"
    run render_resticprofile_config "test-host" "*-*-* 02:00:00"
    [ "$status" -eq 0 ]
    echo "$output" | grep -q "test-host-secondary:"
    echo "$output" | grep -q "repository: \"s3:https://s3.amazonaws.com/sec-bucket\""
  }
  ```
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/resticprofile_config.bats`
  Expected: FAIL
- [ ] **Step 3: `backup.sh` 내 `render_resticprofile_config` 함수 수정**
  - `profiles.yaml` 출력 시 `SECONDARY_BACKEND`가 선언되어 있다면 두 번째 프로필 블록(`$profile_name-secondary`)을 이어서 출력하도록 구현.
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/resticprofile_config.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  ```bash
  git add backup.sh tests/resticprofile_config.bats
  git commit -m "feat: render dual profiles in profiles.yaml"
  ```

---

### Task 5: 2차 원격 저장소 연결성 체크 및 초기화 (`cmd_init`)

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/cmd_init.bats`

**Interfaces:**
- Consumes: `cmd_init`
- Produces: 1차 `init` 후 2차 저장소도 `restic init` 하도록 기동 및 에러 핸들링

- [ ] **Step 1: `cmd_init.bats`에 실패하는 테스트 추가**
  ```bash
  # tests/cmd_init.bats에 추가
  @test "cmd_init initializes secondary remote repository if configured" {
    # stub command 및 환경 변수 구성 후 테스트 진행
    # 2차 init 로직이 동작하는지 호출 체크
    ...
  }
  ```
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/cmd_init.bats`
  Expected: FAIL
- [ ] **Step 3: `backup.sh` 내 `cmd_init` 함수 수정**
  - 1차 초기화 후, `SECONDARY_BACKEND`가 있으면 2차 원격지에 대해 `restic init` 실행 및 에러 감지 로직 구현.
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/cmd_init.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  ```bash
  git add backup.sh tests/cmd_init.bats
  git commit -m "feat: support initializing secondary repositories in cmd_init"
  ```

---

### Task 6: 2차 원격 소산 파이프라인 구동 및 통합 알림 (`cmd_run`)

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/cmd_run.bats`

**Interfaces:**
- Consumes: `cmd_run`, `send_notification`
- Produces: 1차 백업 ➡️ 2차 `restic copy` ➡️ 2차 prune(주기적) ➡️ 1회 통합 알림

- [ ] **Step 1: `cmd_run.bats`에 실패하는 테스트 추가**
  ```bash
  # tests/cmd_run.bats에 추가
  @test "cmd_run triggers restic copy after successful primary backup" {
    ...
  }
  ```
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/cmd_run.bats`
  Expected: FAIL
- [ ] **Step 3: `backup.sh` 내 `cmd_run` 및 알림 함수 수정**
  - 1차 백업 후 `restic copy --repo2 $SECONDARY_RESTIC_REPOSITORY` (또는 `resticprofile copy`) 실행부 체이닝.
  - 2차 `forget --prune` 동작은 일요일 새벽이나 매 N일 주기로 동작하도록 스키핑 로직 적용.
  - 1차 및 2차 결과를 수집해 `send_notification`에서 한꺼번에 포맷팅해 쏘도록 개편.
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/cmd_run.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  ```bash
  git add backup.sh tests/cmd_run.bats
  git commit -m "feat: implement copy pipeline and unified notification in cmd_run"
  ```

---

### Task 7: 이중 상태 진단 및 복구 모의훈련 연동 (`status`, `audit`)

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/cmd_status.bats`, `/home/ppzxc/projects/backup/tests/cmd_audit.bats`

**Interfaces:**
- Consumes: `cmd_status`, `cmd_audit`, `--restore-drill`
- Produces: 1차/2차 저장소 스냅샷 비교 및 양방향 복구 훈련 검증 기능

- [ ] **Step 1: `cmd_status.bats` 및 `cmd_audit.bats`에 실패하는 테스트 추가**
  - 1차/2차 스냅샷 목록을 각각 정상 로드하여 표시하는지 점검하는 테스트 케이스.
  - `--restore-drill` 시 2차 원격지에서 복구가 함께 호출되는지 점검하는 테스트 케이스.
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/cmd_status.bats` & `bats tests/cmd_audit.bats`
  Expected: FAIL
- [ ] **Step 3: `backup.sh` 내 `cmd_status` 및 `cmd_audit` 수정**
  - `status`에서 2차 저장소의 스냅샷 목록 및 연결 상태를 렌더링.
  - `audit` 복구 모의훈련 시 1차 스냅샷뿐만 아니라 2차 스냅샷의 임의의 작은 파일도 복구 테스트하여 결과 보고서에 통합 기록.
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/cmd_status.bats` & `bats tests/cmd_audit.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  ```bash
  git add backup.sh tests/cmd_status.bats tests/cmd_audit.bats
  git commit -m "feat: extend status and audit to support dual remote storage and dual restore drill"
  ```

---

### Task 8: 기존 설정 업그레이드 지원 및 uninstall 연계 (`upgrade-config`, `uninstall`)

**Files:**
- Modify: `/home/ppzxc/projects/backup/backup.sh`
- Test: `/home/ppzxc/projects/backup/tests/cmd_uninstall.bats`, 신규 `/home/ppzxc/projects/backup/tests/cmd_upgrade_config.bats`

**Interfaces:**
- Consumes: `cmd_upgrade_config`, `cmd_uninstall`
- Produces: 기존 설정을 2차 설정으로 변환하는 기능 및 2차 자격증명 정리 기능

- [ ] **Step 1: `cmd_upgrade_config.bats`에 실패하는 테스트 추가**
  - 기존 단일 S3 백업 설정이 존재할 때 `upgrade-config` 명령어가 해당 설정을 `SECONDARY_` 변수로 변환하고 1차를 새로 지정하는지 검증하는 테스트.
- [ ] **Step 2: 테스트 실행 및 실패 확인**
  Run: `bats tests/cmd_upgrade_config.bats`
  Expected: FAIL
- [ ] **Step 3: `backup.sh`에 `cmd_upgrade_config` 명령어 추가 및 `cmd_uninstall` 고도화**
  - 기존 `backup.env` 파싱 후 `SECONDARY_`로 재작성 및 `backup.sh config` 실행 로직 구현.
  - `uninstall --purge` 시 2차 캐시 및 자격증명 정보까지 완전 삭제하도록 래핑.
- [ ] **Step 4: 테스트 실행 및 성공 확인**
  Run: `bats tests/cmd_upgrade_config.bats`
  Expected: PASS
- [ ] **Step 5: 변경사항 커밋**
  - 스크립트 최상단 `BACKUP_SCRIPT_VERSION` 변수를 1단계 상승(범프) 시킵니다.
  ```bash
  git add backup.sh tests/cmd_uninstall.bats tests/cmd_upgrade_config.bats
  git commit -m "feat: add upgrade-config command, clean secondary caches in uninstall, and bump script version"
  ```

# restic 백업 자동화 스크립트 (backup.sh) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** RHEL 계열(dnf) 서버에 배포해 restic 백업을 세팅·운영하는 단일 파일 `backup.sh`를 만든다. 백엔드는 S3 호환 스토리지 또는 rclone SFTP 중 선택하며, `install`/`setting`/`init`/`schedule`/`run`/`status`/`uninstall`/`wizard` 서브커맨드를 제공한다.

**Architecture:** functional core / imperative shell. 순수 함수(`resolve_*`, `validate_*`, `render_*`, `parse_long_opts`)는 외부 명령을 호출하지 않고 입력→stdout/종료코드만 다뤄 bats로 직접 테스트한다. 실제 시스템을 건드리는 함수(`cmd_*`, `dnf_install_packages`, `self_install_copy`, `write_secure_file`, `generate_ssh_key_if_missing`, `systemd_*`)는 순수 함수가 만든 문자열/판정 결과를 받아 실행만 하며, 경로를 담은 전역 변수(`RESTIC_ETC_DIR` 등)를 환경변수로 오버라이드 가능하게 만들어 테스트에서 실서버 대신 임시 디렉터리 + 스텁 커맨드로 검증한다.

**Tech Stack:** Bash(`set -euo pipefail`), bats-core(단위테스트), Docker Compose + MinIO + SFTP 컨테이너(통합테스트), shellcheck(정적분석), restic, rclone, systemd, dnf.

## Global Constraints

- 스크립트는 root(EUID 0)로만 실행 가능해야 한다(테스트에서는 `REQUIRE_ROOT_CHECK=0`으로 우회).
- `curl | bash` 파이프 실행은 지원하지 않는다 — 파일로 내려받아 실행하는 것을 전제로 한다.
- 설정값 우선순위: **CLI 플래그 > 환경변수 > 기존 backup.env(재사용 가능한 필드에 한함) > 기본값.** 백엔드 자격증명(host/port/user/endpoint/bucket/access-key/secret-key)은 기존 backup.env로부터 역추출하지 않는다(범위 밖) — CLI 플래그 또는 `BACKUP_*` 환경변수로만 채운다. targets/excludes/keep-daily/weekly/monthly는 기존 backup.env 값도 폴백으로 사용한다.
- 필수값 누락 시 대화형으로 되묻지 않고, 무엇이 왜 없는지 + 복사해서 쓸 수 있는 다음 명령을 출력하고 종료(exit 1)한다. 이미 아는 값은 채우고 모르는 값은 `<PLACEHOLDER>`로 남긴다. **credential류 값(password/access-key/secret-key)은 알고 있어도 항상 placeholder로 표시한다**(화면 노출 방지).
- 파일/디렉터리 권한: `/etc/restic` 700, `backup.env` 600, SSH 키 600/pub 644, systemd 유닛 644. 매 생성/수정 시 명시적으로 강제한다.
- 로그는 별도 파일 없이 stdout + `logger -t restic-backup`(있으면)만 사용한다. 비밀정보는 로그/화면 요약에 절대 평문 노출하지 않는다.
- SSH 키는 SFTP 전용이며 S3 백엔드에는 적용되지 않는다(HMAC 서명 인증). S3는 최소권한 버킷 정책 JSON 안내로 대체한다.
- 호스트 1대당 restic 저장소 1개(단일 backup.env)만 지원한다. STS AssumeRole, 자동 비밀번호 생성은 범위 밖이다.
- `install`/`setting`은 `--dry-run`을 지원해 실제 변경 없이 계획만 출력할 수 있어야 한다.
- 새로 만드는 파일 중 `tests/`와 `backup.sh`, `AGENTS.md`, `CLAUDE.md`, `.gitignore`만 git으로 추적한다(`docs/`는 의도적으로 미추적).
- (Task 18~) resticprofile 버전/체크섬은 `backup.sh` 상단 상수(`RESTICPROFILE_VERSION`, `RESTICPROFILE_SHA256`, `RESTICPROFILE_URL`)로 고정한다. `curl | sh` 형태의 공식 install.sh는 사용하지 않는다 — 직접 tarball을 받아 SHA256 대조 후에만 설치한다. 버전을 올리려면 이 상수들을 수정하고 재검증 후 배포한다(오버라이드 플래그 없음).
- (Task 18~) GitHub(`github.com`, `objects.githubusercontent.com`) 아웃바운드 접근 가능을 전제로 한다. 접근 불가 환경에 대한 오프라인 설치 경로는 범위 밖이다 — 실패 시 안내 메시지만 출력한다.
- (Task 18~) `backup.env`(및 `RCLONE_CONFIG_*`/`AWS_*`/`RESTIC_PASSWORD` 등 거기서 export되는 모든 값)가 계속 유일한 source of truth다. `profiles.yaml`은 항상 그로부터 파생 렌더링되는 산출물이며, `setting`/`wizard`가 `backup.env`를 쓸 때와 `cmd_run` 실행 시점 양쪽에서 재렌더링한다 — 스케줄 경로는 `wizard.sh`(즉 `backup.sh`)를 거치지 않고 resticprofile이 직접 실행되므로, 저장 시점 렌더링이 없으면 스케줄 경로가 최신 자격증명을 못 받는다.
- (Task 18~) `profiles.yaml`에는 레포 비밀번호/S3·rclone 자격증명이 `env:` 블록으로 그대로 들어간다("YAML엔 비밀값 없음"이라는 초기 가설은 스케줄 경로 조사 중 기각됨) — 따라서 `/etc/restic/profiles.yaml`도 `backup.env`와 동일하게 600 권한을 강제한다.
- (Task 18~) `cmd_status`는 resticprofile로 위임하지 않는다(읽기 전용 조회라 이점이 없고 `profiles.yaml` 존재를 전제하는 새 실패 모드만 늘어남) — 기존처럼 `restic` 직접 호출을 유지한다.

---

## File Structure

```
backup.sh                              # 메인 스크립트 (전체 로직)
tests/test_helper.bash                 # bats 공용 setup: 경로 오버라이드 + 커맨드 스텁 + backup.sh source
tests/help.bats                        # Task 1
tests/resolve_value.bats               # Task 2
tests/validators.bats                  # Task 3
tests/parse_long_opts.bats             # Task 4
tests/render_hints.bats                # Task 5
tests/render_units.bats                # Task 6
tests/cmd_install.bats                 # Task 7
tests/cmd_setting_sftp.bats            # Task 8
tests/cmd_setting_s3.bats              # Task 9
tests/cmd_init.bats                    # Task 10
tests/cmd_schedule.bats                # Task 11
tests/cmd_run.bats                     # Task 12
tests/cmd_status.bats                  # Task 13
tests/cmd_uninstall.bats               # Task 14
tests/cmd_wizard.bats                  # Task 15
tests/integration/docker-compose.yml   # Task 16
tests/integration/run.sh               # Task 16
tests/MANUAL_CHECKLIST.md              # Task 17
tests/resticprofile_config.bats        # Task 20
```

> Task 18~22는 `/grilling` 세션(2026-07-10)에서 합의된 resticprofile 마이그레이션이다.
> resticprofile은 (1) 백업 실행 오케스트레이션(backup+forget+prune), (2) 스케줄링(systemd 유닛
> 생성/enable/disable), (3) stale lock 처리 3곳만 대체한다. install/wizard/setting/SSH
> 키생성/S3 정책 안내/status/uninstall/입력검증은 그대로 커스텀 bash로 남는다.

각 bats 파일은 `load test_helper.bash`로 공용 setup을 재사용하고, 자신이 검증하는 함수/커맨드에만 집중한다.

---

### Task 1: 스크립트 뼈대 + 공용 테스트 하네스 + help/dispatcher

**Files:**
- Create: `backup.sh`
- Create: `tests/test_helper.bash`
- Create: `tests/help.bats`

**Interfaces:**
- Produces: 전역 변수 `RESTIC_ETC_DIR`, `BACKUP_ENV_FILE`, `BACKUP_SSH_KEY`, `BACKUP_SCRIPT_INSTALL_PATH`, `SYSTEMD_UNIT_DIR`, `SYSTEMD_SERVICE_FILE`, `SYSTEMD_TIMER_FILE`, `DEFAULT_TARGETS`, `DEFAULT_EXCLUDES`, `DEFAULT_KEEP_DAILY`, `DEFAULT_KEEP_WEEKLY`, `DEFAULT_KEEP_MONTHLY`, `DEFAULT_ON_CALENDAR`, `DEFAULT_SFTP_PORT`. 함수 `log_info()`, `log_error()`, `die(msg, [code])`, `require_root()`, `render_help()`, `main()`.
- Consumes: 없음(최초 태스크).

- [ ] **Step 1: 실패하는 테스트 작성 — `tests/test_helper.bash`**

```bash
#!/usr/bin/env bash

setup_backup_sh_env() {
  export TEST_ROOT="${BATS_TEST_TMPDIR}/root"
  mkdir -p "$TEST_ROOT"

  export RESTIC_ETC_DIR="${TEST_ROOT}/etc/restic"
  export BACKUP_ENV_FILE="${RESTIC_ETC_DIR}/backup.env"
  export BACKUP_SSH_KEY="${RESTIC_ETC_DIR}/backup_key"
  export BACKUP_SCRIPT_INSTALL_PATH="${TEST_ROOT}/usr/local/sbin/backup.sh"
  export SYSTEMD_UNIT_DIR="${TEST_ROOT}/etc/systemd/system"
  mkdir -p "$SYSTEMD_UNIT_DIR" "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"

  export STUB_BIN="${BATS_TEST_TMPDIR}/stub-bin"
  mkdir -p "$STUB_BIN"
  export PATH="${STUB_BIN}:${PATH}"

  export REQUIRE_ROOT_CHECK=0

  # shellcheck source=/dev/null
  source "${BATS_TEST_DIRNAME}/../backup.sh"
}

stub_command() {
  local name="$1" body="$2"
  cat > "${STUB_BIN}/${name}" <<STUB
#!/usr/bin/env bash
${body}
STUB
  chmod +x "${STUB_BIN}/${name}"
}

stub_call_log() {
  printf '%s\n' "${STUB_BIN}/${1}.calls"
}
```

```bash
cat > tests/help.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_help prints usage mentioning all subcommands" {
  run render_help
  [ "$status" -eq 0 ]
  [[ "$output" == *"install"* ]]
  [[ "$output" == *"setting"* ]]
  [[ "$output" == *"init"* ]]
  [[ "$output" == *"schedule"* ]]
  [[ "$output" == *"run"* ]]
  [[ "$output" == *"status"* ]]
  [[ "$output" == *"uninstall"* ]]
  [[ "$output" == *"wizard"* ]]
}

@test "main with no args prints help and exits 0" {
  run main
  [ "$status" -eq 0 ]
  [[ "$output" == *"install"* ]]
}

@test "main with -h exits 0" {
  run main -h
  [ "$status" -eq 0 ]
}

@test "main with --help exits 0" {
  run main --help
  [ "$status" -eq 0 ]
}

@test "main with unknown subcommand prints help and exits 1" {
  run main bogus-command
  [ "$status" -eq 1 ]
  [[ "$output" == *"install"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/help.bats`
Expected: FAIL (`backup.sh: No such file or directory` 또는 함수 없음 에러)

- [ ] **Step 3: `backup.sh` 최소 구현 작성**

```bash
#!/usr/bin/env bash
set -euo pipefail

RESTIC_ETC_DIR="${RESTIC_ETC_DIR:-/etc/restic}"
BACKUP_ENV_FILE="${BACKUP_ENV_FILE:-${RESTIC_ETC_DIR}/backup.env}"
BACKUP_SSH_KEY="${BACKUP_SSH_KEY:-${RESTIC_ETC_DIR}/backup_key}"
BACKUP_SCRIPT_INSTALL_PATH="${BACKUP_SCRIPT_INSTALL_PATH:-/usr/local/sbin/backup.sh}"
SYSTEMD_UNIT_DIR="${SYSTEMD_UNIT_DIR:-/etc/systemd/system}"
SYSTEMD_SERVICE_FILE="${SYSTEMD_UNIT_DIR}/restic-backup.service"
SYSTEMD_TIMER_FILE="${SYSTEMD_UNIT_DIR}/restic-backup.timer"

DEFAULT_TARGETS="/var/log"
DEFAULT_EXCLUDES="/tmp/*,/var/tmp/*"
DEFAULT_KEEP_DAILY=7
DEFAULT_KEEP_WEEKLY=4
DEFAULT_KEEP_MONTHLY=12
DEFAULT_ON_CALENDAR="*-*-* 02:00:00"
DEFAULT_SFTP_PORT=22

log_info() {
  printf '%s\n' "$1"
  command -v logger >/dev/null 2>&1 && logger -t restic-backup -- "$1" || true
}

log_error() {
  printf 'ERROR: %s\n' "$1" >&2
  command -v logger >/dev/null 2>&1 && logger -t restic-backup -- "ERROR: $1" || true
}

die() {
  log_error "$1"
  exit "${2:-1}"
}

require_root() {
  if [[ "${REQUIRE_ROOT_CHECK:-1}" == "1" && "${EUID}" -ne 0 ]]; then
    die "이 명령은 root 권한으로 실행해야 합니다. sudo로 다시 실행하세요." 1
  fi
}

render_help() {
  cat <<'EOF'
backup.sh - restic 기반 백업 설치/운영 스크립트

사용법:
  backup.sh install [--force] [--dry-run]
  backup.sh setting --backend <s3|sftp> [옵션...] [--force] [--dry-run]
  backup.sh init
  backup.sh schedule enable [--on-calendar "<OnCalendar식>"]
  backup.sh schedule disable
  backup.sh run
  backup.sh status
  backup.sh uninstall [--purge]
  backup.sh wizard
  backup.sh -h | --help
EOF
}

main() {
  if [[ $# -eq 0 ]]; then
    render_help
    return 0
  fi

  case "$1" in
    -h|--help)
      render_help
      return 0
      ;;
    install|setting|init|schedule|run|status|uninstall|wizard)
      : # 다음 태스크에서 각 cmd_* 로 분기
      ;;
    *)
      render_help
      return 1
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
fi
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/help.bats`
Expected: 5개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/test_helper.bash tests/help.bats
git commit -m "feat: add backup.sh skeleton with help dispatcher"
```

---

### Task 2: `resolve_value` — 설정값 우선순위 해석

**Files:**
- Modify: `backup.sh`
- Test: `tests/resolve_value.bats`

**Interfaces:**
- Produces: `resolve_value <cli> <env> <file> <default>` — 첫 번째 non-empty 값을 stdout에 출력하고 0 반환. 전부 비어있으면 아무것도 출력하지 않고 1 반환.
- Consumes: 없음(순수 함수, Task 1의 스크립트 파일에 함수만 추가).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/resolve_value.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "cli value wins over everything" {
  run resolve_value "cli-val" "env-val" "file-val" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "cli-val" ]
}

@test "env value wins when cli is empty" {
  run resolve_value "" "env-val" "file-val" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "env-val" ]
}

@test "file value wins when cli and env are empty" {
  run resolve_value "" "" "file-val" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "file-val" ]
}

@test "default is used when everything else is empty" {
  run resolve_value "" "" "" "default-val"
  [ "$status" -eq 0 ]
  [ "$output" = "default-val" ]
}

@test "returns 1 and prints nothing when all are empty" {
  run resolve_value "" "" "" ""
  [ "$status" -eq 1 ]
  [ -z "$output" ]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/resolve_value.bats`
Expected: FAIL (`resolve_value: command not found`)

- [ ] **Step 3: 구현 추가**

`backup.sh`의 `render_help()` 함수 앞에 추가:

```bash
resolve_value() {
  local cli="$1" env="$2" file="$3" default="$4"
  if [[ -n "$cli" ]]; then printf '%s' "$cli"; return 0; fi
  if [[ -n "$env" ]]; then printf '%s' "$env"; return 0; fi
  if [[ -n "$file" ]]; then printf '%s' "$file"; return 0; fi
  if [[ -n "$default" ]]; then printf '%s' "$default"; return 0; fi
  return 1
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/resolve_value.bats`
Expected: 5개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/resolve_value.bats
git commit -m "feat: add resolve_value config precedence resolver"
```

---

### Task 3: 값 검증 함수 (`validate_backend`, `validate_port`, `validate_positive_int`)

**Files:**
- Modify: `backup.sh`
- Test: `tests/validators.bats`

**Interfaces:**
- Produces: `validate_backend <value>`, `validate_port <value>`, `validate_positive_int <value> <label>` — 유효하면 아무 출력 없이 0 반환, 무효하면 한 줄 에러 메시지를 stdout에 출력하고 1 반환.
- Consumes: 없음(순수 함수).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/validators.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "validate_backend accepts s3" {
  run validate_backend "s3"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "validate_backend accepts sftp" {
  run validate_backend "sftp"
  [ "$status" -eq 0 ]
}

@test "validate_backend rejects unknown value" {
  run validate_backend "ftp"
  [ "$status" -eq 1 ]
  [[ "$output" == *"s3"* ]]
  [[ "$output" == *"sftp"* ]]
  [[ "$output" == *"ftp"* ]]
}

@test "validate_port accepts 22" {
  run validate_port "22"
  [ "$status" -eq 0 ]
}

@test "validate_port accepts 65535" {
  run validate_port "65535"
  [ "$status" -eq 0 ]
}

@test "validate_port rejects 0" {
  run validate_port "0"
  [ "$status" -eq 1 ]
}

@test "validate_port rejects 65536" {
  run validate_port "65536"
  [ "$status" -eq 1 ]
}

@test "validate_port rejects non-numeric" {
  run validate_port "abc"
  [ "$status" -eq 1 ]
  [[ "$output" == *"abc"* ]]
}

@test "validate_positive_int accepts positive integer" {
  run validate_positive_int "7" "keep-daily"
  [ "$status" -eq 0 ]
}

@test "validate_positive_int rejects zero" {
  run validate_positive_int "0" "keep-daily"
  [ "$status" -eq 1 ]
  [[ "$output" == *"keep-daily"* ]]
}

@test "validate_positive_int rejects non-numeric" {
  run validate_positive_int "seven" "keep-weekly"
  [ "$status" -eq 1 ]
  [[ "$output" == *"keep-weekly"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/validators.bats`
Expected: FAIL (함수 없음)

- [ ] **Step 3: 구현 추가**

`resolve_value()` 다음에 추가:

```bash
validate_backend() {
  local value="$1"
  if [[ "$value" == "s3" || "$value" == "sftp" ]]; then
    return 0
  fi
  printf "backend은 's3' 또는 'sftp'여야 합니다 (입력값: '%s')\n" "$value"
  return 1
}

validate_port() {
  local value="$1"
  if [[ "$value" =~ ^[0-9]+$ ]] && (( value >= 1 && value <= 65535 )); then
    return 0
  fi
  printf "port는 1~65535 사이의 정수여야 합니다 (입력값: '%s')\n" "$value"
  return 1
}

validate_positive_int() {
  local value="$1" label="$2"
  if [[ "$value" =~ ^[0-9]+$ ]] && (( value >= 1 )); then
    return 0
  fi
  printf "%s는 1 이상의 정수여야 합니다 (입력값: '%s')\n" "$label" "$value"
  return 1
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/validators.bats`
Expected: 11개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/validators.bats
git commit -m "feat: add validate_backend/validate_port/validate_positive_int"
```

---

### Task 4: `parse_long_opts` — 범용 long-option 파서

**Files:**
- Modify: `backup.sh`
- Test: `tests/parse_long_opts.bats`

**Interfaces:**
- Produces: `parse_long_opts <spec> -- <args...>`. `spec`는 공백구분 플래그 목록이며 값이 필요한 플래그는 `이름:` 형식(예: `"backend: host: port: force dry-run"`). stdout에 인식된 플래그마다 `이름<TAB>값` 한 줄씩 출력(불리언 플래그는 값이 `1`), 반복 플래그는 여러 줄로 출력. 알 수 없는 플래그/값 누락/예상치 못한 위치인자는 에러 메시지를 stdout에 출력하고 1 반환.
- Consumes: 없음(순수 함수). 이후 모든 `cmd_*`가 이 함수를 사용한다.

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/parse_long_opts.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "parses a single value flag with space form" {
  run parse_long_opts "host:" -- --host 1.2.3.4
  [ "$status" -eq 0 ]
  [ "$output" = $'host\t1.2.3.4' ]
}

@test "parses a single value flag with equals form" {
  run parse_long_opts "host:" -- --host=1.2.3.4
  [ "$status" -eq 0 ]
  [ "$output" = $'host\t1.2.3.4' ]
}

@test "parses a boolean flag" {
  run parse_long_opts "force" -- --force
  [ "$status" -eq 0 ]
  [ "$output" = $'force\t1' ]
}

@test "parses repeated value flags into multiple lines" {
  run parse_long_opts "exclude:" -- --exclude '/tmp/*' --exclude '/var/tmp/*'
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = $'exclude\t/tmp/*' ]
  [ "${lines[1]}" = $'exclude\t/var/tmp/*' ]
}

@test "parses mixed value and boolean flags" {
  run parse_long_opts "backend: force" -- --backend s3 --force
  [ "$status" -eq 0 ]
  [ "${lines[0]}" = $'backend\ts3' ]
  [ "${lines[1]}" = $'force\t1' ]
}

@test "rejects unknown flag" {
  run parse_long_opts "host:" -- --bogus x
  [ "$status" -eq 1 ]
  [[ "$output" == *"bogus"* ]]
}

@test "rejects value flag missing its value" {
  run parse_long_opts "host:" -- --host
  [ "$status" -eq 1 ]
  [[ "$output" == *"host"* ]]
}

@test "rejects unexpected positional argument" {
  run parse_long_opts "host:" -- extra-arg
  [ "$status" -eq 1 ]
  [[ "$output" == *"extra-arg"* ]]
}

@test "empty args with empty spec succeeds with no output" {
  run parse_long_opts "" --
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/parse_long_opts.bats`
Expected: FAIL (함수 없음)

- [ ] **Step 3: 구현 추가**

`validate_positive_int()` 다음에 추가:

```bash
parse_long_opts() {
  local spec="$1"
  shift
  if [[ "${1:-}" != "--" ]]; then
    printf 'parse_long_opts: 내부 오류, "--" 구분자가 필요합니다\n'
    return 1
  fi
  shift

  local -A takes_value=()
  local name
  for name in $spec; do
    if [[ "$name" == *: ]]; then
      takes_value["${name%:}"]=1
    else
      takes_value["$name"]=0
    fi
  done

  while [[ $# -gt 0 ]]; do
    local arg="$1"
    case "$arg" in
      --*=*)
        local key="${arg%%=*}"
        key="${key#--}"
        local val="${arg#*=}"
        if [[ -z "${takes_value[$key]+x}" ]]; then
          printf '알 수 없는 플래그: --%s\n' "$key"
          return 1
        fi
        printf '%s\t%s\n' "$key" "$val"
        shift
        ;;
      --*)
        local key="${arg#--}"
        if [[ -z "${takes_value[$key]+x}" ]]; then
          printf '알 수 없는 플래그: --%s\n' "$key"
          return 1
        fi
        if [[ "${takes_value[$key]}" == 1 ]]; then
          if [[ $# -lt 2 ]]; then
            printf -- '--%s 플래그는 값이 필요합니다\n' "$key"
            return 1
          fi
          printf '%s\t%s\n' "$key" "$2"
          shift 2
        else
          printf '%s\t1\n' "$key"
          shift
        fi
        ;;
      *)
        printf '예상치 못한 인자: %s\n' "$arg"
        return 1
        ;;
    esac
  done
  return 0
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/parse_long_opts.bats`
Expected: 9개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/parse_long_opts.bats
git commit -m "feat: add parse_long_opts generic long-option parser"
```

---

### Task 5: 안내 메시지 렌더러 (`render_placeholder_or_value`, `render_setting_hint_sftp`, `render_setting_hint_s3`, `render_missing_settings_message`)

**Files:**
- Modify: `backup.sh`
- Test: `tests/render_hints.bats`

**Interfaces:**
- Produces:
  - `render_placeholder_or_value <value> <placeholder_name>` → value가 있으면 그대로, 없으면 `<placeholder_name>` 출력.
  - `render_setting_hint_sftp <host> <port> <user>` → 알려진 값은 채우고 모르는 값은 placeholder, password는 항상 `<REPO_PASSWORD>`로 표시한 복사 가능한 `backup.sh setting --backend sftp ...` 명령 한 줄을 출력.
  - `render_setting_hint_s3 <endpoint> <bucket>` → access-key/secret-key/password는 항상 placeholder로 표시한 `backup.sh setting --backend s3 ...` 명령 한 줄을 출력.
  - `render_missing_settings_message` → backup.env가 아예 없거나 backend 자체가 안 정해졌을 때 쓰는 일반 안내 문구(양쪽 backend 예시 모두 언급).
- Consumes: 없음(순수 함수). Task 8/9/10/12/13에서 사용됨.

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/render_hints.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_placeholder_or_value returns value when present" {
  run render_placeholder_or_value "1.2.3.4" "NAS_IP"
  [ "$output" = "1.2.3.4" ]
}

@test "render_placeholder_or_value returns placeholder when empty" {
  run render_placeholder_or_value "" "NAS_IP"
  [ "$output" = "<NAS_IP>" ]
}

@test "render_setting_hint_sftp fills known values and placeholders unknown ones" {
  run render_setting_hint_sftp "1.2.3.4" "" "backup_restic"
  [[ "$output" == *"--backend sftp"* ]]
  [[ "$output" == *"--host 1.2.3.4"* ]]
  [[ "$output" == *"--port <PORT>"* ]]
  [[ "$output" == *"--user backup_restic"* ]]
  [[ "$output" == *"--password '<REPO_PASSWORD>'"* ]]
}

@test "render_setting_hint_s3 always placeholders credentials" {
  run render_setting_hint_s3 "https://s3.example.com" "my-bucket"
  [[ "$output" == *"--backend s3"* ]]
  [[ "$output" == *"--endpoint https://s3.example.com"* ]]
  [[ "$output" == *"--bucket my-bucket"* ]]
  [[ "$output" == *"--access-key <ACCESS_KEY>"* ]]
  [[ "$output" == *"--secret-key '<SECRET_KEY>'"* ]]
  [[ "$output" == *"--password '<REPO_PASSWORD>'"* ]]
}

@test "render_missing_settings_message mentions both backends and setting command" {
  run render_missing_settings_message
  [[ "$output" == *"setting"* ]]
  [[ "$output" == *"s3"* ]]
  [[ "$output" == *"sftp"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/render_hints.bats`
Expected: FAIL (함수 없음)

- [ ] **Step 3: 구현 추가**

`parse_long_opts()` 다음에 추가:

```bash
render_placeholder_or_value() {
  local value="$1" placeholder="$2"
  if [[ -n "$value" ]]; then
    printf '%s' "$value"
  else
    printf '<%s>' "$placeholder"
  fi
}

render_setting_hint_sftp() {
  local host="$1" port="$2" user="$3"
  printf "backup.sh setting --backend sftp --host %s --port %s --user %s --password '<REPO_PASSWORD>'\n" \
    "$(render_placeholder_or_value "$host" "NAS_IP")" \
    "$(render_placeholder_or_value "$port" "PORT")" \
    "$(render_placeholder_or_value "$user" "NAS_USER")"
}

render_setting_hint_s3() {
  local endpoint="$1" bucket="$2"
  printf "backup.sh setting --backend s3 --endpoint %s --bucket %s --access-key <ACCESS_KEY> --secret-key '<SECRET_KEY>' --password '<REPO_PASSWORD>'\n" \
    "$(render_placeholder_or_value "$endpoint" "S3_ENDPOINT")" \
    "$(render_placeholder_or_value "$bucket" "BUCKET_NAME")"
}

render_missing_settings_message() {
  cat <<'EOF'
[!] 설정이 없습니다. 먼저 아래 중 하나로 설정을 완료하세요:

    backup.sh setting --backend sftp --host <NAS_IP> --port <PORT> --user <NAS_USER> --password '<REPO_PASSWORD>'
    backup.sh setting --backend s3 --endpoint <S3_ENDPOINT> --bucket <BUCKET_NAME> --access-key <ACCESS_KEY> --secret-key '<SECRET_KEY>' --password '<REPO_PASSWORD>'

자세한 옵션은 'backup.sh --help'를 참고하세요.
EOF
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/render_hints.bats`
Expected: 5개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/render_hints.bats
git commit -m "feat: add setting-guidance message renderers"
```

---

### Task 6: systemd 유닛 파일 렌더러

**Files:**
- Modify: `backup.sh`
- Test: `tests/render_units.bats`

**Interfaces:**
- Produces: `render_service_unit` (인자 없음, `$BACKUP_SCRIPT_INSTALL_PATH`를 참조해 `ExecStart` 생성), `render_timer_unit <on_calendar>`.
- Consumes: 전역 변수 `BACKUP_SCRIPT_INSTALL_PATH` (Task 1).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/render_units.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_service_unit references the installed script path and run subcommand" {
  run render_service_unit
  [[ "$output" == *"ExecStart=${BACKUP_SCRIPT_INSTALL_PATH} run"* ]]
  [[ "$output" == *"Type=oneshot"* ]]
}

@test "render_timer_unit embeds the given OnCalendar value" {
  run render_timer_unit "*-*-* 03:30:00"
  [[ "$output" == *"OnCalendar=*-*-* 03:30:00"* ]]
  [[ "$output" == *"Persistent=true"* ]]
  [[ "$output" == *"WantedBy=timers.target"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/render_units.bats`
Expected: FAIL (함수 없음)

- [ ] **Step 3: 구현 추가**

`render_missing_settings_message()` 다음에 추가:

```bash
render_service_unit() {
  cat <<EOF
[Unit]
Description=Restic System Backup Service (ISMS Compliance)
After=network-online.target

[Service]
Type=oneshot
ExecStart=${BACKUP_SCRIPT_INSTALL_PATH} run
User=root
Group=root
Restart=no
EOF
}

render_timer_unit() {
  local on_calendar="$1"
  cat <<EOF
[Unit]
Description=Run Restic Backup on schedule

[Timer]
OnCalendar=${on_calendar}
Persistent=true

[Install]
WantedBy=timers.target
EOF
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/render_units.bats`
Expected: 2개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/render_units.bats
git commit -m "feat: add systemd unit file renderers"
```

---

### Task 7: `cmd_install` — 패키지 설치 + self-copy + 디렉터리 생성

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_install.bats`

**Interfaces:**
- Produces: `dnf_install_packages()`, `self_install_copy <source_path> <force>`, `ensure_restic_dir()`, `cmd_install <args...>`. `main()`의 `install` 분기가 `cmd_install "$@"`를 호출하도록 연결.
- Consumes: `parse_long_opts`(Task 4), `require_root`/`die`/`log_info`(Task 1), 전역 변수 `RESTIC_ETC_DIR`/`BACKUP_SCRIPT_INSTALL_PATH`(Task 1).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_install.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "dnf" 'echo "dnf $*" >> "'"${STUB_BIN}"'/dnf.calls"'
  stub_command "install" 'echo "install $*" >> "'"${STUB_BIN}"'/install.calls"; cp "${@: -2:1}" "${@: -1}"'
}

@test "cmd_install installs packages, self-copies, and creates the restic dir" {
  run cmd_install
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/dnf.calls"
  [[ "$output" == *"install -y epel-release restic rclone"* ]]
  [ -d "$RESTIC_ETC_DIR" ]
  perm=$(stat -c '%a' "$RESTIC_ETC_DIR")
  [ "$perm" = "700" ]
  [ -f "$BACKUP_SCRIPT_INSTALL_PATH" ]
}

@test "cmd_install --dry-run makes no changes" {
  run cmd_install --dry-run
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/dnf.calls" ]
  [ ! -d "$RESTIC_ETC_DIR" ]
  [[ "$output" == *"dry-run"* ]]
}

@test "cmd_install does not overwrite an existing install without --force" {
  mkdir -p "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"
  echo "old-content" > "$BACKUP_SCRIPT_INSTALL_PATH"
  run cmd_install
  [ "$status" -eq 0 ]
  run cat "$BACKUP_SCRIPT_INSTALL_PATH"
  [ "$output" = "old-content" ]
}

@test "cmd_install --force overwrites an existing install" {
  mkdir -p "$(dirname "$BACKUP_SCRIPT_INSTALL_PATH")"
  echo "old-content" > "$BACKUP_SCRIPT_INSTALL_PATH"
  run cmd_install --force
  [ "$status" -eq 0 ]
  run cat "$BACKUP_SCRIPT_INSTALL_PATH"
  [ "$output" != "old-content" ]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_install.bats`
Expected: FAIL (`cmd_install: command not found`)

- [ ] **Step 3: 구현 추가**

`render_timer_unit()` 다음에 추가:

```bash
dnf_install_packages() {
  dnf install -y epel-release restic rclone
}

self_install_copy() {
  local source_path="$1" force="$2"
  if [[ -e "$BACKUP_SCRIPT_INSTALL_PATH" && "$force" != 1 ]]; then
    log_info "이미 설치되어 있습니다: ${BACKUP_SCRIPT_INSTALL_PATH} (덮어쓰려면 install --force)"
    return 0
  fi
  install -m 0755 "$source_path" "$BACKUP_SCRIPT_INSTALL_PATH"
}

ensure_restic_dir() {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
}

cmd_install() {
  require_root
  local parsed
  parsed=$(parse_long_opts "force dry-run" -- "$@") || die "$parsed"

  local force=0 dry_run=0
  while IFS=$'\t' read -r key val; do
    case "$key" in
      force) force=1 ;;
      dry-run) dry_run=1 ;;
    esac
  done <<< "$parsed"

  if (( dry_run )); then
    cat <<EOF
[dry-run] dnf install -y epel-release restic rclone
[dry-run] install -m 0755 "\$0" "${BACKUP_SCRIPT_INSTALL_PATH}"
[dry-run] mkdir -p "${RESTIC_ETC_DIR}" && chmod 700 "${RESTIC_ETC_DIR}"
EOF
    return 0
  fi

  dnf_install_packages
  self_install_copy "$0" "$force"
  ensure_restic_dir
  log_info "install 완료"
}
```

`main()`의 `install|setting|init|schedule|run|status|uninstall|wizard)` 분기를 아래로 교체:

```bash
    install)
      shift
      cmd_install "$@"
      return $?
      ;;
    setting|init|schedule|run|status|uninstall|wizard)
      : # 이후 태스크에서 각 cmd_* 로 분기
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_install.bats`
Expected: 4개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_install.bats
git commit -m "feat: add cmd_install with dry-run and force-guarded self-copy"
```

---

### Task 8: `cmd_setting` — SFTP 백엔드

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_setting_sftp.bats`

**Interfaces:**
- Produces: `write_secure_file <path> <mode> <content>`, `generate_ssh_key_if_missing()`, `render_backup_env_sftp <hostname_tag> <host> <port> <user> <ssh_key_path> <password> <targets> <excludes_csv> <keep_daily> <keep_weekly> <keep_monthly>` (순수), `render_sftp_registration_notice <pubkey_content>` (순수), `cmd_setting <args...>`(sftp 분기 포함).
- Consumes: `parse_long_opts`, `resolve_value`, `validate_backend`/`validate_port`/`validate_positive_int`, `render_setting_hint_sftp`, `render_missing_settings_message`, `write_secure_file`, `generate_ssh_key_if_missing`(모두 이전 태스크).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_setting_sftp.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "ssh-keygen" '
    keyfile=""
    while [[ $# -gt 0 ]]; do
      if [[ "$1" == "-f" ]]; then keyfile="$2"; fi
      shift
    done
    echo "fake-private-key" > "$keyfile"
    echo "ssh-ed25519 AAAAFAKEKEY test@stub" > "${keyfile}.pub"
  '
}

@test "render_backup_env_sftp produces expected export lines" {
  run render_backup_env_sftp "host1" "1.2.3.4" "22" "backup_restic" "/etc/restic/backup_key" "secret" "/var/log" "/tmp/*,/var/tmp/*" "7" "4" "12"
  [[ "$output" == *'export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_PORT="22"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"'* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="secret"'* ]]
  [[ "$output" == *'export BACKUP_TARGETS="/var/log"'* ]]
  [[ "$output" == *'export KEEP_DAILY="7"'* ]]
}

@test "render_sftp_registration_notice includes the pubkey and next command" {
  run render_sftp_registration_notice "ssh-ed25519 AAAAFAKEKEY test@stub"
  [[ "$output" == *"ssh-ed25519 AAAAFAKEKEY test@stub"* ]]
  [[ "$output" == *"backup.sh init"* ]]
}

@test "cmd_setting sftp writes backup.env with 600 perms and generates ssh key" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  perm=$(stat -c '%a' "$BACKUP_ENV_FILE")
  [ "$perm" = "600" ]
  [ -f "$BACKUP_SSH_KEY" ]
  [ -f "${BACKUP_SSH_KEY}.pub" ]
  key_perm=$(stat -c '%a' "$BACKUP_SSH_KEY")
  [ "$key_perm" = "600" ]
  [[ "$output" == *"ssh-ed25519"* ]]
}

@test "cmd_setting sftp fails with actionable hint when --host is missing" {
  run cmd_setting --backend sftp --port 22 --user backup_restic --password secret
  [ "$status" -eq 1 ]
  [[ "$output" == *"--host <NAS_IP>"* ]]
  [[ "$output" == *"--user backup_restic"* ]]
}

@test "cmd_setting refuses to overwrite existing backup.env without --force" {
  cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret
  run cmd_setting --backend sftp --host 9.9.9.9 --port 22 --user someone --password other
  [ "$status" -eq 1 ]
  [[ "$output" == *"--force"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_setting_sftp.bats`
Expected: FAIL (함수 없음)

- [ ] **Step 3: 구현 추가**

`ensure_restic_dir()` 다음에 공용 헬퍼 추가:

```bash
write_secure_file() {
  local path="$1" mode="$2" content="$3"
  mkdir -p "$(dirname "$path")"
  printf '%s\n' "$content" > "$path"
  chmod "$mode" "$path"
}

generate_ssh_key_if_missing() {
  if [[ ! -f "$BACKUP_SSH_KEY" ]]; then
    ssh-keygen -t ed25519 -f "$BACKUP_SSH_KEY" -N ""
  fi
  chmod 600 "$BACKUP_SSH_KEY"
  chmod 644 "${BACKUP_SSH_KEY}.pub"
}
```

`cmd_install()` 다음에 추가:

```bash
render_backup_env_sftp() {
  local hostname_tag="$1" host="$2" port="$3" user="$4" ssh_key_path="$5" \
        password="$6" targets="$7" excludes_csv="$8" \
        keep_daily="$9" keep_weekly="${10}" keep_monthly="${11}"
  cat <<EOF
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/${hostname_tag}"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="${host}"
export RCLONE_CONFIG_SYNO_BACKUP_USER="${user}"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="${port}"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${ssh_key_path}"
export RESTIC_PASSWORD="${password}"
export BACKUP_TARGETS="${targets}"
export BACKUP_EXCLUDES="${excludes_csv}"
export KEEP_DAILY="${keep_daily}"
export KEEP_WEEKLY="${keep_weekly}"
export KEEP_MONTHLY="${keep_monthly}"
EOF
}

render_sftp_registration_notice() {
  local pubkey_content="$1"
  cat <<EOF
아래 공개키를 NAS의 authorized_keys(또는 File Station)에 등록하세요:
----------------------------------------------------------
${pubkey_content}
----------------------------------------------------------
등록 후 'backup.sh init'을 실행하세요.
EOF
}

cmd_setting() {
  require_root
  local parsed
  parsed=$(parse_long_opts "backend: targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: force dry-run" -- "$@") || die "$parsed"

  local backend="" targets="" password="" keep_daily="" keep_weekly="" keep_monthly=""
  local endpoint="" bucket="" access_key="" secret_key="" host="" port="" user=""
  local force=0 dry_run=0
  local -a excludes=()

  while IFS=$'\t' read -r key val; do
    case "$key" in
      backend) backend="$val" ;;
      targets) targets="$val" ;;
      exclude) excludes+=("$val") ;;
      password) password="$val" ;;
      keep-daily) keep_daily="$val" ;;
      keep-weekly) keep_weekly="$val" ;;
      keep-monthly) keep_monthly="$val" ;;
      endpoint) endpoint="$val" ;;
      bucket) bucket="$val" ;;
      access-key) access_key="$val" ;;
      secret-key) secret_key="$val" ;;
      host) host="$val" ;;
      port) port="$val" ;;
      user) user="$val" ;;
      force) force=1 ;;
      dry-run) dry_run=1 ;;
    esac
  done <<< "$parsed"

  # 실제 사용자가 export한 환경변수는 backup.env를 source하기 전에 미리 캡처해둔다.
  # (source 이후에는 같은 변수명이 파일 값으로 덮어써지므로, 미리 캡처하지 않으면
  #  "환경변수 값"과 "기존 backup.env 값"을 구분할 수 없다.)
  local env_targets="${BACKUP_TARGETS:-}"
  local env_keep_daily="${KEEP_DAILY:-}"
  local env_keep_weekly="${KEEP_WEEKLY:-}"
  local env_keep_monthly="${KEEP_MONTHLY:-}"
  local env_password="${BACKUP_PASSWORD:-}"
  local env_host="${BACKUP_HOST:-}"
  local env_port="${BACKUP_PORT:-}"
  local env_user="${BACKUP_USER:-}"
  local env_endpoint="${BACKUP_ENDPOINT:-}"
  local env_bucket="${BACKUP_BUCKET:-}"
  local env_access_key="${BACKUP_ACCESS_KEY:-}"
  local env_secret_key="${BACKUP_SECRET_KEY:-}"

  if [[ -z "$backend" ]]; then
    die "$(render_missing_settings_message)"
  fi
  local err
  if ! err=$(validate_backend "$backend"); then die "$err"; fi

  if [[ -f "$BACKUP_ENV_FILE" && "$force" != 1 ]]; then
    die "이미 설정이 있습니다: ${BACKUP_ENV_FILE} (덮어쓰려면 setting --force)"
  fi

  local file_targets="" file_keep_daily="" file_keep_weekly="" file_keep_monthly="" file_excludes=""
  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    # shellcheck source=/dev/null
    source "$BACKUP_ENV_FILE"
    file_targets="${BACKUP_TARGETS:-}"
    file_keep_daily="${KEEP_DAILY:-}"
    file_keep_weekly="${KEEP_WEEKLY:-}"
    file_keep_monthly="${KEEP_MONTHLY:-}"
    file_excludes="${BACKUP_EXCLUDES:-}"
  fi

  targets=$(resolve_value "$targets" "$env_targets" "$file_targets" "$DEFAULT_TARGETS")
  keep_daily=$(resolve_value "$keep_daily" "$env_keep_daily" "$file_keep_daily" "$DEFAULT_KEEP_DAILY")
  keep_weekly=$(resolve_value "$keep_weekly" "$env_keep_weekly" "$file_keep_weekly" "$DEFAULT_KEEP_WEEKLY")
  keep_monthly=$(resolve_value "$keep_monthly" "$env_keep_monthly" "$file_keep_monthly" "$DEFAULT_KEEP_MONTHLY")
  password=$(resolve_value "$password" "$env_password" "" "") || die "저장소 비밀번호(--password 또는 BACKUP_PASSWORD)가 필요합니다"

  if ! err=$(validate_positive_int "$keep_daily" "keep-daily"); then die "$err"; fi
  if ! err=$(validate_positive_int "$keep_weekly" "keep-weekly"); then die "$err"; fi
  if ! err=$(validate_positive_int "$keep_monthly" "keep-monthly"); then die "$err"; fi

  # excludes는 반복 가능한 --exclude 플래그로만 CLI에서 받으므로 환경변수 계층은 없다.
  # CLI에서 하나도 주지 않았으면 기존 backup.env 값을 재사용하고, 그것도 없으면 기본값.
  local excludes_csv
  if [[ ${#excludes[@]} -eq 0 ]]; then
    excludes_csv=$(resolve_value "" "" "$file_excludes" "$DEFAULT_EXCLUDES")
  else
    excludes_csv=$(IFS=,; printf '%s' "${excludes[*]}")
  fi

  if [[ "$backend" == "sftp" ]]; then
    # resolve_value returns 1 (with no output) when every source is empty; under
    # set -euo pipefail an unguarded `var=$(...)` assignment would abort the
    # script right here instead of reaching the emptiness check below, so `|| true`
    # neutralizes that exit status and lets host/user legitimately end up empty.
    host=$(resolve_value "$host" "$env_host" "" "") || true
    port=$(resolve_value "$port" "$env_port" "" "$DEFAULT_SFTP_PORT") || true
    user=$(resolve_value "$user" "$env_user" "" "") || true

    if [[ -z "$host" || -z "$user" ]]; then
      die "$(render_setting_hint_sftp "$host" "$port" "$user")"
    fi
    if ! err=$(validate_port "$port"); then die "$err"; fi

    if (( dry_run )); then
      log_info "[dry-run] backup.env(sftp) 생성 예정: ${BACKUP_ENV_FILE}"
      return 0
    fi

    ensure_restic_dir
    generate_ssh_key_if_missing

    local content
    content=$(render_backup_env_sftp "$(hostname)" "$host" "$port" "$user" "$BACKUP_SSH_KEY" "$password" "$targets" "$excludes_csv" "$keep_daily" "$keep_weekly" "$keep_monthly")
    write_secure_file "$BACKUP_ENV_FILE" 600 "$content"

    render_sftp_registration_notice "$(cat "${BACKUP_SSH_KEY}.pub")"
    log_info "setting(sftp) 완료"
    return 0
  fi

  # s3 분기는 Task 9에서 추가
  die "backend 's3'는 아직 구현되지 않았습니다"
}
```

`main()`의 `setting|init|...` 분기를 아래로 교체:

```bash
    setting)
      shift
      cmd_setting "$@"
      return $?
      ;;
    init|schedule|run|status|uninstall|wizard)
      : # 이후 태스크에서 각 cmd_* 로 분기
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_setting_sftp.bats`
Expected: 5개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_setting_sftp.bats
git commit -m "feat: add cmd_setting sftp backend"
```

---

### Task 9: `cmd_setting` — S3 백엔드

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_setting_s3.bats`

**Interfaces:**
- Produces: `render_backup_env_s3 <hostname_tag> <endpoint> <bucket> <access_key> <secret_key> <password> <targets> <excludes_csv> <keep_daily> <keep_weekly> <keep_monthly>` (순수), `render_s3_bucket_policy <bucket>` (순수). `cmd_setting`의 s3 분기를 완성.
- Consumes: Task 8의 `cmd_setting` 공통 파싱/검증 로직, `render_setting_hint_s3`(Task 5).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_setting_s3.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_backup_env_s3 produces expected export lines" {
  run render_backup_env_s3 "host1" "https://s3.example.com" "my-bucket" "AKIA123" "secretkey" "repopass" "/var/log" "/tmp/*,/var/tmp/*" "7" "4" "12"
  [[ "$output" == *'export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"'* ]]
  [[ "$output" == *'export AWS_ACCESS_KEY_ID="AKIA123"'* ]]
  [[ "$output" == *'export AWS_SECRET_ACCESS_KEY="secretkey"'* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="repopass"'* ]]
  [[ "$output" == *'export BACKUP_TARGETS="/var/log"'* ]]
}

@test "render_s3_bucket_policy scopes actions and resource to the given bucket" {
  run render_s3_bucket_policy "my-bucket"
  [[ "$output" == *'"arn:aws:s3:::my-bucket"'* ]]
  [[ "$output" == *'"arn:aws:s3:::my-bucket/*"'* ]]
  [[ "$output" == *"ListBucket"* ]]
  [[ "$output" == *"GetObject"* ]]
  [[ "$output" == *"PutObject"* ]]
  [[ "$output" == *"DeleteObject"* ]]
}

@test "cmd_setting s3 writes backup.env with 600 perms and prints bucket policy" {
  run cmd_setting --backend s3 --endpoint https://s3.example.com --bucket my-bucket --access-key AKIA123 --secret-key secretkey --password repopass
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  perm=$(stat -c '%a' "$BACKUP_ENV_FILE")
  [ "$perm" = "600" ]
  [[ "$output" == *"arn:aws:s3:::my-bucket"* ]]
}

@test "cmd_setting s3 fails with actionable hint when --bucket is missing" {
  run cmd_setting --backend s3 --endpoint https://s3.example.com --access-key AKIA123 --secret-key secretkey --password repopass
  [ "$status" -eq 1 ]
  [[ "$output" == *"--endpoint https://s3.example.com"* ]]
  [[ "$output" == *"--bucket <BUCKET_NAME>"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_setting_s3.bats`
Expected: FAIL (`render_backup_env_s3: command not found`, 마지막 s3 분기 테스트는 "backend 's3'는 아직 구현되지 않았습니다" 에러로 실패)

- [ ] **Step 3: 구현 추가**

`render_sftp_registration_notice()` 다음에 추가:

```bash
render_backup_env_s3() {
  local hostname_tag="$1" endpoint="$2" bucket="$3" access_key="$4" secret_key="$5" \
        password="$6" targets="$7" excludes_csv="$8" \
        keep_daily="$9" keep_weekly="${10}" keep_monthly="${11}"
  cat <<EOF
export RESTIC_REPOSITORY="s3:${endpoint}/${bucket}/${hostname_tag}"
export AWS_ACCESS_KEY_ID="${access_key}"
export AWS_SECRET_ACCESS_KEY="${secret_key}"
export RESTIC_PASSWORD="${password}"
export BACKUP_TARGETS="${targets}"
export BACKUP_EXCLUDES="${excludes_csv}"
export KEEP_DAILY="${keep_daily}"
export KEEP_WEEKLY="${keep_weekly}"
export KEEP_MONTHLY="${keep_monthly}"
EOF
}

render_s3_bucket_policy() {
  local bucket="$1"
  cat <<EOF
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": ["s3:ListBucket", "s3:GetObject", "s3:PutObject", "s3:DeleteObject"],
      "Resource": ["arn:aws:s3:::${bucket}", "arn:aws:s3:::${bucket}/*"]
    }
  ]
}
EOF
}
```

`cmd_setting()` 안의 마지막 줄 `die "backend 's3'는 아직 구현되지 않았습니다"`를 아래로 교체:

```bash
  if [[ "$backend" == "s3" ]]; then
    # Same `|| true` guard as the sftp branch: resolve_value returns 1 with no
    # output when every source is empty, and under set -euo pipefail an
    # unguarded assignment would abort here instead of reaching the emptiness
    # check below.
    endpoint=$(resolve_value "$endpoint" "$env_endpoint" "" "") || true
    bucket=$(resolve_value "$bucket" "$env_bucket" "" "") || true
    access_key=$(resolve_value "$access_key" "$env_access_key" "" "") || true
    secret_key=$(resolve_value "$secret_key" "$env_secret_key" "" "") || true

    if [[ -z "$endpoint" || -z "$bucket" ]]; then
      die "$(render_setting_hint_s3 "$endpoint" "$bucket")"
    fi
    if [[ -z "$access_key" || -z "$secret_key" ]]; then
      die "$(render_setting_hint_s3 "$endpoint" "$bucket")"
    fi

    if (( dry_run )); then
      log_info "[dry-run] backup.env(s3) 생성 예정: ${BACKUP_ENV_FILE}"
      return 0
    fi

    ensure_restic_dir

    local content
    content=$(render_backup_env_s3 "$(hostname)" "$endpoint" "$bucket" "$access_key" "$secret_key" "$password" "$targets" "$excludes_csv" "$keep_daily" "$keep_weekly" "$keep_monthly")
    write_secure_file "$BACKUP_ENV_FILE" 600 "$content"

    log_info "최소권한 버킷 정책을 아래와 같이 적용하세요:"
    render_s3_bucket_policy "$bucket"
    log_info "setting(s3) 완료"
    return 0
  fi

  die "지원하지 않는 backend입니다: ${backend}"
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_setting_s3.bats tests/cmd_setting_sftp.bats`
Expected: 모든 테스트 PASS (두 백엔드 모두 동작)

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_setting_s3.bats
git commit -m "feat: add cmd_setting s3 backend"
```

---

### Task 10: `cmd_init` — 저장소 초기화

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_init.bats`

**Interfaces:**
- Produces: `restic_is_initialized()` (exit 0/1), `cmd_init()`.
- Consumes: `render_missing_settings_message`(Task 5), `BACKUP_ENV_FILE`(Task 1).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_init.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "cmd_init fails with guidance when backup.env is missing" {
  run cmd_init
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_init runs restic init when repository is not yet initialized" {
  echo 'export RESTIC_REPOSITORY="local:/tmp/fake-repo"' > "$BACKUP_ENV_FILE"
  echo 'export RESTIC_PASSWORD="secret"' >> "$BACKUP_ENV_FILE"
  stub_command "restic" '
    case "$1" in
      snapshots) exit 1 ;;
      init) echo "restic init $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0 ;;
    esac
  '
  run cmd_init
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" == *"init"* ]]
}

@test "cmd_init skips restic init when already initialized" {
  echo 'export RESTIC_REPOSITORY="local:/tmp/fake-repo"' > "$BACKUP_ENV_FILE"
  echo 'export RESTIC_PASSWORD="secret"' >> "$BACKUP_ENV_FILE"
  stub_command "restic" '
    case "$1" in
      snapshots) exit 0 ;;
      init) echo "restic init $*" >> "'"${STUB_BIN}"'/restic.calls"; exit 0 ;;
    esac
  '
  run cmd_init
  [ "$status" -eq 0 ]
  [ ! -f "${STUB_BIN}/restic.calls" ]
  [[ "$output" == *"이미"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_init.bats`
Expected: FAIL (`cmd_init: command not found`)

- [ ] **Step 3: 구현 추가**

`render_s3_bucket_policy()` 다음에 추가:

```bash
restic_is_initialized() {
  restic snapshots >/dev/null 2>&1
}

cmd_init() {
  require_root
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi

  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"

  if restic_is_initialized; then
    log_info "이미 초기화된 저장소입니다. 스킵합니다."
    return 0
  fi

  restic init
  log_info "restic init 완료"
}
```

`main()`의 `init|schedule|...` 분기를 아래로 교체:

```bash
    init)
      shift
      cmd_init "$@"
      return $?
      ;;
    schedule|run|status|uninstall|wizard)
      : # 이후 태스크에서 각 cmd_* 로 분기
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_init.bats`
Expected: 3개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_init.bats
git commit -m "feat: add cmd_init with idempotent restic init"
```

---

### Task 11: `cmd_schedule` — systemd 타이머 enable/disable

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_schedule.bats`

**Interfaces:**
- Produces: `systemd_enable_timer()`, `systemd_disable_timer()`, `cmd_schedule <enable|disable> [args...]`.
- Consumes: `render_service_unit`/`render_timer_unit`(Task 6), `write_secure_file`(Task 8), `parse_long_opts`(Task 4).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_schedule.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"'
}

@test "cmd_schedule enable writes unit files with default schedule and enables the timer" {
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  [ -f "$SYSTEMD_SERVICE_FILE" ]
  [ -f "$SYSTEMD_TIMER_FILE" ]
  grep -q 'OnCalendar=\*-\*-\* 02:00:00' "$SYSTEMD_TIMER_FILE"
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"daemon-reload"* ]]
  [[ "$output" == *"enable --now restic-backup.timer"* ]]
}

@test "cmd_schedule enable honors --on-calendar" {
  run cmd_schedule enable --on-calendar "*-*-* 03:15:00"
  [ "$status" -eq 0 ]
  grep -q 'OnCalendar=\*-\*-\* 03:15:00' "$SYSTEMD_TIMER_FILE"
}

@test "cmd_schedule disable disables and removes the timer" {
  cmd_schedule enable
  run cmd_schedule disable
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"disable --now restic-backup.timer"* ]]
}

@test "cmd_schedule rejects unknown action" {
  run cmd_schedule bogus
  [ "$status" -eq 1 ]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_schedule.bats`
Expected: FAIL (`cmd_schedule: command not found`)

- [ ] **Step 3: 구현 추가**

`cmd_init()` 다음에 추가:

```bash
systemd_enable_timer() {
  systemctl daemon-reload
  systemctl enable --now restic-backup.timer
}

systemd_disable_timer() {
  systemctl disable --now restic-backup.timer 2>/dev/null || true
}

cmd_schedule() {
  require_root
  local action="${1:-}"
  shift || true

  case "$action" in
    enable)
      local parsed
      parsed=$(parse_long_opts "on-calendar:" -- "$@") || die "$parsed"
      local on_calendar="$DEFAULT_ON_CALENDAR"
      while IFS=$'\t' read -r key val; do
        case "$key" in
          on-calendar) on_calendar="$val" ;;
        esac
      done <<< "$parsed"

      write_secure_file "$SYSTEMD_SERVICE_FILE" 644 "$(render_service_unit)"
      write_secure_file "$SYSTEMD_TIMER_FILE" 644 "$(render_timer_unit "$on_calendar")"
      systemd_enable_timer
      log_info "schedule enable 완료 (${on_calendar})"
      ;;
    disable)
      systemd_disable_timer
      log_info "schedule disable 완료"
      ;;
    *)
      die "schedule은 'enable' 또는 'disable'만 지원합니다 (입력값: '${action}')"
      ;;
  esac
}
```

`main()`의 `schedule|run|...` 분기를 아래로 교체:

```bash
    schedule)
      shift
      cmd_schedule "$@"
      return $?
      ;;
    run|status|uninstall|wizard)
      : # 이후 태스크에서 각 cmd_* 로 분기
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_schedule.bats`
Expected: 4개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_schedule.bats
git commit -m "feat: add cmd_schedule enable/disable"
```

---

### Task 12: `cmd_run` — 수동/주기 백업 실행

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_run.bats`

**Interfaces:**
- Produces: `cmd_run()`.
- Consumes: `render_missing_settings_message`(Task 5), `BACKUP_ENV_FILE`(Task 1).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_run.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="local:/tmp/fake-repo"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_EXCLUDES="/tmp/*,/var/tmp/*"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
ENV
}

@test "cmd_run fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_run
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_run backs up then forgets/prunes on success" {
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$1" in
      unlock) exit 0 ;;
      backup) exit 0 ;;
      forget) exit 0 ;;
    esac
  '
  run cmd_run
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" == *"unlock --stale"* ]]
  [[ "$output" == *"backup /var/log --exclude=/tmp/* --exclude=/var/tmp/*"* ]]
  [[ "$output" == *"forget --keep-daily 7 --keep-weekly 4 --keep-monthly 12 --prune"* ]]
}

@test "cmd_run stops before forget/prune when backup fails" {
  stub_command "restic" '
    echo "restic $*" >> "'"${STUB_BIN}"'/restic.calls"
    case "$1" in
      unlock) exit 0 ;;
      backup) exit 1 ;;
      forget) exit 0 ;;
    esac
  '
  run cmd_run
  [ "$status" -eq 1 ]
  run cat "${STUB_BIN}/restic.calls"
  [[ "$output" != *"forget"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_run.bats`
Expected: FAIL (`cmd_run: command not found`)

- [ ] **Step 3: 구현 추가**

`cmd_schedule()` 다음에 추가:

```bash
cmd_run() {
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi

  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"

  restic unlock --stale >/dev/null 2>&1 || true

  # IFS=',' prefixed directly on the `read` command scopes the field separator
  # to that single command only (no global IFS mutation, no `local IFS`
  # gymnastics needed). Populating via `read -ra` into an array, then iterating
  # with a quoted "${arr[@]}", also avoids unquoted-expansion pathname
  # (glob) expansion — an earlier draft looped over an unquoted
  # `${BACKUP_EXCLUDES:-}` directly, which silently expanded patterns like
  # "/tmp/*" into whatever files actually exist under /tmp on the machine
  # running the backup, defeating the exclude entirely.
  local -a exclude_flags=()
  local -a excludes_arr=()
  IFS=',' read -ra excludes_arr <<< "${BACKUP_EXCLUDES:-}"
  local ex
  for ex in "${excludes_arr[@]}"; do
    exclude_flags+=("--exclude=${ex}")
  done

  local -a targets=()
  IFS=',' read -ra targets <<< "${BACKUP_TARGETS:-}"

  if restic backup "${targets[@]}" "${exclude_flags[@]}"; then
    log_info "백업 성공"
  else
    die "restic backup 실패"
  fi

  restic forget --keep-daily "${KEEP_DAILY}" --keep-weekly "${KEEP_WEEKLY}" --keep-monthly "${KEEP_MONTHLY}" --prune
  log_info "만료 스냅샷 정리 완료"
}
```

`main()`의 `run|status|...` 분기를 아래로 교체:

```bash
    run)
      shift
      cmd_run "$@"
      return $?
      ;;
    status|uninstall|wizard)
      : # 이후 태스크에서 각 cmd_* 로 분기
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_run.bats`
Expected: 3개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_run.bats
git commit -m "feat: add cmd_run backup+forget flow"
```

---

### Task 13: `cmd_status`

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_status.bats`

**Interfaces:**
- Produces: `cmd_status()`.
- Consumes: `render_missing_settings_message`(Task 5), `BACKUP_ENV_FILE`/`RESTIC_ETC_DIR`(Task 1).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_status.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "cmd_status fails with guidance when backup.env is missing" {
  run cmd_status
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_status masks secrets and reports snapshot/timer state" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RESTIC_PASSWORD="super-secret"
export BACKUP_TARGETS="/var/log"
ENV
  chmod 600 "$BACKUP_ENV_FILE"
  stub_command "restic" 'case "$1" in snapshots) echo "[]" ;; esac'
  stub_command "systemctl" 'echo "inactive"'

  run cmd_status
  [ "$status" -eq 0 ]
  [[ "$output" == *"rclone:syno_backup:/backup/host1"* ]]
  [[ "$output" != *"super-secret"* ]]
  [[ "$output" == *"700"* ]]
  [[ "$output" == *"600"* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_status.bats`
Expected: FAIL (`cmd_status: command not found`)

- [ ] **Step 3: 구현 추가**

`cmd_run()` 다음에 추가:

```bash
cmd_status() {
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi

  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"

  printf '저장소 위치: %s\n' "${RESTIC_REPOSITORY:-알 수 없음}"
  printf '백업 대상: %s\n' "${BACKUP_TARGETS:-알 수 없음}"

  printf '최근 스냅샷:\n'
  restic snapshots --json 2>/dev/null || printf '(조회 실패 또는 미초기화)\n'

  # systemctl is-active is NOT silent on failure - it prints the state word
  # (e.g. "inactive") to stdout even when it exits nonzero. A bare
  # `$(cmd || echo unknown)` would capture both lines concatenated. Capture
  # the value first, guard only the exit status with `|| true`, then fall
  # back to "unknown" only if the variable ended up empty.
  local timer_state
  timer_state=$(systemctl is-active restic-backup.timer 2>/dev/null) || true
  printf '타이머 상태: %s\n' "${timer_state:-unknown}"

  printf '%s 권한: %s\n' "$RESTIC_ETC_DIR" "$(stat -c '%a' "$RESTIC_ETC_DIR" 2>/dev/null || echo '?')"
  printf '%s 권한: %s\n' "$BACKUP_ENV_FILE" "$(stat -c '%a' "$BACKUP_ENV_FILE" 2>/dev/null || echo '?')"
}
```

`main()`의 `status|uninstall|wizard)` 분기를 아래로 교체:

```bash
    status)
      shift
      cmd_status "$@"
      return $?
      ;;
    uninstall|wizard)
      : # 이후 태스크에서 각 cmd_* 로 분기
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_status.bats`
Expected: 2개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_status.bats
git commit -m "feat: add cmd_status with secret masking"
```

---

### Task 14: `cmd_uninstall`

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_uninstall.bats`

**Interfaces:**
- Produces: `cmd_uninstall <args...>`.
- Consumes: `systemd_disable_timer`(Task 11), `parse_long_opts`(Task 4).

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_uninstall.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"'
  mkdir -p "$RESTIC_ETC_DIR"
  echo "export RESTIC_PASSWORD=secret" > "$BACKUP_ENV_FILE"
  echo "unit" > "$SYSTEMD_SERVICE_FILE"
  echo "unit" > "$SYSTEMD_TIMER_FILE"
}

@test "cmd_uninstall without --purge disables timer but keeps /etc/restic" {
  run cmd_uninstall
  [ "$status" -eq 0 ]
  [ ! -f "$SYSTEMD_SERVICE_FILE" ]
  [ ! -f "$SYSTEMD_TIMER_FILE" ]
  [ -f "$BACKUP_ENV_FILE" ]
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"disable --now restic-backup.timer"* ]]
}

@test "cmd_uninstall --purge also removes the restic config dir" {
  run cmd_uninstall --purge
  [ "$status" -eq 0 ]
  [ ! -d "$RESTIC_ETC_DIR" ]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_uninstall.bats`
Expected: FAIL (`cmd_uninstall: command not found`)

- [ ] **Step 3: 구현 추가**

`cmd_status()` 다음에 추가:

```bash
cmd_uninstall() {
  require_root
  local parsed
  parsed=$(parse_long_opts "purge" -- "$@") || die "$parsed"

  local purge=0
  while IFS=$'\t' read -r key val; do
    case "$key" in
      purge) purge=1 ;;
    esac
  done <<< "$parsed"

  systemd_disable_timer
  rm -f "$SYSTEMD_SERVICE_FILE" "$SYSTEMD_TIMER_FILE"

  if (( purge )); then
    rm -rf "$RESTIC_ETC_DIR"
    log_info "uninstall --purge 완료 (${RESTIC_ETC_DIR} 삭제됨)"
  else
    log_info "uninstall 완료 (${RESTIC_ETC_DIR}는 유지됨)"
  fi
}
```

`main()`의 `uninstall|wizard)` 분기를 아래로 교체:

```bash
    uninstall)
      shift
      cmd_uninstall "$@"
      return $?
      ;;
    wizard)
      : # Task 15에서 연결
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_uninstall.bats`
Expected: 2개 테스트 모두 PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_uninstall.bats
git commit -m "feat: add cmd_uninstall with optional purge"
```

---

### Task 15: `cmd_wizard` — 대화형 원샷 설정

**Files:**
- Modify: `backup.sh`
- Test: `tests/cmd_wizard.bats`

**Interfaces:**
- Produces: `cmd_wizard()`. `main()`의 `wizard` 분기 완성.
- Consumes: `cmd_install`, `cmd_setting`, `cmd_init`, `cmd_schedule`(Tasks 7~11) — 새 로직 없이 표준입력을 읽어 그대로 호출.

wizard는 `read -r` 로 표준입력을 읽으므로, bats 테스트에서는 `run bash -c 'printf "...\n" | ( source backup.sh; cmd_wizard )'` 형태로 응답을 파이프로 주입해 검증한다.

- [ ] **Step 1: 실패하는 테스트 작성**

```bash
cat > tests/cmd_wizard.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "dnf" 'true'
  stub_command "install" 'cp "${@: -2:1}" "${@: -1}"'
  stub_command "ssh-keygen" '
    keyfile=""
    while [[ $# -gt 0 ]]; do
      if [[ "$1" == "-f" ]]; then keyfile="$2"; fi
      shift
    done
    echo "fake-private-key" > "$keyfile"
    echo "ssh-ed25519 AAAAFAKEKEY test@stub" > "${keyfile}.pub"
  '
  stub_command "restic" 'case "$1" in snapshots) exit 1 ;; init) exit 0 ;; esac'
  stub_command "systemctl" 'true'
}

@test "wizard walks through sftp setup end to end and writes backup.env" {
  run bash -c '
    source "'"${BATS_TEST_DIRNAME}"'/../backup.sh"
    printf "2\n1.2.3.4\n22\nbackup_restic\nrepo-pass\n\ny\n\n" | cmd_wizard
  '
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  grep -q 'RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"' "$BACKUP_ENV_FILE"
  [[ "$output" == *"ssh-ed25519"* ]]
  [[ "$output" == *"저장소 위치:"* ]]
}
BATS
```

입력 순서(8줄, `read` 8회에 대응): 백엔드 선택(2=sftp) → host → port → user → password → **진행 확인(빈 값=Y 기본)** → 공개키 등록 완료 ack(y) → 스케줄 등록 여부(빈 값=Y 기본).

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_wizard.bats`
Expected: FAIL (`cmd_wizard: command not found`)

- [ ] **Step 3: 구현 추가**

`cmd_uninstall()` 다음에 추가:

```bash
cmd_wizard() {
  require_root

  if [[ ! -f "$BACKUP_SCRIPT_INSTALL_PATH" ]]; then
    log_info "패키지를 설치합니다..."
    cmd_install
  fi

  printf '백엔드를 선택하세요:\n'
  printf '  [1] S3 호환 스토리지 - HTTPS 기반 오브젝트 스토리지(AWS S3, MinIO 등)\n'
  printf '  [2] SFTP(NAS) - SSH로 접속하는 시놀로지 NAS 등\n'
  printf '선택 (1/2): '
  local choice
  read -r choice

  local backend
  case "$choice" in
    1) backend="s3" ;;
    2) backend="sftp" ;;
    *) die "1 또는 2를 입력하세요" ;;
  esac

  local -a setting_args=(--backend "$backend" --force)

  if [[ "$backend" == "sftp" ]]; then
    printf 'NAS_IP: 백업 데이터를 저장할 NAS의 IP 주소입니다.\nNAS IP 주소 입력: '
    local host; read -r host
    printf 'PORT: NAS의 SSH/SFTP 포트입니다. Enter로 기본값(%s) 사용.\n포트 입력: ' "$DEFAULT_SFTP_PORT"
    local port; read -r port
    port="${port:-$DEFAULT_SFTP_PORT}"
    printf 'USER: NAS에 접속할 SFTP 계정입니다.\n사용자 입력: '
    local user; read -r user
    setting_args+=(--host "$host" --port "$port" --user "$user")
  else
    printf 'S3_ENDPOINT: 접속할 S3 호환 엔드포인트 URL입니다.\n엔드포인트 입력: '
    local endpoint; read -r endpoint
    printf 'BUCKET: 백업을 저장할 버킷 이름입니다.\n버킷 입력: '
    local bucket; read -r bucket
    printf 'ACCESS_KEY: 버킷 접근용 access key입니다.\naccess key 입력: '
    local access_key; read -r access_key
    printf 'SECRET_KEY: 버킷 접근용 secret key입니다.\nsecret key 입력: '
    local secret_key; read -r secret_key
    setting_args+=(--endpoint "$endpoint" --bucket "$bucket" --access-key "$access_key" --secret-key "$secret_key")
  fi

  printf '저장소 비밀번호: 분실 시 백업 데이터를 복구할 수 없습니다. 안전한 곳에 별도 보관하세요.\n비밀번호 입력(화면에 표시되지 않습니다): '
  local password
  read -rs password
  printf '\n'
  setting_args+=(--password "$password")

  printf '\n다음 설정으로 진행합니다:\n'
  printf '  백엔드: %s\n' "$backend"
  if [[ "$backend" == "sftp" ]]; then
    printf '  NAS: %s:%s (사용자: %s)\n' "$host" "$port" "$user"
  else
    printf '  S3 엔드포인트: %s\n' "$endpoint"
    printf '  버킷: %s\n' "$bucket"
  fi
  printf '이대로 진행할까요? [Y/n]: '
  local confirm
  read -r confirm
  if [[ -n "$confirm" && ! "$confirm" =~ ^[Yy]$ ]]; then
    log_info "설정을 취소했습니다."
    return 0
  fi

  cmd_setting "${setting_args[@]}"

  printf '위 안내(공개키 등록 또는 버킷 정책 적용)를 완료하셨으면 Enter를 누르세요: '
  local _ack; read -r _ack

  cmd_init

  printf '지금 정기 백업 스케줄을 등록할까요? 기본값은 매일 새벽 2시입니다. [Y/n]: '
  local schedule_choice; read -r schedule_choice
  local schedule_enabled=0
  if [[ -z "$schedule_choice" || "$schedule_choice" =~ ^[Yy]$ ]]; then
    cmd_schedule enable
    schedule_enabled=1
  fi

  # 요약 출력을 위해 backup.env에서 실제 저장소 위치를 읽어온다(하드코딩된 형식
  # 문자열을 다시 조립하지 않고 render_backup_env_* 가 실제로 쓴 값을 그대로 사용).
  local repo_location=""
  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    # shellcheck source=/dev/null
    source "$BACKUP_ENV_FILE"
    repo_location="${RESTIC_REPOSITORY:-}"
  fi

  printf '\n=========================================\n'
  printf ' 설정이 완료되었습니다\n'
  printf '=========================================\n'
  printf ' 백엔드: %s\n' "$backend"
  printf ' 저장소 위치: %s\n' "${repo_location:-알 수 없음}"
  if (( schedule_enabled )); then
    printf ' 정기 백업: 등록됨 (%s)\n' "$DEFAULT_ON_CALENDAR"
  else
    printf ' 정기 백업: 등록하지 않음 (필요시 backup.sh schedule enable 실행)\n'
  fi
  printf ' 이후에는 backup.sh run / status / uninstall 을 사용하세요.\n'
  printf '=========================================\n'
  log_info "wizard 완료"
}
```

`main()`의 `wizard)` 분기를 아래로 교체:

```bash
    wizard)
      shift
      cmd_wizard "$@"
      return $?
      ;;
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_wizard.bats`
Expected: PASS

- [ ] **Step 5: 커밋**

```bash
git add backup.sh tests/cmd_wizard.bats
git commit -m "feat: add cmd_wizard interactive onboarding"
```

---

### Task 16: Tier 2 — docker-compose 통합 테스트 (MinIO + SFTP)

**Files:**
- Create: `tests/integration/docker-compose.yml`
- Create: `tests/integration/run.sh`

**Interfaces:**
- Consumes: 완성된 `backup.sh`(Task 1~15).
- Produces: 로컬/CI에서 실행 가능한 `tests/integration/run.sh` — 종료 코드 0(성공)/비0(실패)으로 결과 보고.

- [ ] **Step 1: `docker-compose.yml` 작성**

```yaml
services:
  minio:
    image: quay.io/minio/minio
    command: server /data
    environment:
      MINIO_ROOT_USER: AKIAIOSFODNN7EXAMPLE
      MINIO_ROOT_PASSWORD: wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
    healthcheck:
      test: ["CMD", "mc", "ready", "local"]
      interval: 2s
      retries: 20

  sftp:
    image: atmoz/sftp:latest
    command: backup_restic::1001
    volumes:
      - sftp_data:/home/backup_restic/upload

  app:
    image: rockylinux:9
    depends_on:
      - minio
      - sftp
    volumes:
      - ../../backup.sh:/workspace/backup.sh:ro
      - ./run.sh:/workspace/run.sh:ro
    working_dir: /workspace
    entrypoint: ["sleep", "infinity"]

volumes:
  sftp_data:
```

- [ ] **Step 2: `run.sh` 시나리오 스크립트 작성**

```bash
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

echo "=== S3 시나리오: install -> setting -> init -> run ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh install
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

echo "=== SFTP 시나리오: setting -> 키 등록 -> init -> run ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh setting --backend sftp \
    --host sftp --port 22 --user backup_restic \
    --password test-repo-password --force
'
docker compose exec -T app cat /etc/restic/backup_key.pub | \
  docker compose exec -T sftp sh -c 'mkdir -p /home/backup_restic/.ssh && cat >> /home/backup_restic/.ssh/keys/id.pub'
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

echo "=== 모든 통합 테스트 통과 ==="
```

- [ ] **Step 3: 로컬 실행으로 검증 (Docker 필요)**

Run: `chmod +x tests/integration/run.sh && ./tests/integration/run.sh`
Expected: `=== 모든 통합 테스트 통과 ===` 출력 후 종료 코드 0. (참고: `atmoz/sftp` 이미지의 공개키 등록 절차는 이미지 버전에 따라 마운트 경로가 다를 수 있으므로, 실행 시 이미지 문서에 맞춰 `run.sh`의 키 등록 부분을 조정한다.)

- [ ] **Step 4: 커밋**

```bash
git add tests/integration/docker-compose.yml tests/integration/run.sh
git commit -m "test: add docker-compose integration tests for s3 and sftp backends"
```

---

### Task 17: Tier 3 체크리스트 + shellcheck + 최종 점검

**Files:**
- Create: `tests/MANUAL_CHECKLIST.md`
- Modify: `backup.sh` (shellcheck 지적사항 수정)

**Interfaces:**
- Consumes: 전체 `backup.sh`(Task 1~16).

- [ ] **Step 1: `tests/MANUAL_CHECKLIST.md` 작성**

```markdown
# 수동 검증 체크리스트 (Tier 3)

테스트 VM(RHEL 계열, dnf) 또는 실제 대상 서버에서 1회씩 수행한다.

## SFTP 경로
- [ ] `backup.sh install` → epel/restic/rclone 설치 확인, `/usr/local/sbin/backup.sh` 생성 확인
- [ ] `backup.sh setting --backend sftp --host <NAS_IP> --port <PORT> --user <USER> --password <PW>` → `/etc/restic/backup.env`(600), `backup_key`(600)/`backup_key.pub`(644) 생성 확인
- [ ] 출력된 공개키를 실제 NAS `authorized_keys`에 등록
- [ ] `backup.sh init` → 저장소 초기화 성공
- [ ] `backup.sh schedule enable` → `systemctl list-timers`에 `restic-backup.timer` 노출 확인
- [ ] 타이머 시각을 임시로 1분 뒤로 맞추고 실제로 백업이 도는지 확인(또는 `systemctl start restic-backup.service`로 즉시 트리거)
- [ ] `backup.sh status` → 스냅샷/타이머 상태 정상 출력, 비밀번호 미노출 확인
- [ ] `backup.sh schedule disable` → 타이머 비활성화 확인
- [ ] `backup.sh uninstall --purge` → `/etc/restic` 삭제, 유닛 파일 제거 확인

## S3 경로
- [ ] `backup.sh install` → 위와 동일
- [ ] `backup.sh setting --backend s3 --endpoint <EP> --bucket <BUCKET> --access-key <AK> --secret-key <SK> --password <PW>` → backup.env 생성, 버킷 정책 JSON 출력 확인
- [ ] 출력된 최소권한 정책을 실제 버킷/IAM에 적용
- [ ] `backup.sh init` → 저장소 초기화 성공
- [ ] `backup.sh run` → 실제 오브젝트가 버킷에 생성되는지 확인
- [ ] `backup.sh uninstall --purge`

## wizard
- [ ] `backup.sh wizard` 전체 흐름을 SFTP로 1회, S3로 1회 수행하며 각 질문의 설명 문구가 이해하기 쉬운지 확인

## 필수값 누락 시나리오
- [ ] `backup.sh setting --backend sftp --port 22` (host/user 누락) → 안내 명령에 `--port 22`는 채워지고 `--host`/`--user`는 placeholder로 나오는지 확인
- [ ] `backup.sh init`을 `setting` 전에 실행 → 안내 메시지가 s3/sftp 두 예시 모두 보여주는지 확인
```

- [ ] **Step 2: shellcheck 실행 및 수정**

Run: `shellcheck backup.sh`
Expected: 경고 0건. 지적사항이 있으면(예: 미사용 변수, quoting 누락) 해당 라인을 수정하고 재실행해 0건이 될 때까지 반복한다.

- [ ] **Step 3: 전체 bats 스위트 최종 실행**

Run: `bats tests/*.bats`
Expected: 모든 테스트 PASS (Task 1~15에서 작성한 전체 테스트)

- [ ] **Step 4: 커밋**

```bash
git add tests/MANUAL_CHECKLIST.md backup.sh
git commit -m "docs: add manual verification checklist and shellcheck fixes"
```

---

### Task 18: 프로파일 이름 설정값 (`validate_profile_name` + `--profile-name`)

**Files:**
- Modify: `backup.sh`
- Modify: `tests/validators.bats`, `tests/cmd_setting_sftp.bats`, `tests/cmd_setting_s3.bats`

**Interfaces:**
- Produces: `validate_profile_name(value)` — 성공 시 stdout 없이 exit 0, 실패 시 `ERROR: profile-name ...`를 stdout에 찍고 exit 1 (`validate_backend`/`validate_positive_int`와 동일한 계약). `cmd_setting`의 `--profile-name` 플래그. `render_backup_env_sftp`/`render_backup_env_s3`는 마지막에 `profile_name` 인자가 하나 늘어나고, 둘 다 `export BACKUP_PROFILE_NAME="${profile_name}"`를 출력한다.
- Consumes: `resolve_value`(Task 2), `parse_long_opts`(Task 4).

- [ ] **Step 1: 실패하는 테스트 작성 — `validate_profile_name`**

`tests/validators.bats` 끝에 추가:

```bash

@test "validate_profile_name accepts letters, digits, underscore, hyphen" {
  run validate_profile_name "web01-backup_1"
  [ "$status" -eq 0 ]
  [ -z "$output" ]
}

@test "validate_profile_name rejects a value containing a slash" {
  run validate_profile_name "web01/backup"
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}

@test "validate_profile_name rejects a value containing a space" {
  run validate_profile_name "web01 backup"
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}

@test "validate_profile_name rejects an empty value" {
  run validate_profile_name ""
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/validators.bats`
Expected: FAIL (`validate_profile_name: command not found`)

- [ ] **Step 3: `validate_profile_name` 구현 추가**

`validate_positive_int()` 바로 다음에 추가:

```bash
validate_profile_name() {
  local value="$1"
  if [[ -z "$value" ]]; then
    printf 'ERROR: profile-name must not be empty\n'
    return 1
  fi
  if ! [[ "$value" =~ ^[a-zA-Z0-9_-]+$ ]]; then
    printf 'ERROR: profile-name must contain only letters, digits, _ or -, got: %s\n' "$value"
    return 1
  fi
  return 0
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/validators.bats`
Expected: 새 테스트 4개 포함 전체 PASS

- [ ] **Step 5: `render_backup_env_sftp`/`render_backup_env_s3`에 `profile_name` 인자 추가 — 실패하는 테스트부터**

`tests/cmd_setting_sftp.bats`의 `render_backup_env_sftp` 테스트를 아래로 교체(마지막 인자 `"web01"` 추가 + 새 assertion 1줄):

```bash
@test "render_backup_env_sftp produces expected export lines" {
  run render_backup_env_sftp "host1" "1.2.3.4" "22" "backup_restic" "/etc/restic/backup_key" "secret" "/var/log" "/tmp/*,/var/tmp/*" "7" "4" "12" "web01"
  [[ "$output" == *'export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_PORT="22"'* ]]
  [[ "$output" == *'export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"'* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="secret"'* ]]
  [[ "$output" == *'export BACKUP_TARGETS="/var/log"'* ]]
  [[ "$output" == *'export KEEP_DAILY="7"'* ]]
  [[ "$output" == *'export BACKUP_PROFILE_NAME="web01"'* ]]
}
```

`tests/cmd_setting_s3.bats`의 `render_backup_env_s3` 테스트도 동일하게 교체(마지막 인자 `"web01"` 추가):

```bash
@test "render_backup_env_s3 produces expected export lines" {
  run render_backup_env_s3 "host1" "https://s3.example.com" "my-bucket" "AKIA123" "secretkey" "repopass" "/var/log" "/tmp/*,/var/tmp/*" "7" "4" "12" "web01"
  [[ "$output" == *'export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"'* ]]
  [[ "$output" == *'export AWS_ACCESS_KEY_ID="AKIA123"'* ]]
  [[ "$output" == *'export AWS_SECRET_ACCESS_KEY="secretkey"'* ]]
  [[ "$output" == *'export RESTIC_PASSWORD="repopass"'* ]]
  [[ "$output" == *'export BACKUP_TARGETS="/var/log"'* ]]
  [[ "$output" == *'export BACKUP_PROFILE_NAME="web01"'* ]]
}
```

Run: `bats tests/cmd_setting_sftp.bats tests/cmd_setting_s3.bats`
Expected: 위 두 테스트만 FAIL (인자 개수 불일치로 `${11}`이 비어서 `BACKUP_PROFILE_NAME` 줄 자체가 없거나 값이 다름), 나머지는 그대로 PASS

- [ ] **Step 6: `render_backup_env_sftp`/`render_backup_env_s3` 구현 수정**

```bash
render_backup_env_sftp() {
  local hostname_tag="$1" host="$2" port="$3" user="$4" ssh_key_path="$5" \
        password="$6" targets="$7" excludes_csv="$8" \
        keep_daily="$9" keep_weekly="${10}" keep_monthly="${11}" profile_name="${12}"
  cat <<EOF
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/${hostname_tag}"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="${host}"
export RCLONE_CONFIG_SYNO_BACKUP_USER="${user}"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="${port}"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="${ssh_key_path}"
export RESTIC_PASSWORD="${password}"
export BACKUP_TARGETS="${targets}"
export BACKUP_EXCLUDES="${excludes_csv}"
export KEEP_DAILY="${keep_daily}"
export KEEP_WEEKLY="${keep_weekly}"
export KEEP_MONTHLY="${keep_monthly}"
export BACKUP_PROFILE_NAME="${profile_name}"
EOF
}
```

`render_backup_env_s3`도 동일한 패턴으로 마지막에 `profile_name="${12}"`를 추가하고 `export BACKUP_PROFILE_NAME="${profile_name}"` 줄을 추가한다.

- [ ] **Step 7: 테스트 통과 확인**

Run: `bats tests/cmd_setting_sftp.bats tests/cmd_setting_s3.bats`
Expected: 전체 PASS

- [ ] **Step 8: `cmd_setting`에 `--profile-name` 배선 — 실패하는 테스트부터**

`tests/cmd_setting_sftp.bats`에 추가:

```bash

@test "cmd_setting sftp defaults profile-name to hostname and honors --profile-name override" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret
  [ "$status" -eq 0 ]
  grep -q "export BACKUP_PROFILE_NAME=\"$(hostname)\"" "$BACKUP_ENV_FILE"

  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --profile-name web01 --force
  [ "$status" -eq 0 ]
  grep -q 'export BACKUP_PROFILE_NAME="web01"' "$BACKUP_ENV_FILE"
}

@test "cmd_setting rejects an invalid --profile-name" {
  run cmd_setting --backend sftp --host 1.2.3.4 --port 22 --user backup_restic --password secret --profile-name "bad name"
  [ "$status" -eq 1 ]
  [[ "$output" == *"profile-name"* ]]
}
```

- [ ] **Step 9: 테스트 실패 확인**

Run: `bats tests/cmd_setting_sftp.bats`
Expected: 위 2개 신규 테스트 FAIL (`--profile-name`은 아직 인식되지 않는 플래그)

- [ ] **Step 10: `cmd_setting` 구현 수정**

`parse_long_opts` 스펙 문자열에 `profile-name:` 추가:

```bash
  parsed=$(parse_long_opts "backend: targets: exclude: password: keep-daily: keep-weekly: keep-monthly: endpoint: bucket: access-key: secret-key: host: port: user: profile-name: force dry-run" -- "$@") || die "$parsed"
```

로컬 선언 줄에 `profile_name=""` 추가:

```bash
  local backend="" targets_csv="" password="" keep_daily="" keep_weekly="" keep_monthly="" profile_name=""
```

`case "$key" in` 블록에 추가:

```bash
      profile-name) profile_name="$val" ;;
```

환경변수 사전 캡처 블록(`local env_targets=...` 옆)에 추가:

```bash
  local env_profile_name="${BACKUP_PROFILE_NAME:-}"
```

기존 `backup.env` 값 캡처 블록(`file_targets=...` 옆)에 추가— `local file_targets="" file_keep_daily="" ...` 줄에 `file_profile_name=""`를 추가하고, `source "$BACKUP_ENV_FILE"` 다음 블록에:

```bash
    file_profile_name="${BACKUP_PROFILE_NAME:-}"
```

`targets_csv=$(resolve_value ...)` 근처, `excludes_csv` 계산 이전 아무 곳에 추가:

```bash
  profile_name=$(resolve_value "$profile_name" "$env_profile_name" "$file_profile_name" "$(hostname)")
  if ! err=$(validate_profile_name "$profile_name"); then die "$err"; fi
```

`render_backup_env_sftp`/`render_backup_env_s3` 두 호출 모두 마지막에 `"$profile_name"` 인자를 추가:

```bash
    content=$(render_backup_env_sftp "$(hostname)" "$host" "$port" "$user" "$BACKUP_SSH_KEY" "$password" "$targets_csv" "$excludes_csv" "$keep_daily" "$keep_weekly" "$keep_monthly" "$profile_name")
```

```bash
    content=$(render_backup_env_s3 "$(hostname)" "$endpoint" "$bucket" "$access_key" "$secret_key" "$password" "$targets_csv" "$excludes_csv" "$keep_daily" "$keep_weekly" "$keep_monthly" "$profile_name")
```

- [ ] **Step 11: 테스트 통과 확인**

Run: `bats tests/*.bats`
Expected: 전체 PASS

- [ ] **Step 12: shellcheck + 커밋**

Run: `shellcheck backup.sh`
Expected: 0건

```bash
git add backup.sh tests/validators.bats tests/cmd_setting_sftp.bats tests/cmd_setting_s3.bats
git commit -m "feat: add configurable --profile-name (default: hostname)"
```

---

### Task 19: resticprofile 설치 (버전 고정 + SHA256 검증)

**Files:**
- Modify: `backup.sh`
- Modify: `tests/cmd_install.bats`

**Interfaces:**
- Produces: 상수 `RESTICPROFILE_VERSION`, `RESTICPROFILE_SHA256`, `RESTICPROFILE_URL`, `RESTICPROFILE_INSTALL_PATH`. 함수 `install_resticprofile()`.
- Consumes: `die()`, `log_info()`. `cmd_install()`에서 `dnf_install_packages` 다음, `self_install_copy` 이전에 호출.

pinned 버전 사실관계(2026-07-10 기준 실제 확인값, GitHub Releases `creativeprojects/resticprofile` v0.33.1의 `resticprofile_no_self_update_0.33.1_linux_amd64.tar.gz`):
- `no_self_update` 빌드를 쓴다: 일반 빌드는 자체 업데이트 기능이 있어 체크섬 검증을 거친 바이너리가 나중에 검증 없이 스스로를 새 버전으로 교체할 수 있다 — 우리가 고정한 버전/체크섬의 의미가 없어진다.
- SHA256(직접 다운로드 후 `sha256sum`으로 재확인함): `1d7027d15e3e2456e585a210f811d0f72ec40f6b3388f00425642ed579165d70`

- [ ] **Step 1: 실패하는 테스트 작성**

`tests/cmd_install.bats` 끝에 추가(기존 `setup()`은 그대로 재사용):

```bash

@test "cmd_install downloads, verifies checksum, and installs resticprofile" {
  stub_command "curl" '
    # 마지막 인자가 -o 뒤에 오는 목적지 경로
    dest=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    cp "'"${BATS_TEST_DIRNAME}"'/fixtures/resticprofile-fake.tar.gz" "$dest"
  '
  run cmd_install
  [ "$status" -eq 0 ]
  [ -x "$RESTICPROFILE_INSTALL_PATH" ]
  run "$RESTICPROFILE_INSTALL_PATH"
  [[ "$output" == *"fake-resticprofile"* ]]
}

@test "cmd_install dies when the downloaded resticprofile checksum does not match" {
  stub_command "curl" '
    dest=""
    prev=""
    for a in "$@"; do
      if [[ "$prev" == "-o" ]]; then dest="$a"; fi
      prev="$a"
    done
    echo "corrupted content, not the real tarball" > "$dest"
  '
  run cmd_install
  [ "$status" -eq 1 ]
  [[ "$output" == *"체크섬"* ]]
  [ ! -e "$RESTICPROFILE_INSTALL_PATH" ]
}

@test "cmd_install skips the resticprofile download when already installed" {
  mkdir -p "$(dirname "$RESTICPROFILE_INSTALL_PATH")"
  printf '#!/usr/bin/env bash\necho already-here\n' > "$RESTICPROFILE_INSTALL_PATH"
  chmod +x "$RESTICPROFILE_INSTALL_PATH"
  stub_command "curl" 'echo "curl should not run" >&2; exit 1'
  run cmd_install
  [ "$status" -eq 0 ]
  run "$RESTICPROFILE_INSTALL_PATH"
  [[ "$output" == "already-here" ]]
}
```

같은 디렉터리에 고정된 가짜 tarball fixture를 만든다(실제 curl/네트워크 없이 체크섬 검증 로직만 검증하기 위함):

```bash
mkdir -p tests/fixtures
printf '#!/usr/bin/env bash\necho fake-resticprofile\n' > /tmp/resticprofile
chmod +x /tmp/resticprofile
tar -czf tests/fixtures/resticprofile-fake.tar.gz -C /tmp resticprofile
rm /tmp/resticprofile
sha256sum tests/fixtures/resticprofile-fake.tar.gz
```

이 `sha256sum` 출력값을 테스트 안에서 쓸 것이므로, `test_helper.bash`의 `setup_backup_sh_env` 호출 전에 아래를 `cmd_install.bats`의 `setup()`에 추가해 `RESTICPROFILE_SHA256`을 fixture의 실제 체크섬으로 오버라이드한다(그래야 진짜 GitHub 버전 체크섬 없이도 검증 로직이 통과):

```bash
setup() {
  export RESTICPROFILE_SHA256
  RESTICPROFILE_SHA256=$(sha256sum "${BATS_TEST_DIRNAME}/fixtures/resticprofile-fake.tar.gz" | awk '{print $1}')
  export RESTICPROFILE_URL="http://fixture.invalid/resticprofile.tar.gz"
  setup_backup_sh_env
  export RESTICPROFILE_INSTALL_PATH="${TEST_ROOT}/usr/local/bin/resticprofile"
  stub_command "dnf" 'echo "dnf $*" >> "'"${STUB_BIN}"'/dnf.calls"'
  stub_command "install" 'echo "'"${STUB_BIN}"'/install.calls"; cp "${@: -2:1}" "${@: -1}"'
}
```

(기존 `setup()`을 이 내용으로 교체 — `RESTICPROFILE_SHA256`/`RESTICPROFILE_URL`/`RESTICPROFILE_INSTALL_PATH`을 테스트별로 오버라이드할 수 있어야 하므로 `setup_backup_sh_env`보다 먼저 export한다.)

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_install.bats`
Expected: 새 테스트 3개 FAIL (`install_resticprofile: command not found` 또는 `$RESTICPROFILE_INSTALL_PATH` 미생성)

- [ ] **Step 3: 전역 변수 + `install_resticprofile()` 구현 추가**

`SYSTEMD_TIMER_FILE=...` 줄 다음에 추가:

```bash
RESTICPROFILE_INSTALL_PATH="${RESTICPROFILE_INSTALL_PATH:-/usr/local/bin/resticprofile}"
RESTICPROFILE_VERSION="0.33.1"
RESTICPROFILE_SHA256="${RESTICPROFILE_SHA256:-1d7027d15e3e2456e585a210f811d0f72ec40f6b3388f00425642ed579165d70}"
RESTICPROFILE_URL="${RESTICPROFILE_URL:-https://github.com/creativeprojects/resticprofile/releases/download/v${RESTICPROFILE_VERSION}/resticprofile_no_self_update_${RESTICPROFILE_VERSION}_linux_amd64.tar.gz}"
RESTICPROFILE_CONFIG_FILE="${RESTICPROFILE_CONFIG_FILE:-${RESTIC_ETC_DIR}/profiles.yaml}"
RESTICPROFILE_UNIT_TEMPLATE="${RESTICPROFILE_UNIT_TEMPLATE:-${RESTIC_ETC_DIR}/resticprofile-service.tmpl}"
RESTICPROFILE_TIMER_TEMPLATE="${RESTICPROFILE_TIMER_TEMPLATE:-${RESTIC_ETC_DIR}/resticprofile-timer.tmpl}"
```

`dnf_install_packages()` 다음에 추가:

```bash
install_resticprofile() {
  if [[ -x "$RESTICPROFILE_INSTALL_PATH" ]]; then
    return 0
  fi

  local tmp_dir
  tmp_dir=$(mktemp -d)
  curl -fsSL -o "${tmp_dir}/resticprofile.tar.gz" "$RESTICPROFILE_URL"

  local actual_sha256
  actual_sha256=$(sha256sum "${tmp_dir}/resticprofile.tar.gz" | awk '{print $1}')
  if [[ "$actual_sha256" != "$RESTICPROFILE_SHA256" ]]; then
    rm -rf "$tmp_dir"
    die "resticprofile 체크섬 불일치 (예상: ${RESTICPROFILE_SHA256}, 실제: ${actual_sha256}) - 설치를 중단합니다"
  fi

  tar -xzf "${tmp_dir}/resticprofile.tar.gz" -C "$tmp_dir" resticprofile
  mkdir -p "$(dirname "$RESTICPROFILE_INSTALL_PATH")"
  install -m 0755 "${tmp_dir}/resticprofile" "$RESTICPROFILE_INSTALL_PATH"
  rm -rf "$tmp_dir"
}
```

`cmd_install()`의 dry-run 블록과 실제 실행 블록에 각각 한 줄씩 추가:

```bash
  if (( dry_run )); then
    cat <<EOF
[dry-run] dnf install -y epel-release
[dry-run] dnf install -y restic rclone
[dry-run] resticprofile ${RESTICPROFILE_VERSION} 다운로드+체크섬 검증 후 ${RESTICPROFILE_INSTALL_PATH}에 설치
[dry-run] install -m 0755 "\$0" "${BACKUP_SCRIPT_INSTALL_PATH}"
[dry-run] mkdir -p "${RESTIC_ETC_DIR}" && chmod 700 "${RESTIC_ETC_DIR}"
EOF
    return 0
  fi

  dnf_install_packages
  install_resticprofile
  self_install_copy "$0" "$force"
  ensure_restic_dir
  log_info "install 완료"
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_install.bats`
Expected: 전체 PASS

- [ ] **Step 5: shellcheck + 커밋**

Run: `shellcheck backup.sh`
Expected: 0건

```bash
git add backup.sh tests/cmd_install.bats tests/fixtures/resticprofile-fake.tar.gz
git commit -m "feat: install a checksum-verified, version-pinned resticprofile binary"
```

---

### Task 20: `profiles.yaml`/systemd 템플릿 렌더러 + `cmd_schedule`을 resticprofile 위임으로 교체

**Files:**
- Modify: `backup.sh`
- Create: `tests/resticprofile_config.bats`
- Modify: `tests/cmd_schedule.bats`, `tests/cmd_status.bats`

**Interfaces:**
- Produces: `render_resticprofile_config(profile_name, on_calendar, targets_csv, excludes_csv, keep_daily, keep_weekly, keep_monthly)`, `render_resticprofile_unit_template()`, `render_resticprofile_timer_template()`. `cmd_schedule enable/disable`은 이제 `resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" schedule`/`unschedule`을 호출한다. 결정론적 유닛 이름 규칙: `resticprofile-backup@profile-<profile_name>.timer`.
- Consumes: Task 18의 `BACKUP_PROFILE_NAME`, Task 19의 `RESTICPROFILE_*` 경로 상수, `write_secure_file`(Task 8), `parse_long_opts`(Task 4). `render_service_unit`/`render_timer_unit`/`systemd_enable_timer`/`systemd_disable_timer`(Task 6/11)는 이 태스크에서 제거된다 — 더 이상 아무도 호출하지 않는다.

resticprofile 사실관계(context7 `/creativeprojects/resticprofile` 문서로 확인):
- 스케줄 등록/해제: `resticprofile --config <path> --name <profile> schedule` / `unschedule` (systemd 유닛 파일 생성·삭제·enable/disable을 resticprofile이 전부 처리).
- 유닛 이름: `resticprofile-backup@profile-<name>.timer`/`.service` (resticprofile이 고정하는 규칙, 우리가 짓는 게 아니다).
- 커스텀 유닛/타이머 템플릿: `global.systemd-unit-template`/`global.systemd-timer-template`에 템플릿 **파일 경로**를 지정한다. 템플릿에서 쓸 수 있는 변수: `JobDescription`, `TimerDescription`, `WorkingDirectory`, `CommandLine`, `OnCalendar`(배열), `SystemdProfile`, `Nice`, `Environment`(배열). 정적 텍스트(ISMS 문구, `User=root`)는 변수 없이 그냥 템플릿에 하드코딩하면 된다.
- lock: `global.restic-lock-retry-after`, `global.restic-stale-lock-age`로 재시도/stale 판정 주기를 정하고, **프로파일 쪽에 `force-inactive-lock: true`가 있어야** 실제로 stale lock을 풀고 재시도한다(둘 다 있어야 동작 — 이번 조사에서 확인).
- retention: `retention.after-backup: true` + `retention.prune: true`(기본 false, 명시해야 함) + `retention.keep-daily/keep-weekly/keep-monthly`.
- 비밀값: `env:` 블록에 그대로 키:값으로 넣는다(`password-file`처럼 별도 파일을 만들 필요 없음 — REPO 비밀번호도 `env.RESTIC_PASSWORD`로 넣는다).
- **⚠️ 실측 확인(2026-07-10, docker 컨테이너에서 실제 `resticprofile schedule` 실행 후 생성된 유닛 파일을 직접 열어봄): `.Environment` 템플릿 변수를 렌더링하면 `Environment="RESTIC_PASSWORD=<평문>"`이 644 권한의 `/etc/systemd/system/*.service` 파일에 그대로 노출된다(기본 내장 템플릿으로 실험해도 동일하게 발생 — 우리 템플릿 문제가 아니라 resticprofile이 스케줄 등록 시 `RESTIC_PASSWORD`를 유닛에 주입하는 동작 자체가 원인).** 반대로 `env:` 블록의 `AWS_ACCESS_KEY_ID`/`AWS_SECRET_ACCESS_KEY`/`RCLONE_CONFIG_*`는 유닛에 노출되지 않았다(resticprofile이 `RESTIC_PASSWORD`만 특별 취급하는 것으로 보임). 생성된 유닛의 `ExecStart`는 `resticprofile ... --config <원본 config 경로> run-schedule backup@<profile>`로, 실행 시점에 config 파일을 다시 읽으므로 유닛에 `Environment=`가 없어도 비밀값은 `env:` 블록에서 정상 공급된다. **따라서 우리 커스텀 유닛 템플릿에서는 `{{ range .Environment }}` 블록을 아예 빼야 한다** — Step 1/Step 3에 반영되어 있다.

- [ ] **Step 1: 실패하는 테스트 작성 — `render_resticprofile_config`**

```bash
cat > tests/resticprofile_config.bats <<'BATS'
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "render_resticprofile_config embeds repository, secrets, retention, and schedule" {
  export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
  export RESTIC_PASSWORD="super-secret"
  export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
  export RCLONE_CONFIG_SYNO_BACKUP_HOST="1.2.3.4"
  export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_restic"
  export RCLONE_CONFIG_SYNO_BACKUP_PORT="22"
  export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"

  run render_resticprofile_config "web01" "*-*-* 02:00:00" "/var/log,/etc" "/tmp/*" "7" "4" "12"
  [ "$status" -eq 0 ]
  [[ "$output" == *'repository: "rclone:syno_backup:/backup/host1"'* ]]
  [[ "$output" == *'force-inactive-lock: true'* ]]
  [[ "$output" == *'RESTIC_PASSWORD: "super-secret"'* ]]
  [[ "$output" == *'RCLONE_CONFIG_SYNO_BACKUP_HOST: "1.2.3.4"'* ]]
  [[ "$output" == *'after-backup: true'* ]]
  [[ "$output" == *'prune: true'* ]]
  [[ "$output" == *'keep-daily: 7'* ]]
  [[ "$output" == *'keep-weekly: 4'* ]]
  [[ "$output" == *'keep-monthly: 12'* ]]
  [[ "$output" == *'schedule: "*-*-* 02:00:00"'* ]]
  [[ "$output" == *'schedule-permission: system'* ]]
  [[ "$output" == *'- "/var/log"'* ]]
  [[ "$output" == *'- "/etc"'* ]]
  [[ "$output" == *'exclude:'* ]]
  [[ "$output" == *'- "/tmp/*"'* ]]
  [[ "$output" == *'systemd-unit-template:'* ]]
  [[ "$output" == *'systemd-timer-template:'* ]]
}

@test "render_resticprofile_config embeds s3 credentials when AWS_* is set" {
  export RESTIC_REPOSITORY="s3:https://s3.example.com/my-bucket/host1"
  export RESTIC_PASSWORD="super-secret"
  export AWS_ACCESS_KEY_ID="AKIA123"
  export AWS_SECRET_ACCESS_KEY="secretkey"

  run render_resticprofile_config "web01" "*-*-* 02:00:00" "/var/log" "" "7" "4" "12"
  [ "$status" -eq 0 ]
  [[ "$output" == *'AWS_ACCESS_KEY_ID: "AKIA123"'* ]]
  [[ "$output" == *'AWS_SECRET_ACCESS_KEY: "secretkey"'* ]]
}

@test "render_resticprofile_unit_template keeps the ISMS description and hardens the service" {
  run render_resticprofile_unit_template
  [[ "$output" == *"ISMS Compliance"* ]]
  [[ "$output" == *"User=root"* ]]
  [[ "$output" == *'ExecStart={{ .CommandLine }}'* ]]
}

@test "render_resticprofile_unit_template never emits the .Environment block (would leak RESTIC_PASSWORD into a 644 unit file)" {
  run render_resticprofile_unit_template
  [[ "$output" != *".Environment"* ]]
  [[ "$output" != *"Environment="* ]]
}

@test "render_resticprofile_timer_template keeps the ISMS description" {
  run render_resticprofile_timer_template
  [[ "$output" == *"ISMS Compliance"* ]]
  [[ "$output" == *'{{ range .OnCalendar -}}'* ]]
}
BATS
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/resticprofile_config.bats`
Expected: FAIL (`render_resticprofile_config: command not found`)

- [ ] **Step 3: 렌더러 구현 추가**

`render_timer_unit()` 다음에 추가(기존 `render_service_unit`/`render_timer_unit`는 이 태스크에서 더 이상 호출되지 않지만, Task 6 테스트(`tests/render_units.bats`)와의 하위호환을 위해 삭제하지 않고 남겨둔다):

```bash
render_resticprofile_unit_template() {
  # 의도적으로 `{{ range .Environment }}` 블록을 넣지 않는다: 넣으면 resticprofile이
  # RESTIC_PASSWORD를 이 파일에 평문 `Environment=`로 주입한다(644, /etc/systemd/system/ -
  # 2026-07-10 docker 컨테이너에서 실측 확인, 기본 템플릿에서도 재현됨). 유닛의 ExecStart는
  # 실행 시점에 `--config <profiles.yaml>`을 그대로 다시 읽으므로(run-schedule), Environment
  # 블록이 없어도 비밀값은 profiles.yaml의 env: 블록에서 정상 공급된다.
  cat <<'EOF'
[Unit]
Description={{ .JobDescription }} (ISMS Compliance)
{{ if .AfterNetworkOnline }}After=network-online.target
{{ end }}
[Service]
Type=notify
User=root
Group=root
WorkingDirectory={{ .WorkingDirectory }}
ExecStart={{ .CommandLine }}
{{ if .Nice }}Nice={{ .Nice }}
{{ end -}}
EOF
}

render_resticprofile_timer_template() {
  cat <<'EOF'
[Unit]
Description={{ .TimerDescription }} (ISMS Compliance)

[Timer]
{{ range .OnCalendar -}}
OnCalendar={{ . }}
{{ end -}}
Unit={{ .SystemdProfile }}
Persistent=true

[Install]
WantedBy=timers.target
EOF
}

render_resticprofile_config() {
  local profile_name="$1" on_calendar="$2" targets_csv="$3" excludes_csv="$4" \
        keep_daily="$5" keep_weekly="$6" keep_monthly="$7"
  local -a sources=() excludes=()
  IFS=',' read -ra sources <<< "$targets_csv"
  if [[ -n "$excludes_csv" ]]; then
    IFS=',' read -ra excludes <<< "$excludes_csv"
  fi

  printf 'version: "1"\n\n'
  printf 'global:\n'
  printf '  restic-lock-retry-after: 1m\n'
  printf '  restic-stale-lock-age: 2h\n'
  printf '  systemd-unit-template: %s\n' "$RESTICPROFILE_UNIT_TEMPLATE"
  printf '  systemd-timer-template: %s\n\n' "$RESTICPROFILE_TIMER_TEMPLATE"

  printf '%s:\n' "$profile_name"
  printf '  repository: "%s"\n' "${RESTIC_REPOSITORY:-}"
  printf '  force-inactive-lock: true\n'
  printf '  env:\n'
  printf '    RESTIC_PASSWORD: "%s"\n' "${RESTIC_PASSWORD:-}"
  if [[ -n "${AWS_ACCESS_KEY_ID:-}" ]]; then
    printf '    AWS_ACCESS_KEY_ID: "%s"\n' "$AWS_ACCESS_KEY_ID"
    printf '    AWS_SECRET_ACCESS_KEY: "%s"\n' "${AWS_SECRET_ACCESS_KEY:-}"
  fi
  if [[ -n "${RCLONE_CONFIG_SYNO_BACKUP_TYPE:-}" ]]; then
    printf '    RCLONE_CONFIG_SYNO_BACKUP_TYPE: "%s"\n' "$RCLONE_CONFIG_SYNO_BACKUP_TYPE"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_HOST: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_HOST:-}"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_USER: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_USER:-}"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_PORT: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_PORT:-}"
    printf '    RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE: "%s"\n' "${RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE:-}"
  fi

  printf '  retention:\n'
  printf '    after-backup: true\n'
  printf '    prune: true\n'
  printf '    keep-daily: %s\n' "$keep_daily"
  printf '    keep-weekly: %s\n' "$keep_weekly"
  printf '    keep-monthly: %s\n' "$keep_monthly"

  printf '  backup:\n'
  printf '    schedule: "%s"\n' "$on_calendar"
  printf '    schedule-permission: system\n'
  printf '    source:\n'
  local s
  for s in "${sources[@]}"; do
    printf '      - "%s"\n' "$s"
  done
  if [[ ${#excludes[@]} -gt 0 ]]; then
    printf '    exclude:\n'
    local e
    for e in "${excludes[@]}"; do
      printf '      - "%s"\n' "$e"
    done
  fi
}
```

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/resticprofile_config.bats`
Expected: 전체 PASS

- [ ] **Step 5: `cmd_schedule`을 resticprofile 위임으로 교체 — 실패하는 테스트부터**

`tests/cmd_schedule.bats`를 아래로 전체 교체:

```bash
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/web01"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_EXCLUDES="/tmp/*"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="web01"
ENV
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"'
}

@test "cmd_schedule enable renders profiles.yaml and delegates to resticprofile schedule" {
  run cmd_schedule enable
  [ "$status" -eq 0 ]
  [ -f "$RESTICPROFILE_CONFIG_FILE" ]
  perm=$(stat -c '%a' "$RESTICPROFILE_CONFIG_FILE")
  [ "$perm" = "600" ]
  grep -q 'schedule: "\*-\*-\* 02:00:00"' "$RESTICPROFILE_CONFIG_FILE"
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--config ${RESTICPROFILE_CONFIG_FILE} --name web01 schedule"* ]]
}

@test "cmd_schedule enable honors --on-calendar" {
  run cmd_schedule enable --on-calendar "*-*-* 03:15:00"
  [ "$status" -eq 0 ]
  grep -q 'schedule: "\*-\*-\* 03:15:00"' "$RESTICPROFILE_CONFIG_FILE"
}

@test "cmd_schedule disable delegates to resticprofile unschedule" {
  cmd_schedule enable
  run cmd_schedule disable
  [ "$status" -eq 0 ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--config ${RESTICPROFILE_CONFIG_FILE} --name web01 unschedule"* ]]
}

@test "cmd_schedule rejects unknown action" {
  run cmd_schedule bogus
  [ "$status" -eq 1 ]
}

@test "cmd_schedule fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_schedule enable
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}
```

- [ ] **Step 6: 테스트 실패 확인**

Run: `bats tests/cmd_schedule.bats`
Expected: FAIL (여전히 `restic-backup.timer`를 직접 다루는 옛 구현이 실행됨)

- [ ] **Step 7: `cmd_schedule` 구현 교체**

`systemd_enable_timer()`/`systemd_disable_timer()`/`cmd_schedule()`을 통째로 아래로 교체(이 태스크에서 `systemd_enable_timer`/`systemd_disable_timer`는 삭제된다 — `cmd_uninstall`이 아직 `systemd_disable_timer`를 호출하고 있으므로, 이 Step에서는 우선 남겨두고 Task 22에서 함께 제거한다):

```bash
cmd_schedule() {
  require_root
  local action="${1:-}"
  shift || true

  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi
  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"
  local profile_name="${BACKUP_PROFILE_NAME:-$(hostname)}"

  case "$action" in
    enable)
      local parsed
      parsed=$(parse_long_opts "on-calendar:" -- "$@") || die "$parsed"
      local on_calendar="$DEFAULT_ON_CALENDAR"
      local key val
      while IFS=$'\t' read -r key val; do
        case "$key" in
          on-calendar) on_calendar="$val" ;;
        esac
      done <<< "$parsed"

      write_secure_file "$RESTICPROFILE_UNIT_TEMPLATE" 644 "$(render_resticprofile_unit_template)"
      write_secure_file "$RESTICPROFILE_TIMER_TEMPLATE" 644 "$(render_resticprofile_timer_template)"
      write_secure_file "$RESTICPROFILE_CONFIG_FILE" 600 \
        "$(render_resticprofile_config "$profile_name" "$on_calendar" "${BACKUP_TARGETS:-}" "${BACKUP_EXCLUDES:-}" "${KEEP_DAILY}" "${KEEP_WEEKLY}" "${KEEP_MONTHLY}")"
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" schedule
      log_info "schedule enable 완료 (${on_calendar})"
      ;;
    disable)
      resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" unschedule 2>/dev/null || true
      log_info "schedule disable 완료"
      ;;
    *)
      die "schedule은 'enable' 또는 'disable'만 지원합니다 (입력값: '${action}')"
      ;;
  esac
}
```

- [ ] **Step 8: 테스트 통과 확인**

Run: `bats tests/cmd_schedule.bats`
Expected: 전체 PASS

- [ ] **Step 9: `cmd_status`가 결정론적 유닛 이름을 조회하도록 수정 — 실패하는 테스트부터**

`tests/cmd_status.bats`의 두 시나리오 테스트에 `export BACKUP_PROFILE_NAME="host1"` 한 줄씩 추가하고, `systemctl` 스텁 검증을 유닛 이름까지 확인하도록 강화한다(파일 전체를 아래로 교체):

```bash
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
}

@test "cmd_status fails with guidance when backup.env is missing" {
  run cmd_status
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_status masks secrets and reports snapshot/timer state" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RESTIC_PASSWORD="super-secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_PROFILE_NAME="host1"
ENV
  chmod 600 "$BACKUP_ENV_FILE"
  stub_command "restic" 'case "$1" in snapshots) echo "[]" ;; esac'
  stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; echo "inactive"'

  run cmd_status
  [ "$status" -eq 0 ]
  [[ "$output" == *"rclone:syno_backup:/backup/host1"* ]]
  [[ "$output" != *"super-secret"* ]]
  [[ "$output" == *"700"* ]]
  [[ "$output" == *"600"* ]]
  run cat "${STUB_BIN}/systemctl.calls"
  [[ "$output" == *"is-active resticprofile-backup@profile-host1.timer"* ]]
}

@test "cmd_status reports 'inactive' correctly when systemctl is-active exits nonzero (realistic stub)" {
  mkdir -p "$RESTIC_ETC_DIR"
  chmod 700 "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/host1"
export RESTIC_PASSWORD="super-secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_PROFILE_NAME="host1"
ENV
  chmod 600 "$BACKUP_ENV_FILE"
  stub_command "restic" 'case "$1" in snapshots) echo "[]" ;; esac'
  stub_command "systemctl" '
    if [[ "$1" == "is-active" ]]; then
      echo "inactive"
      exit 3
    fi
  '

  run cmd_status
  [ "$status" -eq 0 ]
  [[ "$output" == *"타이머 상태: inactive"* ]]
  [[ "$output" != *$'\nunknown'* ]]
}
```

- [ ] **Step 10: 테스트 실패 확인**

Run: `bats tests/cmd_status.bats`
Expected: 첫 번째 시나리오 테스트 FAIL (`systemctl.calls`에 `restic-backup.timer`가 찍혀 있어 `resticprofile-backup@profile-host1.timer` 문자열이 없음)

- [ ] **Step 11: `cmd_status` 구현 수정**

`timer_state=$(systemctl is-active restic-backup.timer ...)` 줄을 아래로 교체:

```bash
  local profile_name="${BACKUP_PROFILE_NAME:-$(hostname)}"
  local timer_state
  timer_state=$(systemctl is-active "resticprofile-backup@profile-${profile_name}.timer" 2>/dev/null) || true
  printf '타이머 상태: %s\n' "${timer_state:-unknown}"
```

- [ ] **Step 12: 테스트 통과 확인**

Run: `bats tests/*.bats`
Expected: 전체 PASS

- [ ] **Step 13: shellcheck + 커밋**

Run: `shellcheck backup.sh`
Expected: 0건

```bash
git add backup.sh tests/resticprofile_config.bats tests/cmd_schedule.bats tests/cmd_status.bats
git commit -m "feat: delegate scheduling to resticprofile, keep ISMS unit wording via custom templates"
```

---

### Task 21: `cmd_run`을 `resticprofile backup` 호출로 교체

**Files:**
- Modify: `backup.sh`
- Modify: `tests/cmd_run.bats`

**Interfaces:**
- Produces: `cmd_run()`이 이제 `restic backup`/`restic forget`을 직접 호출하지 않고, `profiles.yaml`을 재렌더링한 뒤 `resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" backup`을 호출한다(retention의 `after-backup: true` + `prune: true` 덕분에 forget/prune은 resticprofile이 알아서 잇따라 실행한다). `restic unlock --stale` 줄은 제거되고 `force-inactive-lock: true` + `restic-stale-lock-age`(Task 20에서 이미 렌더링됨)로 대체된다.
- Consumes: Task 20의 `render_resticprofile_config`, `write_secure_file`, `RESTICPROFILE_CONFIG_FILE`.

- [ ] **Step 1: 실패하는 테스트 작성**

`tests/cmd_run.bats`를 아래로 전체 교체:

```bash
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_REPOSITORY="local:/tmp/fake-repo"
export RESTIC_PASSWORD="secret"
export BACKUP_TARGETS="/var/log"
export BACKUP_EXCLUDES="/tmp/*,/var/tmp/*"
export KEEP_DAILY="7"
export KEEP_WEEKLY="4"
export KEEP_MONTHLY="12"
export BACKUP_PROFILE_NAME="web01"
ENV
}

@test "cmd_run fails with guidance when backup.env is missing" {
  rm -f "$BACKUP_ENV_FILE"
  run cmd_run
  [ "$status" -eq 1 ]
  [[ "$output" == *"setting"* ]]
}

@test "cmd_run renders profiles.yaml fresh and delegates the backup to resticprofile" {
  stub_command "resticprofile" '
    echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"
    exit 0
  '
  run cmd_run
  [ "$status" -eq 0 ]
  [ -f "$RESTICPROFILE_CONFIG_FILE" ]
  perm=$(stat -c '%a' "$RESTICPROFILE_CONFIG_FILE")
  [ "$perm" = "600" ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--config ${RESTICPROFILE_CONFIG_FILE} --name web01 backup"* ]]
  [[ "$output" != *"restic unlock"* ]]
}

@test "cmd_run re-renders profiles.yaml every run so a stale copy never gets reused" {
  stub_command "resticprofile" 'exit 0'
  echo "stale placeholder, must be overwritten" > "$RESTICPROFILE_CONFIG_FILE"
  chmod 600 "$RESTICPROFILE_CONFIG_FILE"
  run cmd_run
  [ "$status" -eq 0 ]
  grep -q 'repository: "local:/tmp/fake-repo"' "$RESTICPROFILE_CONFIG_FILE"
}

@test "cmd_run dies when resticprofile fails" {
  stub_command "resticprofile" 'exit 1'
  run cmd_run
  [ "$status" -eq 1 ]
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_run.bats`
Expected: FAIL (여전히 `restic`/`restic unlock`을 직접 호출하는 옛 구현이 실행됨)

- [ ] **Step 3: `cmd_run` 구현 교체**

`cmd_run()` 전체를 아래로 교체:

```bash
cmd_run() {
  if [[ ! -f "$BACKUP_ENV_FILE" ]]; then
    die "$(render_missing_settings_message)"
  fi

  # shellcheck source=/dev/null
  source "$BACKUP_ENV_FILE"
  local profile_name="${BACKUP_PROFILE_NAME:-$(hostname)}"

  write_secure_file "$RESTICPROFILE_UNIT_TEMPLATE" 644 "$(render_resticprofile_unit_template)"
  write_secure_file "$RESTICPROFILE_TIMER_TEMPLATE" 644 "$(render_resticprofile_timer_template)"
  write_secure_file "$RESTICPROFILE_CONFIG_FILE" 600 \
    "$(render_resticprofile_config "$profile_name" "$DEFAULT_ON_CALENDAR" "${BACKUP_TARGETS:-}" "${BACKUP_EXCLUDES:-}" "${KEEP_DAILY}" "${KEEP_WEEKLY}" "${KEEP_MONTHLY}")"

  if resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" backup; then
    log_info "백업 성공"
  else
    die "resticprofile backup 실패"
  fi
}
```

(retention의 `after-backup: true`+`prune: true`가 forget/prune을 자동으로 잇따라 실행하므로, 이전의 `restic forget --keep-daily ... --prune` 호출과 그 뒤의 `log_info "만료 스냅샷 정리 완료"`는 더 이상 필요 없다 — resticprofile 자신의 출력이 그 역할을 대신한다.)

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_run.bats`
Expected: 전체 PASS

- [ ] **Step 5: shellcheck + 전체 bats + 커밋**

Run: `shellcheck backup.sh && bats tests/*.bats`
Expected: shellcheck 0건, bats 전체 PASS

```bash
git add backup.sh tests/cmd_run.bats
git commit -m "feat: delegate backup+forget+prune orchestration and stale-lock handling to resticprofile"
```

---

### Task 22: `cmd_uninstall` 갱신 + Tier 2 통합 테스트를 resticprofile 경로로 업데이트 + 최종 점검

**Files:**
- Modify: `backup.sh`
- Modify: `tests/cmd_uninstall.bats`, `tests/integration/docker-compose.yml`, `tests/integration/run.sh`

**Interfaces:**
- Produces: `cmd_uninstall()`이 이제 (purge 여부와 무관하게) `resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" unschedule`을 호출하고, `--purge`일 때 `/etc/restic` 전체 삭제에 더해 `/root/.cache/restic`도 삭제한다. `render_service_unit`/`render_timer_unit`/`systemd_enable_timer`/`systemd_disable_timer`/`SYSTEMD_SERVICE_FILE`/`SYSTEMD_TIMER_FILE`은 이제 아무도 호출하지 않지만, `tests/render_units.bats`(Task 6)가 이미 이 함수들을 직접 단위 테스트하고 있으므로 함수 자체는 삭제하지 않는다(죽은 코드지만 회귀 테스트 자산으로 남긴다 — 실제로 아무 `cmd_*`도 더는 참조하지 않는다는 것은 `grep -n "render_service_unit\|render_timer_unit\|systemd_enable_timer\|systemd_disable_timer" backup.sh`로 확인 가능).
- Consumes: Task 20/21의 `RESTICPROFILE_CONFIG_FILE`, `BACKUP_PROFILE_NAME`.

- [ ] **Step 1: 실패하는 테스트 작성**

`tests/cmd_uninstall.bats`를 아래로 전체 교체:

```bash
#!/usr/bin/env bats

load test_helper.bash

setup() {
  setup_backup_sh_env
  stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"'
  mkdir -p "$RESTIC_ETC_DIR"
  cat > "$BACKUP_ENV_FILE" <<'ENV'
export RESTIC_PASSWORD=secret
export BACKUP_PROFILE_NAME=web01
ENV
}

@test "cmd_uninstall without --purge unschedules via resticprofile but keeps /etc/restic" {
  run cmd_uninstall
  [ "$status" -eq 0 ]
  [ -f "$BACKUP_ENV_FILE" ]
  run cat "${STUB_BIN}/resticprofile.calls"
  [[ "$output" == *"--name web01 unschedule"* ]]
}

@test "cmd_uninstall --purge also removes the restic config dir and restic's cache" {
  mkdir -p "${TEST_ROOT}/root/.cache/restic"
  export HOME="${TEST_ROOT}/root"
  run cmd_uninstall --purge
  [ "$status" -eq 0 ]
  [ ! -d "$RESTIC_ETC_DIR" ]
  [ ! -d "${HOME}/.cache/restic" ]
}

@test "cmd_uninstall survives resticprofile unschedule failing (nothing was ever scheduled)" {
  stub_command "resticprofile" 'exit 1'
  run cmd_uninstall
  [ "$status" -eq 0 ]
}
```

- [ ] **Step 2: 테스트 실패 확인**

Run: `bats tests/cmd_uninstall.bats`
Expected: 첫 번째/세 번째 테스트 FAIL(옛 구현은 `resticprofile`을 전혀 호출하지 않음), 두 번째 테스트도 캐시 삭제 부분 FAIL

- [ ] **Step 3: `cmd_uninstall` 구현 수정**

`systemd_disable_timer` 호출 줄과 유닛 파일 삭제 줄을 아래로 교체:

```bash
cmd_uninstall() {
  require_root
  local parsed
  parsed=$(parse_long_opts "purge" -- "$@") || die "$parsed"

  local purge=0
  local key val
  while IFS=$'\t' read -r key val; do
    case "$key" in
      purge) purge=1 ;;
    esac
  done <<< "$parsed"

  if [[ -f "$BACKUP_ENV_FILE" ]]; then
    # shellcheck source=/dev/null
    source "$BACKUP_ENV_FILE"
    local profile_name="${BACKUP_PROFILE_NAME:-$(hostname)}"
    resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" unschedule 2>/dev/null || true
  fi

  if (( purge )); then
    rm -rf "$RESTIC_ETC_DIR"
    rm -rf "${HOME:-/root}/.cache/restic"
    log_info "uninstall --purge 완료 (${RESTIC_ETC_DIR} 삭제됨)"
  else
    log_info "uninstall 완료 (${RESTIC_ETC_DIR}는 유지됨)"
  fi
}
```

(resticprofile 바이너리 자체(`$RESTICPROFILE_INSTALL_PATH`)는 `restic`/`rclone` 패키지와 마찬가지로 `--purge`에도 삭제하지 않는다 — "설치된 도구는 유지, 우리가 만든 설정/데이터만 지운다"는 기존 원칙 그대로.)

- [ ] **Step 4: 테스트 통과 확인**

Run: `bats tests/cmd_uninstall.bats`
Expected: 전체 PASS

- [ ] **Step 5: Tier 2 통합 테스트를 resticprofile 경로로 업데이트**

`tests/integration/run.sh`의 install 단계에 resticprofile 다운로드가 GitHub에 실제로 나가는 것은 그대로 두고(이 컨테이너는 아웃바운드 가능한 환경이므로 Global Constraints의 "GitHub 접근 가능 전제"와 일치), S3/SFTP 각 시나리오의 `run` 단계 다음에 스케줄 delegation도 함께 검증하는 단계를 추가한다.

**사전 실측(2026-07-10, `tests/integration`의 `app` 컨테이너에 resticprofile 바이너리를 직접 복사하고 `dnf install -y systemd`로 systemctl까지 넣은 뒤 `resticprofile schedule`을 실행해서 확인):**
- `docker-compose.yml`의 `app` 서비스는 `entrypoint: ["sleep", "infinity"]`라 systemd가 PID 1이 아니다. `resticprofile ... schedule`은 유닛 파일을 `/etc/systemd/system/`에 **먼저 쓴 다음** `systemctl daemon-reload`/`enable` 단계에서 `Failed to connect to bus: Host is down`로 실패한다 — 즉 `backup.sh schedule enable` 자체는 이 컨테이너에서 **exit 1로 끝나지만, 유닛 파일은 이미 디스크에 존재한다.**
- 그래서 "타이머가 실제로 활성화됐는가"(`systemctl list-timers`)는 이 컨테이너에서 검증할 수 없지만(→ `tests/MANUAL_CHECKLIST.md`로 넘긴다), **"생성된 유닛 파일에 비밀값이 새지 않는가"는 이 컨테이너에서도 그대로 검증할 수 있다** — 오히려 이 프로젝트 입장에선 이쪽이 더 중요한 확인이다(Task 20의 `render_resticprofile_unit_template`가 `.Environment`를 렌더링하지 않아야 하는 이유이기도 하다).

`run.sh`의 S3 시나리오 블록(`bash backup.sh init` / `bash backup.sh run` 다음)에 추가:

```bash
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
```

SFTP 시나리오 블록에도 동일하게 추가(`bash backup.sh run` 다음, `AWS_SECRET_ACCESS_KEY` 대신 SFTP 쪽 비밀값인 `RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE`은 파일 경로라 비밀값이 아니므로 검사 대상이 아니다 — SFTP 시나리오에선 `RESTIC_PASSWORD` 부재만 확인하면 된다):

```bash
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
```

`docker-compose.yml`의 `app` 서비스 이미지(`rockylinux:9`)에는 `systemctl` 바이너리 자체가 없으므로(기본 최소 이미지), 위 블록이 동작하려면 `install` 단계 이후에 `dnf install -y systemd`를 한 번 추가해야 한다(이 태스크 Step 5에서 `run.sh`의 install 블록에 반영):

```bash
echo "=== install ==="
docker compose exec -T app bash -c '
  set -euo pipefail
  bash backup.sh install
  dnf install -y openssh-clients systemd
'
```

- [ ] **Step 6: 통합 테스트 실행**

Run: `cd tests/integration && ./run.sh`
Expected: 전체 통과 — `schedule enable`/`disable` 자체는 이 컨테이너에서 실패해도(`|| true`로 무시) 무방하고, 유닛 파일 내용 검증(ISMS 문구 존재 + 비밀값 부재)만 통과하면 된다. `systemctl list-timers`로 실제 활성화까지 확인하는 항목은 `tests/MANUAL_CHECKLIST.md`에 남긴다(다음 Step에서 추가).

- [ ] **Step 6.5: `tests/MANUAL_CHECKLIST.md`에 실제 타이머 활성화 확인 항목 추가**

`tests/MANUAL_CHECKLIST.md`의 "SFTP 경로" 섹션, 기존 `backup.sh schedule enable` 항목 바로 다음 줄에 추가:

```markdown
- [ ] `systemctl list-timers`에 `resticprofile-backup@profile-<profile-name>.timer` 노출 확인(Tier 2 컨테이너는 systemd가 PID 1이 아니라 실제 활성화까지는 검증 못 함 — 유닛 파일 내용/비밀값 부재만 자동 검증됨)
- [ ] 위 타이머를 즉시 트리거해 실제로 백업이 도는지 확인(`systemctl start resticprofile-backup@profile-<profile-name>.service` 또는 시각을 1분 뒤로 맞춤)
```

- [ ] **Step 7: 전체 shellcheck + bats 최종 점검**

Run: `shellcheck backup.sh && bats tests/*.bats`
Expected: shellcheck 0건, bats 전체 PASS

- [ ] **Step 8: 커밋**

```bash
git add backup.sh tests/cmd_uninstall.bats tests/integration/run.sh tests/MANUAL_CHECKLIST.md
git commit -m "feat: delegate uninstall's unschedule step to resticprofile, purge restic cache too"
```

---

## 마이그레이션 완료 후 남는 것 (요약)

resticprofile로 대체되는 영역: (1) 백업 실행 오케스트레이션(backup→forget→prune), (2) 스케줄링(systemd 유닛 생성/enable/disable), (3) stale lock 처리.

여전히 커스텀 bash로 남는 영역: `cmd_install`의 패키지/자기설치, `cmd_wizard`의 대화형 온보딩, `cmd_setting`의 SSH 키생성/S3 버킷정책 안내/입력검증(`validate_*`)/에러 힌트 메시지(`render_*_hint_*`), `cmd_status`의 비밀번호 마스킹 조회, `cmd_uninstall`의 `/etc/restic` 정리, `backup.env` 단일 소스 관리. 즉 "100% 대체"는 아니고, `backup.sh`(≈880줄 예상) 중 실행 엔진 부분만 resticprofile에 위임되고 나머지 절반 이상은 그대로 남는다.

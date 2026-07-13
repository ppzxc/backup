# AI Agent Guide (`AGENTS.md`)

이 문서는 AI 코딩 에이전트(Claude Code, Cursor 등)가 이 저장소에서 작업할 때 준수해야 하는 개발 가이드 및 제약사항입니다.

## 🚀 자주 사용하는 명령 (Common Commands)

* **린트 (Lint)**: `shellcheck backup.sh` (커밋 전 경고 0건 유지 필수)
* **전체 단위 테스트**: `bats tests/`
* **단일 테스트 실행**: `bats tests/<file>.bats`
* **단일 테스트 케이스**: `bats tests/<file>.bats -f "<test-case-name>"`
* **통합 테스트 (E2E)**: `cd tests/integration && ./run.sh` (MinIO/SFTP 도커 환경 활용)

## 🏗️ 아키텍처 및 설계 규칙 (Architecture & Rules)

* **Functional Core / Imperative Shell** 구조 준수:
  * 입출력 및 외부 명령 호출이 없는 순수 함수(`resolve_*`, `validate_*`, `render_*`, `parse_long_opts`)는 독립적으로 실행할 수 있도록 설계하고 `bats` 테스트에서 직접 검증합니다.
  * 파일 쓰기, `systemctl`, `restic`, `rclone` 같은 외부 명령 호출은 얇은 래퍼 함수(Shell)에 격리시키고, 테스트 시 `stub_command`로 스텁 처리하여 원격 종속성을 차단합니다.
* **설정값 우선순위 및 단일 원천 (Single Source of Truth)**:
  * 모든 설정값은 `resolve_value`를 거쳐 `CLI 플래그 > 환경변수 > 기존 backup.env > 내장 기본값` 순으로 결정합니다.
  * `/etc/restic/backup.env` 파일이 호스트별 설정의 유일한 단일 원천(Source of Truth)이며, `profiles.yaml`과 같은 산출물은 `backup.env`를 원천으로 가동 시점에 동적 렌더링되어야 합니다.

## 🔒 보안 및 컴플라이언스 (Security & Compliance)

* **권한 강제**: `/etc/restic` 디렉터리는 권한 `700`, `backup.env` 및 `profiles.yaml` 파일은 권한 `600`을 생성/수정 시 명시적으로 강제 적용합니다.
* **비밀값 노출 차단**: 
  * 비밀번호 등 민감 자격 증명은 화면 출력(`status`, `log_*`)이나 로그, `systemd` 유닛 파일(`.Environment` 블록 배제)에 절대 평문으로 노출해선 안 되며 마스킹 처리해야 합니다.
  * SFTP 연결 상태 조회 시 `ssh`/`sftp` 쉘 세션 로그 대신 `rclone` 바이너리(`rclone_check_connectivity`)를 활용해 인증 오탐 및 보안 우회를 방지합니다.

## 🧪 개발 및 테스트 컨벤션 (Conventions)

* **테스트 주도 개발 (TDD)**: 새로운 기능 정의나 버그 수정 시, 실패하는 `bats` 테스트를 먼저 작성하여 실패를 확인하고, 구현 후 성공을 확인하는 워크플로우를 고수합니다.
* **버전 관리 (Versioning)**: 백업 스크립트 코드(`backup.sh`) 수정 시, 파일 최상단의 `BACKUP_SCRIPT_VERSION` 변수 버전을 반드시 범프(상승)해야 합니다. 버전이 변경되지 않으면 GitHub Actions의 병합 방지 워크플로우가 실패합니다.
* **정적 분석 우회 주석**: `shellcheck` 우회가 불가피한 경우(예: nameref나 동적 소싱 경계 등), 주석 위에 구체적인 사유를 1줄로 기재한 후 `# shellcheck disable=<code>`를 선언합니다.
* **Git 추적 규칙**: `.gitignore`는 허용 목록(Allow-list, `/*` 패턴으로 전체 차단 후 개별 `!` 예외 지정)으로 구현되어 있습니다. 새로운 최상위 파일이나 디렉터리를 추가할 때는 `.gitignore`에 이를 명시적으로 추가해야만 git 추적 대상에 들어옵니다.

## Agent skills

### Issue tracker

Issues and PRDs for this repo live as GitHub issues. PRs are not treated as an external request surface. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage labels map to the default vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

Domain layout is configured as single-context. See `docs/agents/domain.md`.


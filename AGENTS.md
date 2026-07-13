# AGENTS.md

이 문서는 AI 코딩 에이전트(Claude Code 등)가 이 저장소에서 작업할 때 참고하는 가이드입니다.

## 이 저장소는 무엇인가

`backup.sh` 하나로 이루어진 bash 스크립트로, systemd 기반 Linux 서버(주로 RHEL
계열)에 `restic` 백업 파이프라인을 설치·운영합니다. 저장 백엔드로 S3 호환
오브젝트 스토리지 또는 rclone을 통한 SFTP/NAS 대상 중 하나를 지원하며, 다음
서브커맨드를 제공합니다:

`install` · `setting` · `init` · `schedule` · `run` · `status` · `audit` · `uninstall` · `wizard`

백업 실행(backup/forget/prune 조율), 스케줄링(systemd 타이머 생성), stale
lock 처리는 [resticprofile](https://creativeprojects.github.io/resticprofile/)에
위임합니다. `restic`/`rclone`/`resticprofile` 셋 다 `cmd_install`이 **버전을
고정하고 체크섬을 검증한 뒤 GitHub 릴리스에서 직접 내려받아 설치**합니다(각각
`RESTIC_VERSION`/`RESTIC_SHA256`, `RCLONE_VERSION`/`RCLONE_SHA256`,
`RESTICPROFILE_VERSION`/`RESTICPROFILE_SHA256` — `backup.sh` 상단 참고). `dnf`/
`EPEL`에는 더 이상 의존하지 않습니다 — 서버마다 EPEL 활성화 여부나 배포판에
따라 restic/rclone 버전이 들쭉날쭉해지는 문제(실사용 중 발견)를 피하기 위함이며,
부수 효과로 RHEL 계열이 아닌 systemd 배포판에서도 동작할 여지가 생겼습니다.
그 외(설정값 검증, SSH 키 생성, 비밀값을 가린 상태 보고, 대화형 wizard 등)는
평범한 bash입니다.

## 빌드·린트·테스트

- 린트: `shellcheck backup.sh` (커밋 전 0건이어야 함)
- 전체 유닛 테스트: `bats tests/*.bats`
- 단일 테스트 파일: `bats tests/cmd_run.bats`
- 단일 테스트 케이스: `bats tests/cmd_run.bats -f "cmd_run dies when resticprofile fails"`
- Tier 2 통합 테스트(docker compose로 MinIO + SFTP + rockylinux:9 컨테이너를
  띄우고 install/setting/init/run/schedule을 실제 백엔드에 대고 end-to-end로
  검증): `cd tests/integration && ./run.sh` — Docker와 GitHub로의 아웃바운드
  네트워크 접근(실제 restic/rclone/resticprofile 다운로드용)이 필요합니다.
- Tier 3 수동 검증 체크리스트(Tier 2로 재현 안 되는 것들 — 실제 NAS/버킷 등록,
  실제 systemd 타이머 활성화, wizard 프롬프트 문구): `tests/MANUAL_CHECKLIST.md`

## 아키텍처

`backup.sh`는 core/imperative 스타일 셸로 작성돼 있습니다: 작고 순수한
함수들(`resolve_value`, `validate_*`, `render_*`, `parse_long_opts`)은
`tests/test_helper.bash`의 `setup_backup_sh_env`로 스크립트를 소싱해 bats
테스트에서 단독으로 검증할 수 있고, 각 서브커맨드를 구현하는 `cmd_*` 함수들은
파일 맨 아래 `main()`에서 디스패치됩니다. 부수 효과(파일 쓰기,
`restic`/`rclone`/`resticprofile`/`systemctl` 같은 외부 명령 호출)는 얇은
래퍼 함수(`write_secure_file`, `install_restic`, `install_rclone`,
`rclone_check_connectivity` 등) 뒤에 숨겨져 있어, 테스트는 `cmd_*` 함수 내부를
깊이 모킹하는 대신 `tests/test_helper.bash`의 `stub_command`로 외부 명령
자체를 스텁합니다.

설정값 해석 순서는 `resolve_value` 한 곳에서만 구현합니다: CLI 플래그 >
환경변수 > 기존 `backup.env` 값 > 내장 기본값. `cmd_setting`의 모든 플래그가
이 경로를 거칩니다.

`backup.env`(`cmd_setting`이 작성하고, `require_backup_env` 헬퍼를 통해
`cmd_init`/`cmd_schedule`/`cmd_run`/`cmd_status`/`cmd_audit`/`cmd_uninstall`이
읽음)가 호스트별 백엔드 설정·보존 정책·프로파일 이름의 단일 진실 공급원이며,
나머지는 실행 시점에 여기서 파생됩니다.

SFTP 백엔드의 `cmd_init`은 `restic init`을 부르기 전에 `rclone lsd <remote>:`로
연결/인증을 먼저 점검합니다(`rclone_check_connectivity`). 시스템 `ssh`/`sftp`
클라이언트 대신 **restic이 실제로 spawn하는 것과 같은 rclone 바이너리**로
점검하는 이유는 두 가지입니다: (1) NAS 계정이 SFTP 전용(쉘 로그인 권한 없음)으로
제한된 경우 공개키 인증에 성공하고도 일반 exec/쉘 세션은 거부되어, 실제로는
문제 없는 설정을 오탐으로 막던 문제(실사용 중 재현)를 없애고, (2) 별도
openssh-clients 의존성이 사라집니다. 점검 대상은 백업 서브경로가 아니라
remote 루트인데, 최초 init 시점엔 그 서브경로가 아직 없는 게 정상이라 서브경로를
직접 보면 인증에 성공했어도 "directory not found"로 오탐하기 때문입니다.

모든 하위 명령은 위치와 무관하게 전역 `-v`/`--verbose` 플래그를 받습니다
(`main()`에서 파싱해 `BACKUP_VERBOSE`로 전달). 기본값에서는 조용하고,
`--verbose`를 주면 SFTP 연결 실패 시 rclone 자체의 진단 메시지를, `init`/`run`
실행 시 `restic`/`resticprofile`의 상세 로그를 함께 보여줍니다.

`cmd_audit`은 ISMS 등 감사 대응용으로 백업 정책·보존 정책·스케줄·접근 통제를
한 화면에 모아 보여주는 컴플라이언스 리포트입니다(`cmd_status`는 빠른 운영
확인용으로 계속 별도 유지). systemd 타이머 유닛에서 `OnCalendar`/등록·실행
상태/다음 실행 예정을 읽고, `restic snapshots` 원본 출력을 백업 이력으로 그대로
보여줍니다.

구현 히스토리와 설계 근거 전체(resticprofile 마이그레이션 결정 — 예: 커스텀
systemd 유닛 템플릿이 `.Environment` 블록을 일부러 빼는 이유 등 포함)는
`docs/superpowers/plans/2026-07-10-restic-backup-script.md`에 기록돼 있습니다.

## 컨벤션

- `shellcheck backup.sh`는 항상 0건을 유지합니다. 경고가 알려진 false
  positive인 경우(예: nameref나 소싱된 env 파일 경계를 shellcheck이 못 보는
  `SC2153`/`SC2034`), 무시하지 말고 인라인 `# shellcheck disable=<code>` 주석과
  그 위에 이유를 한 줄로 남겨 억제합니다. 반대로 실제로 트리거되지 않는
  불필요한 disable 주석은 남기지 않습니다.
- 새 동작은 `docs/superpowers/plans/2026-07-10-restic-backup-script.md`의
  패턴대로 테스트부터 작성합니다(bats): 실패하는 테스트를 먼저 쓰고, 실패를
  확인한 뒤, 구현하고, 통과를 확인합니다.
- `.gitignore`는 deny list가 아니라 allow list(`/*` 다음 명시적 `!` 예외)입니다
  — 새로 추적할 최상위 파일/디렉토리는 여기 명시적으로 추가해야 하며, 안 그러면
  `git add`가 조용히 아무 일도 하지 않습니다.

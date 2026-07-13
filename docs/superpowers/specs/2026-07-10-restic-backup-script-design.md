# restic 백업 자동화 스크립트 설계

## 배경 / 목적

여러 리눅스 서버(RHEL 계열, dnf 기반)에 배포해서 restic 기반 백업을 세팅·운영할 수 있는 단일 셸 스크립트 `backup.sh`를 만든다. 백엔드는 **S3 호환 스토리지** 또는 **rclone SFTP(시놀로지 NAS 등)** 중 하나를 선택해 연결한다.

"1차 백업 / 2차 소산백업"은 이 스크립트가 직접 오케스트레이션하는 개념이 아니다. 이는 조직 차원의 논리적 백업 정책이며, 어떤 서버에서 어떤 백엔드로 이 스크립트를 실행하느냐(예: A 서버는 로컬 NAS로, NAS 자체는 별도 프로세스로 오프사이트 복제)에 따라 자연히 결정된다. 스크립트는 **호스트 1대당 restic 저장소 1개(단일 backup.env)** 를 설정하는 것까지만 책임진다.

## 배포 방식

- `backup.sh` 단일 파일로 배포한다 (curl/scp로 대상 서버에 파일을 내려받아 실행).
- `curl | bash` 파이프 실행은 지원하지 않는다 — `install` 단계에서 스크립트가 자기 자신을 `/usr/local/sbin/backup.sh`로 복사(self-install)해 systemd가 항상 안정된 경로를 참조하게 하는데, 파이프 실행 시 안정적인 원본 파일 경로를 알 수 없기 때문이다. 파일을 먼저 내려받아 실행하는 방식만 지원한다(보안·ISMS 관점에서도 curl\|bash보다 낫다).
- 스크립트는 root 권한으로만 실행 가능하다(EUID 체크, 아니면 즉시 안내 후 종료).

## 디렉터리/파일 구조

- `/usr/local/sbin/backup.sh` — install 시 자기 복사본. systemd가 이 경로를 호출한다.
- `/etc/restic/` (권한 700, root:root)
  - `backup.env` (권한 600) — 아래 값을 담는 셸 소스 가능한 환경변수 파일:
    - `RESTIC_REPOSITORY`, `RESTIC_PASSWORD`
    - `BACKUP_TARGETS` (콤마 구분 경로), `BACKUP_EXCLUDES` (반복 가능)
    - `KEEP_DAILY`, `KEEP_WEEKLY`, `KEEP_MONTHLY`
    - 백엔드가 sftp면: `RCLONE_CONFIG_*` 계열 변수
    - 백엔드가 s3면: `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` (및 필요시 endpoint/region 관련 restic 옵션)
  - `backup_key` / `backup_key.pub` (권한 600/644) — sftp 백엔드일 때만 생성되는 SSH 키페어
- `/etc/systemd/system/restic-backup.service`, `restic-backup.timer`
- 로그는 별도 파일을 두지 않고 `logger -t restic-backup`으로 journald/syslog에 남긴다(중앙 로그 수집·로테이션 관리 불필요, ISMS 감사 추적에 유리).

## 설정값 우선순위

**CLI 플래그 > 환경변수(`BACKUP_*` 등) > 기존 `/etc/restic/backup.env` > 스크립트 내 기본값**

이는 restic 자신이 `--repository-file`/`RESTIC_REPOSITORY_FILE` vs `RESTIC_REPOSITORY`에서 따르는 패턴과 동일한 업계 표준 컨벤션이다. 필수값이 이 어느 경로에도 없으면, 대화형으로 되묻지 않고 **무엇이 왜 없는지 + 다음에 실행해야 할 명령어를 그대로 복사해 쓸 수 있는 형태로** 출력하고 종료한다(아래 "에러 처리 원칙" 참고).

## CLI 서브커맨드

```
backup.sh                          → 도움말 출력 후 exit 0
backup.sh -h | --help              → 도움말 출력 후 exit 0

backup.sh install [--force] [--dry-run]
    - epel-release, restic, rclone 설치
    - 자기 자신을 /usr/local/sbin/backup.sh로 복사
    - /etc/restic 디렉터리(700) 생성
    - 설정값(backup.env)은 다루지 않는다 — 순수 "소프트웨어 설치"만 담당

backup.sh setting --backend <s3|sftp> [옵션...] [--force] [--dry-run]
    공통 옵션:
      --targets <path[,path...]>   (기본 /var/log)
      --exclude <path>             (반복 가능, 기본 /tmp/*, /var/tmp/*)
      --password <repo password>   (또는 BACKUP_PASSWORD 환경변수)
      --keep-daily/--keep-weekly/--keep-monthly N  (기본 7/4/12)
    --backend s3:   --endpoint, --bucket, --access-key, --secret-key
    --backend sftp: --host, --port(기본 22), --user
                     (SSH 키는 setting이 자동 생성, 플래그로 받지 않음)
    - backup.env 작성(600) (+ sftp면 SSH 키 생성)
    - 완료 후 안내 출력:
        sftp → 생성된 공개키 + "NAS의 authorized_keys에 등록하세요"
        s3   → 최소권한(ListBucket/GetObject/PutObject/DeleteObject, 해당 버킷 ARN 한정) 버킷 정책 JSON

backup.sh init
    - restic init 실행(저장소 최초 1회 초기화)
    - 이미 초기화된 저장소면 스킵(멱등)
    - backup.env가 없으면: "먼저 setting을 실행하세요" + 복사 가능한 예시 명령 출력 후 종료

backup.sh schedule enable [--on-calendar "*-*-* 02:00:00"]  (기본 매일 새벽 2시)
backup.sh schedule disable
    - restic-backup.service/.timer 생성/제거, daemon-reload, enable(--now)/disable

backup.sh run
    - 즉시 1회 수동 백업(restic unlock --stale → backup → 성공 시 forget --prune)
    - systemd service의 ExecStart도 동일하게 `backup.sh run`을 호출한다

backup.sh status
    - 백엔드 종류, 저장소 위치(비밀정보 마스킹), 최근 스냅샷, systemd 타이머 상태,
      /etc/restic 하위 권한(700/600) 자가진단 결과를 출력

backup.sh uninstall [--purge]
    - 기본: timer/service 비활성화 및 제거
    - --purge: 추가로 /etc/restic 전체 삭제(원격 저장소 자체는 건드리지 않음)

backup.sh wizard
    - 아래 "wizard 흐름" 참고. 새 로직을 만들지 않고 install/setting/init/schedule의
      내부 함수를 그대로 호출하는 얇은 대화형 레이어.
```

모든 서브커맨드는 **플래그를 주면 그 자리에서 즉시 실행**된다. 대화형 프롬프트나 확인 단계는 없다(단, `wizard`는 예외 — 처음부터 대화형으로 설계된 서브커맨드).

## wizard 흐름

```
backup.sh wizard
  0) root 권한 확인
  1) 패키지 설치 여부 확인 → 미설치 시 install 로직 재사용해 자동 설치
  2) "백엔드를 선택하세요: [1] S3 호환 스토리지  [2] SFTP(NAS)" + 각 옵션 한 줄 설명
  3) 선택한 백엔드에 필요한 값을 하나씩 질문 (질문 위에 그 값이 무엇인지 설명 한 줄 추가)
     - 기본값이 있는 항목(포트 22, 보존정책 7/4/12 등)은 Enter로 기본값 사용 가능
  4) 저장소 비밀번호 직접 입력(화면 비표시 입력, "분실 시 복구 불가" 경고 문구 포함)
  5) 입력값 요약 후 "이대로 진행할까요? [Y/n]"
  6) setting 로직 재사용 → backup.env(+SSH 키) 생성
     → wizard로 입력한 모든 값(백엔드, 접속정보, 대상 경로, 보존정책, 비밀번호)은
       전부 이 backup.env 한 파일에 저장된다 — setting을 플래그로 직접 실행했을 때와 동일한 결과물.
  7) 안내 출력(SFTP 공개키 등록 / S3 버킷 정책) + "등록 완료 후 Enter"
  8) init 로직 재사용 → restic init
  9) "지금 정기 백업 스케줄을 등록할까요? [Y/n] (기본 매일 새벽 2시)" → schedule enable 로직 재사용
  10) 최종 요약 출력(백엔드, 저장소 위치, 다음 백업 예정 시각, 이후 쓸 명령어 목록: run/status/uninstall)
```

## 에러 처리 원칙

- `set -euo pipefail` 적용. 모든 외부 명령(dnf, ssh-keygen, restic, rclone, systemctl) 실행 후 종료 코드를 확인하고, 실패 시 `die()` 헬퍼로 원인과 다음 행동을 함께 출력하고 종료한다.
- root가 아님 / 필수값 없음 / backup.env 없음 등은 단순 에러로 끝내지 않고, **무엇이 왜 안 됐는지 + 다음에 실행해야 할 명령을 복사해서 쓸 수 있는 형태**로 출력한다. 이미 알고 있는 값(플래그로 받은 값)은 채워 넣고, 모르는 값만 `<PLACEHOLDER>`로 남긴다.
  ```
  [!] 설정이 없습니다. 먼저 아래 명령으로 설정을 완료하세요:

      backup.sh setting --backend sftp --host <NAS_IP> --port <PORT> --user <USER> --password '<REPO_PASSWORD>'
  ```
- 비밀번호 등 민감정보는 로그(`logger`)나 화면 요약에 절대 평문 노출하지 않는다(마스킹 처리).
- `/etc/restic`(700), `backup.env`(600), SSH 키(600/644) 권한은 생성/수정 시마다 명시적으로 강제 적용한다(ISMS 접근통제 요구사항).

## S3 백엔드 보안에 관한 참고

SSH 키 인증은 SFTP(SSH 프로토콜) 전용이며 S3(호환) REST API에는 적용할 수 없다(HMAC 서명 기반 access key/secret key 인증). 이 스크립트는 SFTP의 "공개키 등록 안내"와 대칭되는 조치로, S3 `setting` 완료 시 **최소권한 버킷 정책 JSON 안내 출력**만 제공한다. STS AssumeRole/임시 세션 토큰 지원은 범위에서 제외한다(일부 S3 호환 스토리지 벤더가 미지원이며, 필요해지면 추후 옵션 플래그로 추가 가능).

## 코드 구성 원칙 (테스트 가능한 아키텍처)

`backup.sh` 내부 함수는 "순수 로직"과 "실제 시스템 조작"을 명확히 분리한다(functional core / imperative shell).

- **순수 함수** (`parse_*`, `validate_*`, `resolve_*`, `render_*` 접두사) — 외부 명령을 호출하지 않고 입력→출력만 수행. 예: long-option 인자 파싱, 설정값 우선순위 해석(flag > env > backup.env > 기본값), 값 검증(포트 숫자 여부, backend가 s3/sftp인지 등), 누락값 안내 메시지 생성(아는 값은 채우고 `<PLACEHOLDER>`만 남기는 로직), help 텍스트 생성, `.service`/`.timer` 유닛 파일 텍스트 렌더링.
- **실행 함수** (`do_*`, `run_*` 접두사) — dnf install, ssh-keygen, restic init/backup/forget, systemctl, 파일 권한 적용 등 실제 부수효과를 일으키는 부분. 순수 함수가 만든 값(파싱된 옵션, 렌더링된 유닛 파일 내용 등)을 받아 실행만 한다.

이 분리 덕분에 순수 함수는 실제 서버/패키지 관리자/NAS 없이도 독립적으로 테스트할 수 있다.

## 검증 방법 (3단계)

**Tier 1 — bats 유닛테스트 (순수 로직, TDD 대상)**
- `tests/backup.bats`에 순수 함수(`parse_*`/`validate_*`/`resolve_*`/`render_*`)에 대해 먼저 실패하는 테스트를 작성한 뒤 구현한다.
- 커버리지: 인자 파싱, 설정값 우선순위 해석, 값 검증(포트/backend 형식 등), 안내 메시지의 플레이스홀더 치환, help 텍스트, systemd 유닛 파일 렌더링 결과.
- 외부 명령을 호출하지 않으므로 CI에서 빠르게, 어떤 서버 없이도 실행 가능하다.

**Tier 2 — docker-compose 통합 테스트 (실제 dnf/restic/rclone 검증)**
- `tests/integration/docker-compose.yml`로 다음을 구성한다:
  - dnf 기반(Rocky/Alma) 컨테이너 — `backup.sh` 실행 대상
  - `quay.io/minio/minio` 컨테이너 — 고정 `MINIO_ROOT_USER`/`MINIO_ROOT_PASSWORD`로 S3 호환 엔드포인트 제공
  - SFTP 서버 컨테이너 — `setting`이 생성한 `backup_key.pub`을 `authorized_keys`로 주입
- 시나리오별로 `install → setting → init → run` 후 `restic snapshots`로 실제 스냅샷 생성을 검증한다(S3 경로, SFTP 경로 각 1개).
- `schedule enable/disable`(systemd 실사용)은 컨테이너에서 진짜 PID 1 systemd를 띄우는 비용이 커서 Tier 2에 넣지 않는다 — 대신 유닛 파일 렌더링은 이미 Tier 1에서 검증되고, 실제 `systemctl enable --now` 동작은 Tier 3 수동 체크리스트로 남긴다.

**Tier 3 — 수동 검증 체크리스트 (systemd·실서버 의존적인 부분)**
- 테스트 VM에서 1회씩: `schedule enable` → 타이머 실제 동작 확인 → `schedule disable`, `uninstall [--purge]`, `wizard` 전체 흐름
- 각 필수값 누락 시나리오에서 안내 메시지의 복사 가능한 명령이 올바르게 채워지는지 확인(이 부분은 Tier 1에서도 일부 커버되지만 실제 화면 출력까지 눈으로 재확인)

정적 검증: `shellcheck backup.sh` 경고 0건을 목표로 한다. `install`/`setting`에는 `--dry-run`을 제공해 실제 변경 없이 사전 검토할 수 있게 한다.

## 범위에서 제외한 것 (YAGNI)

- 호스트 1대에서 복수 저장소(1차+2차) 동시 설정 — 조직 차원의 정책이며 이 스크립트 단일 실행 범위가 아님
- S3 STS AssumeRole / 임시 세션 토큰 지원
- `curl | bash` 원격 실행 지원
- 별도 로그 파일 및 로그 로테이션 관리(journald/syslog로 대체)
- 자동 비밀번호 생성 옵션(wizard 포함, 직접 입력만 지원)
- systemd 타이머 실동작에 대한 컨테이너 자동화 테스트(Tier 2) — 수동 체크리스트(Tier 3)로 대체

# 1차 로컬 및 2차 원격 소산 백업 파이프라인 설계 사양서
- **작성일**: 2026-07-14
- **상태**: 승인됨 (Approved)

---

## 1. 개요 (Overview)

본 설계 사양서는 `backup.sh` 스크립트를 확장하여 기존의 단일 대상 백업 모델에서 **Functional Core / Imperative Shell** 철학을 유지하면서 **1차 로컬 백업 및 2차 원격 소산 백업(restic copy)**을 통합 파이프라인으로 수행하도록 고도화하는 설계를 담고 있습니다.

이를 통해 정보보호 관리체계(ISMS/ISMS-P)의 백업 소산 규정을 준수하고, 로컬 디스크의 신속한 RTO(복구 목표 시간) 장점과 지리적으로 분리된 외부 백업본의 안정성(재해 복구)을 동시에 충족시킵니다.

---

## 2. 핵심 요구사항 및 설계 원칙

1. **원본 서버 단일 통제 (시나리오 A)**
   - 백업 및 소산의 모든 과정은 원본 데이터를 보유한 서버의 `backup.sh` 스크립트와 `systemd timer` 스케줄러에 의해 단일 통제됩니다.
2. **1차 로컬 백업 경로 표준화**
   - 1차 백업의 기본 저장소 경로는 `/data/backup`으로 설정되며, 스크립트 실행 시 디렉터리가 부재할 경우 자동으로 생성하고 보안 권한 `700`을 강제합니다.
3. **2차 소산 명확화 (Secondary 접두사)**
   - 2차 백업 대상 설정 시 CLI 플래그 및 환경 변수에 `secondary-` 및 `SECONDARY_` 접두사를 풀네임으로 기재하여 1차 로컬 설정과 확실하게 구분합니다.
   - 예: `--secondary-backend`, `SECONDARY_RESTIC_REPOSITORY`
4. **패스워드 상속 및 분리**
   - 기본적으로 1차 로컬 저장소와 2차 원격 저장소의 암호화 패스워드는 동일하게 자동 공유(상속)되나, `--secondary-password` 지정 시 다르게 설정하여 암호를 분리할 수 있습니다.
5. **통합 결과 웹훅 알림**
   - 1차 로컬 백업과 2차 소산 백업의 성공/실패 여부를 수집하여 전체 파이프라인 완료 시점에 **단 1회 최종 결과 통합 알림**만 발송합니다.
6. **2차 무결성 검증 및 유지보수 주기 최소화**
   - 1차 로컬 저장소는 매 백업 직후 `prune` 및 `check`를 실행하여 가볍게 유지합니다.
   - 2차 원격 저장소는 네트워크 트래픽 및 시간 단축을 위해 평상시에는 `restic copy`만 수행하고, `prune` 및 `check`는 주 1회 또는 별도의 스케줄러로 한정하여 구동합니다.
7. **이중 복구 모의훈련 (Restore Drill)**
   - `--restore-drill` 검증 시, 1차 로컬 저장소와 2차 원격 저장소 양쪽 모두에서 최근 스냅샷의 임의 파일을 각각 복구 수행하고 성공 여부를 감사 보고서에 기록합니다.

---

## 3. 세부 설계 사양

### 3.1. CLI 플래그 확장 (`cmd_setting`)

`backup.sh setting` 명령어에 아래의 플래그들을 추가합니다:

* **1차 로컬 기본 플래그**:
  * `--local-repo <path>`: 1차 로컬 백업 저장소 경로 (기본값: `/data/backup`)
  * (기타 `--targets`, `--password`, `--exclude` 등 기존 옵션은 1차 백업 사양으로 자동 매핑)
* **2차 원격 소산 플래그 (`secondary-` 계열)**:
  * `--secondary-backend <s3|sftp>`: 2차 원격 저장소 타입 (S3 또는 SFTP)
  * `--secondary-password <pw>`: 2차 원격 저장소 비밀번호 (생략 시 `--password` 공유)
  * `--secondary-endpoint <url>`: S3 호환 오브젝트 스토리지 엔드포인트 URL
  * `--secondary-bucket <name>`: S3 버킷명
  * `--secondary-access-key <key>`: AWS/S3 Access Key
  * `--secondary-secret-key <key>`: AWS/S3 Secret Key
  * `--secondary-host <ip_or_domain>`: SFTP 호환 서버 호스트
  * `--secondary-user <user>`: SFTP 서버 접속 사용자명
  * `--secondary-port <port>`: SFTP 서버 접속 포트 (기본값: 22)
  * `--secondary-keep-daily <N>`: 2차 저장소 일별 보관 스냅샷 개수
  * `--secondary-keep-weekly <N>`: 2차 저장소 주별 보관 스냅샷 개수
  * `--secondary-keep-monthly <N>`: 2차 저장소 월별 보관 스냅샷 개수

### 3.2. 환경 변수 스키마 (`backup.env`)

`/etc/restic/backup.env` 파일 내에 저장될 환경변수 정의 명세:

```bash
# 1차 로컬 백업용 변수
export RESTIC_REPOSITORY="/data/backup"
export RESTIC_PASSWORD="primary_encryption_password"
export BACKUP_TARGETS="/var/log,/etc"
export BACKUP_EXCLUDES="/tmp"
export KEEP_DAILY=7
export KEEP_WEEKLY=4
export KEEP_MONTHLY=3

# 2차 원격 소산 백업용 변수 (S3 예시)
export SECONDARY_BACKEND="s3"
export SECONDARY_RESTIC_REPOSITORY="s3:https://s3.amazonaws.com/my-backup-bucket"
export SECONDARY_RESTIC_PASSWORD="primary_encryption_password" # 또는 다른 암호
export SECONDARY_AWS_ACCESS_KEY_ID="AWSACCESSKEYID123"
export SECONDARY_AWS_SECRET_ACCESS_KEY="AWSSECRETACCESSKEY456"
export SECONDARY_KEEP_DAILY=30
export SECONDARY_KEEP_WEEKLY=12
export SECONDARY_KEEP_MONTHLY=12
```

### 3.3. 다중 프로필 설정 (`profiles.yaml`)

`render_resticprofile_config` 함수가 기동될 때, 하나의 `profiles.yaml` 파일 내에 두 개의 프로필(`local-backup` 및 `secondary-backup`)을 생성하도록 확장합니다:

```yaml
version: "1"

global:
  restic-lock-retry-after: 1m
  restic-stale-lock-age: 2h
  systemd-unit-template: /etc/restic/resticprofile-unit.template
  systemd-timer-template: /etc/restic/resticprofile-timer.template

# 1차 로컬 백업 프로필
local-backup:
  repository: "/data/backup"
  force-inactive-lock: true
  env:
    RESTIC_PASSWORD: "primary_encryption_password"
    HOSTNAME: "target-host-name"
  retention:
    after-backup: true
    prune: true
    group-by: host
    keep-daily: 7
    keep-weekly: 4
    keep-monthly: 3
  backup:
    schedule: "*-*-* 02:00:00"
    schedule-permission: system
    source:
      - "/var/log"
      - "/etc"
    exclude:
      - "/tmp"

# 2차 원격 소산 백업 프로필
secondary-backup:
  repository: "s3:https://s3.amazonaws.com/my-backup-bucket"
  force-inactive-lock: true
  env:
    RESTIC_PASSWORD: "secondary_encryption_password"
    AWS_ACCESS_KEY_ID: "AWSACCESSKEYID123"
    AWS_SECRET_ACCESS_KEY: "AWSSECRETACCESSKEY456"
    HOSTNAME: "target-host-name"
  retention:
    prune: true
    group-by: host
    keep-daily: 30
    keep-weekly: 12
    keep-monthly: 12
```

### 3.4. 파이프라인 기동 및 알림 통제 (`backup.sh run`)

`cmd_run`은 아래의 동작 순서대로 파이프라인을 통제합니다:

1. **1차 디렉터리 준비**:
   - `RESTIC_REPOSITORY`가 로컬 경로(예: `/data/backup`)일 경우, 디렉터리 존재 유무를 점검하고 부재 시 `mkdir -p` 및 `chmod 700`을 강제합니다.
2. **1차 로컬 백업 수행**:
   - `resticprofile -c /etc/restic/profiles.yaml -p local-backup backup` 기동.
3. **2차 원격 소산 백업 (`restic copy`) 수행**:
   - 1차 백업 성공 시, 2차 저장소로 복제 명령을 실행합니다.
   - `restic copy` 직접 실행 혹은 `resticprofile`을 이용한 데이터 이관.
   - 예: `restic -r /data/backup copy --repo2 s3:https://... --password-file2 <(echo $SECONDARY_RESTIC_PASSWORD)`
4. **로컬 및 원격 Forget/Prune 분리**:
   - 로컬은 백업 시 자동으로 `retention`에 의해 `prune`이 연쇄 실행됩니다.
   - 2차 원격지는 매 실행 시마다 `prune`을 돌리지 않고, 주 1회 또는 특정 일요일에만 `resticprofile -p secondary-backup forget --prune`을 가동하도록 조건 분기합니다.
5. **최종 통합 알림 발송**:
   - 1차 백업의 소요시간/성공여부 및 2차 소산의 결과/에러 메시지를 모아서 단 하나의 Slack/Discord/Custom webhook 알림으로 발송합니다.

### 3.5. 기존 설정 업그레이드 지원 (`backup.sh upgrade-config`)

이전 버전의 단일 백업 설정을 마이그레이션하기 위해 `upgrade-config` 서브커맨드를 지원합니다.
* **동작 흐름**:
  1. 기존 `/etc/restic/backup.env` 파일을 백업합니다 (`backup.env.bak`).
  2. 기존 파일에서 원격지 정보(S3 엔드포인트/버킷, SFTP 접속 정보 등)를 읽어옵니다.
  3. 읽어온 원격지 변수명들에 `SECONDARY_` 접두사를 붙여 2차 변수로 리매핑합니다.
  4. 1차 백업 경로를 `/data/backup`으로 신규 초기화하고, 비밀번호는 기존 비밀번호를 그대로 상속합니다.
  5. 새로운 `backup.env`를 생성(권한 `600`)한 뒤, `backup.sh config`를 강제 구동하여 `profiles.yaml`과 systemd 스케줄을 자동으로 업데이트합니다.

---

## 4. ISMS/ISMS-P 인증 증적 준수 계획

* **접근 제어**: 1차 로컬 저장소 `/data/backup`에 대해 파일 권한 `700`을 강제화하여, 루트 외에 백업본을 무단 탈취하지 못하게 차단합니다 (ISMS 3.3.1).
* **소산 입증**: 감사 보고서(`backup.sh audit`)에 로컬 스냅샷 목록 외에 2차 원격 스냅샷 복사 로그 및 무결성 정합성 여부를 동시 렌더링하여 심사 시 백업 소산 증적을 완전히 통과합니다 (ISMS 2.9.3).
* **양방향 재해복구 훈련**: 복구 모의훈련 시 S3/SFTP 원격지에서도 정기적으로 임의 파일을 복구 테스트하여, 재해 시 원격 본이 가용함을 실질적으로 입증합니다 (ISMS 2.12.3).

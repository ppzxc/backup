# 1차 원격 및 2차 원격 소산 백업 파이프라인 설계 사양서 (개정본)
- **작성일**: 2026-07-14
- **상태**: 승인됨 (Approved)

---

## 1. 개요 (Overview)

본 설계 사양서는 `backup.sh` 스크립트를 확장하여 기존의 단일 대상 백업 모델에서 **1차 원격 백업(S3/SFTP) 및 2차 원격 소산 백업(S3/SFTP)**을 통합 파이프라인으로 수행하도록 고도화하는 설계를 담고 있습니다.

원본 서버의 디스크 공간을 백업 용도로 점유하지 않는 장점을 살리면서, 1차 백업이 완료된 후 원본 서버가 직접 1차와 2차 원격 저장소 간에 `restic copy`를 수행함으로써 데이터 소산(이중 백업 및 격리)을 안전하게 달성하고 정보보호 관리체계(ISMS/ISMS-P) 심사 기준을 완전히 충족시킵니다.

---

## 2. 핵심 요구사항 및 설계 원칙

1. **원본 서버 단일 통제 및 실행 (시나리오 A-Remote)**
   - 백업 및 소산의 모든 과정은 원본 데이터를 보유한 서버의 `backup.sh` 스크립트와 `systemd timer` 스케줄러에 의해 단일 통제됩니다.
   - 1차 및 2차 원격지(S3, SFTP NAS 등)에는 Restic을 기동하거나 설치할 필요가 없으며, 원본 서버가 주체가 되어 원격지 간 데이터 복사(`restic copy`)를 중계 처리합니다.
2. **자유로운 프로콜 조합 (S3 / SFTP 자유 혼용)**
   - 1차와 2차의 종류에 상관없이 모든 조합을 지원합니다.
   - 예: 1차 SFTP(사내 고속 NAS) ➡️ 2차 S3(외부 퍼블릭 클라우드), 1차 S3 ➡️ 2차 S3 등.
3. **2차 소산 명확화 (secondary- 및 SECONDARY_ 접두사)**
   - 2차 백업 대상 설정 시 CLI 플래그 및 환경 변수에 `secondary-` 및 `SECONDARY_` 접두사를 풀네임으로 기재하여 1차 원격 설정과 확실하게 구분합니다.
4. **패스워드 상속 및 분리**
   - 기본적으로 1차 원격 저장소와 2차 원격 저장소의 암호화 패스워드는 동일하게 자동 공유(상속)되나, `--secondary-password` 지정 시 다르게 설정하여 암호를 분리할 수 있습니다.
5. **통합 결과 웹훅 알림**
   - 1차 원격 백업과 2차 소산 백업의 성공/실패 여부를 수집하여 전체 파이프라인 완료 시점에 **단 1회 최종 결과 통합 알림**만 발송합니다.
6. **2차 무결성 검증 및 유지보수 주기 최소화**
   - 1차 원격 저장소는 매 백업 직후 `prune` 및 `check`를 실행하여 가볍게 유지합니다.
   - 2차 원격 저장소는 네트워크 트래픽 및 시간 단축을 위해 평상시에는 `restic copy`만 수행하고, `prune` 및 `check`는 주 1회 또는 별도의 스케줄러로 한정하여 구동합니다.
7. **이중 복구 모의훈련 (Restore Drill)**
   - `--restore-drill` 검증 시, 1차 원격 저장소와 2차 원격 저장소 양쪽 모두에서 최근 스냅샷의 임의 파일을 각각 실제 다운로드 받아 복구 수행하고 성공 여부를 감사 보고서에 기록합니다.

---

## 3. 세부 설계 사양

### 3.1. CLI 플래그 확장 (`cmd_setting`)

`backup.sh setting` 명령어에 아래의 플래그들을 추가합니다:

* **1차 원격 기본 플래그** (기존 명칭 그대로 유지):
  * `--backend <s3|sftp>`
  * `--password <비밀번호>`
  * `--targets <경로,...>`
  * `--endpoint`, `--bucket`, `--access-key`, `--secret-key` (S3)
  * `--host`, `--user`, `--port` (SFTP)
* **2차 원격 소산 플래그 (`secondary-` 계열)**:
  * `--secondary-backend <s3|sftp>`: 2차 원격 저장소 타입 (S3 또는 SFTP)
  * `--secondary-password <암호>`: 2차 원격 저장소 비밀번호 (생략 시 `--password` 공유)
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
# 1차 원격 백업용 변수 (예: SFTP)
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/web-server"
export RESTIC_PASSWORD="primary_encryption_password"
export BACKUP_TARGETS="/var/log,/etc"
export BACKUP_EXCLUDES="/tmp"
export KEEP_DAILY=7
export KEEP_WEEKLY=4
export KEEP_MONTHLY=3
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="192.168.1.100"
export RCLONE_CONFIG_SYNO_BACKUP_USER="backup_user"

# 2차 원격 소산 백업용 변수 (예: S3)
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
*(참고: 1차 원격 프로필명을 명칭 일관성 유지를 위해 설정 파일의 기본 프로필명으로 기계 처리하되, 구조상 2차는 `-secondary` 접미사를 붙입니다.)*

```yaml
version: "1"

global:
  restic-lock-retry-after: 1m
  restic-stale-lock-age: 2h
  systemd-unit-template: /etc/restic/resticprofile-unit.template
  systemd-timer-template: /etc/restic/resticprofile-timer.template

# 1차 원격 백업 프로필 (예: SFTP)
web-server:
  repository: "rclone:syno_backup:/backup/web-server"
  force-inactive-lock: true
  env:
    RESTIC_PASSWORD: "primary_encryption_password"
    RCLONE_CONFIG_SYNO_BACKUP_TYPE: "sftp"
    RCLONE_CONFIG_SYNO_BACKUP_HOST: "192.168.1.100"
    RCLONE_CONFIG_SYNO_BACKUP_USER: "backup_user"
    HOSTNAME: "web-server"
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

# 2차 원격 소산 백업 프로필 (예: S3)
web-server-secondary:
  repository: "s3:https://s3.amazonaws.com/my-backup-bucket"
  force-inactive-lock: true
  env:
    RESTIC_PASSWORD: "secondary_encryption_password"
    AWS_ACCESS_KEY_ID: "AWSACCESSKEYID123"
    AWS_SECRET_ACCESS_KEY: "AWSSECRETACCESSKEY456"
    HOSTNAME: "web-server"
  retention:
    prune: true
    group-by: host
    keep-daily: 30
    keep-weekly: 12
    keep-monthly: 12
```

### 3.4. 파이프라인 기동 및 알림 통제 (`backup.sh run`)

`cmd_run`은 아래의 동작 순서대로 파이프라인을 통제합니다:

1. **1차 원격 백업 수행**:
   - `resticprofile -c /etc/restic/profiles.yaml -p [profile-name] backup` 기동.
2. **2차 원격 소산 백업 (`restic copy`) 수행**:
   - 1차 백업 성공 시, 원본 서버가 중계하여 2차 저장소로 복제 명령을 실행합니다.
   - `resticprofile -c /etc/restic/profiles.yaml -p [profile-name] copy --to [profile-name]-secondary` 기동.
3. **로컬 및 원격 Forget/Prune 분리**:
   - 1차 원격지는 백업 시 자동으로 `retention`에 의해 `prune`이 연쇄 실행됩니다.
   - 2차 원격지는 매 실행 시마다 `prune`을 돌리지 않고, 주 1회 또는 특정 일요일에만 `resticprofile -p [profile-name]-secondary forget --prune`을 가동하도록 조건 분기합니다.
4. **최종 통합 알림 발송**:
   - 1차 백업의 소요시간/성공여부 및 2차 소산의 결과/에러 메시지를 모아서 단 하나의 Slack/Discord/Custom webhook 알림으로 발송합니다.

### 3.5. 기존 설정 업그레이드 지원 (`backup.sh upgrade-config`)

이전 버전의 단일 백업 설정을 마이그레이션하기 위해 `upgrade-config` 서브커맨드를 지원합니다.
* **동작 흐름**:
  1. 기존 `/etc/restic/backup.env` 파일을 백업합니다 (`backup.env.bak`).
  2. 기존 파일에서 원격지 정보(S3 엔드포인트/버킷, SFTP 접속 정보 등)를 읽어옵니다.
  3. 읽어온 원격지 변수명들에 `SECONDARY_` 접두사를 붙여 2차 변수로 리매핑합니다.
  4. 1차 백업 경로는 기존에 사용하던 원격 저장소 정보를 그대로 상속하고, 2차 소산지 변수에는 새로 입력받거나 비워둔 채 추후 설정할 수 있도록 처리합니다. (또는 대화형으로 2차 소산지를 신규 입력받음)
  5. 새로운 `backup.env`를 생성(권한 `600`)한 뒤, `backup.sh config`를 강제 구동하여 `profiles.yaml`과 systemd 스케줄을 자동으로 업데이트합니다.

---

## 4. 기타 서브커맨드 영향도 분석 및 대응 사양 (Impact Analysis)

이중 원격 백업 파이프라인 아키텍처 도입에 따른 `backup.sh` 내 각 서브커맨드의 영향도 분석과 대응 명세는 다음과 같습니다.

### 4.1. `init` (저장소 초기화)
* **영향도**: **높음** (2차 원격 저장소 유효성 및 최초 1회 초기화 필수)
* **대응 사양**:
  1. `backup.sh init` 구동 시, 기존 1차 원격 저장소의 연결성 확인 및 `init`을 순차 기동합니다.
  2. 2차 백업 설정(`SECONDARY_BACKEND`)이 존재할 경우, 2차 원격 저장소에 대해서도 연결 테스트(S3 버킷 유효성 또는 SFTP 세션 유효성)를 연속 수행합니다.
  3. 2차 원격지 저장소가 아직 초기화되지 않았다면 `restic init`을 2차용 자격 증명 환경으로 수행하여 이중 저장을 개시할 준비를 마칩니다.

### 4.2. `status` (백업 상태 확인)
* **영향도**: **높음** (2차 원격 소산지의 상태 가시화 필요)
* **대응 사양**:
  - `backup.sh status` 호출 시 화면에 두 개의 섹션으로 구분하여 상태를 보고합니다:
    1. **1차 저장소 상태**: 최근 1차 백업 성공 일시, 백업 스냅샷 이력, 로컬 권한 검증 결과 등.
    2. **2차 소산지 상태**: 2차 저장소로의 이관 완료 여부, 2차 원격지에 보관된 최근 스냅샷 목록, 이관 중단 락(stale lock) 여부 등.

### 4.3. `audit` (ISMS 컴플라이언스 및 복구 모의훈련)
* **영향도**: **매우 높음** (소산 백업 증적 자료의 완전성 및 양방향 복구 훈련)
* **대응 사양**:
  1. **감사 보고서 생성**: 1차 원격 저장소의 스냅샷 카운트와 2차 원격 저장소의 스냅샷 카운트를 비교하여 "동기화 일치도" 및 "소산 격리 준수 여부"를 보고서 텍스트/JSON/HTML에 상세 기록합니다.
  2. **모의훈련 (`--restore-drill`)**: 
     - 1차 원격 저장소에서 임의 스냅샷의 특정 1개 파일을 복구 테스트합니다.
     - 이어서 2차 원격 저장소에서도 해당 스냅샷(또는 2차의 최신 스냅샷)에서 동일 파일(또는 다른 파일)을 다운로드 받아 복구 테스트합니다.
     - 양쪽 저장소 모두 복구가 원활히 성공했음을 증적 HTML에 명시합니다.

### 4.4. `uninstall` (환경 정리)
* **영향도**: **보통**
* **대응 사양**:
  - `backup.sh uninstall --purge` 수행 시, 1차 백업 타이머 뿐만 아니라 2차와 연관된 임시 캐시 및 설정 파일, rclone 소산용 접속 프로필을 흔적 없이 안전하게 삭제하도록 연동합니다.

### 4.5. `migrate` (기존 데이터 다른 원격지로 이관)
* **영향도**: **보통**
* **대응 사양**:
  - 이관은 기본적으로 1차 원격 저장소를 신규 저장소로 교체할 때 구동됩니다.
  - 다중 원격 상태에서는 `migrate` 대상이 1차인지 2차(소산지)인지 헷갈릴 수 있으므로, CLI 도움말에 이관 명세를 1차 대상 이관용으로 한정하여 정리하거나, 2차 소산지 정보를 함께 마이그레이션 할 수 있도록 관련 매개변수를 추가 조율합니다.

---

## 5. ISMS/ISMS-P 인증 증적 준수 계획

* **소산 입증**: 감사 보고서(`backup.sh audit`)에 1차 원격 스냅샷 목록 외에 2차 원격 스냅샷 복사 로그 및 무결성 정합성 여부를 동시 렌더링하여 심사 시 백업 소산 증적을 완전히 통과합니다 (ISMS 2.9.3).
* **양방향 재해복구 훈련**: 복구 모의훈련 시 S3/SFTP 원격지 양쪽 저장소 모두에서 정기적으로 임의 파일을 각각 실제 다운로드 받아 복구 테스트하여, 재해 시 원격 본이 가용함을 실질적으로 입증합니다 (ISMS 2.12.3).

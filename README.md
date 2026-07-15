# Restic Backup Automation Pipeline (`backup.sh`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![ShellCheck](https://img.shields.io/badge/shellcheck-passing-brightgreen.svg)](https://github.com/koalaman/shellcheck)
[![Tested with Bats](https://img.shields.io/badge/tests-Bats-blue.svg)](https://github.com/bats-core/bats-core)

> **안전하고 규격화된 Linux 서버 백업 관리를 위한 단일 파일 셸 스크립트 기반 Restic 백업 파이프라인**

`backup.sh`는 systemd 기반의 Linux 서버(RHEL/Rocky Linux 등)에서 **Restic 백업 솔루션**을 손쉽게 설치, 설정, 운영 및 자동화할 수 있도록 지원하는 엔터프라이즈용 관리 도구입니다.

저장 백엔드로 **S3 호환 오브젝트 스토리지** 또는 **SFTP/NAS 스토리지**를 지원하며, 백업 정책 설정부터 systemd 스케줄러 등록, ISMS 감사 증적 리포트 생성까지 백업 파이프라인의 전 과정을 단 하나의 스크립트로 제어합니다.

---

## 🌟 주요 특징 (Key Features)

* **무의존성 자동 바이너리 설치**: 외부 패키지 저장소(EPEL 등)에 의존하지 않고, 검증된 버전의 `restic`, `rclone`, `resticprofile` 바이너리를 GitHub Release에서 **SHA-256 체크섬 검증** 후 안전하게 다운로드하여 설치합니다.
* **강력한 스케줄러 위임**: 백업 제어, 백업 주기적 보존 정책(Forget/Prune), stale lock 관리 등 핵심 오케스트레이션은 데몬형 도구인 `resticprofile` 및 systemd timer로 위임하여 안정성을 극대화합니다.
* **대화형 설정 마법사 (Wizard)**: 백업이 처음인 관리자도 `backup.sh wizard` 명령을 통해 손쉽게 정책 수립 및 자격 증명 설정을 진행할 수 있습니다.
* **ISMS/ISO 27001 컴플라이언스 감사 지원**: `audit` 서브커맨드를 통해 보안 감사 요건에 부합하는 리포트를 터미널에 아름다운 트리 형태(ANSI Color)로 출력하며, 자동화 수집 도구(SIEM 등)를 위해 텍스트(`txt`/`md`)와 구조화된 `JSON` 파일로 동시 저장 기능을 지원합니다.
* **강력한 접근 제어**: 환경 설정 디렉터리(`/etc/restic` - `700`) 및 백업 자격 증명 환경 변수 파일(`backup.env` - `600`)의 권한을 엄격히 통제합니다.

---

## 🛠️ 핵심 아키텍처 및 철학 (Architecture)

본 도구는 **Functional Core / Imperative Shell** 디자인 패턴에 근거해 작성되어 있습니다.
* **Functional Core**: 자격 증명 해석(`resolve_value`), 입력 값 유효성 체크(`validate_*`), 설정 파일/systemd 유닛 템플릿 렌더링(`render_*`), CLI 옵션 파서(`parse_long_opts`) 등의 핵심 유틸리티는 외부 시스템(파일 쓰기, 외부 프로세스 호출 등)에 영향을 받지 않는 **순수 함수**로 격리되어 있어 `bats` 프레임워크를 통해 철저하게 검증됩니다.
* **Imperative Shell**: 실질적인 시스템 반영 행위(`cmd_*` 서브커맨드 함수군, 파일 쓰기, 패키지 다운로드)는 격리된 얇은 래퍼 함수 레이어를 통해 실행됩니다.

---

## 🚀 빠른 시작 (Quick Start)

### 1. 설치 및 의존성 다운로드
`backup.sh` 스크립트를 다운로드한 뒤 root 권한으로 실행하여 도구 및 필요한 백업 바이너리(Restic, Rclone, Resticprofile)를 자동 설치합니다.
```bash
$ sudo chmod +x backup.sh
$ sudo ./backup.sh install
```

### 2. 백업 설정 구성 (대화형 마법사)
가장 쉬운 설정 방법은 대화형 마법사를 시작하는 것입니다. 백엔드 유형(S3 / SFTP), 엔드포인트 정보, 백업 주기 및 보존 기준(keep-daily 등)을 질문합니다.
```bash
$ sudo ./backup.sh wizard
```

*또는 CLI 명령을 통해 수동으로 설정할 수 있습니다:*
```bash
# SFTP (Synology NAS 등) 백엔드 설정
$ sudo ./backup.sh setting --backend sftp --host 192.168.1.100 --user backup_user --password 'your-repo-password'

# S3 compatible (AWS, MinIO 등) 백엔드 설정
$ sudo ./backup.sh setting --backend s3 --endpoint https://s3.amazonaws.com --bucket my-backup-bucket --access-key ACCESS_KEY --secret-key SECRET_KEY --password 'your-repo-password'
```

### 3. 백업 저장소 초기화 (Init)
설정이 완료되면 Restic 저장소를 최초 1회 초기화합니다. SFTP 백엔드의 경우, 실제 SFTP 인증 및 접속 점검이 선제적으로 진행됩니다.
```bash
$ sudo ./backup.sh init
```

### 4. 스케줄러 등록
systemd 타이머를 통해 매일 새벽에 자동으로 백업이 실행되도록 스케줄을 활성화합니다.
```bash
$ sudo ./backup.sh schedule enable --on-calendar "*-*-* 02:00:00"
```

### 5. 수동 백업 테스트
필요한 경우 백업을 수동으로 즉시 실행할 수 있습니다.
```bash
$ sudo ./backup.sh run
```

---

## 📖 서브커맨드 레퍼런스 (Subcommands)

스크립트는 다음과 같은 서브커맨드 인터페이스를 제공합니다:

| 명령어 | 설명 | 예시 |
| :--- | :--- | :--- |
| **`install`** | `restic`, `rclone`, `resticprofile` 바이너리를 설치하고 스크립트 복사 | `backup.sh install` |
| **`setting`** | CLI 플래그 및 환경 변수를 해석해 백업 자격 증명(`backup.env`) 파일 생성 | `backup.sh setting --backend s3 [옵션]` |
| **`init`** | 백엔드 유효성 및 원격지 연결 상태 확인 후 restic 저장소 초기화 | `backup.sh init` |
| **`schedule`**| systemd 타이머를 생성·활성화(`enable`) 하거나 비활성화(`disable`) | `backup.sh schedule enable` |
| **`run`** | 백업 정책을 읽어 resticprofile에 실행을 위임하고 백업을 수행 | `backup.sh run` |
| **`status`** | 최근 백업 스냅샷 이력 및 설정 권한 정보를 간결하게 검증 | `backup.sh status` |
| **`audit`** | 컴플라이언스 규정에 부합하는 상세 리포트를 화면에 출력하고 보고서 파일 동시 저장 지원 | `backup.sh audit --report` |
| **`uninstall`**| 생성된 스케줄을 취소하고, `--purge` 추가 시 관련 환경 설정 및 캐시 영구 삭제 | `backup.sh uninstall --purge` |
| **`migrate`**  | 기존 백업 데이터를 신규 백엔드로 이전하고, 호스트 서버 설정 및 스케줄러를 완전 전환 | `backup.sh migrate` |
| **`wizard`** | 초보자를 위해 백업 전 과정을 단계별 대화형으로 세팅해 주는 마법사 | `backup.sh wizard` |

### 📑 `audit` 상세 옵션
* `--report`: 기본 표준 디렉터리(`/var/log/restic-backup/`) 하위에 인간 가독용 텍스트 보고서(`audit_report.txt`)와 기계 가독용 JSON 보고서(`audit_report.json`)를 동시 저장합니다.
* `--report-file <경로>`: 지정된 파일명으로 텍스트 보고서를 저장하며, 동등한 위치에 확장자만 `.json`으로 변환된 JSON 보고서를 함께 자동 생성합니다.

---

## 🔔 백업 성공/실패 알림 설정 (Notifications)

백업 완료 시 Slack, Discord 또는 커스텀 웹훅으로 백업 성공/실패 알림을 전송할 수 있습니다. 알림 설정은 `/etc/restic/backup.env` 파일에 정의하며, `backup.sh config` 또는 `backup.sh setting` 명령 시 CLI 플래그로도 지정할 수 있습니다.

### 1. 주요 설정 변수 (`backup.env`)

| 환경 변수 | CLI 플래그 | 설명 | 허용값 |
| :--- | :--- | :--- | :--- |
| **`BACKUP_NOTIFICATION_URL`** | `--notification-url` | 알림을 전송할 웹훅 URL 주소 | `http://` 또는 `https://` 로 시작하는 URL |
| **`BACKUP_NOTIFICATION_TYPE`** | `--notification-type` | 웹훅 대상 메신저/알림 서비스 타입 | `slack`, `discord`, `custom` |
| **`BACKUP_NOTIFICATION_ON`** | `--notification-on` | 알림을 전송할 트리거 조건 | `both` (기본값), `failure` (실패만), `success` (성공만) |
| **`BACKUP_NOTIFICATION_METHOD`** | - | `custom` 타입 웹훅 시 사용할 HTTP 메소드 | `POST` (기본값), `PUT` 등 |
| **`BACKUP_NOTIFICATION_HEADERS`**| - | `custom` 타입 웹훅 시 전송할 HTTP 헤더 목록 (쉼표 구분) | 예: `Content-Type: application/json, X-Token: secret` |
| **`BACKUP_NOTIFICATION_BODY_SUCCESS`** | - | `custom` 타입 백업 성공 시 전송할 JSON 페이로드 구조 | 예: `'{"status":"ok","msg":"백업 성공"}'` |
| **`BACKUP_NOTIFICATION_BODY_FAILURE`** | - | `custom` 타입 백업 실패 시 전송할 JSON 페이로드 구조 | 예: `'{"status":"error","msg":"${ERROR}"}'` |

---

### 2. 알림 웹훅 구성 예시

#### A. Slack 알림 설정
Slack 웹훅 URL로 백업 성공 및 실패 알림을 모두 받습니다. (페이로드 구성 자동)
```bash
export BACKUP_NOTIFICATION_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
export BACKUP_NOTIFICATION_TYPE="slack"
export BACKUP_NOTIFICATION_ON="both"
```

#### B. Discord 알림 설정
Discord 웹훅 URL로 백업 실패 알림만 받아봅니다. (페이로드 구성 자동)
```bash
export BACKUP_NOTIFICATION_URL="https://discord.com/api/webhooks/YOUR/WEBHOOK/URL"
export BACKUP_NOTIFICATION_TYPE="discord"
export BACKUP_NOTIFICATION_ON="failure"
```

#### C. 사내 시스템 또는 MS Teams 연동 (`custom` 타입)
HTTP Header 및 JSON Body 구조를 완전히 마음대로 설정하여 연동합니다.
```bash
export BACKUP_NOTIFICATION_URL="https://my-monitoring.internal/alerts"
export BACKUP_NOTIFICATION_TYPE="custom"
export BACKUP_NOTIFICATION_ON="both"
export BACKUP_NOTIFICATION_METHOD="POST"
export BACKUP_NOTIFICATION_HEADERS="Content-Type: application/json, Authorization: Bearer TOKEN123"
export BACKUP_NOTIFICATION_BODY_SUCCESS='{"event":"backup_ok","profile":"${PROFILE_NAME}"}'
export BACKUP_NOTIFICATION_BODY_FAILURE='{"event":"backup_fail","profile":"${PROFILE_NAME}","err":"${ERROR}"}'
```

---

### 3. 알림 웹훅에서 지원하는 변수 치환 문구

웹훅 본문(JSON)이나 URL에는 다음 변수를 포함할 수 있으며, `resticprofile`이 런타임에 이를 실제 값으로 동적 치환하여 발송합니다:

* **`${PROFILE_NAME}`**: 현재 작동 중인 백업 프로파일 명칭 (예: `web01`)
* **`${PROFILE_COMMAND}`**: 실행된 명령어 (예: `backup`, `check` 등)
* **`${HOSTNAME}`**: 시스템 호스트명 (스크립트가 `profiles.yaml` 내에 자동 주입)
* **`${ERROR}`**: 백업 최종 실패 시 발생한 오류 메시지 (실패 훅에서만 사용 가능)
* **`${ERROR_COMMANDLINE}`**: 백업 실패 시 실행되었던 restic 명령어 (실패 훅에서만 사용 가능)
* **`${ERROR_EXIT_CODE}`**: 백업 실패 시 종료 코드 (실패 훅에서만 사용 가능)
* **`${ERROR_STDERR}`**: 백업 실패 시 표준 에러(Stderr) 출력 내용 (실패 훅에서만 사용 가능)

---

### 4. 알림 설정 방법 (How to Configure)

알림 웹훅 설정은 다음 세 가지 방법 중 하나를 선택해 구성할 수 있습니다:

#### A. 기존 설정 파일 직접 수정 (추천)
이미 백업이 구성되어 작동 중인 환경에서 가장 간단하게 알림을 연동하는 방법입니다.

1. root 권한으로 백업 설정 파일(`/etc/restic/backup.env`)을 엽니다.
   ```bash
   $ sudo vi /etc/restic/backup.env
   ```
2. 최하단으로 이동해 알림 관련 변수 주석을 해제하고 설정 정보를 입력합니다. (예: Slack 연동)
   ```bash
   export BACKUP_NOTIFICATION_URL='https://hooks.slack.com/services/YOUR/WEBHOOK/URL'
   export BACKUP_NOTIFICATION_TYPE='slack'
   export BACKUP_NOTIFICATION_ON='both'
   ```
3. 저장을 완료한 뒤, 변경된 내용을 systemd 및 resticprofile 구성 파일에 반영하기 위해 설정 리로드를 실행합니다.
   ```bash
   $ sudo backup.sh config
   ```

#### B. CLI 명령어로 부분 수정 (`config`)
설정 파일을 직접 편집하지 않고, CLI 명령 플래그를 이용해 바로 설정을 반영 및 동기화할 수 있습니다.
```bash
$ sudo backup.sh config \
    --notification-url "https://hooks.slack.com/services/YOUR/WEBHOOK/URL" \
    --notification-type "slack" \
    --notification-on "both"
```

#### C. 초기 백업 환경 등록 시 함께 지정 (`setting`)
서버 최초 구성 시점부터 알림 웹훅을 함께 묶어 일괄 등록할 수 있습니다.
```bash
$ sudo backup.sh setting \
    --backend sftp \
    --host 192.168.1.100 \
    --user backupuser \
    --password 'your-repo-password' \
    --targets /etc,/var/log \
    --notification-url "https://hooks.slack.com/services/YOUR/WEBHOOK/URL" \
    --notification-type "slack"
```

---

## 🗄️ 데이터베이스(DB) 백업 연동 및 실증 (Database Backup & Audit)

`backup.sh`는 파일 시스템 백업뿐만 아니라 **상용 데이터베이스(MySQL, MariaDB, PostgreSQL 등)의 온라인 스트리밍 백업**을 자체 지원합니다. 덤프된 데이터를 로컬 디스크에 임시 파일로 쓰지 않고 메모리 상에서 암호화 파이프라인으로 전송(Streaming Stdin Backup)하므로 디스크 I/O를 크게 절약하고 보안 유출 사고를 방지합니다.

### 1. DB 백업 파이프라인 구성 플래그

`setting` 명령어 실행 시 데이터베이스 관련 다음 옵션들을 추가하여 백업을 구성합니다.

| CLI 플래그 | 설명 | 설정 예시 |
| :--- | :--- | :--- |
| **`--db-type`** | 백업할 데이터베이스 종류 | `mysql`, `mariadb`, `postgres`, `custom` |
| **`--db-command`** | 표준 출력(`stdout`)으로 백업 데이터를 내보내는 덤프 명령어 | `"mysqldump -h host ..."` (전체 쌍따옴표 묶음 필수) |
| **`--db-filename`** | 백업 저장소 내에 저장할 가상의 SQL/덤프 파일명 | `db-dump.sql` (기본값) |
| **`--db-keep-daily`** | DB 백업본 보존 정책 (일간) | `7` |
| **`--db-keep-weekly`** | DB 백업본 보존 정책 (주간) | `4` |
| **`--db-keep-monthly`** | DB 백업본 보존 정책 (월간) | `12` |
| **`--db-schedule`** | DB 백업 스케줄 주기 (systemd 타이머 규격) | `"*-*-* 03:00:00"` (기본값: 파일 백업 주기와 동일) |

---

### 2. 주요 데이터베이스별 백업 설정 예시

#### A. MySQL / MariaDB 백업
`mysqldump` 또는 `mariadb-dump` 명령어를 이용하여 데이터베이스 전체를 백업 저장소로 전송합니다.
```bash
$ sudo ./backup.sh setting \
    --backend s3 \
    --endpoint http://minio:9000 \
    --bucket my-backup-bucket \
    --password 'repo-password' \
    --db-type mariadb \
    --db-command "mariadb-dump -h mariadb-server -u root -p'db-password' --all-databases --single-transaction --quick --order-by-primary" \
    --db-filename "db-dump.sql"
```

#### B. PostgreSQL 백업 (`pg_dumpall` 또는 `pg_dump`)
비밀번호 보안 유출을 차단하기 위해 `env PGPASSWORD` 환경 변수를 백업 명령 실행 구간에 바인딩하여 백업을 수행합니다.
```bash
$ sudo ./backup.sh setting \
    --backend s3 \
    --endpoint http://minio:9000 \
    --bucket my-backup-bucket \
    --password 'repo-password' \
    --db-type postgres \
    --db-command "env PGPASSWORD='db-password' pg_dumpall -h postgres-server -U postgres" \
    --db-filename "pg-dump.sql"
```

#### C. 커스텀 파일/데이터 백업 (`custom`)
데이터베이스가 아니더라도 특정 디렉터리를 실시간 압축(`tar`)하여 스트리밍 저장하고자 할 때 유용하게 활용 가능합니다.
```bash
$ sudo ./backup.sh setting \
    --backend s3 \
    --endpoint http://minio:9000 \
    --bucket my-backup-bucket \
    --password 'repo-password' \
    --db-type custom \
    --db-command "tar -czf - /var/lib/my-custom-app" \
    --db-filename "app-data.tar.gz"
```

---

### 3. 복구 모의 훈련 및 무결성 정합성 검증 (`audit`)

ISMS/ISO 감사 요건 준수 및 백업 가용성 실증을 위해 **복구 훈련 서브커맨드**(`audit --restore-drill`)를 실행할 때, 실제 서비스 중인 데이터베이스에 데이터를 임포트(Import)하여 덮어씌우는 위험한 작업 대신 아래와 같이 **안전한 격리 검증**을 자동 수행합니다.

1. **임시 복구**: 백업 저장소에 저장된 DB 스냅샷을 호스트의 임시 디렉터리(`/tmp/restore_drill_db/...`)로 복원합니다.
2. **헤더 마커 검증**: 복원된 SQL 파일의 첫 10줄을 분석하여 아래와 같은 데이터베이스 고유의 덤프 파일 버전 시그니처 정보가 온전히 존재하는지 검사합니다.
   - **`mysql` / `mariadb`**: 헤더 내부 `MySQL dump` 또는 `MariaDB dump` 패턴 매칭
   - **`postgres`**: 헤더 내부 `PostgreSQL database dump` 또는 `PostgreSQL database cluster dump` 패턴 매칭
   - **`custom`**: 백업된 파일의 존재 유무 및 비어있지 않은 파일 크기 검증
3. **안전한 정리**: 검증 완료 즉시 복구 훈련용으로 활용한 임시 데이터를 삭제하여 유출을 차단하고 복구 성공 여부를 감사 보고서(MD/JSON/HTML)에 투명하게 기록합니다.

---

## 🛡️ 정보보호 관리체계 (ISMS / ISMS-P) 컴플라이언스 대응

본 스크립트는 **정보보호 관리체계(ISMS)**와 개인정보 보호 요건이 통합된 **정보보호 및 개인정보보호 관리체계(ISMS-P)** 심사 기준의 백업 통제 요건을 모두 충족하도록 각기 다르게 설계되었습니다. 두 인증 체계의 통제 기준과 본 스크립트의 기능 매핑은 다음과 같습니다.

---

### 1. ISMS (정보보호 관리체계) 대응 방식
**핵심 목표**: 시스템 인프라 및 운영 데이터의 **가용성(Availability)**과 **무결성(Integrity)** 보증, 접근 통제.

| 통제 항목 | 인증 요구사항 및 심사 기준 | 본 스크립트 대응 기능 & 증적 자료 |
| :--- | :--- | :--- |
| **2.9.3 백업 및 복구관리** | **정기 백업 및 스케줄러 가동**<br>주요 정보시스템(OS 설정, 서비스 로그) 백업 자동화 | • `/etc` 및 `/var/log` 기본 강제 백업<br>• systemd timer 자동 스케줄링 가동 (`schedule enable`) |
| **2.9.3 백업 및 복구관리** | **백업 환경 접근 통제**<br>백업 파일 및 자격증명 파일에 대한 권한 통제 | • 설정 디렉터리(`700`) 및 `backup.env` 자격증명 파일(`600`) 로컬 권한 강제 및 위반 탐지 (`status` 검증) |
| **2.9.3 백업 및 복구관리** | **보존 주기 설정 (Keep Policy)**<br>스냅샷 보존 주기를 설정하고 안전한 보관 | • `keep-daily`(7일), `keep-weekly`(4주), `keep-monthly`(12개월) 법적 보존 주기 일괄 세팅 및 정기 Prune 파이프라인 가동 |
| **2.12.3 재해 복구** | **복구 모의훈련 수행 및 RTO 검증**<br>최소 연 1회 재해 시나리오 기반 복구 훈련 수행 및 RTO 달성 검증 | • `backup.sh audit --restore-drill --report`를 통한 실제 파일 복구 및 SQL 쿼리 정합성 자동화 테스트<br>• **`restore_drill_report_YYYYMMDD.html`** 공식 보고서 확보 |

---

### 2. ISMS-P (정보보호 및 개인정보보호 관리체계) 대응 방식
**핵심 목표**: ISMS 요건 + **개인정보의 기밀성(Confidentiality)** 확보 및 **최소주의(Minimization)/파기 정책** 준수.

| 통제 항목 | 인증 요구사항 및 심사 기준 | 본 스크립트 대응 기능 & 증적 자료 |
| :--- | :--- | :--- |
| **3.3.4 개인정보 암호화** | **저장 및 전송 암호화**<br>백업 저장소 및 백업 전송 구간에서의 암호화 | • Restic 엔진의 클라이언트 사이드 AES-256 저장소 자체 암호화 적용<br>• HTTPS(S3 엔드포인트) 및 SSH/SFTP(NAS) 전송 구간 암호화 |
| **3.4.1 개인정보의 파기** | **개인정보 보존 기간 경과 시 파기**<br>백업본 내 보존 기간이 만료된 개인정보의 즉시 파기 | • `restic forget --prune` 파이프라인 자동 연동으로 보존 만료 스냅샷 내의 개인정보 블록을 물리적 저장소 수준에서 완전 소거 |
| **3.3.1 접근 권한 제한**<br>**3.1.2 기밀성 유지** | **비인가 백업 데이터 유출 통제**<br>개인정보 백업본에 대한 비인가자 다운로드/접근 방지 | • 백업 자격증명 노출을 막기 위해 systemd 유닛 파일 내부에서 비밀번호 블록 배제 및 마스킹 처리<br>• `/etc/restic` 디렉터리 권한을 `700`으로 묶어 루트 외 유출 원천 차단 |
| **3.2.1 개인정보 수집 최소화** | **불필요한 개인정보 백업 배제**<br>불필요한 임시 파일이나 민감 정보가 백업본에 쌓이지 않도록 통제 | • `excludes_csv` 설정을 통한 임시/캐시 디렉터리(`/tmp`, `/var/tmp` 등) 백업 배제 필터링 구성 |

---

### 3. 다중 프로필(Multi-Profile) 기반 보관 주기 개별 격리 적용
일반적으로 보안 감사 로그(최소 1~2년 보존)와 데이터베이스 백업(개인정보 파기 정책에 의거 3개월 내 단기 순환)은 요구되는 보존 수명이 다릅니다. 하나의 호스트 서버 내에서 백업 정책을 독립적으로 이원화하여 기동시키려면 환경 변수를 재정의하는 방식으로 다중 프로필을 운용할 수 있습니다.

#### ① DB 백업 프로필 등록 (3개월 보존 주기)
```bash
$ sudo BACKUP_ENV_FILE=/etc/restic/db.env \
       RESTICPROFILE_CONFIG_FILE=/etc/restic/db_profiles.yaml \
       backup.sh setting \
         --backend sftp \
         --host 192.168.1.100 \
         --user backupuser \
         --password 'your-repo-password' \
         --targets "/var/backup/db" \
         --keep-daily 7 \
         --keep-weekly 4 \
         --keep-monthly 3 \
         --profile-name "db-backup"
```

#### ② 로그 백업 프로필 등록 (2년/24개월 보존 주기)
```bash
$ sudo BACKUP_ENV_FILE=/etc/restic/log.env \
       RESTICPROFILE_CONFIG_FILE=/etc/restic/log_profiles.yaml \
       backup.sh setting \
         --backend sftp \
         --host 192.168.1.100 \
         --user backupuser \
         --password 'your-repo-password' \
         --targets "/var/log" \
         --keep-daily 7 \
         --keep-weekly 4 \
         --keep-monthly 24 \
         --profile-name "log-backup"
```

#### ③ 스케줄러 개별 활성화
```bash
# DB 백업 활성화 (매일 새벽 3시)
$ sudo BACKUP_ENV_FILE=/etc/restic/db.env \
       RESTICPROFILE_CONFIG_FILE=/etc/restic/db_profiles.yaml \
       backup.sh schedule enable --on-calendar "*-*-* 03:00:00"

# 로그 백업 활성화 (매일 새벽 4시)
$ sudo BACKUP_ENV_FILE=/etc/restic/log.env \
       RESTICPROFILE_CONFIG_FILE=/etc/restic/log_profiles.yaml \
       backup.sh schedule enable --on-calendar "*-*-* 04:00:00"
```
이렇게 설정하면 systemd timer 가 독립적으로 데몬에 등록되어(`restic-backup@db-backup.timer`, `restic-backup@log-backup.timer`), 각각 다른 주기와 경로로 백업 파이프라인이 자동 실행됩니다.

---

## 🔄 백엔드 마이그레이션 (`migrate`)

기존에 가동 중이던 백업 저장소(SFTP 또는 S3)의 모든 백업 데이터(전체 스냅샷 이력)를 새로운 목적지 저장소로 안전하게 복사하고, 현재 `backup.sh`가 돌아가고 있는 호스트 서버(클라이언트)의 백업 설정(`backup.env`), 프로필 및 `systemd` 스케줄러를 새 저장소 정보로 **완전히 전환**해 주는 고성능 마이그레이션 도구입니다.

### 🛡️ 주요 특징 및 안정성 장치
1. **사전 정합성 점검 (Pre-flight check)**: 마이그레이션 실행 전에 기존 설정의 유효성 검사 및 소스 저장소에 대한 인증/연결성 확인(`snapshots` 호출)을 선제적으로 진행합니다. 소스 접속 오류가 있으면 이관을 중단하고 기존 환경을 유지하여 안전하게 보호합니다.
2. **중복제거율(Deduplication) 100% 보존**: 신규 목적지 저장소가 아직 비어 있는 경우, 단순 신규 생성 대신 소스 저장소의 청커 파라미터를 그대로 복제하여 목적지를 초기화합니다 (`--copy-chunker-params` 계승).
3. **충돌 방지 이관 구조**: SFTP ➡️ SFTP, S3 ➡️ S3와 같이 동일한 프로토콜 간 마이그레이션 시 발생하는 환경변수/인증 정보의 충돌을 방지하기 위해 가상의 임시 rclone 환경변수(`RCLONE_CONFIG_SYNO_BACKUP_DST_*`)를 구성하여 안전하게 데이터 복사(`restic copy`)를 수행합니다.
4. **자동 무결성 검증**: 마이그레이션이 완료되는 즉시 대상 저장소에 대해 데이터 무결성 검증(`restic check`)을 수행하여 복사 중 발생한 손상이 없는지 진단합니다.
5. **호스트 스케줄러 자동 갱신**: 기존에 백업 스케줄(`systemd timer`)이 활성화되어 돌아가고 있던 서버의 경우, 설정 이관이 완료되면 새 목적지 정보를 반영하여 systemd 유닛들을 자동으로 재생성 및 활성화(Reload & Enable)해 줍니다.
6. **안전성 (원본 데이터 보존)**: 마이그레이션 성공 후에도 기존 저장소 내부의 옛날 백업 데이터들은 오작동 방지 및 혹시 모를 안전을 위해 자동으로 삭제하지 않으며, 이관이 완전히 끝난 후 수동 삭제할 수 있도록 가이드를 안내합니다.

### 💻 사용 방법 (CLI / Interactive)

#### A. S3 호환 저장소로 마이그레이션
```bash
$ sudo ./backup.sh migrate \
    --backend s3 \
    --endpoint https://new-s3.example.com \
    --bucket new-backup-bucket \
    --access-key NEW_ACCESS_KEY \
    --secret-key NEW_SECRET_KEY \
    [--new-password 새저장소암호] \
    [--skip-check]
```

#### B. SFTP 저장소로 마이그레이션
```bash
$ sudo ./backup.sh migrate \
    --backend sftp \
    --host 192.168.1.200 \
    --user new_backup_user \
    [--port 22] \
    [--new-password 새저장소암호] \
    [--skip-check]
```

#### C. 대화형 설정 마법사 (Interactive Wizard)
목적지 백엔드 옵션 플래그가 주어지지 않은 상태에서 터미널(TTY) 모드로 실행하면 대화형 프롬프트를 통해 마이그레이션할 대상 저장소 설정을 차례로 질문합니다.
```bash
$ sudo ./backup.sh migrate
```

* `--new-password`: 새로운 저장소에 지정할 비밀번호를 입력합니다. (생략 시 기존 저장소의 비밀번호를 기본 사용)
* `--skip-check`: 마이그레이션 완료 후 무결성 검사(`restic check`) 단계를 스킵합니다. (대량 데이터 전송 시 시간 단축용)

---

## 🧪 개발 및 테스트 (Development & Testing)

스크립트의 안정성을 유지하기 위해 강력한 테스트 환경을 갖추고 있습니다.

### 정적 분석 (Linting)
ShellCheck 도구를 이용해 구문 오류 및 포터빌리티 위반 사항을 점검합니다:
```bash
$ shellcheck backup.sh
```

### 단위 테스트 (Unit Tests)
`bats` 테스트 프레임워크를 기반으로 148개의 단위 테스트 케이스를 수행하여 뼈대 로직의 완전함을 무결하게 입증합니다:
```bash
# 전체 테스트 실행
$ bats tests/
```

### 통합 테스트 (Integration Tests)
Docker Compose를 사용하여 실제 가상 환경(Rockylinux 9 컨테이너 + MinIO 오브젝트 스토리지 + SFTP 원격지 서버)을 띄우고 백업 파이프라인 전 단계를 실제 가동하여 엔드투엔드로 검증합니다:
```bash
$ bats tests/integration/integration.bats
```

---

## 📄 라이선스 (License)

This project is licensed under the [MIT License](LICENSE).

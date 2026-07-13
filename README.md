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
| **`wizard`** | 초보자를 위해 백업 전 과정을 단계별 대화형으로 세팅해 주는 마법사 | `backup.sh wizard` |

### 📑 `audit` 상세 옵션
* `--report`: 기본 표준 디렉터리(`/var/log/restic-backup/`) 하위에 인간 가독용 텍스트 보고서(`audit_report.txt`)와 기계 가독용 JSON 보고서(`audit_report.json`)를 동시 저장합니다.
* `--report-file <경로>`: 지정된 파일명으로 텍스트 보고서를 저장하며, 동등한 위치에 확장자만 `.json`으로 변환된 JSON 보고서를 함께 자동 생성합니다.

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
$ cd tests/integration
$ ./run.sh
```

---

## 📄 라이선스 (License)

This project is licensed under the [MIT License](LICENSE).

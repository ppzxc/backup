# Restic Backup Automation Tool (`backup`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![Coverage](https://img.shields.io/badge/coverage-80%25%2B-brightgreen.svg)](#)
[![ISMS-P](https://img.shields.io/badge/compliance-ISMS--P-emerald.svg)](#)

> **안전하고 규격화된 Linux 서버 백업 관리를 위한 Rust 기반 Restic 백업 자동화 CLI 도구**

`backup`은 systemd 기반의 Linux 서버(RHEL, Rocky Linux, Ubuntu 등)에서 **Restic 백업 파이프라인**을 설치, 설정, 운영 및 자동화하고 ISMS-P 감사 증적 보고서를 생성하는 고성능 백업 관리 CLI 도구입니다.

저장 백엔드로 **SFTP/NAS 스토리지** 및 **S3 호환 오브젝트 스토리지**(AWS S3, MinIO, Synology Rclone 등)를 지원하며, DB 스트리밍 백업, 1차/2차 저장소 마이그레이션, systemd 스케줄러 동기화, 모의복구 훈련(RTO 측정)까지 백업의 전 과정을 제어합니다.

---

## 🌟 주요 특징 (Key Features)

* **고성능 & 안전한 Rust CLI**: 빠른 실행 속도, 강력한 타입 안정성 및 메모리 안전성 보장.
* **Functional Core / Imperative Shell 계층 구조**: 비즈니스 로직과 외부 I/O 레이어를 엄격히 분리하여 100% Mocking 및 단위 테스트 가능.
* **심층 아키텍처 (Deep Module Seams)**:
  * **Configuration Registry**: `/etc/backup` (`700`), `config.yml` / `backup.env` (`600`) POSIX 권한 자동 강제 및 `profiles.yaml` 파생 설정 동기화.
  * **Doctor Diagnostic Engine**: ISMS-P 인증 감사 규정 검증(보안 권한, NTP 시각 동기화, DB 헤더 검증 및 RTO 측정 HTML 보고서 생성).
  * **Pipeline Engine**: DB 덤프 스트리밍 -> 1차 백업 -> 2차 2차 백업 복제 -> 보관 주기(Retention) 정리 수동/자동 원스톱 파이프라인 수행.
* **보안 및 자격 증명 보호**: 비밀번호 및 Access Key 등 민감 자격 증명(`SecretString`) 마스킹 처리.

---

## 🏗️ 핵심 아키텍처 (Architecture)

본 프로젝트는 **Functional Core / Imperative Shell** 및 **Deep Module** 아키텍처 원칙을 준수합니다.

```
┌─────────────────────────────────────────────────────────┐
│                    backup CLI (main)                    │
└────────────────────────────┬────────────────────────────┘
                             │
     ┌───────────────────────┼───────────────────────┐
     ▼                       ▼                       ▼
┌──────────────┐    ┌─────────────────┐    ┌──────────────────┐
│ SetupEngine  │    │ ConfigRegistry  │    │  Doctor Engine   │
│ (Inquire/TUI)│    │ (0700/0600 Sync)│    │ (ISMS Reporting) │
└──────────────┘    └─────────────────┘    └──────────────────┘
                             │
                             ▼
                 ┌───────────────────────┐
                 │    PipelineEngine     │
                 └───────────┬───────────┘
                             │
                             ▼
                ┌─────────────────────────┐
                │ SystemExecutor (Runner) │
                └────────────┬────────────┘
                             │ (Command execution)
        ┌────────────────────┼────────────────────┐
        ▼                    ▼                    ▼
   [ restic ]         [ resticprofile ]       [ rclone ]
```

---

## 🚀 시작하기 (Quick Start)

### 1. 빌드 및 테스트
```bash
# 빌드
$ cargo build --release

# 전체 단위 및 통합 테스트 실행 (62개 테스트)
$ cargo test

# 시나리오 E2E 통합 테스트 실행
$ cargo test --test e2e_full_workflow
```

### 2. 코드 커버리지 및 뮤테이션 측정
```bash
# 커버리지 측정 (cargo-llvm-cov)
$ cargo llvm-cov

# 품질 측정 스크립트 실행 (커버리지 & cargo-mutants)
$ ./scripts/test_coverage.sh
```

---

## 📖 CLI 서브커맨드 명세 (Command Reference)

`backup` CLI 도구는 유비쿼터스 도메인 용어에 맞추어 설계된 서브커맨드를 제공합니다:

| 서브커맨드 | 주요 역할 및 설명 | 실행 예시 |
| :--- | :--- | :--- |
| **`setup`** | TUI 마법사로 환경/프로필 초기화, 바이너리 의존성 자동 설치, 저장소 초기화 | `backup setup`<br>`backup setup dependencies` |
| **`config`** | 설정값 조회 (비밀값 마스킹), 설정 파일 편집 및 레거시 Bash `backup.env` 이관 | `backup config show`<br>`backup config edit` |
| **`run`** | 전체 백업 파이프라인 수동/드라이런 실행 (DB -> Primary -> Secondary -> Retention) | `backup run`<br>`backup run --dry-run` |
| **`doctor`** | 권한, 시각 동기화(NTP), 모의복구 훈련(RTO) 진단 및 ISMS-P HTML 보고서 생성 | `backup doctor`<br>`backup doctor environment --file report.html` |
| **`backend`** | 저장소 간 2차 복사(`restic copy`) 및 신규 백엔드 마이그레이션 | `backup backend migrate` |
| **`schedule`**| Systemd Timer 자동 백업 스케줄 등록, 해제 및 현재 상태 조회 | `backup schedule enable`<br>`backup schedule status` |
| **`restore`** | 스냅샷 복원 및 DB 무결성 복구 실행 | `backup restore --snapshot latest` |
| **`snapshots`**| 1차/2차 저장소에 보관된 전체 스냅샷 목록 조회 | `backup snapshots` |
| **`status`** | 저장소 연결 상태 및 최근 백업 실행 결과 종합 조회 | `backup status` |
| **`uninstall`**| 자동 백업 스케줄 해제 및 바이너리/환경 정리 | `backup uninstall --purge` |

---

## 🧪 테스트 및 품질 검증 (Quality Assurance)

* **Unit & Command Tests**: `tests/cmd_*_test.rs`, `tests/config_test.rs`, `tests/runner_test.rs`
* **Logical & Security Tests**: `tests/logical_validation_test.rs`, `tests/isms_audit_test.rs`
* **Docker Testcontainers Matrix**: S3, SFTP, MySQL, PostgreSQL 백엔드에 대한 E2E 통합 테스트 (`tests/e2e_*_test.rs`)

---

## 📄 라이선스 (License)

This project is licensed under the [MIT License](LICENSE).

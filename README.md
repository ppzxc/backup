# Restic Backup Automation Tool (`backup`)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![Coverage](https://img.shields.io/badge/coverage-80%25%2B-brightgreen.svg)](#)

> **안전하고 규격화된 Linux 서버 백업 관리를 위한 Rust 기반 Restic 백업 자동화 CLI 도구**

`backup`은 systemd 기반의 Linux 서버(RHEL/Rocky Linux/Ubuntu 등)에서 **Restic 백업 솔루션**을 손쉽게 설치, 설정, 운영 및 자동화할 수 있도록 지원하는 고성능 백업 관리 CLI 도구입니다.

저장 백엔드로 **S3 호환 오브젝트 스토리지** 또는 **SFTP/NAS 스토리지**를 지원하며, 백업 정책 설정부터 systemd 스케줄러 등록, 헬스 체크, 마이그레이션까지 백업 파이프라인의 전 과정을 제어합니다.

---

## 🌟 주요 특징 (Key Features)

* **고성능 & 안전한 Rust CLI**: Rust 프로그래밍 언어로 개발되어 빠른 실행 속도와 강력한 타입 안정성, 메모리 안전성을 보장합니다.
* **Functional Core / Imperative Shell 아키텍처**: 순수 비즈니스 로직과 외부 I/O 레이어를 엄격히 분리하여 유닛 테스트 및 뮤테이션 테스트 가능성을 최상으로 유지합니다.
* **다양한 백엔드 지원**: SFTP 및 S3(AWS S3, MinIO 등) 백엔드 지원, 1차/2차 저장소 구성 가능.
* **systemd 스케줄러 연동**: systemd service 및 timer 유닛 자동 생성 및 스케줄링 관리.
* **보안 및 컴플라이언스**: 민감 자격 증명(`SecretString`)의 마스킹 처리, `/etc/backup` 디렉터리(`700`) 및 설정 파일(`600`)의 엄격한 POSIX 권한 강제.
* **테스트 및 검증 체계**: `cargo-llvm-cov` 기반 소스 코드 커버리지 측정 및 `cargo-mutants` 기반 결함 검수 능력(Mutation Testing) 측정 체계 구축.

---

## 🛠️ 핵심 아키텍처 (Architecture)

본 도구는 **Functional Core / Imperative Shell** 디자인 패턴에 근거해 작성되어 있습니다.
* **Functional Core**: `BackupConfig` 검증, 설정 파일 파싱, 디폴트 생성, `systemd` 유닛 렌더링, CLI 명령어 변환 등 외부 I/O가 없는 순수 함수로 구성되어 있습니다.
* **Imperative Shell**: 외부 명령 호출(`restic`, `rclone`, `systemctl`)은 `CommandRunner` / `Executor` Trait (`src/runner/executor.rs`)으로 추상화되어 테스트 시 `MockExecutor`로 원격 종속성을 완전히 차단하고 동작을 검증합니다.

---

## 🚀 시작하기 (Quick Start)

### 1. 빌드 및 테스트
```bash
# 빌드
$ cargo build --release

# 전체 단위 및 통합 테스트 실행
$ cargo test

# 특정 테스트 파일 실행
$ cargo test --test cmd_config_test
```

### 2. 코드 커버리지 및 퀄리티 측정
```bash
# 통합 커버리지 & 뮤테이션 측정 스크립트 실행
$ ./scripts/test_coverage.sh

# 개별 실행: 코드 커버리지 리포트 (cargo-llvm-cov)
$ cargo llvm-cov

# 개별 실행: 뮤테이션 테스트 (cargo-mutants)
$ cargo mutants --file src/config/model.rs --file src/runner/executor.rs
```

---

## 📖 서브커맨드 레퍼런스 (Subcommands)

`backup` CLI 도구는 다음과 같은 서브커맨드를 제공합니다:

| 명령어 | 설명 | 예시 |
| :--- | :--- | :--- |
| **`setup`** | 대화형 마법사 또는 기본 백업 설정 파일 생성 | `backup setup` |
| **`config`** | 현재 등록된 백업 설정 조회(마스킹 지원) 및 수정 | `backup config show` |
| **`run`** | 등록된 백업 프로필 실행 및 백업 수행 | `backup run` |
| **`restore`** | 스냅샷 조회 및 복원 수행 | `backup restore` |
| **`schedule`** | systemd service 및 timer 유닛 생성 및 등록 | `backup schedule enable` |
| **`doctor`** | 시스템 의존성 바이너리 및 저장소 연결 상태 점검 | `backup doctor` |
| **`uninstall`**| 생성된 스케줄 해제 및 설치 환경 정리 | `backup uninstall --yes` |

---

## 🧪 테스트 및 퀄리티 검증 (Testing & Quality Assurance)

프로젝트는 TDD(Test-Driven Development) 원칙을 준수하며 다음과 같은 검증 체계를 포함합니다:

* **단위 테스트 (Unit Tests)**: `tests/config_test.rs`, `tests/legacy_import_test.rs`, `tests/runner_test.rs` 등
* **서브커맨드 검증**: `tests/subcommand_test.rs`, `tests/cmd_*_test.rs`
* **Docker 컨테이너 통합 테스트**: `tests/integration_s3.rs`, `tests/integration_sftp.rs`, `tests/integration_db.rs`
* **시나리오 E2E 테스트**: `tests/integration_scenario.rs`
* **커버리지 리포트**: `./scripts/test_coverage.sh` 구동 시 `target/coverage/html/index.html`에 HTML 리포트 자동 생성

---

## 📄 라이선스 (License)

This project is licensed under the [MIT License](LICENSE).

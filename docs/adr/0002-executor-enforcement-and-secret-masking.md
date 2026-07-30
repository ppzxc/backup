# 0002. Enforce Executor Trait, Mask Secrets, and Complete Pipeline Implementation

## Status

Accepted

## Context

코드베이스 종합 리뷰 과정에서 다음 세 가지 주요 문제점이 식별되었습니다.

1. **외부 명령 호출 추상화 누락**: `setup.rs` (`ssh-keygen`) 및 `uninstall.rs` (`systemctl`) 등 일부 명령에서 `CommandRunner` / `Executor` Trait을 우회하고 `std::process::Command`를 직접 사용하여 테스트 환경 격리(Mocking) 규칙을 위반함.
2. **보안 권한 처리 및 민감 정보 마스킹**: `/etc/backup` 권한 설정 시 에러를 무시하는 코드(`let _ = set_permissions(...)`)와 `SecretString` 직렬화 시 평문 노출 가능성이 존재함.
3. **핵심 도메인 로직 미완성**: `restore.rs` 복구 로직 및 `run.rs` DB 백업 스트리밍 파이프라인이 실제 바이너리 실행 없이 스텁 메시지 출력에 머물러 있음.

## Decision

1. **Executor Trait 완전 강제**: 모든 외부 시스템 명령 호출(`ssh-keygen`, `systemctl`, `rm` 등)을 `Executor` Trait으로 이관하여 단위/통합 테스트 시 `MockExecutor`로 원격 종속성을 차단함.
2. **권한 오류 엄격 반환 및 SecretString 보호**: 디렉터리(`700`) 및 설정 파일(`600`) 권한 적용 실패 시 즉시 `Result::Err`를 반환하도록 변경하고, 디스플레이/출력용 마스킹 처리를 분리 강제함.
3. **파이프라인 및 복구 구현 완성**: `Database Backup Adapter`와 `ResticRunner`를 실제 `Executor` 인터페이스와 연결하여 `restore` 복구 흐름 및 DB 백업 스트리밍 파이프라인을 완성함.

## Consequences

- `setup.rs`, `uninstall.rs`, `restore.rs`, `run.rs`의 단위 테스트 시 외부 환경 의존성 없이 100% Mocking이 가능해짐.
- 권한 오류가 무시되지 않아 보안 컴플라이언스(ISMS-P 감사) 요구사항을 완벽히 충족함.
- `restore` 및 DB 백업 파이프라인의 실동작이 가능해져 E2E 및 시나리오 테스트의 실효성이 확보됨.

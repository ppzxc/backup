# 0004. Run container E2E in a dedicated CI job

## Status

Superseded by ADR-0005

## Decision

Docker와 외부 백업 도구가 필요한 Container E2E Matrix는 기본 `cargo test`에서 분리하고, Docker가 준비된 전용 CI 작업에서 반드시 실행한다. 기본 테스트는 임시 경로와 MockExecutor를 사용해 호스트 상태를 변경하지 않아야 한다. 이 경계는 빠르고 재현 가능한 개발 루프와 실제 S3/SFTP/DB 복구 검증을 함께 보장한다.

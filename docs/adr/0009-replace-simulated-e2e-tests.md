# 0009. Replace simulated E2E tests with one isolated matrix

## Status

Superseded by ADR-0014

## Decision

직접 파일을 써서 복원을 흉내 내거나 컨테이너 포트만 확인하던 기존 E2E 파일을 제거한다. `e2e_isolated_container_test.rs` 하나가 S3/SFTP 저장, 양방향 복사·복원, MariaDB/PostgreSQL Database Stream을 실제 CLI 경로로 순차 검증한다. 이름과 검증 대상이 일치하게 한다.

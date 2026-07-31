# 0007. Require Docker E2E prerequisites in `cargo test`

## Status

Accepted

## Decision

`cargo test`는 Docker daemon, 이미지 pull, 그리고 `docker/Dockerfile.e2e_runner` 기반 runner 이미지 빌드를 필수 전제조건으로 둔다. 누락 또는 실패를 skip이나 조건부 성공으로 바꾸지 않는다. 개발·CI 모두 실제 Backup Pipeline을 검증한다는 동일한 계약을 가진다.

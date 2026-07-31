# 0005. Run container E2E from the default test suite

## Status

Accepted

## Decision

`cargo test`는 Container E2E Matrix를 포함해 unit 및 integration 테스트 전체를 실행한다. Docker와 `restic`, `rclone`, DB 클라이언트는 호스트가 아닌 전용 runner 컨테이너에서 제공하고, 테스트 설정과 산출물은 임시 경로와 컨테이너 네트워크에 격리한다. 이 결정은 빠른 기본 루프보다 전체 백업 파이프라인의 지속적 검증을 우선한다.

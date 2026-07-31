# 0010. Separate CLI contracts from external lifecycle effects

## Status

Accepted

## Decision

CLI lifecycle tests는 공통 `--profiles` 경로 오버라이드와 임시 통합 설정을 사용해 비파괴적인 명령 계약만 검증한다. 의존성 설치, resticprofile 스케줄 등록, systemd 같은 외부 효과는 격리 runner의 Container E2E Matrix에서만 검증한다. 테스트가 호스트의 통합 설정을 생성·변경·삭제하지 않는다.

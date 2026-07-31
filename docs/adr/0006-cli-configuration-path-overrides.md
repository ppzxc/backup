# 0006. Make configuration paths explicit CLI inputs

## Status

Accepted

## Decision

모든 `backup` 서브커맨드는 공통 `--profiles <path>` 옵션으로 통합 resticprofile v2 및 애플리케이션 설정 파일 경로를 오버라이드할 수 있다. 기본 경로는 `/etc/backup/profiles.yaml`이다. 테스트와 운영 복구 절차가 동일한 명시적 CLI 경로를 사용하게 해 호스트 전역 설정 의존성을 제거한다.

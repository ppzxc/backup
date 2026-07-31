# 0012. Verify systemd scheduling in a privileged E2E runner

## Status

Superseded by ADR-0014

## Decision

Container E2E Matrix는 systemd를 PID 1로 실행하는 privileged runner를 사용하고 `/sys/fs/cgroup`을 쓰기 가능한 bind mount로 제공한다. systemd는 cgroup 계층을 생성·관리해야 하므로 read-only mount는 Docker Desktop/WSL 등 cgroup v2 호스트에서 PID 1을 `exit=255`로 종료시킨다. `backup schedule enable` 뒤 timer 존재와 활성 상태를, `disable` 뒤 timer 부재를 실제 `systemctl`로 검증한다. Docker 권한 또는 이 환경을 제공하지 못하면 기본 `cargo test`는 실패한다.

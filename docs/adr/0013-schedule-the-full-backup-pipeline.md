# 0013. Schedule the full Backup Pipeline

## Status

Accepted

## Decision

`backup setup`은 일별 스케줄러와 실행 리포트 보관을 자동으로 설정한다. 스케줄러는 resticprofile의 단일 `backup` 작업이 아니라 `backup run`을 호출하여 Primary Backend Adapter 백업, 활성화된 Secondary Backend Adapter 동기화, Retention, 성공·실패 실행 리포트 생성을 하나의 예약 백업 실행으로 완료한다. `backup schedule`은 자동 설정된 스케줄의 임의 등록·해제·상태 조회를 위한 관리 명령으로 남긴다. Setup Wizard 재실행은 새 설정·저장소 준비·스케줄 등록이 모두 성공한 뒤에만 기존 구성을 교체하며, 실패 시 기존 설정과 스케줄을 유지한다.

Systemd capability probe가 systemd unavailable을 확인한 경우에만 Cron fallback을 사용한다. 선택된 scheduler의 실제 등록 실패는 다른 scheduler로 조용히 전환하지 않고 command failure로 전파한다.

`backup schedule disable`은 Unified Backup Configuration의 존재와 독립적으로 실행할 수 있으며, Backup CLI가 소유한 systemd timer/service와 Cron marker를 모두 탐지해 멱등적으로 제거한다. 대상이 없으면 성공으로 보고하고, 제거 실패는 다른 scheduler 등록으로 전환하지 않고 command failure로 전파한다. 다른 애플리케이션의 scheduler 항목은 변경하지 않는다.

`backup schedule status`도 설정 파일과 독립적으로 동작하며, capability probe가 선택한 scheduler 하나만 조회한다. active, inactive, scheduler 없음은 조회 성공이므로 exit code `0`이고, capability probe 또는 선택된 scheduler 조회 실패는 inactive로 변환하지 않고 exit code `1`이다.

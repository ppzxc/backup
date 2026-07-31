# 0013. Schedule the full Backup Pipeline

## Status

Accepted

## Decision

`backup setup`은 일별 스케줄러와 실행 리포트 보관을 자동으로 설정한다. 스케줄러는 resticprofile의 단일 `backup` 작업이 아니라 `backup run`을 호출하여 Primary Backend Adapter 백업, 활성화된 Secondary Backend Adapter 동기화, Retention, 성공·실패 실행 리포트 생성을 하나의 예약 백업 실행으로 완료한다. `backup schedule`은 자동 설정된 스케줄의 임의 등록·해제·상태 조회를 위한 관리 명령으로 남긴다. Setup Wizard 재실행은 새 설정·저장소 준비·스케줄 등록이 모두 성공한 뒤에만 기존 구성을 교체하며, 실패 시 기존 설정과 스케줄을 유지한다.

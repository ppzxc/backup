# 0014. Verify the Setup-to-Recovery E2E Matrix

## Status

Accepted

## Decision

Container E2E Matrix는 Setup Wizard로만 생성된 설정을 사용하여 S3 primary, SFTP primary, S3→S3, S3→SFTP, SFTP→S3, SFTP→SFTP의 여섯 저장소 구성을 검증한다. 각 구성은 수동 `backup run`, primary와 활성화된 secondary에서의 독립 `backup restore`, Setup Wizard가 자동 등록한 스케줄러의 실제 `backup run`, Backup Execution Report 파일을 검증한다. scheduler는 systemd 우선 및 Cron fallback을 모두 실제 실행으로 검증하며, 파일 왕복은 경로·내용·빈 파일·중첩 구조·유니코드 파일명·실행 권한을 비교한다. Database Stream의 import·행 검증은 별도의 Database E2E Support Matrix에 남긴다.

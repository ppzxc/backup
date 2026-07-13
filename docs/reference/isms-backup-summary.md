# ISMS 관련 백업 정리

> Notion 원본: https://app.notion.com/p/398e73514644806fb606df40335c990a
> 최종 편집: 2026-07-13 (스냅샷 기준)

## backup.sh

- restic 기반 rclone 백엔드 활용하는 sftp 증분 백업
- 사내 호스트 4개 대상으로 실증
- synology sftp 서버에 /var/log 만 우선 대상으로 백업 진행중

(원본 페이지에는 구성 스크린샷 이미지가 첨부되어 있습니다 — Notion 원본 참고)

## reference

- [오픈소스 S3 백업 도구 최종 선택 가이드라인](oss-s3-backup-tool-guide.md)
- [로그 및 소산 백업용 S3 호환 클라우드 도입 가이드라인](log-offsite-backup-s3-storage-guide.md)
- [restic 백업 재시도 및 안정성 확보 베스트 프랙티스 가이드](restic-retry-reliability-guide.md)
- [ISMS 인증 준수 Rocky Linux 9 시스템 백업 가이드라인 (restic + SFTP)](isms-rocky9-restic-sftp-guide.md)
- [restic to sftp for test](restic-sftp-test-script.md)

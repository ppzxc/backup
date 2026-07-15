# 수동 검증 체크리스트 (Tier 3)

테스트 VM(RHEL 계열, dnf) 또는 실제 대상 서버에서 1회씩 수행한다.

## SFTP 경로
- [ ] `backup.sh install` → epel/restic/rclone 설치 확인, `/usr/local/sbin/backup.sh` 생성 확인
- [ ] `backup.sh setting --backend sftp --host <NAS_IP> --port <PORT> --user <USER> --password <PW>` → `/etc/restic/backup.env`(600), `backup_key`(600)/`backup_key.pub`(644) 생성 확인
- [ ] 출력된 공개키를 실제 NAS `authorized_keys`에 등록
- [ ] `backup.sh init` → 저장소 초기화 성공
- [ ] `backup.sh schedule enable` → `systemctl list-timers`에 `resticprofile-backup@profile-<profile-name>.timer` 노출 확인(Tier 2 컨테이너는 systemd가 PID 1이 아니라 실제 활성화까지는 검증 못 함 — 유닛 파일 내용/비밀값 부재만 자동 검증됨)
- [ ] 위 타이머를 즉시 트리거해 실제로 백업이 도는지 확인(`systemctl start resticprofile-backup@profile-<profile-name>.service` 또는 시각을 1분 뒤로 맞춤)
- [ ] `backup.sh status` → 스냅샷/타이머 상태 정상 출력, 비밀번호 미노출 확인
- [ ] `backup.sh schedule disable` → 타이머 비활성화 확인
- [ ] `backup.sh uninstall --purge` → `/etc/restic` 삭제, 유닛 파일 제거(resticprofile unschedule) 확인

## S3 경로
- [ ] `backup.sh install` → 위와 동일
- [ ] `backup.sh setting --backend s3 --endpoint <EP> --bucket <BUCKET> --access-key <AK> --secret-key <SK> --password <PW>` → backup.env 생성, 버킷 정책 JSON 출력 확인
- [ ] 출력된 최소권한 정책을 실제 버킷/IAM에 적용
- [ ] `backup.sh init` → 저장소 초기화 성공
- [ ] `backup.sh run` → 실제 오브젝트가 버킷에 생성되는지 확인
- [ ] `backup.sh uninstall --purge`

## wizard
- [ ] `backup.sh wizard` 전체 흐름을 SFTP로 1회, S3로 1회 수행하며 각 질문의 설명 문구가 이해하기 쉬운지 확인

## 필수값 누락 시나리오
- [ ] `backup.sh setting --backend sftp --port 22` (host/user 누락) → 안내 명령에 `--port 22`는 채워지고 `--host`/`--user`는 placeholder로 나오는지 확인
- [ ] `backup.sh init`을 `setting` 전에 실행 → 안내 메시지가 s3/sftp 두 예시 모두 보여주는지 확인

## 참고: Tier 2(docker-compose) 자동화 테스트로 이미 확인된 항목
`tests/integration/integration.bats`가 MinIO + atmoz/sftp + rockylinux:9 컨테이너로
S3/SFTP 각각의 install → setting → init → run → 스냅샷 확인까지는 이미
자동으로 검증한다. 위 체크리스트는 컨테이너로 재현 불가능한 부분
(systemd 타이머 실동작, 실제 NAS/버킷 등록, 대화형 wizard 문구 검토)에
집중한다.

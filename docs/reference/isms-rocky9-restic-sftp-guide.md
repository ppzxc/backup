# 🛡️ ISMS 인증 준수 Rocky Linux 9 시스템 백업 가이드라인 (restic + SFTP)

> Notion 원본: https://app.notion.com/p/398e7351464480ecb6ebf85daa57bf22
> 상위 페이지: [ISMS 관련 백업 정리](isms-backup-summary.md)
> 최종 편집: 2026-07-09 (스냅샷 기준)
>
> **주의:** 이 문서는 `dnf install restic`(공식 배포 패키지) 기준으로 작성된
> 초기 리서치 문서입니다. 현재 `backup.sh`는 dnf/EPEL 대신 restic/rclone을
> 버전 고정 + 체크섬 검증 후 GitHub에서 직접 받아 설치합니다(자세한 내용은
> 저장소 루트의 `AGENTS.md` 참고). 통제항목 매핑·정책·심사 대응 방향은
> 여전히 유효합니다.

> **문서 버전:** 1.0 (KISA ISMS 인증 기준 단일 최적화)
> **대상 시스템:** Rocky Linux 9 (OS 설정 및 서비스 애플리케이션, 시스템
> 로그 대상 *[DB 제외]*)
> **백업 방식:** 사내망 Synology NAS (SFTP 프로토콜) 기반 1차 백업

---

## 1. ISMS 통제항목 매핑 및 설계 방향

본 가이드라인은 **KISA ISMS 보호대책 요구사항 중 '2.10 시스템 및 서비스
운영관리' 및 '2.7 접근통제'** 항목을 충족하도록 설계되었습니다.

- **백업 및 복구 관리 (통제항목 2.10.4):** 백업 대상 선정, 주기적 백업
  수행, 정기적인 복구 테스트 수행 및 이력 관리를 체계화합니다.
- **암호통제 (통제항목 2.10.5):** `restic`은 전송(SFTP/SSH) 및 저장(AES-256
  군용 등급 암호화) 전 과정에서 데이터를 강제 암호화하므로 암호화 저장
  요건을 완벽히 충족합니다.
- **권한 오남용 방지 (통제항목 2.7.1):** 백업에 사용되는 암호화 키 파일 및
  자격 증명 환경 변수는 `/etc/restic` 내에 `root(0600)` 권한으로 격리하여
  타 계정의 접근을 원천 차단합니다.
- **로그 보존 (통제항목 2.11.1):** 백업 스케줄링을 OS 표준 메커니즘인
  **`Systemd Timer`**로 관리하여, 백업 성공/실패 기록이 시스템 감사
  로그(`journalctl`)에 영구 추적되도록 구성합니다.

---

## 2. 백업 정책 정의 (Policy)

### 백업 대상 및 제외 범위

- **백업 대상 (Scope):**
  - `/etc/` (OS 구성 및 네트워크 설정 파일)
  - `/var/log/` (시스템 로그 및 감사 로그 - *법적 규제 준수*)
  - `/home/`, `/root/` (운영자/사용자 데이터)
  - `/opt/`, `/var/www/` (서비스 애플리케이션 소스 및 엔진 설정 파일)
- **백업 제외 (Exclude):**
  - 임시/런타임 디렉토리 (`/tmp`, `/var/tmp`, `/run`, `/proc`, `/sys`, `/dev`)
  - **데이터베이스 데이터 경로 (`/var/lib/mysql`, `/var/lib/pgsql` 등 -
    별도 DB 가이드라인 적용)**

### 백업 주기 및 보관 기간 (Retention)

- **실행 주기:** 매일 새벽 02:00 (시스템 및 서비스 부하가 가장 적은 시간대)
- **보관 주기 정책:**
  - `최근 7일` 일별 백업 스냅샷 전량 보관
  - `최근 4주` 주별 최신 백업 스냅샷 1개씩 보관
  - `최근 12개월` 월별 최신 백업 스냅샷 1개씩 보관 (ISMS 로그 보존 최소
    1년 규정 준수)

---

## 3. Rocky Linux 9 환경 구성 및 초기화

### ① restic 설치 및 보안 폴더 생성

Rocky Linux 9 공식 EPEL 레포지토리를 활성화한 후 restic을 설치합니다.

```bash
# EPEL 레포지토리 활성화 및 restic 설치
sudo dnf install -y epel-release
sudo dnf install -y restic

# 백업 설정 보관 폴더 생성 및 권한 통제
sudo mkdir -p /etc/restic
sudo chmod 700 /etc/restic
```

### ② 자격 증명 환경 변수 파일 생성 (`/etc/restic/backup.env`)

심사 시 스크립트 내 비밀번호 평문 노출은 주요 결함 사유입니다. 자격 증명을
별도 파일로 분리하고 권한을 제한합니다.

```bash
sudo cat << 'EOF' > /etc/restic/backup.env
# 1. SFTP 저장소 (Synology NAS IP 및 호스트별 격리 경로)
RESTIC_REPOSITORY="sftp:backup_user@<NAS_IP>:/volume1/backup_shared/repos/$(hostname)"

# 2. restic 암호화 비밀번호 (최소 16자리 이상 영문/숫자/특수문자 조합)
RESTIC_PASSWORD="<REPOSITORY_PASSWORD>"

# 3. SFTP 접속용 SSH Key 경로 및 포트 지정 (패스워드 인증 방식 금지)
RESTIC_SSH_ARGS="-i /etc/restic/backup_key -p 22"
EOF

# 소유자(root) 외 읽기/쓰기 금지 설정 (ISMS 필수 권한 통제)
sudo chmod 600 /etc/restic/backup.env
```

### ③ SSH 비대칭키 생성 및 저장소 초기화

```bash
# 백업 전용 SSH Key 생성 (비밀번호 없이 엔터)
sudo ssh-keygen -t ed25519 -f /etc/restic/backup_key -N ""
sudo chmod 600 /etc/restic/backup_key

# [필독] 생성된 /etc/restic/backup_key.pub 내용을
# 타겟 Synology NAS의 backup_user 계정 내 .ssh/authorized_keys에 등록해야 정상 작동합니다.

# 환경 변수 로드 후 restic SFTP 원격 저장소 최초 1회 초기화
source /etc/restic/backup.env
restic init
```

---

## 4. 프로덕션 백업 실행 스크립트

### 📜 `/etc/restic/run-backup.sh`

```bash
#!/bin/bash
# Description: ISMS Compliant System Backup Script for Rocky Linux 9

# 1. 환경 변수 검증 및 로드
if [ -f /etc/restic/backup.env ]; then
    source /etc/restic/backup.env
else
    echo "❌ 치명적 오류: 백업 환경 설정 파일이 없습니다." | logger -t restic-backup
    exit 1
fi

echo "🚀 [$(hostname)] restic 시스템 백업을 시작합니다." | logger -t restic-backup

# 2. 안전장치: 비정상 종료로 멈춘 stale 락(Lock)만 선택적 해제
restic unlock --stale > /dev/null 2>&1

# 3. 데이터 중복 제거 백업 수행 (시스템 설정 및 로그 중심, DB 제외)
restic backup \
    /etc \
    /var/log \
    /home \
    /root \
    /opt \
    --exclude="/tmp/*" \
    --exclude="/var/tmp/*" \
    --exclude="/var/lib/mysql/*" \
    --exclude="/var/lib/pgsql/*"

# 4. 결과 검증 및 ISMS 기준 보관 정책(Retention) 적용
if [ $? -eq 0 ]; then
    echo "✅ 백업 성공. 만료된 스냅샷 만료 처리를 시작합니다." | logger -t restic-backup

    # 설정된 주기에 따라 스냅샷을 만료시키고, 실제 NAS 디스크 용량 정리(prune)까지 통합 수행
    restic forget \
        --keep-daily 7 \
        --keep-weekly 4 \
        --keep-monthly 12 \
        --prune > /dev/null 2>&1

    echo "🧹 만료 데이터 및 디스크 용량 정리가 완료되었습니다." | logger -t restic-backup
else
    echo "🚨 치명적 오류: restic 백업 프로세스가 실패했습니다!" | logger -t restic-backup
    # (필요 시 사내 슬랙/잔디 등 모니터링 알림 웹훅 연동)
    exit 1
fi
```

```bash
# 스크립트 실행 권한 부여
sudo chmod 700 /etc/restic/run-backup.sh
```

---

## 5. Systemd 활용 자동화 및 감사 추적 등록

크론탭 대신 Rocky Linux 9의 로깅 메커니즘(`journald`)과 연동되는 Systemd
Timer를 활용하여 통제 이력을 투명하게 관리합니다.

### ① 서비스 파일 등록 (`/etc/systemd/system/restic-backup.service`)

```ini
[Unit]
Description=Restic System Backup Service (ISMS Compliance)
After=network-online.target

[Service]
Type=oneshot
ExecStart=/etc/restic/run-backup.sh
User=root
Group=root
Restart=no
```

### ② 타이머 파일 등록 (`/etc/systemd/system/restic-backup.timer`)

```ini
[Unit]
Description=Run Restic Backup Daily at 2 AM

[Timer]
# 매일 새벽 02:00:00 정각 실행
OnCalendar=*-*-* 02:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

### ③ 스케줄러 가동 및 상시 활성화

```bash
# systemd 데몬 재로드
sudo systemctl daemon-reload

# 타이머 가동 및 부팅 시 자동 시작 등록
sudo systemctl enable --now restic-backup.timer

# 타이머 정상 스케줄링 작동 여부 확인
systemctl list-timers --all | grep restic
```

---

## 6. ISMS 인증 심사 대응 가이드 (증적 자료 추출)

심사원이 **백업 관리(2.10.4)**에 대한 실효성 증적을 요구할 시 아래 명령어를
활용해 보고서를 추출합니다.

### ① 주기적 백업 수행 로그 (최근 1주일 이력)

```bash
sudo journalctl -u restic-backup.service --since "1 week ago"
```

### ② 암호화 저장소 내 스냅샷 보유 현황 (보관 주기 준수 증적)

```bash
source /etc/restic/backup.env
restic snapshots
```

### ③ 정기 복구 테스트 이력 (연 1회 이상 필수 요구사항)

실제 데이터 복구가 가능한지 테스트 환경에서 모의 훈련을 수행하고 이를
캡처하여 이력서로 보관해야 합니다.

```bash
# 임시 복구 검증 폴더 생성
mkdir -p /tmp/restore_test

# 최신(latest) 백업본을 임시 폴더로 전량 복구 수행
source /etc/restic/backup.env
restic restore latest --target /tmp/restore_test

# 복구 데이터 무결성 육안 확인 후 테스트 데이터 삭제
ls -la /tmp/restore_test
rm -rf /tmp/restore_test
```

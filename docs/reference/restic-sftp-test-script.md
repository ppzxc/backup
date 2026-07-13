# restic to sftp for test

> Notion 원본: https://app.notion.com/p/398e735146448014ac36ec3d84c48703
> 상위 페이지: [ISMS 관련 백업 정리](isms-backup-summary.md)
> 최종 편집: 2026-07-09 (스냅샷 기준)
>
> **주의:** 이 문서는 `backup.sh`가 만들어지기 전의 초기 프로토타입
> 원샷 스크립트입니다(`dnf install restic rclone` 기준, `wizard`/`setting`
> 같은 서브커맨드 분리 이전). 지금은 `backup.sh wizard`가 이 흐름을
> 대체하며, restic/rclone 설치도 dnf가 아니라 버전 고정 + 체크섬 검증 후
> GitHub에서 직접 받아옵니다. 히스토리 참고용으로 원문 그대로 보존합니다.

```shell
# =================================================================
# 1. 환경 설정 변수 (필요시 이 5줄만 수정하여 사용하세요)
# =================================================================
NAS_IP="<NAS_IP>"
NAS_PORT="<NAS_PORT>"
NAS_USER="<BACKUP_USER>"
RESTIC_PASS="<REPOSITORY_PASSWORD>"
BACKUP_DIR="/var/log"

# =================================================================
# 2. 패키지 설치 (EPEL, restic, rclone)
# =================================================================
sudo dnf install -y epel-release
sudo dnf install -y restic rclone

# =================================================================
# 3. 보안 폴더 구조 생성 및 SSH Key 발급 (비대화형 자동 생성)
# =================================================================
sudo mkdir -p /etc/restic
sudo chmod 700 /etc/restic

if [ ! -f /etc/restic/backup_key ]; then
    sudo ssh-keygen -t ed25519 -f /etc/restic/backup_key -N ""
fi
sudo chmod 600 /etc/restic/backup_key

# =================================================================
# 4. 통합 자격 증명 환경 변수 파일 생성 (backup.env)
# =================================================================
sudo tee /etc/restic/backup.env > /dev/null << EOF
export RESTIC_REPOSITORY="rclone:syno_backup:/backup/\$(hostname)"
export RCLONE_CONFIG_SYNO_BACKUP_TYPE="sftp"
export RCLONE_CONFIG_SYNO_BACKUP_HOST="${NAS_IP}"
export RCLONE_CONFIG_SYNO_BACKUP_USER="${NAS_USER}"
export RCLONE_CONFIG_SYNO_BACKUP_PORT="${NAS_PORT}"
export RCLONE_CONFIG_SYNO_BACKUP_KEY_FILE="/etc/restic/backup_key"
export RESTIC_PASSWORD="${RESTIC_PASS}"
export BACKUP_TARGETS="${BACKUP_DIR}"
EOF
sudo chmod 600 /etc/restic/backup.env

# =================================================================
# 5. 프로덕션 백업 실행 스크립트 생성 (run-backup.sh)
# =================================================================
sudo tee /etc/restic/run-backup.sh > /dev/null << 'EOF'
#!/bin/bash
if [ -f /etc/restic/backup.env ]; then
    source /etc/restic/backup.env
else
    echo "❌ 치명적 오류: 백업 환경 설정 파일이 없습니다." | logger -t restic-backup
    exit 1
fi

echo "🚀 [$(hostname)] restic 시스템 백업을 시작합니다." | logger -t restic-backup
restic unlock --stale > /dev/null 2>&1

restic backup \
    $BACKUP_TARGETS \
    --exclude="/tmp/*" \
    --exclude="/var/tmp/*" \
    --exclude="/var/lib/mysql/*" \
    --exclude="/var/lib/pgsql/*"

if [ $? -eq 0 ]; then
    echo "✅ 백업 성공. 만료된 스냅샷 관리를 시작합니다." | logger -t restic-backup
    restic forget --keep-daily 7 --keep-weekly 4 --keep-monthly 12 --prune > /dev/null 2>&1
    echo "🧹 만료 데이터 정리가 완료되었습니다." | logger -t restic-backup
else
    echo "🚨 치명적 오류: restic 백업 프로세스가 실패했습니다!" | logger -t restic-backup
    exit 1
fi
EOF
sudo chmod 700 /etc/restic/run-backup.sh

# =================================================================
# 6. Systemd 타이머 및 서비스 등록 (ISMS 감사 추적용)
# =================================================================
sudo tee /etc/systemd/system/restic-backup.service > /dev/null << EOF
[Unit]
Description=Restic System Backup Service (ISMS Compliance)
After=network-online.target

[Service]
Type=oneshot
ExecStart=/etc/restic/run-backup.sh
User=root
Group=root
Restart=no
EOF

sudo tee /etc/systemd/system/restic-backup.timer > /dev/null << EOF
[Unit]
Description=Run Restic Backup Daily at 2 AM

[Timer]
OnCalendar=*-*-* 02:00:00
Persistent=true

[Install]
WantedBy=timers.target
EOF

# =================================================================
# 7. 스케줄러 데몬 가동
# =================================================================
sudo systemctl daemon-reload
sudo systemctl enable --now restic-backup.timer

# =================================================================
# 8. 최종 마무리를 위한 안내 출력
# =================================================================
clear
echo "=========================================================="
echo " 🎉 [$(hostname)] 기본 백업 자동화 환경 구성 완료!"
echo "=========================================================="
echo " 1. 아래 출력된 이 서버의 '공개키'를 복사하여 "
echo "    시놀로지 File Station (.ssh/authorized_keys)에 추가하세요."
echo "----------------------------------------------------------"
cat /etc/restic/backup_key.pub
echo "----------------------------------------------------------"
echo " 2. 키 등록 후, 아래 명령어를 '그대로 입력'하여 저장소를 최초 1회 초기화하세요."
echo "    (초기화가 끝나면 스케줄러에 의해 매일 새벽 2시 자동 백업됩니다.)"
echo ""
echo "    source /etc/restic/backup.env && restic init"
echo "=========================================================="
```

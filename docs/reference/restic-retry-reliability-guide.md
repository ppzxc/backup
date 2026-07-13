# 🛠️ restic 백업 재시도 및 안정성 확보 베스트 프랙티스 가이드

> Notion 원본: https://app.notion.com/p/398e7351464480e58560f52ac9eac230
> 상위 페이지: [ISMS 관련 백업 정리](isms-backup-summary.md)
> 최종 편집: 2026-07-09 (스냅샷 기준)

> **문서 목적:** 100대 대규모 서버 환경에서 restic 백업 실패(네트워크 순단,
> 락 충돌 등) 리스크를 최소화하고, 재시도(Retry) 구조를 안정적으로
> 자동화하기 위한 엔터프라이즈급 가이드라인

---

## 1. 대규모 환경에서 잊지 말아야 할 3대 운영 원칙

### ① 리포지토리 분리를 통한 락(Lock) 충돌 방지 (필수)

restic은 하나의 저장소에 동시에 쓰기 작업이 들어오면 데이터 보호를 위해
전역 락(Lock)을 겁니다. 100대 서버가 Synology NAS의 단일 경로를 공유하면
매분마다 락 충돌이 발생하여 무한 재시도 지옥에 빠집니다.

- **해결책:** NAS 내 하위 경로에 **서버별 호스트명(`$HOSTNAME`) 단위로
  저장소를 격리**하여 충돌을 원천 차단합니다.

### ② 무조건적인 `unlock` 금지 (`-stale` 활용)

백업 시작 전 에러를 막겠다고 `restic unlock` 명령어를 무조건 실행하면,
실제로 대용량 데이터를 정상 업로드 중이던 다른 프로세스의 락까지 해제하여
백업 본이 깨질 수 있습니다.

- **해결책:** 끊긴 지 오래된 고정 락만 안전하게 지우는 `restic unlock
  --stale` 옵션을 사용합니다.

### ③ 오픈소스 래퍼(Wrapper) 도구 도입 적극 검토

정교한 재시도, 슬랙 알림, 성공/실패 훅(Hook)을 복잡한 쉘 스크립트로 짜서
100대 서버에 배포하는 것보다, 전 세계 엔지니어들이 검증한 **`resticprofile`**
같은 YAML 기반 도구를 도입하는 것이 장기적으로 유지보수 비용을 수십 배
아끼는 길입니다.

---

## 2. [추천 안 A] 생산성 극대화: resticprofile 도입 (YAML 관리)

서버마다 가볍게 바이너리 하나와 설정 파일만 배포하면 자체적으로 재시도(Retry)
및 알림을 수행합니다.

### 📄 `/etc/resticprofile/profiles.yaml` 설정 예시

```yaml
version: "1"

default:
  # 호스트명별로 저장소 경로를 분리하여 락 충돌 방지
  repository: "<sftp:backup_user@NAS_IP>:/volume1/backup_shared/repos/${HOSTNAME}"
  password-file: "/etc/restic/password.txt"

  # 💡 베스트 프랙티스: 자체 내장 재시도 정책
  retry:
    count: 3          # 실패 시 최대 3회 재시도
    interval: 1m      # 재시도 간격 (1분)

  # 백업 실행 전/후 제어
  hooks:
    before:
      - "restic unlock --stale" # 멈춘지 오래된 고상 락만 안전하게 해제
    on-failure:
      - "curl -X POST -H 'Content-type: application/json' --data '{\"text\":\"❌ [${HOSTNAME}] restic 백업 최종 실패!\"}' https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

# 실행 명령: resticprofile backup
```

---

## 3. [추천 안 B] 가성비 중심: 프로덕션급 자동화 쉘 스크립트

추가 도구 설치 없이 기존 크론탭(Cron)이나 Systemd에 그대로 이식할 수 있도록
안전장치가 고도화된 래퍼 스크립트입니다.

### 📜 `restic-safe-backup.sh`

```bash
#!/bin/bash

# ==========================================
# 1. 인프라 및 환경 변수 설정
# ==========================================
# 호스트명을 경로에 추가하여 100대 서버 간 락 충돌 방지
export RESTIC_REPOSITORY="sftp:backup_user@<NAS_IP>:/volume1/backup_shared/repos/$(hostname)"
export RESTIC_PASSWORD_FILE="/etc/restic/password.txt"
SLACK_WEBHOOK_URL="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"

BACKUP_TARGET="/대상/디렉토리"
MAX_RETRIES=3       # 최대 재시도 횟수
RETRY_DELAY=60      # 실패 시 대기 시간 (초)

# ==========================================
# 2. 백업 실행 및 예외 처리 로직
# ==========================================
echo "🚀 [$(hostname)] restic 안전 백업 프로세스를 시작합니다."

# 오래된 비정상 락 파일 선제적 해제 (다른 서버에 영향 없음)
restic unlock --stale > /dev/null 2>&1

for ((i=1; i<=MAX_RETRIES; i++)); do
    echo "🔄 [시도 $i/$MAX_RETRIES] 백업 중..."

    # 백업 실행 (이어올리기는 restic이 내부적으로 기본 지원)
    restic backup "$BACKUP_TARGET"

    # 종료 코드 확인 (0: 성공)
    if [ $? -eq 0 ]; then
        echo "✅ 백업이 성공적으로 완료되었습니다."
        exit 0
    else
        echo "⚠️ 백업 실패 (시도 $i/$MAX_RETRIES)"
        if [ $i -lt $MAX_RETRIES ]; then
            echo "💤 ${RETRY_DELAY}초 후 재시도합니다..."
            sleep $RETRY_DELAY
            # 재시도 전 혹시 모를 잠금 상태 다시 체크
            restic unlock --stale > /dev/null 2>&1
        fi
    fi
done

# ==========================================
# 3. 최종 실패 시 사내 모니터링 채널 전송
# ==========================================
echo "❌ 치명적 오류: 최대 재시도 횟수를 초과하여 백업에 실패했습니다."
payload="{\"text\": \"🚨 *[백업 실패]* \n• 호스트: \`$(hostname)\`\n• 확인 사항: 내부망 단절 또는 Synology NAS 디스크 풀 확인 요망\"}"
curl -X POST -H 'Content-type: application/json' --data "$payload" "$SLACK_WEBHOOK_URL" > /dev/null 2>&1

exit 1
```

---

## 🛠️ 구축 후 최종 체크리스트

- [ ] 100대 서버가 저장소를 바라볼 때 반드시 하위 경로에 `$(hostname)` 또는
      고유 ID가 적용되었는가?
- [ ] 백업 실패 시 알림을 받아볼 수 있는 Webhook 채널(Slack/Teams 등)이
      연동되었는가?
- [ ] 무조건적인 `unlock` 대신 `-stale` 플래그를 사용하여 실행 중인 다른
      프로세스를 보호했는가?
- [ ] restic의 기본 기능인 **이어 올리기(Resume)** 덕분에 재시도 시 대역폭
      낭비가 없는지 모니터링을 통해 확인했는가?

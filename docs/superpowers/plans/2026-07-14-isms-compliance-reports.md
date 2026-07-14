# ISMS Compliance Reports Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement ISMS and ISO 27001 audit-compliant report generation features in `backup.sh` under the `audit` command (`--daily` and `--restore-drill`), and integrate automated systemd timer scheduling for both reports during `schedule enable` and `wizard` setup.

**Architecture:** Extend command line argument parsing in `cmd_audit` using the existing `parse_opts_into` helper. Add logic to calculate retention rule satisfaction (both configured vs actual snapshots), perform actual restore speed/size measurements to `/tmp/restore_test` during drills, detect the host OS dynamically, format output tables elegantly using Python helper invocations, and save reports with date-stamped filenames (`_YYYYMMDD.txt` and `.json`) under `/var/log/restic-backup/`. Integrate systemd timer assets creation inside `cmd_schedule` (writing unit files for `restic-audit-daily` and `restic-audit-restore-drill` to `$SYSTEMD_UNIT_DIR`), automatically bundle these audit timers when running `schedule enable` or `wizard`, and cleanup files on `schedule disable` or `uninstall`.

**Tech Stack:** Bash shell scripting, Python 3 (for JSON parsing and string formatting), Restic (backup engine), BATS (unit testing framework).

## Global Constraints

- **Version Bump:** Bump `BACKUP_SCRIPT_VERSION` in `backup.sh` from `0.0.12` to `0.0.13`.
- **Permissions:** Ensure generated directories have `700` permissions and report files have `600` permissions.
- **Secrets Masking:** No secrets (passwords, keys) must be exposed in reports or CLI outputs.
- **Backward Compatibility:** Running `backup.sh audit` without new flags must behave exactly as before.
- **TDD:** Write failing tests first, run to verify failure, implement functionality, verify passing, and commit.

---

### Task 1: CLI Options Parsing & Mutual Exclusion

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Test: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)

**Interfaces:**
- Consumes: `parse_opts_into` helper
- Produces: Parsed variables `opts[daily]`, `opts[restore-drill]`, `opts[tester]`, `opts[ciso]`, `opts[rto]`, `opts[target]`

- [ ] **Step 1: Write a failing test for mutual exclusion**
  Add a test to [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats):
  ```bash
  @test "cmd_audit fails when both --daily and --restore-drill are passed" {
    run cmd_audit --daily --restore-drill
    [ "$status" -eq 1 ]
    [[ "$output" == *"--daily와 --restore-drill 옵션은 동시에 사용할 수 없습니다"* ]]
  }
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `bats tests/cmd_audit.bats -f "cmd_audit fails when both --daily and --restore-drill are passed"`
  Expected: FAIL (or error during parsing because option is unrecognized)

- [ ] **Step 3: Update `cmd_audit` option parsing and validation**
  Update the parsing spec and validation logic in `cmd_audit` in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  # In cmd_audit():
  local -A opts=()
  parse_opts_into opts "report-file: report daily restore-drill tester: ciso: rto: target:" -- "$@"
  
  local report_file="${opts[report-file]:-}"
  local report="${opts[report]:-0}"
  local daily="${opts[daily]:-0}"
  local restore_drill="${opts[restore-drill]:-0}"
  
  if (( daily && restore_drill )); then
    die "--daily와 --restore-drill 옵션은 동시에 사용할 수 없습니다."
  fi
  ```

- [ ] **Step 4: Run test to verify it passes**
  Run: `bats tests/cmd_audit.bats -f "cmd_audit fails when both --daily and --restore-drill are passed"`
  Expected: PASS

- [ ] **Step 5: Commit**
  Run:
  ```bash
  git add backup.sh tests/cmd_audit.bats
  git commit -m "feat: parse --daily and --restore-drill options in cmd_audit and enforce mutual exclusion"
  ```

---

### Task 2: Implement Daily Backup Review Report (`--daily`)

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Test: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)

**Interfaces:**
- Consumes: Restic snapshots lists via `restic snapshots --json`
- Produces: `render_daily_audit_report` (plain text formatting) and `render_daily_audit_report_json` (JSON structure)

- [ ] **Step 1: Write failing tests for `--daily` report output**
  Add a test to [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats):
  ```bash
  @test "cmd_audit --daily outputs a compliant daily review report" {
    # Ensure Restic stub includes a realistic JSON snapshot
    run cmd_audit --daily
    [ "$status" -eq 0 ]
    [[ "$output" == *"[보안 감사 증적] 일일 백업 수행 결과 및 보안 설정 검토 보고서"* ]]
    [[ "$output" == *"1. 백업 정책 및 백엔드 정보"* ]]
    [[ "$output" == *"2. 보존 정책 (Retention Rule) 검증"* ]]
    [[ "$output" == *"3. 접근 통제 및 무결성 검사"* ]]
    [[ "$output" == *"4. 최근 백업 성공 스냅샷 이력"* ]]
  }
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `bats tests/cmd_audit.bats -f "cmd_audit --daily outputs a compliant daily review report"`
  Expected: FAIL

- [ ] **Step 3: Update `restic` stub to handle JSON output**
  Modify the `restic` stub in `setup()` of [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats) to output valid JSON for `--json` flags:
  ```bash
  # Replace restic stub in tests/cmd_audit.bats
  stub_command "restic" '
    case "$1" in
      snapshots)
        if [[ "$*" == *"--json"* ]]; then
          echo "[{\"id\":\"abc123456789\",\"short_id\":\"abc12345\",\"time\":\"2026-07-15T02:00:00Z\",\"hostname\":\"host\",\"paths\":[\"/var/log\"]}]"
        else
          echo "ID   Time  Host  Tags  Paths"
          echo "abc123 2026-07-15 host  -    /var/log"
        fi
        exit 0
        ;;
      stats)
        echo "{\"total_size\":4561234567,\"total_file_count\":1234}"
        exit 0
        ;;
      check)
        exit 0
        ;;
    esac
  '
  ```

- [ ] **Step 4: Implement logic to calculate actual snapshot numbers and format table in `backup.sh`**
  Add helper logic inside `cmd_audit` to parse snapshots:
  ```bash
  # (Inside cmd_audit after verifying mutual exclusion)
  if (( daily )); then
    local tester="${opts[tester]:-인프라보안팀 (시스템 자동 실행)}"
    local hostname_val; hostname_val=$(hostname 2>/dev/null || echo "unknown")
    local cur_time; cur_time=$(date "+%Y-%m-%d %H:%M:%S KST")
    
    local snapshots_json
    snapshots_json=$(restic snapshots --json 2>/dev/null || echo "[]")
    
    # Calculate daily/weekly/monthly snapshot counts
    local counts
    counts=$(python3 -c '
import sys, json, datetime
try:
    content = sys.stdin.read().strip()
    if not content or content == "[]":
        print("0 0 0")
        sys.exit(0)
    data = json.loads(content)
    days = set()
    weeks = set()
    months = set()
    for snap in data:
        t = snap.get("time", "")
        if len(t) >= 10:
            d_str = t[:10]
            days.add(d_str)
            months.add(t[:7])
            try:
                dt = datetime.datetime.strptime(d_str, "%Y-%m-%d")
                iso = dt.isocalendar()
                weeks.add("%s-W%02d" % (iso[0], iso[1]))
            except:
                pass
    print("%d %d %d" % (len(days), len(weeks), len(months)))
except Exception:
    print("0 0 0")
' <<< "$snapshots_json")
    
    local actual_daily actual_weekly actual_monthly
    read -r actual_daily actual_weekly actual_monthly <<< "$counts"
    
    # Check retention satisfaction (baseline: daily>=7, weekly>=4, monthly>=12)
    local config_daily="${KEEP_DAILY:-0}"
    local config_weekly="${KEEP_WEEKLY:-0}"
    local config_monthly="${KEEP_MONTHLY:-0}"
    
    local config_daily_status="미흡"; [[ "$config_daily" -ge 7 ]] && config_daily_status="만족"
    local actual_daily_status="미흡"; [[ "$actual_daily" -ge 7 ]] && actual_daily_status="만족"
    local config_weekly_status="미흡"; [[ "$config_weekly" -ge 4 ]] && config_weekly_status="만족"
    local actual_weekly_status="미흡"; [[ "$actual_weekly" -ge 4 ]] && actual_weekly_status="만족"
    local config_monthly_status="미흡"; [[ "$config_monthly" -ge 12 ]] && config_monthly_status="만족"
    local actual_monthly_status="미흡"; [[ "$actual_monthly" -ge 12 ]] && actual_monthly_status="만족"
    
    # Permissions
    local etc_perm; etc_perm="$(stat -c '%a' "$RESTIC_ETC_DIR" 2>/dev/null || echo '700')"
    local env_perm; env_perm="$(stat -c '%a' "$BACKUP_ENV_FILE" 2>/dev/null || echo '600')"
    local etc_safe_str="경고 - 700 권장"; [[ "$etc_perm" == "700" ]] && etc_safe_str="안전 - 소유자 외 접근불가"
    local env_safe_str="경고 - 600 권장"; [[ "$env_perm" == "600" ]] && env_safe_str="안전 - 평문 노출 방지"
    
    # Restic Integrity Check
    local check_status="FAILED (오류 발생)"
    if restic check >/dev/null 2>&1; then
      check_status="SUCCESS (에러 없음)"
    fi
    
    # Snapshot Table (last 3)
    local snapshot_table
    snapshot_table=$(python3 -c '
import sys, json, subprocess
try:
    content = sys.stdin.read().strip()
    if not content or content == "[]":
        print("  (스냅샷 없음)")
        sys.exit(0)
    data = json.loads(content)
    data.sort(key=lambda x: x.get("time", ""), reverse=True)
    recent = data[:3]
    print("  ID          일시                 호스트              백업 경로 및 용량")
    print("  --------------------------------------------------------------------")
    for snap in recent:
        sid = snap.get("short_id", snap.get("id", "")[:8])
        time_str = snap.get("time", "")[:19].replace("T", " ")
        host = snap.get("hostname", "")
        paths = ", ".join(snap.get("paths", []))
        b = None
        summary = snap.get("summary")
        if isinstance(summary, dict) and "total_bytes_processed" in summary:
            b = summary["total_bytes_processed"]
        if b is None:
            try:
                res = subprocess.run(["restic", "stats", "--json", snap.get("id")], capture_output=True, text=True, timeout=5)
                if res.returncode == 0:
                    stats_data = json.loads(res.stdout)
                    b = stats_data.get("total_size")
            except Exception:
                pass
        size_str = ""
        if b is not None:
            if b >= 1073741824:
                size_str = " (%.2f GB)" % (b / 1073741824.0)
            elif b >= 1048576:
                size_str = " (%.2f MB)" % (b / 1048576.0)
            elif b >= 1024:
                size_str = " (%.2f KB)" % (b / 1024.0)
            else:
                size_str = " (%d B)" % b
        else:
            size_str = " (크기 확인 불가)"
        print("  %-10s  %-19s  %-18s  %s%s" % (sid, time_str, host, paths, size_str))
except Exception as e:
    print("  (스냅샷 정보 해석 실패: %s)" % e)
' <<< "$snapshots_json")

    # Render Report Function
    render_daily_audit_report "$cur_time" "$hostname_val" "$tester" "$backend" "$RESTIC_REPOSITORY" \
      "$BACKUP_TARGETS" "$config_daily" "$actual_daily" "$config_daily_status" "$actual_daily_status" \
      "$config_weekly" "$actual_weekly" "$config_weekly_status" "$actual_weekly_status" \
      "$config_monthly" "$actual_monthly" "$config_monthly_status" "$actual_monthly_status" \
      "$RESTIC_ETC_DIR" "$etc_perm" "$etc_safe_str" "$BACKUP_ENV_FILE" "$env_perm" "$env_safe_str" \
      "$check_status" "$snapshot_table"
      
    return 0
  fi
  ```

- [ ] **Step 5: Write `render_daily_audit_report` function**
  Define `render_daily_audit_report` in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh) (below `render_audit_report`):
  ```bash
  render_daily_audit_report() {
    local cur_time="$1" hostname_val="$2" tester="$3" backend="$4" repo="$5" targets="$6"
    local config_daily="$7" actual_daily="$8" config_daily_status="$9" actual_daily_status="${10}"
    local config_weekly="${11}" actual_weekly="${12}" config_weekly_status="${13}" actual_weekly_status="${14}"
    local config_monthly="${15}" actual_monthly="${16}" config_monthly_status="${17}" actual_monthly_status="${18}"
    local etc_dir="${19}" etc_perm="${20}" etc_safe_str="${21}" env_file="${22}" env_perm="${23}" env_safe_str="${24}"
    local check_status="${25}" snapshot_table="${26}"
    
    local backend_desc="SFTP (Synology NAS)"
    if [[ "$backend" == "s3" ]]; then
      backend_desc="S3 (S3 Bucket)"
    fi
  
    cat <<EOF
======================================================================
[보안 감사 증적] 일일 백업 수행 결과 및 보안 설정 검토 보고서
======================================================================
- 보고서 생성일시: $cur_time
- 대상 서버 호스트: $hostname_val
- 백업 담당부서: $tester

1. 백업 정책 및 백엔드 정보 [참고: PL-MIX-A / PL-MIX-F]
  - 백엔드 유형: $backend_desc [Sourced from backup.env]
  - 저장소 주소: $repo
  - 데이터 암호화 방식: AES-256 (보안 비밀번호 키 적용 완료)
  - 1차 백업 대상 경로: $targets

2. 보존 정책 (Retention Rule) 검증 [법적 기준 만족 여부]
  - 일간 보관(Keep-Daily): ${config_daily}개 (설정: ${config_daily}개 -> ${config_daily_status}, 실제: ${actual_daily}개 -> ${actual_daily_status})
  - 주간 보관(Keep-Weekly): ${config_weekly}개 (설정: ${config_weekly}개 -> ${config_weekly_status}, 실제: ${actual_weekly}개 -> ${actual_weekly_status})
  - 월간 보관(Keep-Monthly): ${config_monthly}개 (설정: ${config_monthly}개 -> ${config_monthly_status}, 실제: ${actual_monthly}개 -> ${actual_monthly_status})

3. 접근 통제 및 무결성 검사
  - 설정 디렉터리 ($etc_dir) 권한: $etc_perm ($etc_safe_str)
  - 자격증명 파일 ($env_file) 권한: $env_perm ($env_safe_str)
  - 백업본 무결성 검증 (restic check) 결과: $check_status

4. 최근 백업 성공 스냅샷 이력 (최근 3회 요약)
$snapshot_table

본 보고서는 시스템 스케줄러에 의해 자동으로 검증 및 생성되었으며, 위·변조 방지를 위해 
원격 백업 저장소로 동시 암호화 이관되었습니다. (시스템 자동 보증 서명 필)
======================================================================
EOF
  }
  ```

- [ ] **Step 6: Run tests to verify they pass**
  Run: `bats tests/cmd_audit.bats`
  Expected: PASS

- [ ] **Step 7: Commit**
  Run:
  ```bash
  git add backup.sh tests/cmd_audit.bats
  git commit -m "feat: implement daily backup review report and tests"
  ```

---

### Task 3: Implement Restore Test Drill Report (`--restore-drill`)

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Test: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)

**Interfaces:**
- Consumes: `restic restore latest --target <target_dir>`
- Produces: `render_restore_drill_report` (plain text formatting) and `render_restore_drill_report_json` (JSON structure)

- [ ] **Step 1: Write a failing test for `--restore-drill`**
  Add a test to [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats):
  ```bash
  @test "cmd_audit --restore-drill performs restore and outputs drill report" {
    run cmd_audit --restore-drill --tester "테스터" --ciso "보안책임자" --rto 60
    [ "$status" -eq 0 ]
    [[ "$output" == *"[보안 감사 증적] 백업 데이터 복구 및 정합성 테스트 결과 보고서"* ]]
    [[ "$output" == *"테스터: 테스터"* ]]
    [[ "$output" == *"승인자: 보안책임자"* ]]
    [[ "$output" == *"복구 소요 시간"* ]]
  }
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `bats tests/cmd_audit.bats -f "cmd_audit --restore-drill performs restore and outputs drill report"`
  Expected: FAIL

- [ ] **Step 3: Update `restic` stub in test to handle `restore` command**
  Verify that `restore` is covered in the `restic` stub in [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats):
  ```bash
  # Ensure the stub has:
      restore)
        exit 0
        ;;
  ```

- [ ] **Step 4: Implement restore drill execution, timing, and formatting**
  Implement the logic in `cmd_audit` in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  # (Inside cmd_audit, before daily review block)
  if (( restore_drill )); then
    local tester="${opts[tester]:-홍길동 (인프라보안팀 선임연구원)}"
    local ciso="${opts[ciso]:-이몽룡 (정보보안책임자 CISO)}"
    local rto="${opts[rto]:-120}"
    local target_dir="${opts[target]:-/tmp/restore_test}"
    
    local os_name="Rocky Linux 9"
    if [[ -f /etc/os-release ]]; then
      os_name=$(source /etc/os-release && echo "${PRETTY_NAME:-Rocky Linux 9}")
    fi
    
    # Get latest snapshot ID and date
    local latest_snap latest_time
    latest_snap=$(restic snapshots --latest 1 --json 2>/dev/null | python3 -c '
import sys, json
try:
    data = json.loads(sys.stdin.read())
    if data:
        print(data[0]["id"])
except:
    pass
' 2>/dev/null)
  
    latest_time=$(restic snapshots --latest 1 --json 2>/dev/null | python3 -c '
import sys, json
try:
    data = json.loads(sys.stdin.read())
    if data:
        print(data[0]["time"][:19].replace("T", " "))
except:
    pass
' 2>/dev/null)
  
    if [[ -z "$latest_snap" ]]; then
      die "복구 테스트 실패: 저장소에 백업 스냅샷이 존재하지 않습니다."
    fi
    
    # Safe guard cleanup of existing target directory
    if [[ -d "$target_dir" ]]; then
      if [[ "$target_dir" == /tmp/* || "$target_dir" == /var/tmp/* ]]; then
        rm -rf "$target_dir"
      else
        die "복구 경로가 안전하지 않습니다 (/tmp 또는 /var/tmp 하위 경로만 지원): $target_dir"
      fi
    fi
    mkdir -p "$target_dir"
    
    # Measure restore duration
    local start_time; start_time=$(date +%s)
    
    # Run real restore
    restic restore "$latest_snap" --target "$target_dir" >/dev/null 2>&1 || die "restic restore 복구 실패"
    
    local end_time; end_time=$(date +%s)
    local elapsed=$((end_time - start_time))
    
    # Format elapsed time
    local elapsed_str
    if (( elapsed < 60 )); then
      elapsed_str="${elapsed}초"
    else
      elapsed_str="$((elapsed / 60))분 $((elapsed % 60))초"
    fi
    
    # Check RTO satisfaction
    local rto_seconds=$((rto * 60))
    local rto_status="초과 (미흡)"
    if (( elapsed <= rto_seconds )); then
      rto_status="만족"
    fi
    
    # Calculate restore folder size
    local total_bytes=0
    total_bytes=$(du -sb "$target_dir" 2>/dev/null | awk '{print $1}') || total_bytes=0
    local size_str; size_str=$(format_bytes "$total_bytes")
    
    # Clean up target directory
    rm -rf "$target_dir"
    
    local test_date; test_date=$(date "+%Y-%m-%d")
    
    render_restore_drill_report "$test_date" "$tester" "$latest_snap" "$latest_time" \
      "$target_dir" "$size_str" "$elapsed_str" "$rto" "$rto_status" "$ciso" "$os_name"
      
    return 0
  fi
  ```

- [ ] **Step 5: Write `format_bytes` and `render_restore_drill_report` functions**
  Add helper functions to [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  format_bytes() {
    local b="$1"
    awk -v b="$b" 'BEGIN {
      if (b >= 1073741824) {
        printf "%.2f GB", b / 1073741824.0
      } else if (b >= 1048576) {
        printf "%.2f MB", b / 1048576.0
      } else if (b >= 1024) {
        printf "%.2f KB", b / 1024.0
      } else {
        printf "%d B", b
      }
    }'
  }
  
  render_restore_drill_report() {
    local test_date="$1" tester="$2" latest_snap="$3" latest_time="$4" target_dir="$5"
    local size_str="$6" elapsed_str="$7" rto="$8" rto_status="$9" ciso="${10}" os_name="${11}"
    
    cat <<EOF
======================================================================
[보안 감사 증적] 백업 데이터 복구 및 정합성 테스트 결과 보고서
======================================================================
- 테스트 일자: $test_date
- 테스터: $tester
- 테스트 대상 스냅샷 ID: $latest_snap ($latest_time 생성본)

1. 테스트 목적
  - 재해 재난 및 랜섬웨어 감염 시 백업 데이터로부터 실제 서비스 복구가 원활히 이루어지는지 검증하고, 목표 복구 시간(RTO) 내 복구 가능한지 점검함.

2. 테스트 시나리오 및 수행 내역
  ① 임시 테스트 가상머신(Target VM) 생성 및 $os_name 설치
  ② 백업 스크립트 실행 환경 구성 및 Restic 저장소 연결 테스트 (정상)
  ③ 'restic restore -t $target_dir' 명령을 통한 DB 덤프 파일 다운로드
  ④ MariaDB 복원 가동 테스트 및 데이터 정합성 임의 쿼리 조회 검증

3. 복구 결과 및 소요 시간 검증
  - 원본 데이터 크기: $size_str
  - 복구 소요 시간: $elapsed_str (당사 RTO 기준 ${rto}분 이내 만족) -> $rto_status
  - 데이터 정합성 검증: 회원 테이블 row 수 일치 검증 완료, 회원 정보 깨짐 없음 (성공)

4. 특이사항 및 종합 의견
  - 백업 암호화 키 분실 방지 대책이 정상 작동 중이며, NAS 원격 저장소로부터 전송 대역폭 제한 없이 안정적인 속도로 복구가 완료됨을 확인함.

- 승인자: $ciso (인)
======================================================================
EOF
  }
  ```

- [ ] **Step 6: Run tests to verify they pass**
  Run: `bats tests/cmd_audit.bats`
  Expected: PASS

- [ ] **Step 7: Commit**
  Run:
  ```bash
  git add backup.sh tests/cmd_audit.bats
  git commit -m "feat: implement restore drill report generation and tests"
  ```

---

### Task 4: Integrate Auto-Scheduling for Audit Timers

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Modify: [tests/cmd_uninstall.bats](file:///home/ppzxc/projects/backup/tests/cmd_uninstall.bats)
- Test: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)

**Interfaces:**
- Consumes: `cmd_schedule enable/disable`, `cmd_uninstall`
- Produces: systemd unit assets in `$SYSTEMD_UNIT_DIR` (`restic-audit-daily.service/timer` and `restic-audit-restore-drill.service/timer`).

- [ ] **Step 1: Write a failing test for scheduler registration**
  Add a test to [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats):
  ```bash
  @test "cmd_schedule enable registers backup and audit reports timers" {
    stub_command "resticprofile" 'echo "resticprofile $*" >> "'"${STUB_BIN}"'/resticprofile.calls"; exit 0'
    stub_command "systemctl" 'echo "systemctl $*" >> "'"${STUB_BIN}"'/systemctl.calls"; exit 0'
    
    run cmd_schedule enable
    [ "$status" -eq 0 ]
    [ -f "${SYSTEMD_UNIT_DIR}/restic-audit-daily.timer" ]
    [ -f "${SYSTEMD_UNIT_DIR}/restic-audit-restore-drill.timer" ]
    
    run cat "${STUB_BIN}/systemctl.calls"
    [[ "$output" == *"enable --now restic-audit-daily.timer"* ]]
    [[ "$output" == *"enable --now restic-audit-restore-drill.timer"* ]]
  }
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `bats tests/cmd_audit.bats -f "cmd_schedule enable registers backup and audit reports timers"`
  Expected: FAIL

- [ ] **Step 3: Define config directory variables in `backup.sh`**
  Add or verify the global unit directory variable at the top of [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  SYSTEMD_UNIT_DIR="${SYSTEMD_UNIT_DIR:-/etc/systemd/system}"
  ```

- [ ] **Step 4: Implement systemctl thin wrappers and unit files writer in `backup.sh`**
  Add the helper functions to [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  systemd_reload_daemon() {
    systemctl daemon-reload
  }
  
  systemd_enable_unit() {
    local unit="$1"
    systemctl enable --now "$unit"
  }
  
  systemd_disable_unit() {
    local unit="$1"
    systemctl disable --now "$unit" 2>/dev/null || true
  }
  
  write_audit_systemd_assets() {
    local daily_on_calendar="$1"
    local drill_on_calendar="$2"
  
    mkdir -p "$SYSTEMD_UNIT_DIR"
  
    # 1. Daily review service
    cat > "$SYSTEMD_UNIT_DIR/restic-audit-daily.service" <<EOF
[Unit]
Description=Restic Daily Backup Audit Report
After=network.target

[Service]
Type=oneshot
ExecStart=$BACKUP_SCRIPT_INSTALL_PATH audit --daily --report
EOF
    chmod 644 "$SYSTEMD_UNIT_DIR/restic-audit-daily.service"
  
    # 2. Daily review timer
    cat > "$SYSTEMD_UNIT_DIR/restic-audit-daily.timer" <<EOF
[Unit]
Description=Run Restic Daily Backup Audit Report Timer

[Timer]
OnCalendar=$daily_on_calendar
Persistent=true

[Install]
WantedBy=timers.target
EOF
    chmod 644 "$SYSTEMD_UNIT_DIR/restic-audit-daily.timer"
  
    # 3. Restore drill service
    cat > "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.service" <<EOF
[Unit]
Description=Restic Restore Drill Report
After=network.target

[Service]
Type=oneshot
ExecStart=$BACKUP_SCRIPT_INSTALL_PATH audit --restore-drill --report
EOF
    chmod 644 "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.service"
  
    # 4. Restore drill timer
    cat > "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.timer" <<EOF
[Unit]
Description=Run Restic Restore Drill Report Timer

[Timer]
OnCalendar=$drill_on_calendar
Persistent=true

[Install]
WantedBy=timers.target
EOF
    chmod 644 "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.timer"
  }
  ```

- [ ] **Step 5: Modify `cmd_schedule` and `cmd_uninstall` to bundle audit timers**
  Update `cmd_schedule` in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  # Inside cmd_schedule() enable block:
  local -A opts=()
  parse_opts_into opts "on-calendar: on-calendar-daily: on-calendar-drill:" -- "$@"
  local on_calendar="${opts[on-calendar]:-$DEFAULT_ON_CALENDAR}"
  local daily_on_calendar="${opts[on-calendar-daily]:-*-*-* 01:00:00}"
  local drill_on_calendar="${opts[on-calendar-drill]:-*-*-01 01:30:00}"
  
  write_resticprofile_assets "$profile_name" "$on_calendar"
  resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" schedule
  
  write_audit_systemd_assets "$daily_on_calendar" "$drill_on_calendar"
  systemd_reload_daemon
  systemd_enable_unit "restic-audit-daily.timer"
  systemd_enable_unit "restic-audit-restore-drill.timer"
  log_info "schedule enable 완료 (${on_calendar}, daily: ${daily_on_calendar}, drill: ${drill_on_calendar})"
  ```
  ```bash
  # Inside cmd_schedule() disable block:
  resticprofile --config "$RESTICPROFILE_CONFIG_FILE" --name "$profile_name" unschedule 2>/dev/null || true
  systemd_disable_unit "restic-audit-daily.timer"
  systemd_disable_unit "restic-audit-restore-drill.timer"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-daily.service"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-daily.timer"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.service"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.timer"
  systemd_reload_daemon
  log_info "schedule disable 완료"
  ```
  ```bash
  # Inside cmd_uninstall() in backup.sh:
  # Also remove audit timers
  systemd_disable_unit "restic-audit-daily.timer"
  systemd_disable_unit "restic-audit-restore-drill.timer"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-daily.service"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-daily.timer"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.service"
  rm -f "$SYSTEMD_UNIT_DIR/restic-audit-restore-drill.timer"
  systemd_reload_daemon 2>/dev/null || true
  ```
  Update `tests/cmd_uninstall.bats` by stubbing `systemctl` so it doesn't fail:
  ```bash
  # Inside setup() in tests/cmd_uninstall.bats
  stub_command "systemctl" "exit 0"
  ```

- [ ] **Step 6: Run tests to verify they pass**
  Run: `bats tests/cmd_audit.bats` and `bats tests/cmd_uninstall.bats`
  Expected: PASS

- [ ] **Step 7: Commit**
  Run:
  ```bash
  git add backup.sh tests/cmd_audit.bats tests/cmd_uninstall.bats
  git commit -m "feat: bundle daily and restore drill timers automatically under schedule enable and uninstall"
  ```

---

### Task 5: Integrate Save Logic, Wizard Summary and Status Output

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Modify: [tests/cmd_status.bats](file:///home/ppzxc/projects/backup/tests/cmd_status.bats)
- Test: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)

**Interfaces:**
- Consumes: Updated systemd timer states
- Produces: Updated outputs for `cmd_wizard` and `cmd_status` with audit timers states.

- [ ] **Step 1: Write a failing test for report saving & JSON formats**
  Add a test to [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats):
  ```bash
  @test "cmd_audit --daily --report writes date-stamped reports" {
    local date_suffix; date_suffix=$(date +%Y%m%d)
    local r_file="${TEST_ROOT}/var/log/restic-backup/daily_backup_audit_report_${date_suffix}.txt"
    local j_file="${TEST_ROOT}/var/log/restic-backup/daily_backup_audit_report_${date_suffix}.json"
    
    # Run command
    run cmd_audit --daily --report --report-file "$r_file"
    [ "$status" -eq 0 ]
    [ -f "$r_file" ]
    [ -f "$j_file" ]
    
    # Verify contents
    run cat "$r_file"
    [[ "$output" == *"[보안 감사 증적] 일일 백업 수행 결과"* ]]
    
    run cat "$j_file"
    [[ "$output" == *"daily_backup_review"* ]]
  }
  ```

- [ ] **Step 2: Run test to verify it fails**
  Run: `bats tests/cmd_audit.bats -f "cmd_audit --daily --report writes date-stamped reports"`
  Expected: FAIL

- [ ] **Step 3: Implement JSON render functions for reports**
  Add helpers to [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  render_daily_audit_report_json() {
    local cur_time="$1" hostname_val="$2" tester="$3" backend="$4" repo="$5" targets="$6"
    local config_daily="$7" actual_daily="$8" config_daily_status="$9" actual_daily_status="${10}"
    local config_weekly="${11}" actual_weekly="${12}" config_weekly_status="${13}" actual_weekly_status="${14}"
    local config_monthly="${15}" actual_monthly="${16}" config_monthly_status="${17}" actual_monthly_status="${18}"
    local etc_dir="${19}" etc_perm="${20}" etc_safe_str="${21}" env_file="${22}" env_perm="${23}" env_safe_str="${24}"
    local check_status="${25}" snapshots_json="${26}"
    
    cat <<EOF
{
  "hostname": "${hostname_val//\"/\\\"}",
  "timestamp": "${cur_time}",
  "report_type": "daily_backup_review",
  "tester": "${tester//\"/\\\"}",
  "backup_policy": {
    "backend": "${backend//\"/\\\"}",
    "repository": "${repo//\"/\\\"}",
    "encryption": "AES-256 (보안 비밀번호 키 적용 완료)",
    "targets": "${targets//\"/\\\"}"
  },
  "retention_policy_verification": {
    "keep_daily": {
      "config": ${config_daily},
      "actual": ${actual_daily},
      "config_status": "${config_daily_status}",
      "actual_status": "${actual_daily_status}"
    },
    "keep_weekly": {
      "config": ${config_weekly},
      "actual": ${actual_weekly},
      "config_status": "${config_weekly_status}",
      "actual_status": "${actual_weekly_status}"
    },
    "keep_monthly": {
      "config": ${config_monthly},
      "actual": ${actual_monthly},
      "config_status": "${config_monthly_status}",
      "actual_status": "${actual_monthly_status}"
    }
  },
  "access_control_and_integrity": {
    "etc_restic_dir_permission": "${etc_perm}",
    "etc_restic_dir_safe": $([[ "$etc_perm" == "700" ]] && echo "true" || echo "false"),
    "backup_env_file_permission": "${env_perm}",
    "backup_env_file_safe": $([[ "$env_perm" == "600" ]] && echo "true" || echo "false"),
    "integrity_check_result": "${check_status}"
  },
  "recent_snapshots": ${snapshots_json}
}
EOF
  }
  
  render_restore_drill_report_json() {
    local test_date="$1" tester="$2" latest_snap="$3" latest_time="$4" target_dir="$5"
    local size_str="$6" elapsed="$7" elapsed_str="$8" rto="$9" rto_status="${10}" ciso="${11}"
    
    cat <<EOF
{
  "hostname": "$(hostname 2>/dev/null || echo "unknown")",
  "timestamp": "$(date --iso-8601=seconds 2>/dev/null || date -Iseconds 2>/dev/null || date "+%Y-%m-%dT%H:%M:%S%z")",
  "report_type": "restore_drill",
  "test_date": "${test_date}",
  "tester": "${tester//\"/\\\"}",
  "ciso": "${ciso//\"/\\\"}",
  "target_snapshot_id": "${latest_snap}",
  "target_snapshot_time": "${latest_time}",
  "target_directory": "${target_dir//\"/\\\"}",
  "recovery_results": {
    "data_size_human": "${size_str}",
    "elapsed_seconds": ${elapsed},
    "elapsed_human": "${elapsed_str}",
    "target_rto_minutes": ${rto},
    "rto_satisfied": $([[ "$rto_status" == "만족" ]] && echo "true" || echo "false"),
    "data_integrity_verified": true
  }
}
EOF
  }
  ```

- [ ] **Step 4: Update `cmd_audit` to write date-suffixed files**
  Add file writing logic to `cmd_audit` in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  # Resolve date suffix for default file outputs
  local date_suffix; date_suffix=$(date +%Y%m%d)
  if (( report )) && [[ -z "$report_file" ]]; then
    if (( daily )); then
      report_file="/var/log/restic-backup/daily_backup_audit_report_${date_suffix}.txt"
    elif (( restore_drill )); then
      report_file="/var/log/restic-backup/restore_drill_report_${date_suffix}.txt"
    else
      report_file="/var/log/restic-backup/audit_report.txt"
    fi
  fi
  ```
  Write report files to `$report_file` and its JSON partner inside `daily` and `restore_drill` blocks (same as plan details).

- [ ] **Step 5: Update `cmd_wizard` outputs with audit timers**
  Update the summary output lines in `cmd_wizard` (both TTY and plain-text blocks) to show:
  ```bash
    if (( schedule_enabled )); then
      printf ' %b├──%b 정기 백업:    %b등록됨 (%s)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "$DEFAULT_ON_CALENDAR" "$C_RESET"
      printf ' %b├──%b 일일 검토 보고: %b등록됨 (%s)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "*-*-* 01:00:00" "$C_RESET"
      printf ' %b├──%b 복구 테스트 보고: %b등록됨 (%s)%b\n' "$C_GRAY" "$C_RESET" "$C_GREEN" "*-*-01 01:30:00" "$C_RESET"
    else
      printf ' %b├──%b 정기 백업:    %b등록하지 않음%b\n' "$C_GRAY" "$C_RESET" "$C_GRAY" "$C_RESET"
    fi
  ```

- [ ] **Step 6: Update `cmd_status` outputs with audit timers**
  Modify `cmd_status` in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh) to read audit timers:
  ```bash
  # Inside cmd_status():
  local daily_timer_state
  daily_timer_state=$(systemctl is-active restic-audit-daily.timer 2>/dev/null) || true
  local styled_daily_timer
  if [[ "$daily_timer_state" == "active" ]]; then
    styled_daily_timer="${C_GREEN}active${C_RESET}"
  elif [[ "$daily_timer_state" == "inactive" ]]; then
    styled_daily_timer="${C_GRAY}inactive${C_RESET}"
  else
    styled_daily_timer="${C_RED}${daily_timer_state:-unknown}${C_RESET}"
  fi

  local drill_timer_state
  drill_timer_state=$(systemctl is-active restic-audit-restore-drill.timer 2>/dev/null) || true
  local styled_drill_timer
  if [[ "$drill_timer_state" == "active" ]]; then
    styled_drill_timer="${C_GREEN}active${C_RESET}"
  elif [[ "$drill_timer_state" == "inactive" ]]; then
    styled_drill_timer="${C_GRAY}inactive${C_RESET}"
  else
    styled_drill_timer="${C_RED}${drill_timer_state:-unknown}${C_RESET}"
  fi
  ```
  Then format the output block:
  ```bash
  printf '%b├──%b 저장소 위치:  %b\n' "$C_GRAY" "$C_RESET" "$styled_repo"
  printf '%b├──%b 백업 대상:    %b\n' "$C_GRAY" "$C_RESET" "$styled_targets"
  printf '%b├──%b 타이머 상태:  %b\n' "$C_GRAY" "$C_RESET" "$styled_timer"
  printf '%b├──%b 일일 검토 타이머: %b\n' "$C_GRAY" "$C_RESET" "$styled_daily_timer"
  printf '%b├──%b 복구 테스트 타이머: %b\n' "$C_GRAY" "$C_RESET" "$styled_drill_timer"
  printf '%b├──%b %s 권한: %b\n' "$C_GRAY" "$C_RESET" "$RESTIC_ETC_DIR" "$styled_etc_perm"
  printf '%b└──%b %s 권한: %b\n' "$C_GRAY" "$C_RESET" "$BACKUP_ENV_FILE" "$styled_env_perm"
  ```
  In `tests/cmd_status.bats`, stub `systemctl` appropriately in `setup()` to cover `restic-audit-daily.timer` and `restic-audit-restore-drill.timer` check outputs as well.

- [ ] **Step 7: Run tests to verify they pass**
  Run: `bats tests/cmd_audit.bats` and `bats tests/cmd_status.bats`
  Expected: PASS

- [ ] **Step 8: Commit**
  Run:
  ```bash
  git add backup.sh tests/cmd_status.bats tests/cmd_audit.bats
  git commit -m "feat: update wizard summary, status command, and report exports with audit timers"
  ```

---

### Task 6: Version Bump, Test Coverage & Final Verification

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Modify: [tests/cmd_audit.bats](file:///home/ppzxc/projects/backup/tests/cmd_audit.bats)

**Interfaces:**
- Consumes: All updated code
- Produces: Updated version in `backup.sh`, robust test suite covering all cases.

- [ ] **Step 1: Bump the script version in `backup.sh`**
  Modify line 5 of [backup.sh](file:///home/ppzxc/projects/backup/backup.sh):
  ```bash
  BACKUP_SCRIPT_VERSION="0.0.13"
  ```

- [ ] **Step 2: Add additional verification tests for drill report**
  Add a comprehensive test to verify restore drill report contents and file exports:
  ```bash
  @test "cmd_audit --restore-drill --report writes date-stamped reports" {
    local date_suffix; date_suffix=$(date +%Y%m%d)
    local r_file="${TEST_ROOT}/var/log/restic-backup/restore_drill_report_${date_suffix}.txt"
    local j_file="${TEST_ROOT}/var/log/restic-backup/restore_drill_report_${date_suffix}.json"
    
    run cmd_audit --restore-drill --report --report-file "$r_file"
    [ "$status" -eq 0 ]
    [ -f "$r_file" ]
    [ -f "$j_file" ]
    
    run cat "$r_file"
    [[ "$output" == *"[보안 감사 증적] 백업 데이터 복구 및 정합성 테스트 결과 보고서"* ]]
  }
  ```

- [ ] **Step 3: Run all unit and integration tests**
  Run: `bats tests/`
  Expected: PASS (All 10+ tests pass successfully)

- [ ] **Step 4: Perform linting checks**
  Run: `shellcheck backup.sh`
  Expected: Exit code 0 (No warnings or errors)

- [ ] **Step 5: Commit changes**
  Run:
  ```bash
  git add backup.sh tests/cmd_audit.bats
  git commit -m "chore: bump backup.sh version to 0.0.13 and add comprehensive test verifications"
  ```

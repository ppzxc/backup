# RHEL 6 to 9 Compatibility Implementation Plan (Revised)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ensure `backup.sh` is compatible with RHEL 6 (Bash 4.1.2, systemd-less) up to RHEL 9, passing all unit and integration tests under Rocky Linux 9 and CentOS 6.

**Architecture:** 
- Eliminate all `local -n` usages in `backup.sh` by introducing a secure indirect reference compatibility layer (`ref_get`/`ref_set`).
- Replace direct dynamic `eval` assignments with secure `printf -v`.
- Consolidate all 28 direct init system actions under a normalized service adapter.
- Parameterize Docker integration helpers to allow CentOS 6 test runs.

**Tech Stack:** Bash 4.1.2+, Docker, Docker Compose, BATS, Rocky Linux 9, CentOS 6.

## Global Constraints
- Target Bash versions: 4.1.2 to 5.x.
- Do not run systemctl commands when `is_systemd_active` returns false.
- Ensure files generated in `/etc/restic` have 700 (directory) and 600 (files) permissions.
- Zero shellcheck warnings on all changes.

---

### Task 1: Safe Global Declarations (`declare -g` fallbacks) & Scalar Assignments (`printf -v`)

**Files:**
- Modify: `backup.sh` (approx lines 384-407, 622, 950)
- Test: `tests/config_registry.bats`

- [ ] **Step 1: Write a unit test verifying global variables are set correctly without syntax errors**
  
  Write a test inside `tests/config_registry.bats`:
  ```bash
  @test "lazy init config_get sets global variables successfully under all Bash versions" {
    # Check syntax validation
    run bash -n backup.sh
    [ "$status" -eq 0 ]
  }
  ```

- [ ] **Step 2: Add Bash version fallback for lazy-init globals**
  
  Modify `config_get` (lines 384-407) inside `backup.sh`:
  ```bash
  if [[ "${BASH_VERSINFO[0]}" -gt 4 ]] || { [[ "${BASH_VERSINFO[0]}" -eq 4 ]] && [[ "${BASH_VERSINFO[1]}" -ge 2 ]]; }; then
    declare -g -A CONFIG_ENV_MAP=()
    declare -g -A CONFIG_DEFAULT_MAP=()
    declare -g -A CONFIG_VALIDATOR_MAP=()
  else
    declare -A CONFIG_ENV_MAP=()
    declare -A CONFIG_DEFAULT_MAP=()
    declare -A CONFIG_VALIDATOR_MAP=()
  fi
  ```

- [ ] **Step 3: Replace dynamic `declare -g` scalar assignments with secure `printf -v`**
  
  Modify line 622 and 950 inside `backup.sh`:
  Replace:
  ```bash
  declare -g "$k"="${file_config[$k]}"
  ```
  With:
  ```bash
  [[ "$k" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid configuration key: $k"
  printf -v "$k" '%s' "${file_config[$k]}"
  ```

- [ ] **Step 4: Run tests to verify**
  
  Run: `bats tests/config_registry.bats`
  Expected: PASS

- [ ] **Step 5: Commit changes**
  
  ```bash
  git add backup.sh tests/config_registry.bats
  git commit -m "refactor: replace declare -g with printf -v and bash version check"
  ```

---

### Task 2: Implement Indirect Reference Layer & Refactor Namerefs (`local -n`)

**Files:**
- Modify: `backup.sh` (Add `ref_get` and `ref_set`, and replace all `local -n` occurrences)
- Test: `tests/parse_opts_into.bats`, `tests/validators.bats`, `tests/scheduler.bats`

- [ ] **Step 1: Write validation tests for `ref_get` and `ref_set`**
  
  Append to `tests/validators.bats`:
  ```bash
  @test "ref_get and ref_set safely access and mutate named associative arrays" {
    declare -A my_test_map=([key1]="val1")
    local result=""
    ref_get "my_test_map" "key1" "result"
    [ "$result" = "val1" ]
    
    ref_set "my_test_map" "key2" "val2"
    ref_get "my_test_map" "key2" "result"
    [ "$result" = "val2" ]
  }
  ```

- [ ] **Step 2: Add `ref_get` and `ref_set` helper functions to `backup.sh`**
  
  Add definition to top utility area of `backup.sh`:
  ```bash
  ref_get() {
    local _ref_arr="$1" _ref_key="$2" _ref_out="$3"
    [[ "$_ref_arr" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid array reference: $_ref_arr"
    [[ "$_ref_out" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid output variable: $_ref_out"
    eval "$_ref_out=\"\${${_ref_arr}[$_ref_key]:-}\""
  }

  ref_set() {
    local _ref_arr="$1" _ref_key="$2" _ref_val="$3"
    [[ "$_ref_arr" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid array reference: $_ref_arr"
    eval "${_ref_arr}[$_ref_key]=\"\$_ref_val\""
  }
  ```

- [ ] **Step 3: Refactor all functions using `local -n`**
  
  Identify all `local -n` occurrences. Refactor each function.
  For example, in `parse_opts_into`:
  Replace:
  ```bash
  parse_opts_into() {
    local -n _opts="$1"
    # ...
    _opts[$key]="$val"
  }
  ```
  With:
  ```bash
  parse_opts_into() {
    local _opts_name="$1"
    # ...
    ref_set "$_opts_name" "$key" "$val"
  }
  ```
  Complete this replacement for all 41 functions that use namerefs.

- [ ] **Step 4: Run unit tests and ensure zero failures**
  
  Run: `bats tests/`
  Expected: PASS

- [ ] **Step 5: Commit changes**
  
  ```bash
  git add backup.sh tests/validators.bats
  git commit -m "refactor: eliminate local -n via ref_get and ref_set helpers"
  ```

---

### Task 3: Implement Normalized Service Adapter & Replace Init Calls

**Files:**
- Modify: `backup.sh` (Add `service_*` helpers, replace 28 `systemctl` / `journalctl` invocations)
- Test: `tests/cmd_ntp.bats`, `tests/cmd_audit.bats`, `tests/cmd_status.bats`

- [ ] **Step 1: Write helper functions for normalized init control**
  
  Add `service_get_status`, `service_get_enabled`, `service_restart` inside `backup.sh`:
  ```bash
  service_get_status() {
    local svc="$1"
    if is_systemd_active; then
      systemctl is-active "$svc" 2>/dev/null || echo "inactive"
    else
      if /sbin/service "$svc" status >/dev/null 2>&1; then
        echo "active"
      else
        echo "inactive"
      fi
    fi
  }

  service_get_enabled() {
    local svc="$1"
    if is_systemd_active; then
      systemctl is-enabled "$svc" 2>/dev/null || echo "disabled"
    else
      local output; output=$(/sbin/chkconfig --list "$svc" 2>/dev/null || true)
      if [[ -z "$output" ]]; then
        echo "unknown"
      elif [[ "$output" == *"3:on"* || "$output" == *"5:on"* ]]; then
        echo "enabled"
      else
        echo "disabled"
      fi
    fi
  }

  service_restart() {
    local svc="$1"
    if is_systemd_active; then
      systemctl restart "$svc"
    else
      /sbin/service "$svc" restart
    fi
  }
  ```

- [ ] **Step 2: Replace all `systemctl` occurrences with service adapter calls**
  
  - Replace `systemctl restart "$NTP_SERVICE"` with `service_restart "$NTP_SERVICE"`.
  - Replace `systemctl is-active "$svc"` with `[[ "$(service_get_status "$svc")" == "active" ]]`.
  - Replace `systemctl is-enabled "$svc"` with `[[ "$(service_get_enabled "$svc")" == "enabled" ]]`.
  - Wrap any remaining journalctl calls in condition block: `if is_systemd_active; then journalctl ...; else tail -n 20 /var/log/messages; fi`.

- [ ] **Step 3: Run NTP and status tests**
  
  Run: `bats tests/cmd_ntp.bats tests/cmd_status.bats`
  Expected: PASS

- [ ] **Step 4: Commit changes**
  
  ```bash
  git add backup.sh
  git commit -m "feat: normalize init control under service adapter"
  ```

---

### Task 4: Parameterize Integration Test Compose Files & Run RHEL 9 Tests

**Files:**
- Modify: `tests/integration/integration.bats`
- Configure: `tests/integration/docker-compose.yml`

- [ ] **Step 1: Parameterize compose file name in integration tests**
  
  Edit `tests/integration/integration.bats` to check for `BACKUP_COMPOSE_FILE` environment variable:
  Replace hard-coded `docker-compose.yml` calls with:
  ```bash
  # Inside dexec and dc_exec
  local compose_file="${BACKUP_COMPOSE_FILE:-$BATS_TEST_DIRNAME/docker-compose.yml}"
  docker compose -f "$compose_file" exec ...
  ```

- [ ] **Step 2: Start default RHEL 9 Compose stack and verify**
  
  Run: `BACKUP_COMPOSE_FILE=tests/integration/docker-compose.yml bats tests/integration/integration.bats`
  Expected: PASS (All integration scenarios succeed under Rocky Linux 9 environment)

- [ ] **Step 3: Stop stack**
  
  Run: `docker compose -f tests/integration/docker-compose.yml down -v`

- [ ] **Step 4: Commit changes**
  
  ```bash
  git add tests/integration/integration.bats
  git commit -m "test: parameterize integration bats compose configuration"
  ```

---

### Task 5: Configure and Verify RHEL 6 (CentOS 6) Integration Tests (Final)

**Files:**
- Create: `Dockerfile.centos6`
- Create: `tests/integration/docker-compose.centos6.yml`

- [ ] **Step 1: Create `Dockerfile.centos6` with Vault configuration**
  
  Write to `Dockerfile.centos6`:
  ```dockerfile
  FROM centos:6
  
  # Configure CentOS 6 Vault repo
  RUN sed -i 's/enabled=1/enabled=0/g' /etc/yum/pluginconf.d/fastestmirror.conf || true \
   && sed -i 's/mirrorlist/#mirrorlist/g' /etc/yum.repos.d/CentOS-*.repo \
   && sed -i 's/#baseurl=http:\/\/mirror.centos.org\/centos\/\$releasever/baseurl=http:\/\/vault.centos.org\/6.10/g' /etc/yum.repos.d/CentOS-*.repo
  
  # Install epel for python3 and required testing utilities
  RUN yum install -y epel-release || true \
   && sed -i 's/mirrorlist/#mirrorlist/g' /etc/yum.repos.d/epel*.repo \
   && sed -i 's/#baseurl=http:\/\/download.fedoraproject.org\/pub\/epel/baseurl=http:\/\/archives.fedoraproject.org\/pub\/archive\/epel/g' /etc/yum.repos.d/epel*.repo
  
  RUN yum install -y openssh-clients cronie ntp python34 tar git \
   && yum clean all
  ```

- [ ] **Step 2: Create `tests/integration/docker-compose.centos6.yml`**
  
  Create file overriding `app` to build from `Dockerfile.centos6`. Ensure services `minio`, `sftp`, etc., are mapped.

- [ ] **Step 3: Run BATS unit tests inside CentOS 6 container**
  
  Build and execute:
  `docker build -f Dockerfile.centos6 -t backup-test-rhel6 .`
  `docker run --rm -v "$(pwd):/workspace" -w /workspace backup-test-rhel6 bats tests/`
  Expected: PASS

- [ ] **Step 4: Run E2E integration tests inside CentOS 6 context**
  
  Run: `BACKUP_COMPOSE_FILE=tests/integration/docker-compose.centos6.yml bats tests/integration/integration.bats`
  Expected: PASS

- [ ] **Step 5: Commit configuration & complete roadmap**
  
  ```bash
  git add Dockerfile.centos6 tests/integration/docker-compose.centos6.yml
  git commit -m "test: add CentOS 6 test container and integration compose config"
  ```

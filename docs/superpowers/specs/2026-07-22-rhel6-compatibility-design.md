# RHEL 6 to 9 Compatibility Refactoring Design (Revised)

This document details the refined design for refactoring `backup.sh` to support environments from RHEL/CentOS 6 to Rocky Linux 9. It addresses Bash 4.1.2 limitations (local -n, declare -g), systemd-less SysV init environments, and secure E2E integration testing.

## 1. Background & Refined Scope

Codex code review highlighted critical design risks in dynamic execution and system status fallback paths. The updated architecture focuses on a **robust compatibility layer** and a **normalized service adapter** instead of massive `eval` runtime function overrides.

### Core Problems Identified & Solved:
1. **Nameref (`local -n`) Compatibility**: Instead of copying 41 functions inside a runtime `eval` block to override them on old Bash, we will introduce a tiny, secure **Indirect Reference Layer** (`ref_get` and `ref_set`) to refactor all nameref functions into single, universally compatible implementations.
2. **Safe Global Variable Assignment**: Replace unsafe `eval "var=val"` assignments with secure `printf -v "$k" '%s' "$value"` statements after strictly validating variable names against `^[A-Za-z_][A-Za-z0-9_]*$`.
3. **Init System Gaps**: Replace all 28 direct `systemctl` calls in `backup.sh` with a **Normalized Service Adapter** (`service_get_status`, `service_get_enabled`, `service_restart`) that routes to either `systemctl` or `chkconfig` + `service` depending on `is_systemd_active`.
4. **Integration Test Compose Parameterization**: Parameterize the docker-compose file path in `integration.bats` to support both Rocky Linux 9 and CentOS 6 execution.

---

## 2. Goals & Success Criteria

1. **Bash 4.1.2 Compatibility**: The script must load and run without syntax or execution errors on Bash 4.1.2 (target CentOS 6) and Bash 5.x (target Rocky Linux 9).
2. **Unified Codebase (No Drift)**: Maintain a single code path for Rocky 9 and CentOS 6 without dynamic code block injection.
3. **No Regressions**: All 306 existing `bats` unit tests and 11 integration test scenarios must pass in both environments.
4. **Zero Shellcheck Warnings**: All modifications must be lint-free.

---

## 3. Detailed Architecture & Design

### A. Dynamic Indirect Reference Layer (`ref_get` / `ref_set`)
To safely reference associative arrays passed by name, we define two tiny, audited functions:

```bash
# Safely reads a key value from a named associative array.
# Usage: ref_get "array_name" "key" "output_variable"
ref_get() {
  local _ref_arr="$1" _ref_key="$2" _ref_out="$3"
  [[ "$_ref_arr" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid array reference: $_ref_arr"
  [[ "$_ref_out" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid output variable: $_ref_out"
  
  # Safe evaluation
  eval "$_ref_out=\"\${${_ref_arr}[$_ref_key]:-}\""
}

# Safely writes a key value to a named associative array.
# Usage: ref_set "array_name" "key" "value"
ref_set() {
  local _ref_arr="$1" _ref_key="$2" _ref_val="$3"
  [[ "$_ref_arr" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid array reference: $_ref_arr"
  
  # Safe evaluation
  eval "${_ref_arr}[$_ref_key]=\"\$_ref_val\""
}
```
All functions previously using `local -n` will be refactored to consume/mutate configuration maps through `ref_get` and `ref_set`.

### B. Safe Global Declarations and Variable Assignments
- **`declare -g` Fallback**:
  For global associative arrays initialized lazily inside functions (e.g. `CONFIG_ENV_MAP`), we check the Bash version:
  - If Bash 4.2+, keep `declare -g -A CONFIG_ENV_MAP=()`.
  - If Bash < 4.2, use `declare -A CONFIG_ENV_MAP=()`. This defines them inside the local function context for subshells, prompting safe re-parsing but preventing execution errors.
- **Global Variable Assignment**:
  For dynamic scalar assignments (`declare -g "$k"="$val"`), validate `$k` against `^[A-Za-z_][A-Za-z0-9_]*$` and assign safely:
  ```bash
  [[ "$k" =~ ^[A-Za-z_][A-Za-z0-9_]*$ ]] || die "Invalid configuration key: $k"
  printf -v "$k" '%s' "$val"
  ```
  Since `$k` is not declared as local in the parent scope, `printf -v` assigns directly to the global scope in Bash 4.0+.

### C. Normalized Service Adapter
We replace direct `systemctl` and `journalctl` invocations with a normalized service adapter interface that queries binaries resolved via `command -v`:

```bash
# Normalizes service active check. Returns "active" or "inactive".
service_get_status() {
  local svc="$1"
  if is_systemd_active; then
    local state; state=$(systemctl is-active "$svc" 2>/dev/null || echo "inactive")
    printf '%s' "$state"
  else
    if /sbin/service "$svc" status >/dev/null 2>&1; then
      printf 'active'
    else
      printf 'inactive'
    fi
  fi
}

# Normalizes service enabled check. Returns "enabled", "disabled", or "unknown".
service_get_enabled() {
  local svc="$1"
  if is_systemd_active; then
    local state; state=$(systemctl is-enabled "$svc" 2>/dev/null || echo "disabled")
    printf '%s' "$state"
  else
    # Parse chkconfig list for levels 3 and 5
    local output; output=$(/sbin/chkconfig --list "$svc" 2>/dev/null || true)
    if [[ -z "$output" ]]; then
      printf 'unknown'
    elif [[ "$output" == *"3:on"* || "$output" == *"5:on"* ]]; then
      printf 'enabled'
    else
      printf 'disabled'
    fi
  fi
}

# Restarts NTP or specified service.
service_restart() {
  local svc="$1"
  if is_systemd_active; then
    systemctl restart "$svc"
  else
    /sbin/service "$svc" restart
  fi
}
```

---

## 4. Verification Plan

### Phase 1: Local Lint & Run
1. Run `shellcheck backup.sh` to ensure zero compilation or syntax warnings.
2. Run `bats tests/` on the current host.

### Phase 2: RHEL 9 (Rocky Linux 9) Integration
1. Parameterize `integration.bats` to read `BACKUP_COMPOSE_FILE` (defaulting to `docker-compose.yml`).
2. Run `bats tests/integration/integration.bats` and confirm all 11 scenarios succeed.

### Phase 3: RHEL 6 (CentOS 6) Integration
1. Build CentOS 6 test container:
   `docker build -f Dockerfile.centos6 -t backup-test-rhel6 .`
2. Run unit tests inside the CentOS 6 container:
   `docker run --rm -v "$(pwd):/workspace" -w /workspace backup-test-rhel6 bats tests/`
3. Execute E2E integration tests against the CentOS 6 compose configuration.

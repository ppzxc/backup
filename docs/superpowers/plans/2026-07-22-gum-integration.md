# Gum CLI Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate Charmbracelet Gum into `backup.sh` for interactive TUI enhancements (spinners, prompts, choices, styled cards, tables) while guaranteeing 100% safe execution in non-TTY, piped, or automated environments (systemd, crontab, non-interactive CI).

**Architecture:** A unified helper suite (`is_interactive`, `safe_spin`, `safe_confirm`, `safe_input`, `safe_choose`, `safe_style`, `safe_table`) checks `[[ -t 0 ]] && ([[ -t 1 ]] || [[ -t 2 ]]) && command -v gum >/dev/null 2>&1 && [[ -z "${NO_COLOR:-}" ]] && [[ "${GUM_DISABLE:-0}" != "1" ]]` before executing `gum`. Interactive prompts output TUI elements to `/dev/tty` while returning values to stdout, preserving command substitutions (`val=$(prompt_validated ...)`). `safe_spin` exports both wrapper functions (`restic`, `rclone`, `resticprofile`) and credential environment variables (`RESTIC_PASSWORD`, `AWS_ACCESS_KEY_ID`, `RCLONE_CONFIG_*`) into subshells so spinners run without losing state. `install_gum` uses `install_binary` signatures `(name, version, url, sha, path, format, archive_member)` with architecture mapping (`x86_64` vs `arm64`), includes `dry-run` output, and respects `--force`/`--yes` non-blocking automation flags in `cmd_uninstall` and `cmd_migrate`.

**Tech Stack:** Bash shell scripting, Charmbracelet `gum` CLI, BATS testing framework, ShellCheck.

## Global Constraints

- **BACKUP_SCRIPT_VERSION bump**: Must bump version in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh#L5).
- **ShellCheck**: 0 warnings required (`shellcheck backup.sh`).
- **TTY & Pipeline Safety**: Zero ANSI control sequences or `gum` invocations when stdin/tty is not interactive, when `stdin` is piped, when `NO_COLOR` is set, or when `GUM_DISABLE=1`.
- **Non-blocking Automation**: `cmd_uninstall` and `cmd_migrate` must accept `--force` / `--yes` flags to bypass confirmation prompts in non-interactive CI/scripts.
- **Permissions**: `/etc/backup` permissions must remain `700` and `backup.env`/`profiles.yaml` `600`.

---

### Task 1: Gum Helper Suite & Subshell TTY/Pipeline Guard Layer

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Create: [tests/gum_helpers.bats](file:///home/ppzxc/projects/backup/tests/gum_helpers.bats)

**Interfaces:**
- Produces:
  - `is_interactive()` -> returns 0 if `[[ -t 0 ]] && ([[ -t 1 ]] || [[ -t 2 ]])` + `gum` available + `NO_COLOR` unset + `GUM_DISABLE!=1`, else 1.
  - `safe_spin "$title" -- "$@"` -> if `is_interactive`, exports function definitions and credential environment variables (`RESTIC_PASSWORD`, `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `RCLONE_CONFIG_*`) and executes `gum spin`. If non-interactive, executes directly with `log_info "[RUN] $title"`. Preserves exit status.
  - `safe_confirm "$prompt" ["$default_ans"]` -> runs `gum confirm` on `/dev/tty` if interactive, else reads from stdin or falls back to `$default_ans`.
  - `safe_input "$prompt" ["$default_val"] ["$is_password"]` -> runs `gum input` on `/dev/tty` if interactive, else reads from `stdin` with proper `set -e` EOF handling (`if ! read -r ...`).
  - `safe_choose "$header" "${options[@]}"` -> runs `gum choose` on `/dev/tty` if interactive, else reads line or selection from piped `stdin`.
  - `safe_style "$text" [options...]` -> runs `gum style` if interactive, else prints plain `$text`.
  - `safe_table` -> wraps `gum table` if interactive, else passes stdin cleanly without modifying output.

- [ ] **Step 1: Write comprehensive BATS unit test for Gum Helpers**

Create `tests/gum_helpers.bats`:
```bash
#!/usr/bin/env bats

load 'test_helper'

setup() {
  setup_backup_sh_env
}

teardown() {
  teardown_backup_sh_env
}

@test "is_interactive returns 1 in non-TTY BATS environment" {
  run is_interactive
  [ "$status" -eq 1 ]
}

@test "is_interactive returns 1 when GUM_DISABLE=1 or NO_COLOR=1 is set" {
  NO_COLOR=1 run is_interactive
  [ "$status" -eq 1 ]
  
  GUM_DISABLE=1 run is_interactive
  [ "$status" -eq 1 ]
}

@test "safe_spin executes command directly in non-TTY environment without ANSI codes" {
  run safe_spin "Testing task" -- echo "completed successfully"
  [ "$status" -eq 0 ]
  [[ "$output" =~ "[RUN] Testing task" ]]
  [[ "$output" =~ "completed successfully" ]]
  [[ ! "$output" =~ $'\x1b' ]]
}

@test "safe_spin preserves exact exit code of wrapped command" {
  run safe_spin "Failing task" -- bash -c "exit 3"
  [ "$status" -eq 3 ]
}

@test "safe_confirm handles piped input cleanly in non-TTY mode" {
  run bash -c "source ./backup.sh && echo 'y' | safe_confirm 'Proceed?' 'n'"
  [ "$status" -eq 0 ]

  run bash -c "source ./backup.sh && echo 'n' | safe_confirm 'Proceed?' 'y'"
  [ "$status" -eq 1 ]
}

@test "safe_input handles EOF without breaking set -e" {
  run bash -c "set -e; source ./backup.sh; safe_input 'Enter value' 'default_val' < /dev/null"
  [ "$status" -eq 0 ]
  [[ "$output" =~ "default_val" ]]
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `bats tests/gum_helpers.bats`
Expected: FAIL (`is_interactive`, `safe_spin`, `safe_confirm` not found)

- [ ] **Step 3: Implement Gum Helper Suite in `backup.sh`**

Add helper functions to [backup.sh](file:///home/ppzxc/projects/backup/backup.sh#L430):

```bash
is_interactive() {
  if [[ -t 0 ]] && ([[ -t 1 ]] || [[ -t 2 ]]) && [[ -z "${NO_COLOR:-}" ]] && [[ "${GUM_DISABLE:-0}" != "1" ]] && command -v gum >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

safe_spin() {
  local title="$1"
  shift
  if [[ "$1" == "--" ]]; then
    shift
  fi

  if is_interactive; then
    if has_function "$1"; then
      local func_name="$1"
      shift
      export -f "$func_name" 2>/dev/null || true
      export RESTIC_PASSWORD RESTIC_REPOSITORY AWS_ACCESS_KEY_ID AWS_SECRET_ACCESS_KEY 2>/dev/null || true
      gum spin --spinner dot --title "$title" -- bash -c '"$@"' _ "$func_name" "$@"
    else
      gum spin --spinner dot --title "$title" -- "$@"
    fi
  else
    log_info "[RUN] $title"
    "$@"
  fi
}

safe_confirm() {
  local prompt="$1"
  local default_ans="${2:-n}"
  if is_interactive && [[ -e /dev/tty ]]; then
    if [[ "$default_ans" == "y" ]]; then
      gum confirm --default=true "$prompt" < /dev/tty > /dev/tty
    else
      gum confirm --default=false "$prompt" < /dev/tty > /dev/tty
    fi
  else
    if [[ ! -t 0 ]]; then
      local user_input=""
      if read -r user_input; then
        case "$user_input" in
          [Yy]* ) return 0 ;;
          [Nn]* ) return 1 ;;
        esac
      fi
    fi
    if [[ "$default_ans" == "y" ]]; then
      return 0
    else
      return 1
    fi
  fi
}

safe_input() {
  local prompt="$1"
  local default_val="${2:-}"
  local is_password="${3:-0}"
  if is_interactive && [[ -e /dev/tty ]]; then
    local opts=(--placeholder "$prompt")
    if [[ -n "$default_val" ]]; then
      opts+=(--value "$default_val")
    fi
    if [[ "$is_password" == "1" ]]; then
      opts+=(--password)
    fi
    gum input "${opts[@]}" < /dev/tty
  else
    local val=""
    if [[ "$is_password" == "1" ]]; then
      printf '%s: ' "$prompt" >&2
      if ! read -r -s val; then val=""; fi
      echo "" >&2
    else
      printf '%s [%s]: ' "$prompt" "$default_val" >&2
      if ! read -r val; then val=""; fi
    fi
    val="${val:-$default_val}"
    printf '%s\n' "$val"
  fi
}

safe_choose() {
  local header="$1"
  shift
  local options=("$@")
  if is_interactive && [[ -e /dev/tty ]]; then
    gum choose --header "$header" "${options[@]}" < /dev/tty
  else
    if [[ ! -t 0 ]]; then
      local val=""
      if read -r val; then
        printf '%s\n' "$val"
        return 0
      fi
    fi
    printf '%s\n' "${options[0]}"
  fi
}

safe_style() {
  local text="$1"
  shift
  if is_interactive; then
    gum style "$@" "$text"
  else
    printf '%s\n' "$text"
  fi
}

safe_table() {
  if is_interactive; then
    gum table "$@"
  else
    cat
  fi
}
```

- [ ] **Step 4: Run tests to verify pass**

Run: `bats tests/gum_helpers.bats`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add backup.sh tests/gum_helpers.bats
git commit -m "feat: add subshell and pipeline-safe gum helper suite"
```

---

### Task 2: Architecture-Aware SHA256 Verified Gum Installation in `cmd_install`

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Create: [tests/install_gum.bats](file:///home/ppzxc/projects/backup/tests/install_gum.bats)

**Interfaces:**
- Consumes: Correct `install_binary` signature `(name, version, url, expected_sha, target_path, format, archive_path)`.
- Produces: `install_gum` step within `cmd_install`. Determines `arch` (`x86_64` vs `arm64`), matches exact SHA256, uses `install_binary`, supports `--dry-run`, and degrades gracefully if offline.

- [ ] **Step 1: Write failing BATS test for Gum installation**

Create `tests/install_gum.bats`:
```bash
#!/usr/bin/env bats

load 'test_helper'

setup() {
  setup_backup_sh_env
}

teardown() {
  teardown_backup_sh_env
}

@test "install_gum succeeds or degrades gracefully without error" {
  run install_gum
  [ "$status" -eq 0 ]
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `bats tests/install_gum.bats`
Expected: FAIL (`install_gum` command not found)

- [ ] **Step 3: Add constants and `install_gum` function in `backup.sh`**

Add constants in [backup.sh](file:///home/ppzxc/projects/backup/backup.sh#L83):
```bash
GUM_INSTALL_PATH="${GUM_INSTALL_PATH:-/usr/local/bin/gum}"
GUM_VERSION="0.15.0"
GUM_SHA256_AMD64="${GUM_SHA256_AMD64:-6919a0a149c7bc2990089f2ee1df1b3531bdf776adcd1ca4eb0cf91fec660565}"
GUM_SHA256_ARM64="${GUM_SHA256_ARM64:-8b2c286a67f1b73bf7e21a221f7c17d3d17dcb1adad98327ef3eefbf7a70a8d6}"
```

Implement `install_gum()` with correct `install_binary` parameters:
```bash
install_gum() {
  if command -v gum >/dev/null 2>&1; then
    log_info "gum이 이미 설치되어 있습니다: $(command -v gum)"
    return 0
  fi

  local uname_arch
  uname_arch=$(uname -m)
  local gum_arch=""
  local gum_sha=""

  case "$uname_arch" in
    x86_64)
      gum_arch="x86_64"
      gum_sha="$GUM_SHA256_AMD64"
      ;;
    aarch64|arm64)
      gum_arch="arm64"
      gum_sha="$GUM_SHA256_ARM64"
      ;;
    *)
      log_warn "지원하지 않는 아키텍처(${uname_arch})이므로 gum 설치를 건너뜁니다."
      return 0
      ;;
  esac

  local gum_url="https://github.com/charmbracelet/gum/releases/download/v${GUM_VERSION}/gum_${GUM_VERSION}_Linux_${gum_arch}.tar.gz"
  local archive_member="gum_${GUM_VERSION}_Linux_${gum_arch}/gum"

  log_info "gum v${GUM_VERSION} (${gum_arch}) 설치 시도 중..."
  if install_binary "gum" "$GUM_VERSION" "$gum_url" "$gum_sha" "$GUM_INSTALL_PATH" "tar.gz" "$archive_member" 2>/dev/null; then
    log_info "gum 설치가 완료되었습니다: ${GUM_INSTALL_PATH}"
  else
    log_warn "gum 자동 설치를 건너뛰었습니다 (선택적 TUI 도구)"
  fi
  return 0
}
```

Integrate `install_gum` inside `cmd_install()` and update `--dry-run` output to mention `gum`.

- [ ] **Step 4: Run test to verify pass**

Run: `bats tests/install_gum.bats`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add backup.sh tests/install_gum.bats
git commit -m "feat: add arch-aware checksum-verified gum installation"
```

---

### Task 3: Refactor Subcommands and Prompt Helpers with Automation Flags & Version Bump

**Files:**
- Modify: [backup.sh](file:///home/ppzxc/projects/backup/backup.sh)
- Modify: [tests/integration/integration.bats](file:///home/ppzxc/projects/backup/tests/integration/integration.bats)

**Interfaces:**
- `prompt_validated`, `prompt_secret_required`, `prompt_backend_choice`: enhanced with `safe_input` / `safe_choose` inside while preserving existing stdin piping and validation loops.
- `cmd_run`, `cmd_init`: wraps binary and function executions in `safe_spin`.
- `cmd_status`, `cmd_config`: enhances visual headers/tables with `safe_style` / `safe_table` when interactive.
- `cmd_uninstall`, `cmd_migrate`: accepts `--force` / `--yes` options, skipping `safe_confirm` prompt when present.
- `BACKUP_SCRIPT_VERSION`: bump from `0.0.55` to `0.0.56`.

- [ ] **Step 1: Verify existing tests before refactoring**

Run: `bats tests/`
Expected: ALL PASS

- [ ] **Step 2: Refactor prompt helpers and subcommands in `backup.sh`**

1. Update `BACKUP_SCRIPT_VERSION="0.0.56"`.
2. Integrate `safe_input` into `prompt_validated` and `prompt_secret_required`.
3. Integrate `safe_choose` into `prompt_backend_choice`.
4. Apply `safe_spin` to `cmd_run` and `cmd_init`.
5. Parse `purge force yes` in `cmd_uninstall` and `yes force` in `cmd_migrate` (bypassing confirmation when `--force` or `--yes` is passed).
6. Apply `safe_style` in `cmd_status` and `cmd_config`.

- [ ] **Step 3: Run full BATS test suite to verify 0 regressions**

Run: `bats tests/`
Expected: ALL PASS (100% clean test execution including piped wizard tests)

- [ ] **Step 4: Run ShellCheck**

Run: `shellcheck backup.sh`
Expected: 0 warnings / clean output.

- [ ] **Step 5: Commit**

```bash
git add backup.sh
git commit -m "feat: apply gum TUI enhancements to subcommands and prompt helpers with version bump 0.0.56"
```

---

### Task 4: Final Verification & Integration Checklist

- [ ] **Step 1: Run ShellCheck on backup.sh**

Run: `shellcheck backup.sh`
Expected: Clean 0 errors.

- [ ] **Step 2: Run all BATS tests**

Run: `bats tests/`
Expected: All tests pass.

- [ ] **Step 3: Commit final plan updates**

```bash
git commit -m "docs: finalize gum integration plan with all codex cli review fixes"
```

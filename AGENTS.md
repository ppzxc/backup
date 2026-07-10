# AGENTS.md

This file provides guidance to AI coding agents (Claude Code, etc.) when working with code in this repository.

## What this repo is

A single-file bash script, `backup.sh`, that installs and operates a `restic` backup pipeline on RHEL-family (dnf) servers. It supports two storage backends — S3-compatible object storage, or an SFTP/NAS target via rclone — and provides these subcommands:

`install` · `setting` · `init` · `schedule` · `run` · `status` · `uninstall` · `wizard`

Backup execution (backup/forget/prune orchestration), scheduling (systemd timer generation), and stale-lock handling are delegated to [resticprofile](https://creativeprojects.github.io/resticprofile/) (installed by `cmd_install` as a version-pinned, checksum-verified binary — see `RESTICPROFILE_VERSION`/`RESTICPROFILE_SHA256` near the top of `backup.sh`). Everything else (package installation, config validation, SSH key generation, secret-masked status reporting, the interactive wizard) is plain bash.

## Build, lint, test

- Lint: `shellcheck backup.sh` (must report 0 findings before committing)
- Full unit test suite: `bats tests/*.bats`
- Single test file: `bats tests/cmd_run.bats`
- Single test case: `bats tests/cmd_run.bats -f "cmd_run dies when resticprofile fails"`
- Tier 2 integration tests (spins up MinIO + SFTP + a rockylinux:9 container via docker compose, exercises install/setting/init/run/schedule end-to-end against real backends): `cd tests/integration && ./run.sh` — requires Docker and outbound network access to GitHub (for the real resticprofile download).
- Tier 3 manual verification checklist (things Tier 2 can't reproduce — real NAS/bucket registration, actual systemd timer activation, wizard prompt wording): `tests/MANUAL_CHECKLIST.md`

## Architecture

`backup.sh` is written as core/imperative shell: small pure functions (`resolve_value`, `validate_*`, `render_*`, `parse_long_opts`) that are unit-testable in isolation by sourcing the script into a bats test (see `tests/test_helper.bash`'s `setup_backup_sh_env`), plus a set of `cmd_*` functions that each implement one subcommand and are dispatched from `main()` at the bottom of the file. Side effects (writing files, calling external commands like `restic`/`rclone`/`resticprofile`/`systemctl`/`dnf`) are isolated behind thin wrapper functions (e.g. `write_secure_file`, `dnf_install_packages`) so tests can stub the external command via `tests/test_helper.bash`'s `stub_command` instead of mocking deep inside a `cmd_*` function.

Config resolution follows a fixed precedence, implemented once in `resolve_value`: CLI flag > environment variable > existing `backup.env` value > built-in default. All of `cmd_setting`'s flags go through this.

`backup.env` (written by `cmd_setting`, sourced by `cmd_init`/`cmd_schedule`/`cmd_run`/`cmd_status`/`cmd_uninstall` via the shared `require_backup_env` helper) is the single source of truth for a host's backend config, retention policy, and profile name — everything else is derived from it at runtime rather than re-resolved.

The full implementation history and design rationale (including the resticprofile migration decisions — e.g. why the custom systemd unit template deliberately omits the `.Environment` block) is recorded in `docs/superpowers/plans/2026-07-10-restic-backup-script.md`.

## Conventions

- `shellcheck backup.sh` must stay at 0 findings. Where a warning is a known false positive (e.g. `SC2153`/`SC2034` across a nameref or a sourced-env-file boundary shellcheck can't see through), suppress it with an inline `# shellcheck disable=<code>` comment plus a one-line reason above the flagged line — don't leave it unexplained or unsuppressed.
- New behavior should be developed test-first (bats) per the pattern in `docs/superpowers/plans/2026-07-10-restic-backup-script.md`: write the failing test, confirm it fails, implement, confirm it passes.
- `.gitignore` uses an allowlist (`/*` then explicit `!` exceptions) rather than a denylist — new tracked top-level files/dirs must be added there explicitly or `git add` will silently no-op on them.

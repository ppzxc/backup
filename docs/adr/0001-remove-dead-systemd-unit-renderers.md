# 0001. Remove the dead systemd-native unit renderers

## Status

Accepted

## Context

`backup.sh` originally generated systemd `.service`/`.timer` units directly
(`render_service_unit`, `render_timer_unit`, backed by the `SYSTEMD_SERVICE_FILE`,
`SYSTEMD_TIMER_FILE`, `SYSTEMD_UNIT_DIR` constants). The migration to
[resticprofile](https://creativeprojects.github.io/resticprofile/) for
scheduling (see the restic-backup-script design plan) replaced this path
with resticprofile's own systemd templates
(`render_resticprofile_unit_template`, `render_resticprofile_timer_template`),
but the old renderers and constants were kept afterward, with an inline
comment explaining they were retained "to pair with
render_service_unit/render_timer_unit's regression-test asset"
(`tests/render_units.bats`).

By 2026-07-10 no `cmd_*` function called either renderer or referenced the
two `SYSTEMD_*_FILE` constants — their only remaining caller was their own
test file. Applying the deletion test (codebase-design vocabulary: "would
deleting this concentrate complexity elsewhere, or does it just vanish?"),
deleting them made nothing reappear: they were pure dead weight left over
from the migration, not a hypothetical seam waiting for a second adapter.

## Decision

Remove `render_service_unit`, `render_timer_unit`, `SYSTEMD_SERVICE_FILE`,
`SYSTEMD_TIMER_FILE`, and `SYSTEMD_UNIT_DIR` from `backup.sh`, and delete
`tests/render_units.bats`.

Scheduling is resticprofile's responsibility going forward
(`render_resticprofile_unit_template`/`render_resticprofile_timer_template`,
`cmd_schedule`). If a future need arises to generate systemd units directly
again (e.g. dropping the resticprofile dependency), that would be a new
decision made with the requirements of that moment, not a resurrection of
this dead code.

## Consequences

- `backup.sh` shrinks by ~35 lines; a reader (human or AI) walking the file
  top to bottom no longer has to determine that these renderers are
  vestigial before dismissing them.
- `tests/test_helper.bash` still exports `SYSTEMD_UNIT_DIR` defensively for
  test environment isolation; this is harmless now that nothing in
  `backup.sh` reads it, and was left as-is to avoid unrelated churn.
- Reintroducing native systemd unit generation later should start from a
  fresh design (informed by why resticprofile was adopted), not from
  restoring this deleted code.

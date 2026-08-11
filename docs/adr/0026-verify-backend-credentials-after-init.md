# Verify backend credentials after initialization

**Status: accepted.**

## Context

`restic init` is idempotent for an existing repository. A successful result can therefore mean
only that the repository exists; it does not establish that the configured repository password
can decrypt one of its keys. SFTP connectivity likewise establishes transport authentication, not
restic repository access. Scheduling a configuration after either partial signal can leave an
operator with a successful setup and a backup pipeline that cannot read or write its repository.

## Decision

For every backend initialization target, Setup performs `init` and then `snapshots` in the
existing deterministic target order. A target is initialized only when both actions succeed.
This applies to interactive setup, `setup --non-interactive`, and `setup backend-init`.

The command output is classified as a repository credential mismatch only when it contains
`wrong password` or `no key found`, case-insensitively. Other errors remain transport,
authorization, or general initialization failures and retain the existing retryable pending
configuration policy.

An interactive newly-created setup stops immediately on a credential mismatch. It restores the
prior configuration, wizard-owned sidecars, and scheduler state; it neither saves pending setup
nor contacts later targets. Non-interactive setup reports failure without registering a schedule.
`backend-init` never promotes, modifies, or removes live/pending configuration after a credential
verification failure.

For a newly configured secondary SFTP backend, the wizard asks whether to reuse the primary
restic key or enter the existing secondary key. A separately entered key must be at least twelve
characters. It is stored only in the `0600` `secondary-password` sidecar. The `secondary` profile
and the backup profile's copy destination both reference that sidecar. An explicit secondary key
takes precedence over the shared `enc` file; an existing safe `secondary-password` is preserved
when reuse is selected.

## Consequences

SFTP connection success and restic credential verification are observable as distinct stages.
Existing repositories using another key now fail setup before scheduling. Operators can retain
ordinary unavailable backends for repair, while key mismatches require correcting the credential
rather than promoting an unusable pending configuration. Secrets remain outside terminal output,
diagnostic logs, and systemd units.

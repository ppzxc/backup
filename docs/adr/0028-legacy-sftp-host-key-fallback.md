# 0028. Use a strict known-host fallback for legacy SFTP

**Status: accepted.**

## Context

CentOS 6-era OpenSSH cannot rely on `StrictHostKeyChecking=accept-new`, and Ed25519 key
generation is not available in the supported runtime profile. Setup and scheduled execution must
still share the central managed `known_hosts` trust state.

## Decision

Choose Ed25519 first when the platform capability supports it, otherwise generate/use the managed
RSA identity (`id_rsa` or `id_rsa_secondary`). On legacy OpenSSH, setup pre-registers the host key
with `ssh-keyscan` and runtime SFTP uses `StrictHostKeyChecking=yes` with the configuration-scoped
`UserKnownHostsFile`. Existing configuration is corrected only when its identity and known_hosts
paths are wizard-managed; arbitrary SSH settings are never guessed or replaced.

## Consequences

Host-key changes remain connection failures. Key-only authentication, `IdentitiesOnly=yes`, and
the shared trust file continue to apply to both the SFTP Connection Test and restic's native
`sftp.args` path.

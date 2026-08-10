---
status: accepted
date: 2026-08-10
decision-makers: [project maintainers]
consulted: [backup operator]
informed: []
---

# Enforce key-only SFTP repository initialization

## Context and Problem Statement

The Setup Wizard's SFTP Connection Test successfully authenticated with the selected SSH identity, but repository initialization through <code>resticprofile</code> invoked a separate SSH path. That path could try unrelated SSH-agent identities, fall back to password authentication, and exhaust the server's authentication-attempt limit before the selected key was accepted. The preflight result was therefore not a reliable indication that the actual SFTP Backend Adapter could initialize the repository.

## Decision Drivers

* Use the explicitly selected SSH identity for both preflight and repository initialization.
* Prevent password prompts and unrelated SSH-agent identities from becoming authentication fallback.
* Preserve host-key change detection across interactive and scheduled executions.
* Keep host, port, and user resolution owned by the restic SFTP backend.
* Make the authentication policy testable without exposing private key material or secrets.

## Considered Options

* Pass authentication arguments through restic's native <code>sftp.args</code> option.
* Replace the complete SSH invocation with a generated <code>sftp.command</code> string.
* Allow password or SSH-agent fallback after the selected key fails.

## Decision Outcome

Chosen option: **Pass authentication arguments through restic's native <code>sftp.args</code> option**, because it keeps repository location parsing in restic while making the authentication policy explicit and shared.

The generated SFTP arguments are equivalent to:

~~~text
-i <managed-key>
-o IdentitiesOnly=yes
-o BatchMode=yes
-o StrictHostKeyChecking=accept-new
-o UserKnownHostsFile=<profiles.yaml-parent>/known_hosts
~~~

The SFTP Connection Test and resticprofile configuration use one pure renderer for these authentication arguments. Setup Wizard accepts only keys it manages beside <code>profiles.yaml</code>; on preflight authentication failure it offers retry, connection-detail re-entry, key re-selection or explicit regeneration, ignore, and cancel. Existing non-standard <code>sftp.command</code> values are not guessed or partially migrated.

### Consequences

* Good, because an SSH agent containing unrelated keys cannot consume the server's authentication-attempt budget.
* Good, because <code>BatchMode=yes</code> prevents an unexpected password prompt during setup, scheduled runs, or repository initialization.
* Good, because <code>accept-new</code> accepts a new host key once and rejects later host-key changes through one configuration-directory <code>known_hosts</code> file.
* Good, because restic derives host, port, and user from the repository URI instead of the application duplicating the complete SSH command.
* Bad, because password authentication and implicit SSH-agent fallback are intentionally unavailable.
* Bad, because setup must manage key files and host-key trust state, including permissions and rollback.

### Confirmation

The implementation is confirmed by unit tests for the shared argument renderer and generated <code>sftp.args</code>, an integration test with multiple SSH-agent identities that verifies only the selected key is usable, and SFTP E2E tests without global SSH settings that mask <code>IdentitiesOnly</code> or host-key behavior. Setup tests also verify that initialization failure can preserve retryable authentication artifacts without enabling the scheduler.

## Pros and Cons of the Options

### Pass authentication arguments through <code>sftp.args</code>

* Good, because restic owns repository URI parsing and process invocation.
* Good, because the application controls only the authentication policy it must enforce.
* Good, because the same rendered arguments can be tested for preflight and initialization.
* Bad, because the <code>sftp.args</code> value still requires careful argument rendering.

### Replace the complete SSH invocation with <code>sftp.command</code>

* Good, because the full command line appears in one configuration value.
* Bad, because host, port, and user are duplicated outside the repository URI.
* Bad, because a command string can drift from the preflight invocation and is harder to migrate safely.

### Allow password or SSH-agent fallback

* Good, because it may connect to servers with incomplete key authorization.
* Bad, because it violates the key-only SFTP security policy.
* Bad, because it causes interactive prompts and can trigger <code>too many authentication failures</code>.
* Bad, because a successful preflight with one key would not guarantee that initialization uses that key.

## More Information

This decision refines [ADR-0019](0019-use-sftp-subsystem-for-connectivity-test.md): its SFTP Connection Test scope remains valid, while the old runtime-command clause is superseded by [ADR-0022](0022-use-native-sftp-args-for-restic.md). Scheduler behavior after initialization failure is defined in [ADR-0021](0021-do-not-schedule-uninitialized-backends.md), host-key storage in [ADR-0023](0023-centralize-sftp-host-key-trust.md), and setup reuse/migration in [ADR-0024](0024-preserve-sftp-authentication-on-setup-reuse.md).

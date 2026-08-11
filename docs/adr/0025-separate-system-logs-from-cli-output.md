# Separate system diagnostic logs from CLI output

## Status

Accepted

## Context

The CLI has two observable output contracts:

- system diagnostics are structured `tracing` events for operators and automation;
- CLI output is user data, progress, completion, interaction, or a command-failure diagnostic.

Putting both in a terminal stream makes report JSON unsafe to consume, damages the setup TUI, and
makes it unclear whether a message is an operational log or an instruction to the user.

## Decision

Structured diagnostics are never written to terminal `stdout` or `stderr`. The system delivery
boundary attempts the following sinks for each event, in order:

1. Journald socket;
2. Syslog socket;
3. a secure daily rotating file, first under `/var/log/backup`, then under the user's
   application state directory when the system directory cannot accept the event.

The sink order is evaluated per event. If every default sink rejects one event, that event is
lost; the command continues and its exit status is unchanged. A directory newly created by the
application is mode `700`; an existing shared parent is not chmod'ed. Rotating files are regular
files with mode `600`. Structured fields are masked before delivery.

`--log-file` is an explicit single-file target. It has no socket or rotation fallback. The file is
created/opened and secured before command dispatch; any failure returns exit code `1`. The normal
non-blocking writer and its `WorkerGuard` are retained so process shutdown flushes queued events.

CLI User Notice and command output are separate from that system boundary:

- ordinary user data, progress, completion, and explicit setup-cancellation notices use
  `CommandOutcome.stdout` or the setup TUI stdout path;
- parser, startup, and command-failure diagnostics use `stderr`;
- `backup doctor` keeps status and details on stdout and emits only a short failure summary on
  stderr for `Fail` or `Unavailable`;
- JSON report output is artifact-only and leaves stdout empty; HTML output may print its artifact
  path notice. An action-level report format takes precedence over the parent format.

This supersedes ADR-0015's terminal diagnostic layer.

## Consequences

- terminal streams remain safe for their declared CLI contracts;
- systemd/journald, syslog, and local file operators can observe the same structured events;
- a temporary loss of default system sinks does not make a backup command fail;
- an explicitly requested log file fails fast, so a typo or permission error cannot silently send
  diagnostics elsewhere;
- JSON consumers can parse stdout without filtering tracing output.

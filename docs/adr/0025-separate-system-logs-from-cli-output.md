# Separate system diagnostic logs from CLI output

## Status

Accepted

Structured `tracing` diagnostics are routed only to the existing system-log pipeline (`Journald -> Syslog -> rotating file`) and are never emitted to terminal `stdout` or `stderr`. User-facing progress, completion, and interaction notices are emitted through the CLI output contract, while parser, startup, and command-failure diagnostics remain plain `stderr`; machine-readable output modes suppress non-essential notices so their `stdout` remains parseable. This supersedes ADR-0015's terminal `stderr` diagnostic layer while preserving its sink order, log format, filtering, masking, and sink-delivery behavior.

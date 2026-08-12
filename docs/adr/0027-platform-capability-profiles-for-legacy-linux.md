# 0027. Use Platform Capability profiles for legacy Linux support

**Status: accepted.**

## Context

CentOS 6.10 x86_64 has no systemd, commonly uses `crond` and `ntpd`, and its OpenSSH release
does not provide the modern Ed25519 and `accept-new` behavior used by the modern Linux path.
Adding an OS-name branch to every command would make scheduler, diagnostics, reports, SFTP, and
Database Stream behavior drift apart.

## Decision

Detect one `PlatformCapabilities` snapshot at process startup and inject it into scheduler,
doctor, report, setup/SFTP, and Database Backup Adapter seams. The supported CentOS profile is
x86_64 only: cron registration requires a running `crond`; ntpd is the time-sync provider; RSA is
the SSH key fallback; strict known-host checking is used; and MariaDB 5.5.56 is the only Database
Stream version in the CentOS 6 support matrix. PostgreSQL is rejected before its dump client is
launched.

Systemd remains the first scheduler when its capability is available. A capability failure may
select cron, but a registration failure in the selected scheduler is returned without retrying
the other scheduler.

## Consequences

Capability-specific findings are represented as `NotApplicable` or `Unavailable` rather than
pretending that chrony/systemd exists. Existing chrony/systemd report fields remain for schema
compatibility, while generic `time_sync_*` and `scheduler_*` fields identify the selected path.

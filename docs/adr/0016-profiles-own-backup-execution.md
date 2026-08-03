# 0016. Profiles Own Backup Execution

## Status

Accepted

## Context

`application` previously mirrored backup profile, storage, retention, and format-version
settings already represented by resticprofile v2. This made the two representations drift and
could place credentials in the shared YAML.

## Decision

The root `version: "2"` and standard `profiles` are the sole authority for backup execution.
`application` is limited to reports, audit metadata, and an optional Database Stream whose
target Backup Profile must exist. Legacy execution keys under `application` are rejected with
an instruction to migrate manually. `audit` is application metadata, never a root setting.

S3 credentials remain in mode-600 sidecar files. Backend profile `env` values only reference
child-process environment variables; they never contain credential values. A profile's `copy`
section owns whether that profile is replicated.

## Consequences

Operators must manually move legacy execution settings into standard profiles. This removes a
convenient compatibility path, but prevents conflicting configuration and secret leakage.
This amends ADR-0003; domain vocabulary remains in `CONTEXT.md`.

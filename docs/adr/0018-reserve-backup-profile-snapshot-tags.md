# Reserve Backup Profile Snapshot Tags

## Status

Accepted

Every snapshot created for a runnable Backup Profile carries the CLI-owned tag `backup-profile:<exact-profile-key>`. Restore Drill uses this exact tag to select the newest concrete snapshot ID for each Backup Profile and Backend Profile combination; it does not infer identity from paths or hostnames and does not retag existing snapshots. This reserves a stable namespace at the cost of requiring one newly tagged backup before legacy profiles can produce Restore Drill Evidence, preventing ambiguous audit attribution in repositories shared by multiple profiles.

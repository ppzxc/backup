# Use native SFTP arguments for restic repository connections

**Status: accepted.** The generated Backend Profile uses restic's native `sftp.args` option to pass the selected identity and key-only authentication flags, rather than replacing the complete SSH command through `sftp.command`. Restic then derives host, port, and user from the repository URI, leaving the application responsible only for the authentication policy. This supersedes the runtime-command clause of ADR-0019; ADR-0019's SFTP Connection Test scope and subsystem requirement remain in force.

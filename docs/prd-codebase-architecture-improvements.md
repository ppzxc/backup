# PRD: Codebase Architecture Deepening via Polymorphic Adapters

## Problem Statement

The backup script contains several shallow modules where storage backend details, notification provider logic, and database-specific commands are hardcoded in the core configuration, resolution, and execution pipelines. This lack of modular depth makes it difficult to maintain existing backends/notifications or add new ones (e.g., Google Cloud Storage, Microsoft Teams, PostgreSQL-specific backup options) without editing multiple unrelated parts of the main execution script, violating locality and reducing leverage for tests and callers.

## Solution

Deepen the codebase architecture by introducing three polymorphic seams:
1. A **Backend Adapter Interface** to unify storage backend validation, priority resolution, and configuration generation (backup.env and resticprofile).
2. A **Notification Adapter Interface** to unify notification payloads, formatting, and webhook dispatching.
3. A **Database Backup Adapter Interface** to unify default backup commands, validation, and metadata reporting for various database types.

This will replace hardcoded case statements with clean, dynamic dispatch interfaces (`module_${adapter}_action`), increasing depth, leverage, and locality.

## User Stories

1. As a system administrator, I want to add a new storage backend adapter (e.g., Azure Blob) without editing the core configuration registry, so that I can extend storage options safely with high locality.
2. As a developer, I want all backend-specific validation logic to reside in the backend's own adapter module, so that I don't have to search through the main script to fix a backend validation bug.
3. As a test suite, I want to stub the backend adapter interface, so that I can verify configuration resolution without executing network requests or file writes.
4. As a system administrator, I want to integrate a new notification channel (e.g., Teams) by creating a notification adapter, so that I don't have to alter the main notification dispatcher function.
5. As a system administrator, I want database-specific credentials and commands to be resolved through a database adapter, so that Postgres and MySQL settings don't pollute the generic backup configuration registry.
6. As a developer, I want to use standard mock adapters for testing backend, notification, and database logic, so that unit tests remain fast, self-contained, and run without external system dependencies.
7. As a security compliance officer, I want backup.env generation to automatically leverage the active backend adapter to render sensitive credentials securely (masking, single-quoting), so that secret exposure is prevented at the seam.
8. As a developer, I want to run BATS tests directly against the backend and database adapter interfaces, so that I can verify their standalone correctness without running a full backup cycle.
9. As a system administrator, I want to define secondary backends that utilize the exact same backend adapter interface, so that the secondary storage setup inherits all validation and resolution capabilities automatically.
10. As a system administrator, I want database backup execution to fetch its dump command dynamically from the database adapter, so that I can customize backup options (like schema-only or compression) via configuration.

## Implementation Decisions

- **Backend Adapter Module**: Introduce a polymorphic interface for backends. The interface will expose the following dynamic methods:
  - `backend_${backend}_env_vars`: Returns tab-separated list of configuration fields and environment shadow variables.
  - `backend_${backend}_resolve`: Resolves precedence (CLI > Env > backup.env > Defaults) for backend fields.
  - `backend_${backend}_validate`: Validates backend fields.
  - `backend_${backend}_prepare`: Runs pre-initialization steps (e.g. generating ssh keys for SFTP).
  - `backend_${backend}_render_env`: Generates environment exports to write to `backup.env`.
  - `backend_${backend}_render_notice`: Generates registration instructions (e.g. public keys or bucket policies).
  - `backend_${backend}_test_connectivity`: Performs a connection check.
- **Notification Adapter Module**: Introduce a polymorphic interface for notification providers. The dispatcher `dispatch_notification` will dynamically call `notification_${type}_send` using `BACKUP_NOTIFICATION_TYPE`. Each provider adapter encapsulates its payload layout and web request execution.
- **Database Backup Adapter Module**: Introduce a polymorphic interface for database engines. The configuration registry and scheduler delegate validation and default command resolution to `database_${db_type}_*` methods.
- **Dynamic Seams**: Invocation of adapters will use dynamic dispatch (`"module_${adapter}_method"`).

## Testing Decisions

- **Seam-level Unit Tests**: Tests should verify external behavior through BATS unit tests, calling the polymorphic functions directly with mock associative arrays, and checking outputs (validations, resolved values, generated environment snippets) rather than verifying implementation details.
- **Mock Adapters**: Utilize mock backend, notification, and database adapters in tests to isolate core logic from network connectivity and database processes.
- **Integration Tests**: Leverage the existing E2E integration test suite (`tests/integration/integration.bats`) to confirm that the unified pipelines execute successfully with SFTP and S3 backends in Docker.
- **Prior Art**: The existing `tests/backend_adapters.bats` and `tests/scheduler.bats` serve as excellent examples of direct adapter and seam testing.

## Out of Scope

- Adding new storage backends (e.g., Azure, GCS) or notification channels (e.g., Teams) beyond refactoring the existing ones (S3, SFTP, Slack, Discord, Custom, MySQL, MariaDB, Postgres) to the new structure.
- Modifying the underlying restic or rclone binaries or their standard execution flow.
- Modifying the systemd timer templates or scheduler adapters.

## Further Notes

None.

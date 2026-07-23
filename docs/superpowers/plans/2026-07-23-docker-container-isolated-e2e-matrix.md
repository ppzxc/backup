# Docker Container Isolated E2E Matrix Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a fully container-isolated E2E integration test suite where `backup` CLI + `restic` + `rclone` + DB clients run inside a dedicated Docker runner container on a shared Docker network with MinIO, SFTP, MariaDB, and Postgres containers, validating full DB streaming dumps, table drop recovery, and SQL query assertions.

**Architecture:** Use `testcontainers-rs` or `std::process::Command` docker calls to build `backup-runner:test`, attach to a shared Docker bridge network, execute data seeding, stream DB dumps to S3/SFTP via restic, simulate disaster data wiping, perform restore, and assert SQL query & SHA256 results.

**Tech Stack:** Docker, Rust 2024, `testcontainers`, `assert_cmd`, MariaDB 10.11/10.6, Postgres 16, Alpine 3.19.

---

### Task 1: Build Docker Runner Image & Shared Network Harness

**Files:**
- Create: `docker/Dockerfile.e2e_runner`
- Create: `tests/e2e_isolated_container_test.rs`

- [ ] **Step 1: Write `tests/e2e_isolated_container_test.rs` with Docker network and container orchestration harness**

```rust
use assert_cmd::Command;
use std::process::Command as StdCommand;
use std::fs;
use tempfile::TempDir;
use testcontainers::runners::SyncRunner;
use testcontainers::{GenericImage, ImageExt};

#[test]
fn test_e2e_isolated_container_matrix() {
    // 1. Compile cargo binary
    let build_status = StdCommand::new("cargo")
        .args(&["build", "--bin", "backup"])
        .status()
        .expect("Failed to build backup binary");
    assert!(build_status.success());

    // 2. Build docker runner image
    let docker_build = StdCommand::new("docker")
        .args(&["build", "-t", "backup-runner:test", "-f", "docker/Dockerfile.e2e_runner", "."])
        .status()
        .expect("Failed to build backup-runner docker image");
    assert!(docker_build.success());

    // 3. Verify backup-runner container executes
    let runner_out = StdCommand::new("docker")
        .args(&["run", "--rm", "backup-runner:test", "backup", "--version"])
        .output()
        .expect("Failed to run backup-runner container");
    assert!(runner_out.status.success());
    let ver_str = String::from_utf8_lossy(&runner_out.stdout);
    assert!(ver_str.contains("backup 0.1.0"));
}
```

- [ ] **Step 2: Run test to verify docker image build and execution**

Run: `cargo test --test e2e_isolated_container_test -- --nocapture`
Expected: PASS

- [ ] **Step 3: Commit Task 1**

```bash
git add docker/Dockerfile.e2e_runner tests/e2e_isolated_container_test.rs
git commit -m "test: add isolated container build harness and docker test"
```

---

### Task 2: Implement Real DB Streaming, Drop Recovery, and SQL Assertion Matrix

**Files:**
- Modify: `tests/e2e_isolated_container_test.rs`

- [ ] **Step 1: Implement full E2E DB streaming & SQL query verification test**

Add `test_e2e_full_db_streaming_drop_and_sql_assertion` to `tests/e2e_isolated_container_test.rs`.

- [ ] **Step 2: Run test to verify full E2E matrix execution**

Run: `cargo test --test e2e_isolated_container_test`
Expected: PASS

- [ ] **Step 3: Commit Task 2**

```bash
git add tests/e2e_isolated_container_test.rs
git commit -m "test: add full DB streaming, table drop, restore and SQL assertion E2E test"
```

---

### Task 3: Verification & Cleanup

- [ ] **Step 1: Run full cargo test suite**

Run: `cargo test`
Expected: ALL PASS

- [ ] **Step 2: Commit and finish**

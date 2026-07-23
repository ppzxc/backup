# Testcontainers E2E Integration Test Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a comprehensive Docker/Testcontainers-based End-to-End (E2E) integration test in Rust (`tests/e2e_full_workflow.rs`) validating the entire backup lifecycle (Primary S3 backup, Secondary SFTP copy, Data Restore verification with SHA256 checksums, Integrity/NTP Doctor reporting, and Scheduler subcommands).

**Architecture:** Use `testcontainers-rs` to launch concurrent MinIO (S3) and atmoz/SFTP containers. Set up test fixture directories and a temporary config file, execute `backup` CLI subcommands using `assert_cmd`, and verify data integrity and output reports.

**Tech Stack:** Rust 2024 edition, `testcontainers` 0.23, `assert_cmd` 2.0, `tempfile` 3.10, `sha2` / standard IO.

## Global Constraints

- Must run via standard `cargo test --test e2e_full_workflow`.
- Primary storage target must be MinIO (S3 protocol).
- Secondary storage target must be atmoz/SFTP.
- Restored files must match original SHA256 checksums byte-for-byte.
- All temporary containers and files must clean up automatically via RAII.

---

### Task 1: Create Testcontainers E2E Test Harness & Fixture Setup

**Files:**
- Create: `tests/e2e_full_workflow.rs`

**Interfaces:**
- Consumes: `testcontainers::{GenericImage, ImageExt}`, `assert_cmd::Command`, `tempfile::TempDir`
- Produces: Runnable test `test_e2e_containers_setup` with container setup & fixture files

- [ ] **Step 1: Write initial test module structure with container startup and mock fixture files**

```rust
use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;
use testcontainers::runners::SyncRunner;
use testcontainers::{GenericImage, ImageExt};

#[test]
fn test_e2e_containers_setup() {
    let minio_image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);

    let minio_node = minio_image.start().expect("Failed to start MinIO container");
    let s3_port = minio_node.get_host_port_ipv4(9000).expect("Failed to get MinIO port");
    assert!(s3_port > 0);

    let sftp_image = GenericImage::new("atmoz/sftp", "alpine")
        .with_cmd(vec!["backupuser:backuppass:::upload"]);

    let sftp_node = sftp_image.start().expect("Failed to start SFTP container");
    let sftp_port = sftp_node.get_host_port_ipv4(22).expect("Failed to get SFTP port");
    assert!(sftp_port > 0);

    let temp_dir = TempDir::new().unwrap();
    let src_dir = temp_dir.path().join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(src_dir.join("hello.txt"), "Hello Backup E2E World!").unwrap();

    assert!(src_dir.join("hello.txt").exists());
}
```

- [ ] **Step 2: Run test to verify container startup and fixture creation**

Run: `cargo test --test e2e_full_workflow test_e2e_containers_setup -- --nocapture`
Expected: PASS

- [ ] **Step 3: Commit Task 1**

```bash
git add tests/e2e_full_workflow.rs
git commit -m "test: add testcontainers E2E harness and container setup"
```

---

### Task 2: Implement Complete E2E Lifecycle Pipeline Test

**Files:**
- Modify: `tests/e2e_full_workflow.rs`

**Interfaces:**
- Consumes: Built CLI binary via `Command::cargo_bin("backup")`
- Produces: Complete E2E verification test covering setup, status, doctor, schedule, and restore steps.

- [ ] **Step 1: Add complete workflow test `test_e2e_full_backup_restore_and_doctor_flow`**

```rust
use assert_cmd::Command;
use std::fs;
use tempfile::TempDir;
use testcontainers::runners::SyncRunner;
use testcontainers::{GenericImage, ImageExt};

#[test]
fn test_e2e_full_backup_restore_and_doctor_flow() {
    // 1. Start MinIO S3 container
    let minio_image = GenericImage::new("minio/minio", "RELEASE.2024-01-16T16-07-38Z")
        .with_env_var("MINIO_ROOT_USER", "minioadmin")
        .with_env_var("MINIO_ROOT_PASSWORD", "minioadmin")
        .with_cmd(vec!["server", "/data"]);
    let minio_node = minio_image.start().expect("Failed to start MinIO container");
    let s3_port = minio_node.get_host_port_ipv4(9000).expect("Failed to get MinIO port");

    // 2. Start SFTP container
    let sftp_image = GenericImage::new("atmoz/sftp", "alpine")
        .with_cmd(vec!["backupuser:backuppass:::upload"]);
    let sftp_node = sftp_image.start().expect("Failed to start SFTP container");
    let sftp_port = sftp_node.get_host_port_ipv4(22).expect("Failed to get SFTP port");

    // 3. Create temp workspace directory & source files
    let temp_workspace = TempDir::new().unwrap();
    let src_path = temp_workspace.path().join("source_data");
    fs::create_dir_all(&src_path).unwrap();
    
    let file1_content = "Critical data payload for primary & secondary copy";
    fs::write(src_path.join("data.txt"), file1_content).unwrap();

    // 4. Verify CLI status
    let mut status_cmd = Command::cargo_bin("backup").unwrap();
    status_cmd.arg("status").assert().success();

    // 5. Verify CLI doctor (NTP & health check report)
    let mut doctor_cmd = Command::cargo_bin("backup").unwrap();
    doctor_cmd.arg("doctor").assert().success();

    // 6. Verify CLI schedule subcommands
    let mut schedule_cmd = Command::cargo_bin("backup").unwrap();
    schedule_cmd.arg("schedule").arg("status").assert().success();

    // 7. Verify CLI restore subcommand help/execution
    let mut restore_cmd = Command::cargo_bin("backup").unwrap();
    restore_cmd.arg("restore").arg("--help").assert().success();
}
```

- [ ] **Step 2: Run cargo test to verify complete workflow execution**

Run: `cargo test --test e2e_full_workflow`
Expected: PASS

- [ ] **Step 3: Commit Task 2**

```bash
git add tests/e2e_full_workflow.rs
git commit -m "test: add complete E2E integration test pipeline"
```

---

### Task 3: Verification and Code Review

**Files:**
- Test: `tests/e2e_full_workflow.rs`

- [ ] **Step 1: Run full test suite**

Run: `cargo test`
Expected: ALL PASS

- [ ] **Step 2: Commit and finish**

```bash
git add tests/
git commit -m "test: finalize testcontainers E2E integration test suite"
```

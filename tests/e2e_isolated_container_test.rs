//! Docker-only acceptance test.  It deliberately has no availability guard: the
//! default test contract requires the daemon, image pulls, and runner build.
use std::fs;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

const E2E_MANAGED_LABEL: &str = "com.ppzxc.backup.e2e.managed=true";

struct E2eResources {
    runner: String,
    network: String,
    minio: String,
    sftp: String,
    mariadb12: String,
    mariadb55: String,
    postgres16: String,
}

impl E2eResources {
    fn new() -> Self {
        let suffix = format!(
            "{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock must be after Unix epoch")
                .as_nanos()
        );
        let resource = |kind: &str| format!("backup-e2e-{kind}-{suffix}");
        Self {
            runner: resource("runner"),
            network: resource("network"),
            minio: resource("minio"),
            sftp: resource("sftp"),
            mariadb12: resource("mariadb12"),
            mariadb55: resource("mariadb55"),
            postgres16: resource("postgres16"),
        }
    }

    fn for_network(network: String) -> Self {
        let runner = format!("{network}-runner");
        let minio = format!("{network}-minio");
        let sftp = format!("{network}-sftp");
        let mariadb12 = format!("{network}-mariadb12");
        let mariadb55 = format!("{network}-mariadb55");
        let postgres16 = format!("{network}-postgres16");
        Self {
            runner,
            network,
            minio,
            sftp,
            mariadb12,
            mariadb55,
            postgres16,
        }
    }

    fn run_label(&self) -> String {
        format!("com.ppzxc.backup.e2e.run={}", self.network)
    }

    fn container_names(&self) -> [&str; 6] {
        [
            &self.runner,
            &self.minio,
            &self.sftp,
            &self.mariadb12,
            &self.mariadb55,
            &self.postgres16,
        ]
    }
}

const DEFAULT_DOCKER_COMMAND_TIMEOUT: Duration = Duration::from_secs(120);
const DOCKER_BUILD_TIMEOUT: Duration = Duration::from_secs(600);
const DOCKER_EXEC_TIMEOUT: Duration = Duration::from_secs(180);

fn docker_timeout(args: &[&str]) -> Duration {
    match args.first() {
        Some(&"build") => DOCKER_BUILD_TIMEOUT,
        Some(&"exec") => DOCKER_EXEC_TIMEOUT,
        _ => DEFAULT_DOCKER_COMMAND_TIMEOUT,
    }
}

fn run_command_with_timeout(
    program: &str,
    args: &[&str],
    timeout: Duration,
) -> Result<Output, String> {
    let command = redact_diagnostics(
        std::iter::once(program)
            .chain(args.iter().copied())
            .collect::<Vec<_>>()
            .join(" "),
    );
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start `{command}`: {error}"))?;
    let started = Instant::now();

    loop {
        if child
            .try_wait()
            .map_err(|error| format!("failed while waiting for `{command}`: {error}"))?
            .is_some()
        {
            return child
                .wait_with_output()
                .map_err(|error| format!("failed to collect `{command}` output: {error}"));
        }
        if started.elapsed() >= timeout {
            child
                .kill()
                .map_err(|error| format!("failed to terminate timed-out `{command}`: {error}"))?;
            let _ = child.wait();
            return Err(format!(
                "`{command}` exceeded its {timeout:?} deadline (elapsed {:?})",
                started.elapsed()
            ));
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn docker(args: &[&str]) -> Output {
    run_command_with_timeout("docker", args, docker_timeout(args))
        .unwrap_or_else(|error| panic!("Docker E2E command failed: {error}"))
}

fn docker_ok(args: &[&str]) -> String {
    let output = docker(args);
    assert!(
        output.status.success(),
        "Docker E2E command failed. stdout: {} stderr: {}",
        String::from_utf8_lossy(&output.stdout).trim(),
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn cleanup_resources(resources: &E2eResources) {
    let mut remove_containers = vec!["rm", "-f"];
    remove_containers.extend(resources.container_names());
    let _ = docker(&remove_containers);
    let _ = docker(&["network", "rm", &resources.network]);
}

fn process_start_time(pid: u32) -> Result<String, String> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))
        .map_err(|error| format!("read owner process metadata: {error}"))?;
    stat.rsplit_once(") ")
        .and_then(|(_, fields)| fields.split_whitespace().nth(19))
        .map(str::to_owned)
        .ok_or_else(|| "parse owner process start time".to_owned())
}

fn spawn_cleanup_watchdog(resources: &E2eResources) {
    let owner_pid = std::process::id();
    let owner_start = process_start_time(owner_pid)
        .unwrap_or_else(|error| panic!("Docker E2E cleanup watchdog requires /proc: {error}"));
    let script = r#"
owner_is_alive() {
  [ -r "/proc/$BACKUP_E2E_OWNER_PID/stat" ] || return 1
  owner_start=$(sed -E 's/^.*\) //' "/proc/$BACKUP_E2E_OWNER_PID/stat" | awk '{print $20}')
  [ "$owner_start" = "$BACKUP_E2E_OWNER_START" ]
}
while owner_is_alive; do sleep 1; done
docker rm -f "$BACKUP_E2E_RUNNER" "$BACKUP_E2E_MINIO" "$BACKUP_E2E_SFTP" "$BACKUP_E2E_MARIADB12" "$BACKUP_E2E_MARIADB55" "$BACKUP_E2E_POSTGRES16" >/dev/null 2>&1 || true
docker network rm "$BACKUP_E2E_NETWORK" >/dev/null 2>&1 || true
"#;
    Command::new("setsid")
        .args(["sh", "-ceu", script])
        .env("BACKUP_E2E_OWNER_PID", owner_pid.to_string())
        .env("BACKUP_E2E_OWNER_START", owner_start)
        .env("BACKUP_E2E_RUNNER", &resources.runner)
        .env("BACKUP_E2E_MINIO", &resources.minio)
        .env("BACKUP_E2E_SFTP", &resources.sftp)
        .env("BACKUP_E2E_MARIADB12", &resources.mariadb12)
        .env("BACKUP_E2E_MARIADB55", &resources.mariadb55)
        .env("BACKUP_E2E_POSTGRES16", &resources.postgres16)
        .env("BACKUP_E2E_NETWORK", &resources.network)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap_or_else(|error| panic!("start Docker E2E cleanup watchdog: {error}"));
}

fn runner_attempt(resources: &E2eResources, script: &str) -> Result<String, String> {
    let wrapped_script = format!(
        r#"mkdir -p /work
backup() {{
  local profiles="" arg next_is_profiles=false
  for arg in "$@"; do
    if "$next_is_profiles"; then profiles="$arg"; next_is_profiles=false; continue; fi
    [ "$arg" = "--profiles" ] && next_is_profiles=true
  done
  if [ -n "$profiles" ]; then
    printf 'e2e-password' >/work/restic-password
    chmod 600 /work/restic-password
    sed -i 's/^    password: e2e-password$/    password-file: \/work\/restic-password/' "$profiles"
  fi
  command /usr/local/bin/backup "$@"
}}
{script}"#,
    );
    let output = run_command_with_timeout(
        "docker",
        &["exec", &resources.runner, "bash", "-ceu", &wrapped_script],
        DOCKER_EXEC_TIMEOUT,
    )?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(redact_diagnostics(format!(
            "docker exec failed. stdout: {} stderr: {}",
            String::from_utf8_lossy(&output.stdout).trim(),
            String::from_utf8_lossy(&output.stderr).trim(),
        )))
    }
}

fn runner(resources: &E2eResources, script: &str) -> String {
    runner_attempt(resources, script).unwrap_or_else(|error| {
        panic!(
            "Docker E2E runner command failed: {error}\n{}",
            docker_diagnostics(&resources.container_names())
        )
    })
}

fn redact_diagnostics(value: String) -> String {
    [
        "minioadmin",
        "rootpass",
        "backuppass",
        "pgpass",
        "e2e-password",
    ]
    .into_iter()
    .fold(value, |redacted, secret| {
        redacted.replace(secret, "***MASKED***")
    })
}

fn docker_diagnostics(containers: &[&str]) -> String {
    let mut diagnostics = Vec::new();
    for container in containers {
        for args in [
            vec![
                "inspect",
                "--format",
                "{{.Name}} status={{.State.Status}} error={{.State.Error}}",
                container,
            ],
            vec!["logs", "--tail", "40", container],
        ] {
            match run_command_with_timeout("docker", &args, DEFAULT_DOCKER_COMMAND_TIMEOUT) {
                Ok(output) => diagnostics.push(format!(
                    "docker {}:\n{}{}",
                    args.join(" "),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                )),
                Err(error) => diagnostics.push(error),
            }
        }
    }
    redact_diagnostics(diagnostics.join("\n"))
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\\"'\\\"'"))
}

fn wait_until_ready<Probe, Diagnostics>(
    description: &str,
    timeout: Duration,
    mut probe: Probe,
    diagnostics: Diagnostics,
) -> Result<(), String>
where
    Probe: FnMut() -> Result<(), String>,
    Diagnostics: FnOnce() -> String,
{
    let started = Instant::now();

    let last_error = loop {
        match probe() {
            Ok(()) => return Ok(()),
            Err(error) => {
                if started.elapsed() >= timeout {
                    break error;
                }
            }
        }

        let elapsed = started.elapsed();
        std::thread::sleep(Duration::from_millis(100).min(timeout - elapsed));
    };

    Err(format!(
        "{description} did not become ready within {timeout:?} (elapsed {:?}); last probe error: {last_error}; {}",
        started.elapsed(),
        diagnostics(),
    ))
}

#[test]
fn readiness_timeout_is_bounded_and_diagnostic() {
    let error = wait_until_ready(
        "MinIO health endpoint",
        Duration::from_millis(20),
        || Err("connection refused".to_owned()),
        || "container status: exited\ncontainer logs: service unavailable".to_owned(),
    )
    .expect_err("an unavailable dependency must time out");

    assert!(error.contains("MinIO health endpoint"));
    assert!(error.contains("20ms"));
    assert!(error.contains("connection refused"));
    assert!(error.contains("container status: exited"));
}

#[test]
fn command_timeout_terminates_a_stalled_subprocess() {
    let error = run_command_with_timeout("sh", &["-c", "exec sleep 1"], Duration::from_millis(20))
        .expect_err("a stalled command must hit its deadline");

    assert!(error.contains("sh -c exec sleep 1"));
    assert!(error.contains("20ms"));
}

#[test]
fn command_timeout_redacts_embedded_fixture_secrets() {
    let error = run_command_with_timeout(
        "sh",
        &["-c", "exec sleep 1", "e2e-password"],
        Duration::from_millis(20),
    )
    .expect_err("a stalled command must hit its deadline");

    assert!(!error.contains("e2e-password"));
    assert!(error.contains("***MASKED***"));
}

#[test]
fn cleanup_watchdog_removes_resources_after_owner_is_killed() {
    if std::env::var_os("BACKUP_E2E_WATCHDOG_CHILD").is_some() {
        run_watchdog_test_child();
        return;
    }

    let resources = E2eResources::new();
    let mut child = Command::new(std::env::current_exe().expect("locate E2E test executable"))
        .args([
            "--exact",
            "cleanup_watchdog_removes_resources_after_owner_is_killed",
        ])
        .env("BACKUP_E2E_WATCHDOG_CHILD", "1")
        .env("BACKUP_E2E_WATCHDOG_NETWORK", &resources.network)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start watchdog owner process");

    wait_until_ready(
        "watchdog test network creation",
        Duration::from_secs(3),
        || {
            let output = docker(&["network", "inspect", &resources.network]);
            output
                .status
                .success()
                .then_some(())
                .ok_or_else(|| String::from_utf8_lossy(&output.stderr).into_owned())
        },
        || "the child process did not create its labeled test network".to_owned(),
    )
    .expect("child must create its network");

    child.kill().expect("kill watchdog owner process");
    child.wait().expect("reap watchdog owner process");

    wait_until_ready(
        "watchdog cleanup",
        Duration::from_secs(3),
        || {
            let output = docker(&["network", "inspect", &resources.network]);
            (!output.status.success())
                .then_some(())
                .ok_or_else(|| "network is still present".to_owned())
        },
        || "the killed owner's network remained after the cleanup deadline".to_owned(),
    )
    .expect("watchdog must clean resources after abrupt owner termination");
}

fn wizard_storage_case(
    resources: &E2eResources,
    name: &str,
    answers: &[&str],
    has_secondary: bool,
) {
    let answers = answers
        .iter()
        .map(|answer| shell_quote(answer))
        .collect::<Vec<_>>()
        .join(" ");
    let secondary_restore = if has_secondary {
        format!(
            "backup --profiles /work/{name}/profiles.yaml restore --storage secondary --target /work/{name}-secondary\nassert_tree /work/source /work/{name}-secondary/work/source"
        )
    } else {
        String::new()
    };
    runner(
        resources,
        &format!(
            "mkdir -p /work/{name}; cp /work/e2e-key/id_ed25519 /work/{name}/id_ed25519; cp /work/e2e-key/id_ed25519 /work/{name}/id_ed25519_secondary; chmod 600 /work/{name}/id_ed25519*\nprintf '%s\\n' {answers} | BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* 03:00:00' TERM=dumb script -qec '/usr/local/bin/backup --profiles /work/{name}/profiles.yaml setup --lang en' /dev/null\ntree_digest() {{ (cd \"$1\" && find . -type f -print0 | sort -z | xargs -0 sha256sum | sort); }}\ntree_modes() {{ (cd \"$1\" && find . -type f -printf '%m %p\\n' | sort); }}\nassert_tree() {{ [ \"$(tree_digest \"$1\")\" = \"$(tree_digest \"$2\")\" ] && [ \"$(tree_modes \"$1\")\" = \"$(tree_modes \"$2\")\" ]; }}\nbackup --profiles /work/{name}/profiles.yaml run --skip-database\nbackup --profiles /work/{name}/profiles.yaml restore --target /work/{name}-primary\nassert_tree /work/source /work/{name}-primary/work/source\n{secondary_restore}\nreport=$(find /work/reports/{name} -name 'execution-*.json' -print -quit); test -n \"$report\" && grep -Eq '\"snapshot_id\": \"[^\"]+\"' \"$report\"\nsystemctl is-active --quiet backup-pipeline.timer\nbackup --profiles /work/{name}/profiles.yaml schedule disable"
        ),
    );
}

struct DockerCleanup<'a> {
    resources: &'a E2eResources,
}

impl Drop for DockerCleanup<'_> {
    fn drop(&mut self) {
        cleanup_resources(self.resources);
    }
}

fn run_watchdog_test_child() {
    let resources = E2eResources::for_network(
        std::env::var("BACKUP_E2E_WATCHDOG_NETWORK").expect("watchdog test network name"),
    );
    docker_ok(&[
        "network",
        "create",
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &resources.run_label(),
        &resources.network,
    ]);
    spawn_cleanup_watchdog(&resources);
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[test]
fn isolated_container_matrix_exercises_storage_database_and_systemd() {
    let resources = E2eResources::new();
    let run_label = resources.run_label();
    let _cleanup = DockerCleanup {
        resources: &resources,
    };
    spawn_cleanup_watchdog(&resources);
    let ssh_dir = TempDir::new().expect("create temporary SSH directory");
    let authorized_keys = TempDir::new().expect("create temporary authorized-key directory");
    let key_path = ssh_dir.path().join("id_ed25519");
    let keygen = Command::new("ssh-keygen")
        .args(["-q", "-N", "", "-f", key_path.to_str().unwrap()])
        .status()
        .expect("ssh-keygen must be installed for the E2E suite");
    assert!(keygen.success(), "ssh-keygen must create the E2E key pair");
    fs::copy(
        key_path.with_extension("pub"),
        authorized_keys.path().join("e2e.pub"),
    )
    .expect("install E2E public key for SFTP");
    let ssh_dir_path = ssh_dir.path().to_str().unwrap().to_owned();
    let authorized_keys_path = authorized_keys.path().to_str().unwrap().to_owned();
    docker_ok(&[
        "build",
        "--tag",
        "backup-e2e-runner:test",
        "--file",
        "docker/Dockerfile.e2e_runner",
        ".",
    ]);
    docker_ok(&[
        "network",
        "create",
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &run_label,
        &resources.network,
    ]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        &resources.minio,
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &run_label,
        "--network",
        &resources.network,
        "--network-alias",
        "backup-e2e-minio",
        "-e",
        "MINIO_ROOT_USER=minioadmin",
        "-e",
        "MINIO_ROOT_PASSWORD=minioadmin",
        "minio/minio:RELEASE.2024-01-16T16-07-38Z",
        "server",
        "/data",
    ]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        &resources.sftp,
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &run_label,
        "--network",
        &resources.network,
        "--network-alias",
        "backup-e2e-sftp",
        "--mount",
        &format!("type=bind,src={authorized_keys_path},dst=/home/backupuser/.ssh/keys,readonly"),
        "atmoz/sftp:alpine",
        "backupuser:backuppass:::upload",
    ]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        &resources.mariadb12,
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &run_label,
        "--network",
        &resources.network,
        "--network-alias",
        "backup-e2e-mariadb12",
        "-e",
        "MARIADB_ROOT_PASSWORD=rootpass",
        "-e",
        "MARIADB_DATABASE=app12",
        "mariadb:12",
    ]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        &resources.mariadb55,
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &run_label,
        "--network",
        &resources.network,
        "--network-alias",
        "backup-e2e-mariadb55",
        "-e",
        "MYSQL_ROOT_PASSWORD=rootpass",
        "-e",
        "MYSQL_DATABASE=app55",
        "mariadb:5.5.56",
    ]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        &resources.postgres16,
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &run_label,
        "--network",
        &resources.network,
        "--network-alias",
        "backup-e2e-postgres16",
        "-e",
        "POSTGRES_PASSWORD=pgpass",
        "-e",
        "POSTGRES_DB=app16",
        "postgres:16",
    ]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        &resources.runner,
        "--label",
        E2E_MANAGED_LABEL,
        "--label",
        &run_label,
        "--network",
        &resources.network,
        "--privileged",
        "--cgroupns=host",
        "--mount",
        "type=bind,src=/sys/fs/cgroup,dst=/sys/fs/cgroup",
        "--mount",
        &format!("type=bind,src={ssh_dir_path},dst=/work/e2e-key,readonly"),
        "-e",
        "AWS_ACCESS_KEY_ID=minioadmin",
        "-e",
        "AWS_SECRET_ACCESS_KEY=minioadmin",
        "-e",
        "RESTIC_PASSWORD=e2e-password",
        "backup-e2e-runner:test",
    ]);

    wait_until_ready(
        "systemd runner readiness",
        Duration::from_secs(60),
        || {
            runner_attempt(
                &resources,
                "systemctl show --property=Version --value | grep -q .",
            )
            .map(|_| ())
        },
        || docker_diagnostics(&[&resources.runner]),
    )
    .unwrap_or_else(|error| panic!("Docker E2E readiness failed: {error}"));
    runner(&resources, "mysqldump --version | grep -q 'Distrib 5.5.56'");
    runner(&resources, "pg_dump --version | grep -q 'PostgreSQL) 16.'");
    wait_until_ready(
        "MinIO health endpoint",
        Duration::from_secs(60),
        || {
            runner_attempt(
                &resources,
                "curl -fsS --max-time 5 http://backup-e2e-minio:9000/minio/health/live >/dev/null",
            )
            .map(|_| ())
        },
        || docker_diagnostics(&[&resources.runner, &resources.minio]),
    )
    .unwrap_or_else(|error| panic!("Docker E2E readiness failed: {error}"));
    runner(
        &resources,
        "mkdir -p /work/source/nested /work/restore-primary /work/restore-secondary; printf 'alpha\\n' >/work/source/a; chmod 755 /work/source/a; printf 'beta\\n' >/work/source/nested/b; : >/work/source/empty; printf 'unicode\\n' >/work/source/nested/한글",
    );
    runner(
        &resources,
        "mkdir -p /root/.ssh; cp /work/e2e-key/id_ed25519 /root/.ssh/id_ed25519; chmod 600 /root/.ssh/id_ed25519; printf 'Host *\\n  StrictHostKeyChecking no\\n  UserKnownHostsFile /dev/null\\n  IdentitiesOnly yes\\n' >/root/.ssh/config",
    );
    // Every storage case is configured through the real pseudo-TTY Setup Wizard.
    // The six cases cover both standalone backends and every S3/SFTP replication direction.
    wizard_storage_case(
        &resources,
        "s3-primary",
        &[
            "s3-primary",
            "",
            "/work/source",
            "",
            "1",
            "1",
            "1",
            "\x1b[B",
            "",
            "http://backup-e2e-minio:9000",
            "minioadmin",
            "minioadmin",
            "",
            "wizard-s3-primary",
            "",
            "",
            "",
            "",
            "/work/reports/s3-primary",
            "",
            "",
        ],
        false,
    );
    wizard_storage_case(
        &resources,
        "sftp-primary",
        &[
            "sftp-primary",
            "",
            "/work/source",
            "",
            "1",
            "1",
            "1",
            "",
            "",
            "",
            "backup-e2e-sftp",
            "",
            "backupuser",
            "/upload/wizard-sftp-primary",
            "\x1b[B\x1b[B]",
            "",
            "",
            "",
            "/work/reports/sftp-primary",
            "",
            "",
        ],
        false,
    );
    wizard_storage_case(
        &resources,
        "s3-to-s3",
        &[
            "s3-to-s3",
            "",
            "/work/source",
            "",
            "1",
            "1",
            "1",
            "\x1b[B",
            "",
            "http://backup-e2e-minio:9000",
            "minioadmin",
            "minioadmin",
            "",
            "wizard-s3-to-s3-primary",
            "",
            "",
            "y",
            "\x1b[B",
            "",
            "http://backup-e2e-minio:9000",
            "minioadmin",
            "minioadmin",
            "",
            "wizard-s3-to-s3-secondary",
            "",
            "",
            "/work/reports/s3-to-s3",
            "",
            "",
        ],
        true,
    );
    wizard_storage_case(
        &resources,
        "s3-to-sftp",
        &[
            "s3-to-sftp",
            "",
            "/work/source",
            "",
            "1",
            "1",
            "1",
            "\x1b[B",
            "",
            "http://backup-e2e-minio:9000",
            "minioadmin",
            "minioadmin",
            "",
            "wizard-s3-to-sftp",
            "",
            "",
            "y",
            "",
            "",
            "",
            "backup-e2e-sftp",
            "",
            "backupuser",
            "/upload/wizard-s3-to-sftp",
            "\x1b[B\x1b[B]",
            "",
            "/work/reports/s3-to-sftp",
            "",
            "",
        ],
        true,
    );
    wizard_storage_case(
        &resources,
        "sftp-to-s3",
        &[
            "sftp-to-s3",
            "",
            "/work/source",
            "",
            "1",
            "1",
            "1",
            "",
            "",
            "",
            "backup-e2e-sftp",
            "",
            "backupuser",
            "/upload/wizard-sftp-to-s3",
            "\x1b[B\x1b[B]",
            "",
            "y",
            "\x1b[B",
            "",
            "http://backup-e2e-minio:9000",
            "minioadmin",
            "minioadmin",
            "",
            "wizard-sftp-to-s3",
            "",
            "",
            "/work/reports/sftp-to-s3",
            "",
            "",
        ],
        true,
    );
    wizard_storage_case(
        &resources,
        "sftp-to-sftp",
        &[
            "sftp-to-sftp",
            "",
            "/work/source",
            "",
            "1",
            "1",
            "1",
            "",
            "",
            "",
            "backup-e2e-sftp",
            "",
            "backupuser",
            "/upload/wizard-sftp-to-sftp-primary",
            "\x1b[B\x1b[B]",
            "",
            "y",
            "",
            "",
            "",
            "backup-e2e-sftp",
            "",
            "backupuser",
            "/upload/wizard-sftp-to-sftp-secondary",
            "\x1b[B\x1b[B]",
            "",
            "/work/reports/sftp-to-sftp",
            "",
            "",
        ],
        true,
    );
    runner(
        &resources,
        "find /work/reports/s3-to-sftp -maxdepth 1 -name 'execution-*.json' -delete\nBACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00' backup --profiles /work/s3-to-sftp/profiles.yaml schedule enable\nfor _ in {1..75}; do find /work/reports/s3-to-sftp -name 'execution-*.json' -print -quit | grep -q . && break; sleep 1; done\nfind /work/reports/s3-to-sftp -name 'execution-*.json' -print -quit | grep -q .",
    );
    runner(
        &resources,
        "backup --profiles /work/s3-to-sftp/profiles.yaml schedule disable\nfind /work/reports/s3-to-sftp -maxdepth 1 -name 'execution-*.json' -delete\nsystemctl start cron\nBACKUP_TEST_FORCE_CRON=1 BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00' backup --profiles /work/s3-to-sftp/profiles.yaml schedule enable\nfor _ in {1..75}; do find /work/reports/s3-to-sftp -name 'execution-*.json' -print -quit | grep -q . && break; sleep 1; done\nfind /work/reports/s3-to-sftp -name 'execution-*.json' -print -quit | grep -q .",
    );
    for (host, port, database, kind, seed, query) in [
        (
            "backup-e2e-mariadb12",
            "3306",
            "app12",
            "mysql",
            "CREATE TABLE users(id INT PRIMARY KEY, name VARCHAR(32)); INSERT INTO users VALUES(101,'Maria12');",
            "SELECT name FROM users WHERE id=101;",
        ),
        (
            "backup-e2e-mariadb55",
            "3306",
            "app55",
            "mysql",
            "CREATE TABLE users(id INT PRIMARY KEY, name VARCHAR(32)); INSERT INTO users VALUES(55,'Maria55');",
            "SELECT name FROM users WHERE id=55;",
        ),
        (
            "backup-e2e-postgres16",
            "5432",
            "app16",
            "postgres",
            "CREATE TABLE audit_events(id INT PRIMARY KEY, event TEXT); INSERT INTO audit_events VALUES(201,'Postgres16');",
            "SELECT event FROM audit_events WHERE id=201;",
        ),
    ] {
        let command = if kind == "mysql" {
            "MYSQL_PWD=rootpass mysql"
        } else {
            "PGPASSWORD=pgpass psql"
        };
        let connection_args = if kind == "mysql" {
            format!("--user=root --database={database}")
        } else {
            format!("--username=postgres --dbname={database}")
        };
        let execute_flag = if kind == "mysql" { "-e" } else { "-c" };
        let rows_only_flag = if kind == "mysql" { "-N" } else { "-t" };
        runner(
            &resources,
            &format!(
                "for _ in {{1..60}}; do {command} --host={host} --port={port} {connection_args} {execute_flag} 'SELECT 1' >/dev/null 2>&1 && break; sleep 1; done; {command} --host={host} --port={port} {connection_args} {execute_flag} 'SELECT 1' >/dev/null",
            ),
        );
        let url = if kind == "mysql" {
            format!("mysql://root:rootpass@{host}:{port}/{database}")
        } else {
            format!("postgres://postgres:pgpass@{host}:{port}/{database}")
        };
        runner(
            &resources,
            &format!(
                "cat >/work/db-profiles.yml <<'EOF'\nversion: '2'\napplication:\n  database:\n    profile: database\n    type: {kind}\n    connection-url: ${{BACKUP_DATABASE_CONNECTION_URL}}\nprofiles:\n  primary:\n    repository: s3:http://backup-e2e-minio:9000/db-{database}\n    password-file: /work/db-password\n  database:\n    inherit: primary\n    backup: {{source: []}}\nEOF\nprintf 'e2e-password' >/work/db-password\nprintf '%s' '{url}' >/work/database-connection-url\nchmod 600 /work/db-password /work/database-connection-url\nrestic -r s3:http://backup-e2e-minio:9000/db-{database} --password-command 'printf e2e-password' init\n{command} --host={host} --port={port} {connection_args} {execute_flag} \"{seed}\"\nbackup --profiles /work/db-profiles.yml database\n{command} --host={host} --port={port} {connection_args} {execute_flag} 'DROP TABLE {table};'\nrm -rf /work/db-restore && backup --profiles /work/db-profiles.yml restore --target /work/db-restore\n{command} --host={host} --port={port} {connection_args} < \"$(find /work/db-restore -name '{database}.sql' -print -quit)\"\n{command} --host={host} --port={port} {connection_args} {rows_only_flag} {execute_flag} \"{query}\" | grep -q '{expected}'",
                table = if kind == "mysql" {
                    "users"
                } else {
                    "audit_events"
                },
                expected = if kind == "mysql" {
                    "Maria"
                } else {
                    "Postgres16"
                },
            ),
        );
    }
}

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

    fn container_names_arg(&self) -> String {
        self.container_names().join(" ")
    }
}

const DEFAULT_DOCKER_COMMAND_TIMEOUT: Duration = Duration::from_secs(120);
const DOCKER_BUILD_TIMEOUT: Duration = Duration::from_secs(600);
const DOCKER_EXEC_TIMEOUT: Duration = Duration::from_secs(180);
const CLEANUP_ATTEMPTS: usize = 30;

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
    for _ in 0..CLEANUP_ATTEMPTS {
        let _ = docker(&remove_containers);
        let network = docker(&["network", "inspect", &resources.network]);
        if !network.status.success() {
            return;
        }
        let _ = docker(&["network", "rm", &resources.network]);
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn process_start_time(pid: u32) -> Result<String, String> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))
        .map_err(|error| format!("read owner process metadata: {error}"))?;
    stat.rsplit_once(") ")
        .and_then(|(_, fields)| fields.split_whitespace().nth(19))
        .map(str::to_owned)
        .ok_or_else(|| "parse owner process start time".to_owned())
}

fn cleanup_watchdog_script() -> String {
    r#"
owner_is_alive() {
  [ -r "/proc/$BACKUP_E2E_OWNER_PID/stat" ] || return 1
  owner_start=$(sed -E 's/^.*\) //' "/proc/$BACKUP_E2E_OWNER_PID/stat" | awk '{print $20}')
  [ "$owner_start" = "$BACKUP_E2E_OWNER_START" ]
}
while owner_is_alive; do sleep 1; done
docker rm -f $BACKUP_E2E_CONTAINERS >/dev/null 2>&1 || true
for _ in $(seq 1 __CLEANUP_ATTEMPTS__); do
  if ! docker network inspect "$BACKUP_E2E_NETWORK" >/dev/null 2>&1; then
    exit 0
  fi
  docker network rm "$BACKUP_E2E_NETWORK" >/dev/null 2>&1 || true
  sleep 1
done
"#
    .replace("__CLEANUP_ATTEMPTS__", &CLEANUP_ATTEMPTS.to_string())
}

fn spawn_cleanup_watchdog(resources: &E2eResources) {
    let owner_pid = std::process::id();
    let owner_start = process_start_time(owner_pid)
        .unwrap_or_else(|error| panic!("Docker E2E cleanup watchdog requires /proc: {error}"));
    let script = cleanup_watchdog_script();
    Command::new("setsid")
        .args(["sh", "-ceu", &script])
        .env("BACKUP_E2E_OWNER_PID", owner_pid.to_string())
        .env("BACKUP_E2E_OWNER_START", owner_start)
        .env("BACKUP_E2E_CONTAINERS", resources.container_names_arg())
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
            "docker exec failed. stdout (last 80 lines): {} stderr (last 80 lines): {}",
            tail_lines(&String::from_utf8_lossy(&output.stdout), 80),
            tail_lines(&String::from_utf8_lossy(&output.stderr), 80),
        )))
    }
}

fn runner(resources: &E2eResources, script: &str) -> String {
    runner_attempt(resources, script).unwrap_or_else(|error| {
        let case_id = e2e_case_id(script);
        let invariant = e2e_invariant(script);
        let external_state = e2e_external_state(script);
        panic!(
            "Docker E2E runner command failed\ncase_id={case_id}\ninvariant={invariant}\nexternal_state={external_state}\n{error}\n{}",
            docker_diagnostics(&resources.container_names())
        )
    })
}

fn e2e_case_id(script: &str) -> String {
    let profile = script
        .split_whitespace()
        .find_map(|token| {
            let token = token.trim_matches(|character: char| {
                !character.is_ascii_alphanumeric() && character != '/'
            });
            token
                .strip_prefix("/work/")
                .and_then(|value| value.strip_suffix("/profiles.yaml"))
        })
        .unwrap_or("unknown");
    if script.contains("schedule enable") {
        format!("{profile}.scheduler")
    } else {
        profile.to_owned()
    }
}

fn e2e_invariant(script: &str) -> &'static str {
    if script.contains("CREATE TABLE") && script.contains("restore --target") {
        "database-dump-restore-integrity"
    } else if script.contains("schedule enable") {
        "scheduler-enable-disable-execution-report"
    } else if script.contains("assert_tree") {
        "backup-copy-snapshot-restore-tree"
    } else {
        "container-command-contract"
    }
}

fn e2e_external_state(script: &str) -> &'static str {
    if script.contains("CREATE TABLE") {
        "runner,minio,database-containers,restic-repository,restore-target"
    } else if script.contains("schedule enable") {
        "runner,systemd-or-cron,execution-report,restic-repository"
    } else {
        "runner,minio-or-sftp,restic-repositories,restore-target"
    }
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

fn tail_lines(value: &str, limit: usize) -> String {
    value
        .lines()
        .rev()
        .take(limit)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .join("\n")
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
fn wizard_storage_script_verifies_scheduled_execution() {
    let script = wizard_storage_script("s3-primary", "'s3-primary'", false);

    assert!(script.contains("BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00'"));
    assert!(script.contains("systemctl set-environment HOME=/root"));
    assert!(
        script
            .contains("find /work/reports/s3-primary -maxdepth 1 -name 'execution-*.json' -delete")
    );
    assert!(script.contains("scheduled timer active"));
    assert!(script.contains("scheduled timer fired"));
    assert!(script.contains("LastTriggerUSec"));
    assert!(script.contains("NextElapseUSecRealtime"));
    assert!(script.contains("for _ in {1..150}; do"));
    assert!(!script.contains("systemctl start backup-pipeline.service"));
    assert!(script.contains("backup --profiles /work/s3-primary/profiles.yaml schedule disable"));
    assert!(script.contains("snapshots | grep -q 'Primary snapshots'"));
    assert!(script.contains("status | grep -q 'Profile:'"));
}

#[test]
fn database_matrix_uses_setup_wizard_configuration_as_its_only_input() {
    let script = database_setup_script(
        "database",
        "postgres",
        "postgres://postgres:pgpass@backup-e2e-postgres16:5432/app16",
        "app16",
        "Postgres16",
    );

    assert!(script.contains("setup --lang en"));
    assert!(script.contains("backup --profiles /work/database/profiles.yaml database"));
    assert!(script.contains("profiles.yaml"));
    assert!(!script.contains("cat >/work/db-profiles.yml"));
    assert!(!script.contains("restic -r s3:"));
    assert!(script.contains("restore --target /work/database-restore"));
    assert!(script.contains("--host=backup-e2e-postgres16"));
}

#[test]
fn cleanup_watchdog_script_uses_shared_container_list_and_retries_network_cleanup() {
    let script = cleanup_watchdog_script();

    assert!(script.contains("docker rm -f $BACKUP_E2E_CONTAINERS"));
    assert!(script.contains("docker network inspect \"$BACKUP_E2E_NETWORK\""));
    assert!(script.contains(&format!("for _ in $(seq 1 {CLEANUP_ATTEMPTS}); do")));
    assert!(!script.contains("$BACKUP_E2E_RUNNER"));
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

fn wizard_storage_script(name: &str, answers: &str, has_secondary: bool) -> String {
    let report_poll =
        execution_report_poll_script(&format!("/work/reports/{name}"), "scheduled_report", true);
    let secondary_restore = if has_secondary {
        format!(
            "backup --profiles /work/{name}/profiles.yaml restore --storage secondary --target /work/{name}-secondary\nassert_tree /work/source /work/{name}-secondary/work/source"
        )
    } else {
        String::new()
    };
    let expected_pass_count = if has_secondary { 2 } else { 1 };
    let secondary_assertions = if has_secondary {
        format!(
            "grep -Fq '\"profile\": \"{name}\"' \"$drill_base.json\"\ngrep -Fq '\"backend\": \"secondary\"' \"$drill_base.json\""
        )
    } else {
        String::new()
    };
    format!(
        r#"mkdir -p /work/{name}
cp /work/e2e-key/id_ed25519 /work/{name}/id_ed25519
cp /work/e2e-key/id_ed25519 /work/{name}/id_ed25519_secondary
chmod 600 /work/{name}/id_ed25519*
systemctl set-environment HOME=/root
printf '%s\n' {answers} | BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00' TERM=dumb script -qec '/usr/local/bin/backup --profiles /work/{name}/profiles.yaml setup --lang en' /dev/null
tree_digest() {{ (cd "$1" && find . -type f -print0 | sort -z | xargs -0 sha256sum | sort); }}
tree_modes() {{ (cd "$1" && find . -type f -printf '%m %p\n' | sort); }}
assert_tree() {{ [ "$(tree_digest "$1")" = "$(tree_digest "$2")" ] && [ "$(tree_modes "$1")" = "$(tree_modes "$2")" ]; }}
if ! systemctl is-active --quiet backup-pipeline.timer; then
  systemctl status backup-pipeline.timer --no-pager || true
  journalctl -u backup-pipeline.timer --no-pager -n 80 || true
  exit 1
fi
echo 'scheduled timer active'
# Verify the timer that Setup Wizard just enabled before manual backup coverage.
find /work/reports/{name} -maxdepth 1 -name 'execution-*.json' -delete
{report_poll}
if [ -z "$scheduled_report" ]; then
  systemctl status backup-pipeline.service --no-pager || true
  journalctl -u backup-pipeline.service --no-pager -n 80 || true
  exit 1
fi
last_trigger=$(systemctl show backup-pipeline.timer --property=LastTriggerUSec --value || true)
if [ -z "$last_trigger" ] || [ "$last_trigger" = "n/a" ]; then
  systemctl status backup-pipeline.timer --no-pager || true
  journalctl -u backup-pipeline.timer --no-pager -n 80 || true
  exit 1
fi
echo 'scheduled timer fired'
if ! grep -Eq '"succeeded": true' "$scheduled_report" || ! grep -Eq '"snapshot_id": "[^"]+"' "$scheduled_report"; then
  cat "$scheduled_report"
  systemctl status backup-pipeline.service --no-pager || true
  journalctl -u backup-pipeline.service --no-pager -n 80 || true
  exit 1
fi
backup --profiles /work/{name}/profiles.yaml run --skip-database
backup --profiles /work/{name}/profiles.yaml snapshots | grep -q 'Primary snapshots'
backup --profiles /work/{name}/profiles.yaml status | grep -q 'Profile:'
backup --profiles /work/{name}/profiles.yaml restore --target /work/{name}-primary
assert_tree /work/source /work/{name}-primary/work/source
{secondary_restore}
report=$(find /work/reports/{name} -name 'execution-*.json' -print -quit); test -n "$report" && grep -Eq '"snapshot_id": "[^"]+"' "$report"
drill_base=/work/reports/{name}/restore-drill-evidence
rm -f "$drill_base.html" "$drill_base.json"
backup --profiles /work/{name}/profiles.yaml report restore-drill --file "$drill_base"
test -s "$drill_base.html" && test -s "$drill_base.json"
execution_id=$(sed -n 's/.*"execution_id": "\([^"]*\)".*/\1/p' "$drill_base.json" | head -n 1)
test -n "$execution_id"
grep -Eq '"overall_status": "pass"' "$drill_base.json"
grep -Fq "$execution_id" "$drill_base.html"
grep -Eq '"snapshot_id": "[^"]+"' "$drill_base.json"
grep -Eq '"file_count": [1-9][0-9]*' "$drill_base.json"
grep -Eq '"total_bytes": [1-9][0-9]*' "$drill_base.json"
grep -Eq '"elapsed_milliseconds": [1-9][0-9]*' "$drill_base.json"
grep -Fq '"profile": "{name}"' "$drill_base.json"
grep -Fq '"backend": "primary"' "$drill_base.json"
{secondary_assertions}
pass_count=$(grep -c '"status": "pass"' "$drill_base.json")
test "$pass_count" -ge {expected_pass_count}
test "$(stat -c '%a' "$drill_base.html")" = 600
test "$(stat -c '%a' "$drill_base.json")" = 600
test "$(stat -c '%a' /work/reports/{name})" = 700
! grep -Eq 'e2e-password|minioadmin|backuppass' "$drill_base.html" "$drill_base.json"
backup --profiles /work/{name}/profiles.yaml schedule disable
if systemctl is-active --quiet backup-pipeline.timer; then
  systemctl status backup-pipeline.timer --no-pager || true
  exit 1
fi
next_trigger=$(systemctl show backup-pipeline.timer --property=NextElapseUSecRealtime --value || true)
if [ -n "$next_trigger" ] && [ "$next_trigger" != "n/a" ]; then
  systemctl status backup-pipeline.timer --no-pager || true
  exit 1
fi"#,
    )
}

fn execution_report_poll_script(
    directory: &str,
    variable: &str,
    stop_on_service_failure: bool,
) -> String {
    let failure_probe = if stop_on_service_failure {
        format!(
            r#"  if systemctl is-failed --quiet backup-pipeline.service; then
    for _ in {{1..10}}; do
      {variable}=$(find {directory} -name 'execution-*.json' -print -quit)
      [ -n "${variable}" ] && break
      sleep 1
    done
    break
  fi
"#
        )
    } else {
        String::new()
    };
    format!(
        r#"for _ in {{1..150}}; do
  {variable}=$(find {directory} -name 'execution-*.json' -print -quit)
  [ -n "${variable}" ] && break
{failure_probe}  sleep 1
done
{variable}=$(find {directory} -name 'execution-*.json' -print -quit)"#
    )
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
    runner(
        resources,
        &wizard_storage_script(name, &answers, has_secondary),
    );
}

fn database_setup_script(
    name: &str,
    database_type: &str,
    connection_url: &str,
    database_name: &str,
    expected_value: &str,
) -> String {
    let report_dir = format!("/work/reports/{name}");
    let answers = vec![
        name.to_owned(),
        "2".to_owned(),
        database_type.to_owned(),
        connection_url.to_owned(),
        String::new(),
        "7".to_owned(),
        "4".to_owned(),
        "12".to_owned(),
        "\x1b[B".to_owned(),
        String::new(),
        "http://backup-e2e-minio:9000".to_owned(),
        "minioadmin".to_owned(),
        "minioadmin".to_owned(),
        String::new(),
        database_name.to_owned(),
        String::new(),
        String::new(),
        String::new(),
        String::new(),
        report_dir,
        String::new(),
        String::new(),
    ]
    .into_iter()
    .map(|answer| shell_quote(&answer))
    .collect::<Vec<_>>()
    .join(" ");
    let table = if database_type == "mysql" {
        "users"
    } else {
        "audit_events"
    };
    let seed = if database_type == "mysql" {
        format!(
            "CREATE TABLE {table}(id INT PRIMARY KEY, name VARCHAR(32)); INSERT INTO {table} VALUES(201,'{expected_value}');"
        )
    } else {
        format!(
            "CREATE TABLE {table}(id INT PRIMARY KEY, event TEXT); INSERT INTO {table} VALUES(201,'{expected_value}');"
        )
    };
    let query = if database_type == "mysql" {
        format!("SELECT name FROM {table} WHERE id=201;")
    } else {
        format!("SELECT event FROM {table} WHERE id=201;")
    };
    let client = if database_type == "mysql" {
        "MYSQL_PWD=rootpass mysql"
    } else {
        "PGPASSWORD=pgpass psql"
    };
    let host = if database_type == "mysql" {
        if database_name == "app12" {
            "backup-e2e-mariadb12"
        } else {
            "backup-e2e-mariadb55"
        }
    } else {
        "backup-e2e-postgres16"
    };
    let connection_args = if database_type == "mysql" {
        format!("--host={host} --user=root --database={database_name}")
    } else {
        format!("--host={host} --user=postgres --dbname={database_name}")
    };
    let execute_flag = if database_type == "mysql" { "-e" } else { "-c" };
    let rows_only_flag = if database_type == "mysql" { "-N" } else { "-t" };
    format!(
        r#"mkdir -p /work/{name}
printf '%s\n' {answers} | BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00' TERM=dumb script -qec '/usr/local/bin/backup --profiles /work/{name}/profiles.yaml setup --lang en' /dev/null
for _ in {{1..60}}; do {client} {connection_args} {execute_flag} 'SELECT 1' >/dev/null 2>&1 && break; sleep 1; done
{client} {connection_args} {execute_flag} "{seed}"
backup --profiles /work/{name}/profiles.yaml database
drill_base=/work/reports/{name}/database-restore-drill
rm -f "$drill_base.html" "$drill_base.json"
backup --profiles /work/{name}/profiles.yaml report restore-drill --file "$drill_base"
test -s "$drill_base.html" && test -s "$drill_base.json"
grep -Eq '"overall_status": "pass"' "$drill_base.json"
grep -Eq '"database_verification": \{{' "$drill_base.json"
grep -Eq '"signature_verified": true' "$drill_base.json"
grep -Eq '"import_performed": false' "$drill_base.json"
grep -Eq '"record_validation_performed": false' "$drill_base.json"
grep -Eq '"file_count": [1-9][0-9]*' "$drill_base.json"
grep -Eq '"total_bytes": [1-9][0-9]*' "$drill_base.json"
grep -Eq '"elapsed_milliseconds": [1-9][0-9]*' "$drill_base.json"
grep -Fq '"profile": "{name}"' "$drill_base.json"
grep -Fq '"backend": "primary"' "$drill_base.json"
test "$(stat -c '%a' "$drill_base.json")" = 600
test "$(stat -c '%a' /work/reports/{name})" = 700
! grep -Eq 'rootpass|pgpass|BACKUP_DATABASE_CONNECTION_URL' "$drill_base.html" "$drill_base.json"
{client} {connection_args} {execute_flag} 'DROP TABLE {table};'
rm -rf /work/{name}-restore
backup --profiles /work/{name}/profiles.yaml restore --target /work/{name}-restore
{client} {connection_args} < "$(find /work/{name}-restore -name '{database_name}.sql' -print -quit)"
{client} {connection_args} {rows_only_flag} {execute_flag} "{query}" | grep -q '{expected_value}'
backup --profiles /work/{name}/profiles.yaml snapshots | grep -q 'Primary snapshots'
backup --profiles /work/{name}/profiles.yaml status | grep -q 'Profile: {name}'"#,
    )
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
    let systemd_report_poll =
        execution_report_poll_script("/work/reports/s3-to-sftp", "scheduled_report", true);
    runner(
        &resources,
        &format!(
            r#"find /work/reports/s3-to-sftp -maxdepth 1 -name 'execution-*.json' -delete
BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00' backup --profiles /work/s3-to-sftp/profiles.yaml schedule enable
{systemd_report_poll}
if [ -z "$scheduled_report" ] || ! grep -Eq '"succeeded": true' "$scheduled_report" || ! grep -Eq '"snapshot_id": "[^"]+"' "$scheduled_report"; then
  cat "$scheduled_report" 2>/dev/null || true
  systemctl show backup-pipeline.timer --no-pager -p ActiveState -p SubState -p OnCalendar -p NextElapseUSecRealtime -p LastTriggerUSec || true
  systemctl status backup-pipeline.service --no-pager || true
  journalctl -u backup-pipeline.service --no-pager -n 80 || true
  exit 1
fi"#
        ),
    );
    let cron_report_poll =
        execution_report_poll_script("/work/reports/s3-to-sftp", "cron_report", false);
    runner(
        &resources,
        &format!(
            r#"backup --profiles /work/s3-to-sftp/profiles.yaml schedule disable
if systemctl is-active --quiet backup-pipeline.timer; then systemctl status backup-pipeline.timer --no-pager || true; exit 1; fi
next_trigger=$(systemctl show backup-pipeline.timer --property=NextElapseUSecRealtime --value || true)
if [ -n "$next_trigger" ] && [ "$next_trigger" != "n/a" ]; then systemctl status backup-pipeline.timer --no-pager || true; exit 1; fi
find /work/reports/s3-to-sftp -maxdepth 1 -name 'execution-*.json' -delete
systemctl start cron
BACKUP_TEST_FORCE_CRON=1 BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00' backup --profiles /work/s3-to-sftp/profiles.yaml schedule enable
{cron_report_poll}
if [ -z "$cron_report" ] || ! grep -Eq '"succeeded": true' "$cron_report" || ! grep -Eq '"snapshot_id": "[^"]+"' "$cron_report"; then
  cat "$cron_report" 2>/dev/null || true
  systemctl status cron --no-pager || true
  journalctl -u cron --no-pager -n 80 || true
  exit 1
fi"#
        ),
    );
    for (kind, database, url, expected) in [
        (
            "mysql",
            "app12",
            "mysql://root:rootpass@backup-e2e-mariadb12:3306/app12",
            "Maria12",
        ),
        (
            "mysql",
            "app55",
            "mysql://root:rootpass@backup-e2e-mariadb55:3306/app55",
            "Maria55",
        ),
        (
            "postgres",
            "app16",
            "postgres://postgres:pgpass@backup-e2e-postgres16:5432/app16",
            "Postgres16",
        ),
    ] {
        runner(
            &resources,
            &database_setup_script(
                match database {
                    "app12" => "database-mariadb12",
                    "app55" => "database-mariadb55",
                    _ => "database-postgres16",
                },
                kind,
                url,
                database,
                expected,
            ),
        );
    }
}

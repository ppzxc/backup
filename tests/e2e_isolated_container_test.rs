//! Docker-only acceptance test.  It deliberately has no availability guard: the
//! default test contract requires the daemon, image pulls, and runner build.
use std::fs;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;

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
}

fn docker(args: &[&str]) -> Output {
    Command::new("docker")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("docker must be available for the E2E suite: {error}"))
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

fn runner(resources: &E2eResources, script: &str) -> String {
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
    docker_ok(&["exec", &resources.runner, "bash", "-ceu", &wrapped_script])
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\\"'\\\"'"))
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
            "mkdir -p /work/{name}; cp /work/e2e-key/id_ed25519 /work/{name}/id_ed25519; cp /work/e2e-key/id_ed25519 /work/{name}/id_ed25519_secondary; chmod 600 /work/{name}/id_ed25519*\nprintf '%s\\n' {answers} | BACKUP_TEST_SCHEDULE_CALENDAR='*-*-* *:*:00' TERM=dumb script -qec '/usr/local/bin/backup --profiles /work/{name}/profiles.yaml setup --lang en' /dev/null\ntree_digest() {{ (cd \"$1\" && find . -type f -print0 | sort -z | xargs -0 sha256sum | sort); }}\ntree_modes() {{ (cd \"$1\" && find . -type f -printf '%m %p\\n' | sort); }}\nassert_tree() {{ [ \"$(tree_digest \"$1\")\" = \"$(tree_digest \"$2\")\" ] && [ \"$(tree_modes \"$1\")\" = \"$(tree_modes \"$2\")\" ]; }}\nbackup --profiles /work/{name}/profiles.yaml run --skip-database\nbackup --profiles /work/{name}/profiles.yaml restore --target /work/{name}-primary\nassert_tree /work/source /work/{name}-primary/work/source\n{secondary_restore}\nreport=$(find /work/reports/{name} -name 'execution-*.json' -print -quit); test -n \"$report\" && grep -Eq '\"snapshot_id\": \"[^\"]+\"' \"$report\"\nsystemctl is-active --quiet backup-pipeline.timer\nfind /work/reports/{name} -maxdepth 1 -name 'execution-*.json' -delete\nfor _ in {{1..75}}; do find /work/reports/{name} -name 'execution-*.json' -print -quit | grep -q . && break; sleep 1; done\nfind /work/reports/{name} -name 'execution-*.json' -print -quit | grep -q .\nbackup --profiles /work/{name}/profiles.yaml schedule disable"
        ),
    );
}

struct DockerCleanup<'a> {
    resources: &'a E2eResources,
}

impl Drop for DockerCleanup<'_> {
    fn drop(&mut self) {
        let _ = docker(&[
            "rm",
            "-f",
            &self.resources.runner,
            &self.resources.minio,
            &self.resources.sftp,
        ]);
        let _ = docker(&[
            "rm",
            "-f",
            &self.resources.mariadb12,
            &self.resources.mariadb55,
            &self.resources.postgres16,
        ]);
        let _ = docker(&["network", "rm", &self.resources.network]);
    }
}

#[test]
fn isolated_container_matrix_exercises_storage_database_and_systemd() {
    let resources = E2eResources::new();
    let _cleanup = DockerCleanup {
        resources: &resources,
    };
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
    docker_ok(&["network", "create", &resources.network]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        &resources.minio,
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

    runner(
        &resources,
        "until systemctl show --property=Version --value | grep -q .; do sleep 1; done",
    );
    runner(&resources, "mysqldump --version | grep -q 'Distrib 5.5.56'");
    runner(&resources, "pg_dump --version | grep -q 'PostgreSQL) 16.'");
    runner(
        &resources,
        "until curl -fsS http://backup-e2e-minio:9000/minio/health/live >/dev/null; do sleep 1; done",
    );
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
                "cat >/work/db-profiles.yml <<'EOF'\nversion: '2'\napplication:\n  version: '1.0'\n  profile: db\n  backup:\n    backupType: !dbStream\n      db_type: {kind}\n      connection_url: '{url}'\n    targets: []\n    excludes: []\n  retention: {{keepDaily: 1, keepWeekly: 1, keepMonthly: 1}}\n  storage:\n    primary: {{backend: s3, repository: 's3:http://backup-e2e-minio:9000/db-{database}', password: e2e-password}}\nprofiles: {{}}\nEOF\nrestic -r s3:http://backup-e2e-minio:9000/db-{database} --password-command 'printf e2e-password' init\n{command} --host={host} --port={port} {connection_args} {execute_flag} \"{seed}\"\nbackup --profiles /work/db-profiles.yml database\n{command} --host={host} --port={port} {connection_args} {execute_flag} 'DROP TABLE {table};'\nrm -rf /work/db-restore && backup --profiles /work/db-profiles.yml restore --target /work/db-restore\n{command} --host={host} --port={port} {connection_args} < \"$(find /work/db-restore -name '{database}.sql' -print -quit)\"\n{command} --host={host} --port={port} {connection_args} {rows_only_flag} {execute_flag} \"{query}\" | grep -q '{expected}'",
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

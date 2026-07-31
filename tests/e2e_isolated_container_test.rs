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
        "mkdir -p /work/source/nested /work/restore-primary /work/restore-secondary; printf 'alpha\\n' >/work/source/a; printf 'beta\\n' >/work/source/nested/b",
    );
    runner(
        &resources,
        "mkdir -p /root/.ssh; cp /work/e2e-key/id_ed25519 /root/.ssh/id_ed25519; chmod 600 /root/.ssh/id_ed25519; printf 'Host *\\n  StrictHostKeyChecking no\\n  UserKnownHostsFile /dev/null\\n  IdentitiesOnly yes\\n' >/root/.ssh/config",
    );
    runner(
        &resources,
        "restic -r s3:http://backup-e2e-minio:9000/primary --password-command 'printf e2e-password' init; restic -r sftp:backupuser@backup-e2e-sftp:/upload/s3-to-sftp --password-command 'printf e2e-password' init; restic -r sftp:backupuser@backup-e2e-sftp:/upload/sftp-to-s3 --password-command 'printf e2e-password' init",
    );
    // The runner uses explicit paths for every CLI invocation; the files never touch /etc/backup.
    runner(
        &resources,
        "tree_digest() { (cd \"$1\" && find . -type f -print0 | sort -z | xargs -0 sha256sum | sort); }\ncat >/work/config.yml <<'EOF'\nversion: '1.0'\nprofile: primary\nbackup:\n  backup_type: directory\n  targets: [/work/source]\n  excludes: []\nretention: {keep_daily: 1, keep_weekly: 1, keep_monthly: 1}\nstorage:\n  primary: {backend: s3, repository: 's3:http://backup-e2e-minio:9000/primary', password: e2e-password}\nEOF\ncat >/work/profiles.yml <<'EOF'\nversion: '2'\nprofiles:\n  primary:\n    repository: s3:http://backup-e2e-minio:9000/primary\n    password: e2e-password\n    initialize: true\n    env: {AWS_ACCESS_KEY_ID: minioadmin, AWS_SECRET_ACCESS_KEY: minioadmin}\n    backup: {source: [/work/source], schedule: '*-*-* 03:00:00'}\n    copy:\n      repository: sftp:backupuser@backup-e2e-sftp:/upload/s3-to-sftp\n      password: e2e-password\n      initialize: true\nEOF\nbackup --config /work/config.yml --profiles /work/profiles.yml run --skip-database --skip-retention\nbackup --config /work/config.yml restore --target /work/restore-primary\n[ \"$(tree_digest /work/source)\" = \"$(tree_digest /work/restore-primary/work/source)\" ]\nbackup --config /work/config.yml --profiles /work/profiles.yml copy --profile primary\ncat >/work/sftp.yml <<'EOF'\nversion: '1.0'\nprofile: sftp\nbackup:\n  backup_type: directory\n  targets: [/work/source]\n  excludes: []\nretention: {keep_daily: 1, keep_weekly: 1, keep_monthly: 1}\nstorage:\n  primary: {backend: sftp, repository: 'sftp:backupuser@backup-e2e-sftp:/upload/s3-to-sftp', password: e2e-password}\nEOF\nbackup --config /work/sftp.yml restore --target /work/restore-secondary\n[ \"$(tree_digest /work/source)\" = \"$(tree_digest /work/restore-secondary/work/source)\" ]",
    );
    runner(
        &resources,
        "tree_digest() { (cd \"$1\" && find . -type f -print0 | sort -z | xargs -0 sha256sum | sort); }\ncat >/work/sftp-primary.yml <<'EOF'\nversion: '1.0'\nprofile: sftp-primary\nbackup:\n  backup_type: directory\n  targets: [/work/source]\n  excludes: []\nretention: {keep_daily: 1, keep_weekly: 1, keep_monthly: 1}\nstorage:\n  primary: {backend: sftp, repository: 'sftp:backupuser@backup-e2e-sftp:/upload/sftp-to-s3', password: e2e-password}\nEOF\ncat >/work/sftp-primary-profiles.yml <<'EOF'\nversion: '2'\nprofiles:\n  sftp-primary:\n    repository: sftp:backupuser@backup-e2e-sftp:/upload/sftp-to-s3\n    password: e2e-password\n    initialize: true\n    backup: {source: [/work/source]}\n    copy:\n      repository: s3:http://backup-e2e-minio:9000/sftp-to-s3\n      password: e2e-password\n      initialize: true\n      env: {AWS_ACCESS_KEY_ID: minioadmin, AWS_SECRET_ACCESS_KEY: minioadmin}\nEOF\nbackup --config /work/sftp-primary.yml --profiles /work/sftp-primary-profiles.yml run --skip-database --skip-retention\nbackup --config /work/sftp-primary.yml --profiles /work/sftp-primary-profiles.yml copy --profile sftp-primary\ncat >/work/reverse.yml <<'EOF'\nversion: '1.0'\nprofile: reverse\nbackup:\n  backup_type: directory\n  targets: [/work/source]\n  excludes: []\nretention: {keep_daily: 1, keep_weekly: 1, keep_monthly: 1}\nstorage:\n  primary: {backend: s3, repository: 's3:http://backup-e2e-minio:9000/sftp-to-s3', password: e2e-password}\nEOF\nrm -rf /work/restore-reverse && backup --config /work/reverse.yml restore --target /work/restore-reverse\n[ \"$(tree_digest /work/source)\" = \"$(tree_digest /work/restore-reverse/work/source)\" ]",
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
                "cat >/work/db.yml <<'EOF'\nversion: '1.0'\nprofile: db\nbackup:\n  backupType: !dbStream\n    db_type: {kind}\n    connection_url: '{url}'\n  targets: []\n  excludes: []\nretention: {{keepDaily: 1, keepWeekly: 1, keepMonthly: 1}}\nstorage:\n  primary: {{backend: s3, repository: 's3:http://backup-e2e-minio:9000/db-{database}', password: e2e-password}}\nEOF\nrestic -r s3:http://backup-e2e-minio:9000/db-{database} --password-command 'printf e2e-password' init\n{command} --host={host} --port={port} {connection_args} {execute_flag} \"{seed}\"\nbackup --config /work/db.yml database\n{command} --host={host} --port={port} {connection_args} {execute_flag} 'DROP TABLE {table};'\nrm -rf /work/db-restore && backup --config /work/db.yml restore --target /work/db-restore\n{command} --host={host} --port={port} {connection_args} < \"$(find /work/db-restore -name '{database}.sql' -print -quit)\"\n{command} --host={host} --port={port} {connection_args} {rows_only_flag} {execute_flag} \"{query}\" | grep -q '{expected}'",
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
    runner(
        &resources,
        "backup --config /work/config.yml --profiles /work/profiles.yml schedule enable",
    );
    runner(
        &resources,
        "timer=resticprofile-backup@profile-primary.timer; systemctl list-timers --all --no-legend | grep -Fq \"$timer\"; systemctl is-active --quiet \"$timer\"",
    );
    runner(
        &resources,
        "backup --config /work/config.yml --profiles /work/profiles.yml schedule disable",
    );
    runner(
        &resources,
        "! systemctl list-timers --all --no-legend | grep -Fq resticprofile-backup@profile-primary.timer",
    );
}

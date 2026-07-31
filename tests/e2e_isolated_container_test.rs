//! Docker-only acceptance test.  It deliberately has no availability guard: the
//! default test contract requires the daemon, image pulls, and runner build.
use std::process::{Command, Output};

const RUNNER: &str = "backup-e2e-runner";
const NETWORK: &str = "backup-e2e-network";

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
        "docker {:?} failed:\n{}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn runner(script: &str) -> String {
    docker_ok(&["exec", RUNNER, "bash", "-ceu", script])
}

struct DockerCleanup;

impl Drop for DockerCleanup {
    fn drop(&mut self) {
        let _ = docker(&["rm", "-f", RUNNER, "backup-e2e-minio", "backup-e2e-sftp"]);
        let _ = docker(&[
            "rm",
            "-f",
            "backup-e2e-mariadb12",
            "backup-e2e-mariadb55",
            "backup-e2e-postgres16",
        ]);
        let _ = docker(&["network", "rm", NETWORK]);
    }
}

#[test]
fn isolated_container_matrix_exercises_storage_database_and_systemd() {
    let _cleanup = DockerCleanup;
    docker_ok(&[
        "build",
        "--tag",
        "backup-e2e-runner:test",
        "--file",
        "docker/Dockerfile.e2e_runner",
        ".",
    ]);
    docker_ok(&["network", "create", NETWORK]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        "backup-e2e-minio",
        "--network",
        NETWORK,
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
        "backup-e2e-sftp",
        "--network",
        NETWORK,
        "atmoz/sftp:alpine",
        "backupuser:backuppass:::upload",
    ]);
    docker_ok(&[
        "run",
        "-d",
        "--name",
        "backup-e2e-mariadb12",
        "--network",
        NETWORK,
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
        "backup-e2e-mariadb55",
        "--network",
        NETWORK,
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
        "backup-e2e-postgres16",
        "--network",
        NETWORK,
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
        RUNNER,
        "--network",
        NETWORK,
        "--privileged",
        "--cgroupns=host",
        "--mount",
        "type=bind,src=/sys/fs/cgroup,dst=/sys/fs/cgroup,readonly",
        "-e",
        "AWS_ACCESS_KEY_ID=minioadmin",
        "-e",
        "AWS_SECRET_ACCESS_KEY=minioadmin",
        "backup-e2e-runner:test",
    ]);

    runner("until systemctl show --property=Version --value | grep -q .; do sleep 1; done");
    runner(
        "until curl -fsS http://backup-e2e-minio:9000/minio/health/live >/dev/null; do sleep 1; done",
    );
    runner(
        "mkdir -p /work/source/nested /work/restore-primary /work/restore-secondary; printf 'alpha\\n' >/work/source/a; printf 'beta\\n' >/work/source/nested/b",
    );
    runner(
        "ssh-keygen -q -N '' -f /root/.ssh/id_ed25519; mkdir -p /root/.ssh; printf 'Host *\\n  StrictHostKeyChecking no\\n' >/root/.ssh/config",
    );
    // The runner uses explicit paths for every CLI invocation; the files never touch /etc/backup.
    runner(
        "cat >/work/config.yml <<'EOF'\nversion: '1.0'\nprofile: primary\nbackup:\n  backup_type: directory\n  targets: [/work/source]\n  excludes: []\nretention: {keep_daily: 1, keep_weekly: 1, keep_monthly: 1}\nstorage:\n  primary: {backend: s3, repository: 's3:http://backup-e2e-minio:9000/primary', password: e2e-password}\nEOF\ncat >/work/profiles.yml <<'EOF'\nversion: '2'\nprofiles:\n  primary:\n    repository: s3:http://backup-e2e-minio:9000/primary\n    password: e2e-password\n    env: {AWS_ACCESS_KEY_ID: minioadmin, AWS_SECRET_ACCESS_KEY: minioadmin}\n    backup: {source: [/work/source]}\nEOF\nrestic -r s3:http://backup-e2e-minio:9000/primary --password-command 'printf e2e-password' init\nbackup --config /work/config.yml --profiles /work/profiles.yml run --skip-database --skip-retention\nbackup --config /work/config.yml restore --target /work/restore-primary\n[ \"$(find /work/source -type f -print0 | xargs -0 sha256sum | awk '{print $1}' | sort)\" = \"$(find /work/restore-primary -type f -print0 | xargs -0 sha256sum | awk '{print $1}' | sort)\" ]",
    );
    // Both migration directions are CLI operations; each endpoint is restored and compared.
    runner(
        "backup --config /work/config.yml --profiles /work/profiles.yml copy --profile primary --dry-run",
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
        runner(&format!(
            "until {command} --host={host} --port={port} --username={} --dbname={database} -c 'SELECT 1' >/dev/null 2>&1; do sleep 1; done",
            if kind == "mysql" { "root" } else { "postgres" },
        ));
        let url = if kind == "mysql" {
            format!("mysql://root:rootpass@{host}:{port}/{database}")
        } else {
            format!("postgres://postgres:pgpass@{host}:{port}/{database}")
        };
        runner(&format!(
            "cat >/work/db.yml <<'EOF'\nversion: '1.0'\nprofile: db\nbackup:\n  backup_type: {{ dbStream: {{ dbType: {kind}, connectionUrl: '{url}' }} }}\n  targets: []\n  excludes: []\nretention: {{keep_daily: 1, keep_weekly: 1, keep_monthly: 1}}\nstorage:\n  primary: {{backend: s3, repository: 's3:http://backup-e2e-minio:9000/db-{database}', password: e2e-password}}\nEOF\nrestic -r s3:http://backup-e2e-minio:9000/db-{database} --password-command 'printf e2e-password' init\n{command} --host={host} --port={port} --username={} --dbname={database} -c \"{seed}\"\nbackup --config /work/db.yml database\n{command} --host={host} --port={port} --username={} --dbname={database} -c 'DROP TABLE {};'\nrm -rf /work/db-restore && backup --config /work/db.yml restore --target /work/db-restore\n{command} --host={host} --port={port} --username={} --dbname={database} < \"$(find /work/db-restore -name '{database}.sql' -print -quit)\"\n{command} --host={host} --port={port} --username={} --dbname={database} -N -e \"{query}\" | grep -q '{}'",
            if kind == "mysql" { "root" } else { "postgres" },
            if kind == "mysql" { "root" } else { "postgres" },
            if kind == "mysql" {
                "users"
            } else {
                "audit_events"
            },
            if kind == "mysql" { "root" } else { "postgres" },
            if kind == "mysql" { "root" } else { "postgres" },
            if kind == "mysql" {
                "Maria"
            } else {
                "Postgres16"
            }
        ));
    }
    runner(
        "backup --config /work/config.yml --profiles /work/profiles.yml schedule enable; systemctl list-timers --all | grep -q resticprofile; backup --config /work/config.yml --profiles /work/profiles.yml schedule disable; ! systemctl list-timers --all | grep -q resticprofile",
    );
}

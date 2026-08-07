use crate::config::model::{BackupConfig, BackupType, DatabaseType, backup_profile_snapshot_tag};
use crate::runner::restic::ResticRunner;
use anyhow::{Result, bail};
use secrecy::ExposeSecret;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatabaseDumpValidation {
    pub database_type: DatabaseType,
    pub expected_signature: String,
    pub signature_verified: bool,
}

impl DatabaseDumpValidation {
    pub fn expected_signature(database_type: DatabaseType) -> &'static str {
        match database_type {
            DatabaseType::Mysql => "MySQL/MariaDB mysqldump SQL signature",
            DatabaseType::Postgres => "PostgreSQL pg_dump SQL signature",
        }
    }
}

/// Performs the pure, structural Database Backup Adapter check for a plain SQL dump.
///
/// This deliberately does not import the dump or execute any schema/table/record query.
pub fn validate_dump_signature(
    content: &str,
    database_type: DatabaseType,
) -> DatabaseDumpValidation {
    let normalized = content.to_ascii_lowercase();
    let detected_type = if normalized
        .lines()
        .map(str::trim_start)
        .any(|line| line.starts_with("-- mysql dump") || line.starts_with("-- mariadb dump"))
    {
        Some(DatabaseType::Mysql)
    } else if normalized
        .lines()
        .map(str::trim_start)
        .any(|line| line.starts_with("-- postgresql database dump"))
    {
        Some(DatabaseType::Postgres)
    } else {
        None
    };
    DatabaseDumpValidation {
        database_type,
        expected_signature: DatabaseDumpValidation::expected_signature(database_type).into(),
        signature_verified: detected_type == Some(database_type),
    }
}

pub fn execute_database_backup_from_profiles<R: ResticRunner + ?Sized>(
    config: &crate::config::model::ResticProfileConfig,
    config_path: &Path,
    runner: &R,
    dry_run: bool,
) -> Result<String> {
    let database = config
        .application
        .as_ref()
        .and_then(|application| application.database.as_ref())
        .ok_or_else(|| {
            anyhow::anyhow!("Database Backup Adapter is not configured; run backup setup first")
        })?;
    let config_dir = config_path.parent().unwrap_or(Path::new("."));
    // Dry-runs render the dump command but never launch a child process, so they
    // may inspect a fixture sidecar without treating it as an executable secret.
    let url = crate::config::model::ResticProfileConfig::secure_sidecar_value(
        config_dir,
        "database-connection-url",
    )?;
    let (program, args, filename, environment) = dump_command(database.db_type, &url)?;
    let backend = config.backend_credentials(config_dir, &database.profile)?;
    if dry_run {
        return Ok(format!(
            "[Dry-Run] Database Stream: {} -> {}",
            program, filename
        ));
    }
    let (repository, password) = backend;
    let mut owned_environment = config.sidecar_environment(config_dir)?;
    owned_environment.extend(environment);
    let environment = owned_environment
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    runner.backup_command_with_env_and_tag(
        &repository,
        &password,
        &filename,
        program,
        &args,
        &backup_profile_snapshot_tag(&database.profile),
        &environment,
    )
}

pub fn execute_database_backup<R: ResticRunner>(
    config: &BackupConfig,
    runner: &R,
    dry_run: bool,
) -> Result<String> {
    let BackupType::DbStream {
        db_type,
        connection_url,
    } = &config.backup.backup_type
    else {
        bail!("Database Backup Adapter is not configured; run backup setup first");
    };
    let url = connection_url
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("Database Backup Adapter requires a connection URL"))?;
    let (program, args, filename, environment) = dump_command(*db_type, url)?;
    tracing::info!(
        db_type = ?db_type,
        program = %program,
        filename = %filename,
        dry_run = %dry_run,
        "Initiating database stream backup"
    );
    if dry_run {
        return Ok(format!(
            "[Dry-Run] Database Stream: {} -> {}",
            program, filename
        ));
    }
    let env_refs = environment
        .iter()
        .map(|(key, value)| (key.as_str(), value.as_str()))
        .collect::<Vec<_>>();
    runner.backup_command_with_env_and_tag(
        &config.storage.primary.repository,
        config.storage.primary.password.expose_secret(),
        &filename,
        program,
        &args,
        &backup_profile_snapshot_tag(&config.profile),
        &env_refs,
    )
}

fn dump_command(
    db_type: DatabaseType,
    url: &str,
) -> Result<(&'static str, Vec<String>, String, Vec<(String, String)>)> {
    let parsed = url::Url::parse(url)?;
    let scheme = parsed.scheme().to_ascii_lowercase();
    let expected_scheme = match db_type {
        DatabaseType::Mysql => ["mysql", "mariadb"].as_slice(),
        DatabaseType::Postgres => ["postgres", "postgresql"].as_slice(),
    };
    if !expected_scheme.iter().any(|expected| *expected == scheme) {
        bail!("Database URL scheme '{scheme}' does not match database type '{db_type}'");
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Database URL must contain a host"))?;
    let database = parsed.path().trim_start_matches('/');
    if database.is_empty() {
        bail!("Database URL must contain a database name");
    }
    let user = parsed.username();
    if user.trim().is_empty() || parsed.password().is_none_or(str::is_empty) {
        bail!("Database URL must contain database credentials");
    }
    let password = parsed.password().unwrap_or_default();
    match db_type {
        DatabaseType::Mysql => {
            let mut args = vec![format!("--host={host}"), format!("--user={user}")];
            if let Some(port) = parsed.port() {
                args.push(format!("--port={port}"));
            }
            args.push(database.into());
            Ok((
                "mysqldump",
                args,
                format!("{database}.sql"),
                vec![("MYSQL_PWD".into(), password.into())],
            ))
        }
        DatabaseType::Postgres => {
            let mut args = vec![
                format!("--host={host}"),
                format!("--username={user}"),
                format!("--dbname={database}"),
            ];
            if let Some(port) = parsed.port() {
                args.push(format!("--port={port}"));
            }
            Ok((
                "pg_dump",
                args,
                format!("{database}.sql"),
                vec![("PGPASSWORD".into(), password.into())],
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{DatabaseType, dump_command, validate_dump_signature};

    #[test]
    fn dump_command_builds_postgres_arguments_and_secret_environment() {
        let (program, args, filename, environment) = dump_command(
            DatabaseType::Postgres,
            "postgres://backup-user:db-secret@db:5432/app",
        )
        .unwrap();

        assert_eq!(program, "pg_dump");
        assert_eq!(filename, "app.sql");
        assert_eq!(
            args,
            [
                "--host=db",
                "--username=backup-user",
                "--dbname=app",
                "--port=5432"
            ]
        );
        assert_eq!(environment, [("PGPASSWORD".into(), "db-secret".into())]);
    }

    #[test]
    fn dump_command_rejects_mismatched_type_and_missing_credentials() {
        let mismatch =
            dump_command(DatabaseType::Mysql, "postgres://user:pass@db/app").unwrap_err();
        assert!(mismatch.to_string().contains("does not match"));

        let missing_credentials =
            dump_command(DatabaseType::Postgres, "postgres://user@db/app").unwrap_err();
        assert!(missing_credentials.to_string().contains("credentials"));
    }

    #[test]
    fn dump_signature_validation_supports_mysql_mariadb_and_postgres() {
        for (database_type, content) in [
            (
                DatabaseType::Mysql,
                "-- MySQL dump 10.13\nCREATE TABLE items (id int);",
            ),
            (
                DatabaseType::Mysql,
                "-- MariaDB dump 10.11\nCREATE TABLE items (id int);",
            ),
            (
                DatabaseType::Postgres,
                "-- PostgreSQL database dump\nCREATE TABLE items (id integer);",
            ),
        ] {
            assert!(validate_dump_signature(content, database_type).signature_verified);
        }
    }

    #[test]
    fn dump_signature_validation_rejects_generic_sql_and_database_type_mismatch() {
        assert!(
            !validate_dump_signature("CREATE TABLE items (id int);", DatabaseType::Postgres,)
                .signature_verified
        );
        assert!(
            !validate_dump_signature(
                "-- PostgreSQL database dump\nCREATE TABLE items (id int);",
                DatabaseType::Mysql,
            )
            .signature_verified
        );
    }
}

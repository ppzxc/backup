use crate::config::model::{BackupConfig, BackupType, DatabaseType};
use crate::runner::restic::ResticRunner;
use anyhow::{Result, bail};
use secrecy::ExposeSecret;

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
    runner.backup_command_with_env(
        &config.storage.primary.repository,
        config.storage.primary.password.expose_secret(),
        &filename,
        program,
        &args,
        &env_refs,
    )
}

fn dump_command(
    db_type: DatabaseType,
    url: &str,
) -> Result<(&'static str, Vec<String>, String, Vec<(String, String)>)> {
    let parsed = url::Url::parse(url)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Database URL must contain a host"))?;
    let database = parsed.path().trim_start_matches('/');
    if database.is_empty() {
        bail!("Database URL must contain a database name");
    }
    let user = parsed.username();
    let password = parsed.password().unwrap_or("");
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

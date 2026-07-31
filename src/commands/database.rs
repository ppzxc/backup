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
    let (program, args, filename) = dump_command(*db_type, url)?;
    if dry_run {
        return Ok(format!(
            "[Dry-Run] Database Stream: {} -> {}",
            program, filename
        ));
    }
    runner.backup_command(
        &config.storage.primary.repository,
        config.storage.primary.password.expose_secret(),
        &filename,
        program,
        &args,
    )
}

fn dump_command(db_type: DatabaseType, url: &str) -> Result<(&'static str, Vec<String>, String)> {
    let parsed = url::Url::parse(url)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("Database URL must contain a host"))?;
    let database = parsed.path().trim_start_matches('/');
    if database.is_empty() {
        bail!("Database URL must contain a database name");
    }
    let user = parsed.username();
    let password = parsed.password();
    match db_type {
        DatabaseType::Mysql => {
            let mut args = vec![format!("--host={host}"), format!("--user={user}")];
            if let Some(port) = parsed.port() {
                args.push(format!("--port={port}"));
            }
            if let Some(password) = password {
                args.push(format!("--password={password}"));
            }
            args.push(database.into());
            Ok(("mysqldump", args, format!("{database}.sql")))
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
            Ok(("pg_dump", args, format!("{database}.sql")))
        }
    }
}

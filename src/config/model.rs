use anyhow::Result;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::path::Path;

pub const DEFAULT_PROFILES_FILENAME: &str = "profiles.yaml";
pub const DEFAULT_PROFILES_PATH: &str = "/etc/backup/profiles.yaml";
pub const BACKUP_PROFILE_TAG_PREFIX: &str = "backup-profile:";

/// Secret values intended for a child-process environment. Values are exposed only when
/// borrowed at the external command boundary.
pub type SecretEnvironment = Vec<(String, SecretString)>;

pub fn borrowed_environment(environment: &SecretEnvironment) -> Vec<(&str, &str)> {
    environment
        .iter()
        .map(|(key, value)| (key.as_str(), value.expose_secret().as_str()))
        .collect()
}

/// Returns the CLI-owned tag that identifies snapshots produced by one exact Backup Profile.
pub fn backup_profile_snapshot_tag(profile: &str) -> String {
    format!("{BACKUP_PROFILE_TAG_PREFIX}{profile}")
}

const APPLICATION_SECRET_PREFIX: &str = "${BACKUP_";

fn legacy_application_version() -> String {
    "1.0".into()
}

fn serialize_secret_string<S>(secret: &SecretString, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(secret.expose_secret())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReportsConfig {
    pub output_dir: String,
    pub enable_daily_reports: bool,
    pub enable_annual_dr_drill_report: bool,
}

/// Backup CLI metadata which is intentionally outside resticprofile's execution schema.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ApplicationConfig {
    #[serde(default)]
    pub reports: ReportsConfig,
    #[serde(default)]
    pub audit: AuditConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<DatabaseConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DatabaseConfig {
    pub profile: String,
    #[serde(rename = "type")]
    pub db_type: DatabaseType,
    pub connection_url: String,
}

fn deserialize_application<'de, D>(deserializer: D) -> Result<Option<ApplicationConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_yaml::Value>::deserialize(deserializer)?;
    let Some(value) = value else { return Ok(None) };
    let mapping = value
        .as_mapping()
        .ok_or_else(|| serde::de::Error::custom("application must be a mapping"))?;
    for key in ["version", "profile", "backup", "retention", "storage"] {
        if mapping.contains_key(serde_yaml::Value::String(key.into())) {
            return Err(serde::de::Error::custom(format!(
                "application.{key} is deprecated; manually move backup execution settings into standard profiles"
            )));
        }
    }
    let application: ApplicationConfig =
        serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
    if let Some(database) = &application.database {
        if database.connection_url != "${BACKUP_DATABASE_CONNECTION_URL}" {
            return Err(serde::de::Error::custom(
                "application.database.connection-url must reference ${BACKUP_DATABASE_CONNECTION_URL}; store the value in the secure sidecar file",
            ));
        }
    }
    Ok(Some(application))
}

impl Default for ReportsConfig {
    fn default() -> Self {
        Self {
            output_dir: "/data/backup/reports".into(),
            enable_daily_reports: true,
            enable_annual_dr_drill_report: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupConfig {
    #[serde(
        default = "legacy_application_version",
        skip_serializing_if = "String::is_empty"
    )]
    pub version: String,
    pub profile: String,
    pub backup: BackupTargets,
    pub retention: RetentionPolicy,
    pub storage: StorageConfig,
    #[serde(default)]
    pub reports: ReportsConfig,
    #[serde(default)]
    pub audit: AuditConfig,
}

pub fn create_secure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

pub fn save_secure_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        create_secure_dir(parent)?;
    }
    fs::write(path, content)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

impl BackupConfig {
    fn profile_password_file(&self, config_dir: &Path, is_secondary: bool) -> Result<String> {
        let (existing_file, inline_password) =
            self.resolve_storage_password(config_dir, is_secondary)?;
        if let Some(path) = existing_file {
            return Ok(path);
        }
        let path = config_dir.join(if is_secondary {
            "secondary-password"
        } else {
            "primary-password"
        });
        save_secure_file(
            &path,
            inline_password
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("Storage password cannot be resolved"))?,
        )?;
        Ok(path.to_string_lossy().into_owned())
    }

    pub fn validate(&self) -> Result<()> {
        if self
            .storage
            .primary
            .password
            .expose_secret()
            .trim()
            .is_empty()
        {
            anyhow::bail!("Primary storage password cannot be empty");
        }
        if let Some(ref sec) = self.storage.secondary {
            if sec.enabled
                && sec.backend != "sftp"
                && sec.password.expose_secret().trim().is_empty()
            {
                anyhow::bail!("Secondary storage password cannot be empty");
            }
        }
        Ok(())
    }

    pub fn redacted(&self) -> Self {
        let mut masked = self.clone();
        masked.storage.primary.password = SecretString::new("******".into());
        if let Some(ref mut s3) = masked.storage.primary.s3 {
            s3.access_key_id = SecretString::new("******".into());
            s3.secret_access_key = SecretString::new("******".into());
        }
        if let Some(ref mut sec) = masked.storage.secondary {
            sec.password = SecretString::new("******".into());
            if let Some(s3) = sec.s3.as_mut() {
                s3.access_key_id = SecretString::new("******".into());
                s3.secret_access_key = SecretString::new("******".into());
            }
        }
        if let BackupType::DbStream {
            connection_url: Some(url),
            ..
        } = &mut masked.backup.backup_type
        {
            if url.contains('@') {
                *url = "******".into();
            }
        }
        masked
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let yaml = serde_yaml::to_string(self)?;
        save_secure_file(path, &yaml)?;
        Ok(())
    }

    pub fn resolve_storage_password(
        &self,
        config_dir: &Path,
        is_secondary: bool,
    ) -> Result<(Option<String>, Option<String>)> {
        let enc_path = config_dir.join("enc");
        if enc_path.is_file() {
            return Ok((Some(enc_path.to_string_lossy().to_string()), None));
        }
        let fallback_enc = Path::new("/etc/backup/enc");
        if fallback_enc.is_file() {
            return Ok((Some(fallback_enc.to_string_lossy().to_string()), None));
        }

        if is_secondary {
            if let Some(ref sec) = self.storage.secondary {
                let sec_pwd = sec.password.expose_secret();
                if !sec_pwd.trim().is_empty() {
                    return Ok((None, Some(sec_pwd.to_string())));
                }
            }
            let primary_pwd = self.storage.primary.password.expose_secret();
            if !primary_pwd.trim().is_empty() {
                return Ok((None, Some(primary_pwd.to_string())));
            }
            anyhow::bail!("Secondary storage password cannot be resolved");
        } else {
            let pwd = self.storage.primary.password.expose_secret();
            if !pwd.trim().is_empty() {
                Ok((None, Some(pwd.to_string())))
            } else {
                anyhow::bail!("Primary storage password cannot be empty");
            }
        }
    }

    pub fn save_and_sync(&self, config_dir: &Path) -> Result<()> {
        self.save_to_profiles_path(&config_dir.join(DEFAULT_PROFILES_FILENAME))
    }

    pub fn save_to_profiles_path(&self, profiles_yaml_path: &Path) -> Result<()> {
        self.validate()?;
        let config_dir = profiles_yaml_path.parent().unwrap_or(Path::new("."));
        if !config_dir.exists() {
            create_secure_dir(config_dir)?;
        }

        let mut restic_config = if profiles_yaml_path.exists() {
            ResticProfileConfig::load_from_path(profiles_yaml_path).unwrap_or_else(|_| {
                ResticProfileConfig {
                    version: "2".into(),
                    application: None,
                    global: None,
                    groups: None,
                    profiles: std::collections::BTreeMap::new(),
                }
            })
        } else {
            ResticProfileConfig {
                version: "2".into(),
                application: None,
                global: None,
                groups: None,
                profiles: std::collections::BTreeMap::new(),
            }
        };

        let existing_profile_tags = restic_config
            .profiles
            .get(&self.profile)
            .and_then(|profile| profile.backup.as_ref())
            .and_then(|backup| backup.tag.as_ref())
            .cloned()
            .unwrap_or_default();

        restic_config.version = "2".into();
        restic_config.application =
            Some(self.application_metadata_with_secret_references(config_dir)?);

        // 1. Populate default profile (truly global options only)
        let mut default_profile = restic_config.profiles.remove("default").unwrap_or_default();
        if default_profile.description.is_none() {
            default_profile.description = Some("Global common options".into());
        }
        default_profile.insecure_tls = Some(true);
        restic_config
            .profiles
            .insert("default".into(), default_profile);

        // 2. Populate primary profile (1st storage configuration)
        let mut primary_profile = restic_config.profiles.remove("primary").unwrap_or_default();
        if primary_profile.description.is_none() {
            primary_profile.description = Some("Primary Storage configuration".into());
        }
        primary_profile.inherit = (self.profile != "default").then(|| "default".into());
        if self.profile == "default" {
            primary_profile.insecure_tls = Some(true);
        }
        primary_profile.repository = Some(self.storage.primary.repository.clone());
        primary_profile.password_file = Some(self.profile_password_file(config_dir, false)?);
        primary_profile.password = None;
        if let Some(s3) = &self.storage.primary.s3 {
            save_secure_file(
                &config_dir.join("primary-aws-access-key-id"),
                s3.access_key_id.expose_secret(),
            )?;
            save_secure_file(
                &config_dir.join("primary-aws-secret-access-key"),
                s3.secret_access_key.expose_secret(),
            )?;
            primary_profile.env = Some(std::collections::BTreeMap::from([
                (
                    "AWS_ACCESS_KEY_ID".into(),
                    "{{ .Env.BACKUP_PRIMARY_AWS_ACCESS_KEY_ID }}".into(),
                ),
                (
                    "AWS_SECRET_ACCESS_KEY".into(),
                    "{{ .Env.BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY }}".into(),
                ),
            ]));
        }
        if primary_profile
            .env
            .as_ref()
            .is_some_and(|env| env.is_empty())
        {
            primary_profile.env = None;
        }
        if let Some(ref sftp) = self.storage.primary.sftp {
            if let Some(sftp_cmd) = sftp.sftp_command() {
                let mut opt_map = primary_profile.option.unwrap_or_default();
                opt_map.insert("sftp.command".into(), sftp_cmd);
                primary_profile.option = Some(opt_map);
            }
        }
        restic_config
            .profiles
            .insert("primary".into(), primary_profile);

        // 3. Populate secondary profile (if enabled)
        if let Some(ref sec) = self.storage.secondary {
            if sec.enabled {
                let mut secondary_profile = restic_config
                    .profiles
                    .remove("secondary")
                    .unwrap_or_default();
                if secondary_profile.description.is_none() {
                    secondary_profile.description = Some("Secondary Storage configuration".into());
                }
                secondary_profile.inherit = Some("default".into());
                secondary_profile.repository = Some(sec.repository.clone());
                secondary_profile.password_file =
                    Some(self.profile_password_file(config_dir, true)?);
                secondary_profile.password = None;
                if let Some(s3) = &sec.s3 {
                    save_secure_file(
                        &config_dir.join("secondary-aws-access-key-id"),
                        s3.access_key_id.expose_secret(),
                    )?;
                    save_secure_file(
                        &config_dir.join("secondary-aws-secret-access-key"),
                        s3.secret_access_key.expose_secret(),
                    )?;
                    secondary_profile.env = Some(std::collections::BTreeMap::from([
                        (
                            "AWS_ACCESS_KEY_ID".into(),
                            "{{ .Env.BACKUP_SECONDARY_AWS_ACCESS_KEY_ID }}".into(),
                        ),
                        (
                            "AWS_SECRET_ACCESS_KEY".into(),
                            "{{ .Env.BACKUP_SECONDARY_AWS_SECRET_ACCESS_KEY }}".into(),
                        ),
                    ]));
                }
                if secondary_profile
                    .env
                    .as_ref()
                    .is_some_and(|env| env.is_empty())
                {
                    secondary_profile.env = None;
                }
                if let Some(ref sftp) = sec.sftp {
                    if let Some(sftp_cmd) = sftp.sftp_command() {
                        let mut opt_map = secondary_profile.option.unwrap_or_default();
                        opt_map.insert("sftp.command".into(), sftp_cmd);
                        secondary_profile.option = Some(opt_map);
                    }
                }
                restic_config
                    .profiles
                    .insert("secondary".into(), secondary_profile);
            }
        }

        // 4. Build target profile section
        let copy_section = if self.storage.secondary.as_ref().map_or(false, |s| s.enabled) {
            let sec = self.storage.secondary.as_ref().unwrap();
            Some(CopyCommandSection {
                profile: Some("secondary".into()),
                repository: Some(sec.repository.clone()),
                password_file: Some(self.profile_password_file(config_dir, true)?),
                password: None,
                initialize: Some(true),
                schedule: None,
                ..Default::default()
            })
        } else {
            None
        };

        let profile_tags = normalize_backup_profile_tags(existing_profile_tags, &self.profile);
        let profile_section = ProfileSection {
            description: Some(format!("Backup profile for {}", self.profile)),
            inherit: Some("primary".into()),
            initialize: Some(true),
            insecure_tls: None,
            backup: Some(BackupCommandSection {
                source: Some(self.backup.targets.clone()),
                exclude: if self.backup.excludes.is_empty() {
                    None
                } else {
                    Some(self.backup.excludes.clone())
                },
                tag: Some(profile_tags.clone()),
                schedule: None,
                schedule_permission: None,
                schedule_priority: None,
                schedule_ignore_on_battery_less_than: None,
                run_before: None,
                run_finally: None,
                send_before: None,
                send_after: None,
                send_after_fail: None,
            }),
            retention: Some(RetentionSection {
                after_backup: Some(true),
                before_backup: None,
                compact: None,
                prune: Some(false),
                keep_daily: Some(self.retention.keep_daily),
                keep_weekly: Some(self.retention.keep_weekly),
                keep_monthly: Some(self.retention.keep_monthly),
                keep_yearly: None,
                keep_hourly: None,
                keep_last: None,
                keep_tag: None,
                tag: Some(vec![self.profile.clone()]),
            }),
            forget: Some(ForgetSection {
                schedule: None,
                prune: Some(false),
                keep_daily: Some(self.retention.keep_daily),
                keep_weekly: Some(self.retention.keep_weekly),
                keep_monthly: Some(self.retention.keep_monthly),
                keep_yearly: None,
                keep_hourly: None,
                keep_last: None,
                keep_tag: None,
                tag: Some(vec![self.profile.clone()]),
            }),
            prune: None,
            check: None,
            repository: None,
            password_file: None,
            password: None,
            env: None,
            option: None,
            copy: copy_section,
        };

        restic_config
            .profiles
            .insert(self.profile.clone(), profile_section);

        // Setup owns the migration of generated runnable profiles to the reserved tag
        // namespace. This changes future configuration only; it never mutates existing
        // repository snapshots or infers their identity.
        restic_config.ensure_reserved_backup_profile_tags()?;

        let yaml_content = serde_yaml::to_string(&restic_config)?;
        save_secure_file(profiles_yaml_path, &yaml_content)?;
        Ok(())
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let profiles = ResticProfileConfig::load_from_path(path)?;
        Self::from_profile_config(&profiles, path)
    }

    /// Builds the compatibility/report view from an already validated unified profile model.
    /// Operational command dispatch must load and validate `ResticProfileConfig` first; this
    /// projection is retained only for report rendering and legacy public APIs.
    pub fn from_profile_config(profiles: &ResticProfileConfig, path: &Path) -> Result<Self> {
        let application = profiles.application.clone().unwrap_or_default();
        let profile_name = application
            .database
            .as_ref()
            .map(|database| database.profile.clone())
            .or_else(|| profiles.profile_names().into_iter().next())
            .ok_or_else(|| anyhow::anyhow!("{} has no Backup Profiles", path.display()))?;
        let profile = profiles.profiles.get(&profile_name).unwrap();
        let backend = profiles
            .profiles
            .get(profile.inherit.as_deref().unwrap_or("primary"))
            .or_else(|| profiles.profiles.get("primary"))
            .unwrap_or(profile);
        let password = backend
            .password_file
            .as_deref()
            .map(fs::read_to_string)
            .transpose()?
            .unwrap_or_default();
        let config_dir = path.parent().unwrap_or(Path::new("."));
        let s3 = match (
            fs::read_to_string(config_dir.join("primary-aws-access-key-id")),
            fs::read_to_string(config_dir.join("primary-aws-secret-access-key")),
        ) {
            (Ok(access_key_id), Ok(secret_access_key)) => Some(S3Config {
                endpoint: String::new(),
                access_key_id: SecretString::new(access_key_id),
                secret_access_key: SecretString::new(secret_access_key),
            }),
            _ => None,
        };
        let database = application
            .database
            .as_ref()
            .map(|database| -> Result<BackupType> {
                Ok(BackupType::DbStream {
                    db_type: database.db_type,
                    connection_url: Some(
                        if database
                            .connection_url
                            .starts_with(APPLICATION_SECRET_PREFIX)
                        {
                            fs::read_to_string(
                                path.parent()
                                    .unwrap_or(Path::new("."))
                                    .join("database-connection-url"),
                            )?
                        } else {
                            database.connection_url.clone()
                        },
                    ),
                })
            })
            .transpose()?
            .unwrap_or(BackupType::Directory);
        Ok(Self {
            version: "2".into(),
            profile: profile_name,
            backup: BackupTargets {
                backup_type: database,
                targets: profile
                    .backup
                    .as_ref()
                    .and_then(|backup| backup.source.clone())
                    .unwrap_or_default(),
                excludes: profile
                    .backup
                    .as_ref()
                    .and_then(|backup| backup.exclude.clone())
                    .unwrap_or_default(),
            },
            retention: RetentionPolicy::standard_defaults(),
            storage: StorageConfig {
                primary: StorageTarget {
                    backend: if backend
                        .repository
                        .as_deref()
                        .unwrap_or("")
                        .starts_with("sftp:")
                    {
                        "sftp".into()
                    } else {
                        "s3".into()
                    },
                    repository: backend.repository.clone().unwrap_or_default(),
                    password: SecretString::new(password),
                    sftp: None,
                    s3,
                },
                secondary: None,
            },
            reports: application.reports,
            audit: application.audit,
        })
    }

    fn application_metadata_with_secret_references(
        &self,
        config_dir: &Path,
    ) -> Result<ApplicationConfig> {
        let mut application = ApplicationConfig {
            reports: self.reports.clone(),
            audit: self.audit.clone(),
            database: None,
        };
        // `profiles.yaml` has one format version: its resticprofile v2 top-level key.
        // Keep accepting the legacy application version on input, but never emit it
        // inside the application namespace.
        if let BackupType::DbStream {
            connection_url: Some(connection_url),
            db_type,
            ..
        } = &self.backup.backup_type
        {
            save_secure_file(&config_dir.join("database-connection-url"), connection_url)?;
            application.database = Some(DatabaseConfig {
                profile: self.profile.clone(),
                db_type: *db_type,
                connection_url: "${BACKUP_DATABASE_CONNECTION_URL}".into(),
            });
        }
        Ok(application)
    }

    pub fn render(&self, format: &str, redacted: bool) -> Result<String> {
        let target = if redacted {
            self.redacted()
        } else {
            self.clone()
        };
        if format == "json" {
            Ok(serde_json::to_string_pretty(&target)?)
        } else {
            Ok(serde_yaml::to_string(&target)?)
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            version: "1.0".into(),
            profile: "default".into(),
            backup: BackupTargets {
                backup_type: BackupType::Directory,
                targets: vec!["/data".into()],
                excludes: vec![],
            },
            retention: RetentionPolicy {
                keep_daily: 7,
                keep_weekly: 4,
                keep_monthly: 12,
            },
            storage: StorageConfig {
                primary: StorageTarget {
                    backend: String::new(),
                    repository: String::new(),
                    password: SecretString::new(String::new()),
                    sftp: None,
                    s3: None,
                },
                secondary: None,
            },
            reports: ReportsConfig::default(),
            audit: AuditConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Mysql,
    Postgres,
}

impl std::str::FromStr for DatabaseType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mysql" | "mariadb" => Ok(DatabaseType::Mysql),
            "postgres" | "postgresql" => Ok(DatabaseType::Postgres),
            _ => anyhow::bail!("Invalid database type: {}", s),
        }
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::Mysql => write!(f, "mysql"),
            DatabaseType::Postgres => write!(f, "postgres"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackupType {
    Directory,
    DbStream {
        db_type: DatabaseType,
        connection_url: Option<String>,
    },
}

impl Default for BackupType {
    fn default() -> Self {
        BackupType::Directory
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTargets {
    #[serde(default, alias = "backup_type")]
    pub backup_type: BackupType,
    pub targets: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    #[serde(alias = "keep_daily")]
    pub keep_daily: u32,
    #[serde(alias = "keep_weekly")]
    pub keep_weekly: u32,
    #[serde(alias = "keep_monthly")]
    pub keep_monthly: u32,
}

impl RetentionPolicy {
    pub fn standard_defaults() -> Self {
        Self {
            keep_daily: 7,
            keep_weekly: 4,
            keep_monthly: 12,
        }
    }

    pub fn long_term_defaults() -> Self {
        Self {
            keep_daily: 180,
            keep_weekly: 12,
            keep_monthly: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageConfig {
    pub primary: StorageTarget,
    pub secondary: Option<SecondaryStorageTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageTarget {
    pub backend: String,
    pub repository: String,
    #[serde(serialize_with = "serialize_secret_string")]
    pub password: SecretString,
    pub sftp: Option<SftpConfig>,
    pub s3: Option<S3Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecondaryStorageTarget {
    pub enabled: bool,
    pub backend: String,
    pub repository: String,
    #[serde(serialize_with = "serialize_secret_string")]
    pub password: SecretString,
    pub sftp: Option<SftpConfig>,
    pub s3: Option<S3Config>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_file: Option<String>,
}

impl SftpConfig {
    pub fn sftp_command(&self) -> Option<String> {
        let key_file = self.key_file.as_ref()?;
        if key_file.trim().is_empty() {
            return None;
        }
        Some(format!(
            "ssh -o StrictHostKeyChecking=no -i {} -p {} {}@{} -s sftp",
            key_file, self.port, self.user, self.host
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub endpoint: String,
    #[serde(serialize_with = "serialize_secret_string")]
    pub access_key_id: SecretString,
    #[serde(serialize_with = "serialize_secret_string")]
    pub secret_access_key: SecretString,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct AuditConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_manager: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_officer: Option<String>,
    /// Restore Drill policy metadata. `None` means the documented default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restore_drill_rto_minutes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restore_drill_timeout_minutes: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub restore_drill_work_dir: Option<String>,
}

impl AuditConfig {
    pub fn system_manager_name<'a>(&'a self, default: &'a str) -> &'a str {
        self.system_manager.as_deref().unwrap_or(default)
    }

    pub fn security_officer_name<'a>(&'a self, default: &'a str) -> &'a str {
        self.security_officer.as_deref().unwrap_or(default)
    }

    pub fn resolved_restore_drill_rto_minutes(&self) -> u64 {
        self.restore_drill_rto_minutes.unwrap_or(120)
    }

    pub fn resolved_restore_drill_timeout_minutes(&self) -> u64 {
        self.restore_drill_timeout_minutes.unwrap_or(240)
    }

    pub fn resolved_restore_drill_work_dir(&self) -> &str {
        self.restore_drill_work_dir
            .as_deref()
            .unwrap_or("/var/lib/backup/restore-drill")
    }

    pub fn validate_restore_drill_policy(&self) -> Result<()> {
        let rto = self.resolved_restore_drill_rto_minutes();
        let timeout = self.resolved_restore_drill_timeout_minutes();
        if rto == 0 {
            anyhow::bail!("restore-drill-rto-minutes must be at least 1");
        }
        if timeout < rto {
            anyhow::bail!(
                "restore-drill-timeout-minutes must be greater than or equal to restore-drill-rto-minutes"
            );
        }
        if self
            .restore_drill_work_dir
            .as_deref()
            .is_some_and(|path| path.trim().is_empty() || path != path.trim())
        {
            anyhow::bail!("restore-drill-work-dir must be an exact, non-empty path");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResticProfileConfig {
    pub version: String,
    /// Application-owned settings intentionally use a dedicated namespace so they
    /// do not collide with resticprofile v2's top-level keys.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_application"
    )]
    pub application: Option<ApplicationConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global: Option<GlobalSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub groups: Option<std::collections::BTreeMap<String, GroupSection>>,
    #[serde(default)]
    pub profiles: std::collections::BTreeMap<String, ProfileSection>,
}

#[derive(Debug, Clone)]
pub struct EffectiveBackupSettings {
    pub source: Vec<String>,
    pub exclude: Vec<String>,
    pub retention: RetentionPolicy,
}

impl ResticProfileConfig {
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let document: serde_yaml::Value = serde_yaml::from_str(&content)?;
        if document
            .as_mapping()
            .is_some_and(|mapping| mapping.contains_key(serde_yaml::Value::String("audit".into())))
        {
            anyhow::bail!("root audit is deprecated; move it to application.audit");
        }
        let config: Self = serde_yaml::from_str(&content)?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != "2" {
            anyhow::bail!("profiles.yaml must declare the resticprofile v2 root version: '2'");
        }
        if let Some(application) = &self.application {
            application.audit.validate_restore_drill_policy()?;
        }
        if let Some(database) = self
            .application
            .as_ref()
            .and_then(|app| app.database.as_ref())
        {
            if !self.profiles.contains_key(&database.profile) {
                anyhow::bail!(
                    "application.database.profile '{}' must name an existing Backup Profile",
                    database.profile
                );
            }
            if matches!(
                database.profile.as_str(),
                "default" | "primary" | "secondary"
            ) {
                anyhow::bail!(
                    "application.database.profile '{}' must name a Backup Profile, not a reserved Backend Profile",
                    database.profile
                );
            }
        }
        Ok(())
    }

    pub fn profile_names(&self) -> Vec<String> {
        self.profiles
            .iter()
            .filter(|(name, profile)| {
                !matches!(name.as_str(), "default" | "primary" | "secondary")
                    || profile.backup.is_some()
            })
            .map(|(name, _)| name.clone())
            .collect()
    }

    /// Resolves child-process-only S3 credentials for resticprofile commands.
    /// The unified configuration determines which sidecars are mandatory; no
    /// credential is retained in the configuration or process-global environment.
    pub fn sidecar_environment(&self, config_dir: &Path) -> Result<SecretEnvironment> {
        let definitions = [
            (
                "primary",
                "BACKUP_PRIMARY_AWS_ACCESS_KEY_ID",
                "BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY",
            ),
            (
                "secondary",
                "BACKUP_SECONDARY_AWS_ACCESS_KEY_ID",
                "BACKUP_SECONDARY_AWS_SECRET_ACCESS_KEY",
            ),
        ];
        let mut environment = Vec::new();
        for (backend, access_var, secret_var) in definitions {
            let Some(profile) = self.profiles.get(backend) else {
                continue;
            };
            let uses_s3_environment = profile.env.as_ref().is_some_and(|environment| {
                aws_sidecar_references(environment)
                    .is_some_and(|(access, secret)| access == access_var && secret == secret_var)
                    || environment
                        .values()
                        .any(|value| value.contains(access_var) || value.contains(secret_var))
            });
            let access_file = sidecar_file_name(access_var).expect("canonical S3 access variable");
            let secret_file = sidecar_file_name(secret_var).expect("canonical S3 secret variable");
            let access_path = config_dir.join(access_file);
            let secret_path = config_dir.join(secret_file);
            if uses_s3_environment {
                environment.push((
                    access_var.into(),
                    SecretString::new(read_secure_sidecar(&access_path)?),
                ));
                environment.push((
                    secret_var.into(),
                    SecretString::new(read_secure_sidecar(&secret_path)?),
                ));
            } else {
                for (variable, path) in [(access_var, access_path), (secret_var, secret_path)] {
                    if path.exists() {
                        environment.push((
                            variable.into(),
                            SecretString::new(read_secure_sidecar(&path)?),
                        ));
                    }
                }
            }
        }
        Ok(environment)
    }

    /// Resolves only the AWS environment required by one Backend Profile's direct restic calls.
    /// Unlike `sidecar_environment`, this never combines Primary and Secondary credentials.
    pub fn backend_sidecar_environment(
        &self,
        config_dir: &Path,
        backend: &str,
    ) -> Result<SecretEnvironment> {
        let Some((access_var, secret_var)) = self.s3_sidecar_references(backend) else {
            return Ok(Vec::new());
        };
        let access_file = sidecar_file_name(&access_var)
            .ok_or_else(|| anyhow::anyhow!("invalid access sidecar variable for '{backend}'"))?;
        let secret_file = sidecar_file_name(&secret_var)
            .ok_or_else(|| anyhow::anyhow!("invalid secret sidecar variable for '{backend}'"))?;
        Ok(vec![
            (
                "AWS_ACCESS_KEY_ID".into(),
                SecretString::new(read_secure_sidecar(&config_dir.join(access_file))?),
            ),
            (
                "AWS_SECRET_ACCESS_KEY".into(),
                SecretString::new(read_secure_sidecar(&config_dir.join(secret_file))?),
            ),
        ])
    }

    /// Resolves the S3 credentials needed by a profile's copy target. The
    /// target profile may inherit its S3 environment from another profile;
    /// callers receive only child-process environment values, never config
    /// fields or process-global state.
    pub fn copy_sidecar_environment(
        &self,
        config_dir: &Path,
        profile: &str,
    ) -> Result<SecretEnvironment> {
        let Some(target_name) = self.effective_copy_profile(profile)? else {
            return Ok(Vec::new());
        };
        let Some((access_var, secret_var)) = self.s3_sidecar_references(&target_name) else {
            return Ok(Vec::new());
        };
        let Some(access_file) = sidecar_file_name(&access_var) else {
            return Ok(Vec::new());
        };
        let Some(secret_file) = sidecar_file_name(&secret_var) else {
            return Ok(Vec::new());
        };
        Ok(vec![
            (
                "AWS_ACCESS_KEY_ID".into(),
                SecretString::new(read_secure_sidecar(&config_dir.join(access_file))?),
            ),
            (
                "AWS_SECRET_ACCESS_KEY".into(),
                SecretString::new(read_secure_sidecar(&config_dir.join(secret_file))?),
            ),
        ])
    }

    fn s3_sidecar_references(&self, profile: &str) -> Option<(String, String)> {
        let mut current = profile;
        let mut remaining = self.profiles.len() + 1;
        while remaining > 0 {
            remaining -= 1;
            let section = self.profiles.get(current)?;
            if let Some(environment) = &section.env {
                if let Some(references) = aws_sidecar_references(environment) {
                    return Some(references);
                }
            }
            current = section.inherit.as_deref()?;
        }
        None
    }

    pub fn application_config(&self) -> ApplicationConfig {
        self.application.clone().unwrap_or_default()
    }

    /// Resolves the operational Backup Profile inheritance chain without projecting it into the
    /// retired legacy configuration model.
    pub fn effective_backup_settings(&self, profile: &str) -> Result<EffectiveBackupSettings> {
        let mut current = profile;
        let mut source = None;
        let mut exclude = None;
        let mut keep_daily = None;
        let mut keep_weekly = None;
        let mut keep_monthly = None;
        let mut remaining = self.profiles.len() + 1;

        while remaining > 0 {
            remaining -= 1;
            let section = self
                .profiles
                .get(current)
                .ok_or_else(|| anyhow::anyhow!("Unknown Backup Profile '{current}'"))?;
            if let Some(backup) = &section.backup {
                if source.is_none() {
                    source = backup.source.clone();
                }
                if exclude.is_none() {
                    exclude = backup.exclude.clone();
                }
            }
            let retention = section.retention.as_ref();
            let forget = section.forget.as_ref();
            if keep_daily.is_none() {
                keep_daily = retention
                    .and_then(|value| value.keep_daily)
                    .or_else(|| forget.and_then(|value| value.keep_daily));
            }
            if keep_weekly.is_none() {
                keep_weekly = retention
                    .and_then(|value| value.keep_weekly)
                    .or_else(|| forget.and_then(|value| value.keep_weekly));
            }
            if keep_monthly.is_none() {
                keep_monthly = retention
                    .and_then(|value| value.keep_monthly)
                    .or_else(|| forget.and_then(|value| value.keep_monthly));
            }
            let Some(parent) = section.inherit.as_deref() else {
                break;
            };
            current = parent;
        }
        if remaining == 0 {
            anyhow::bail!("Backup Profile inheritance is cyclic");
        }

        Ok(EffectiveBackupSettings {
            source: source.unwrap_or_default(),
            exclude: exclude.unwrap_or_default(),
            retention: RetentionPolicy {
                keep_daily: keep_daily.unwrap_or(7),
                keep_weekly: keep_weekly.unwrap_or(4),
                keep_monthly: keep_monthly.unwrap_or(12),
            },
        })
    }

    /// Ensures every runnable Backup Profile has its exact CLI-owned snapshot tag while
    /// preserving ordinary user tags and removing stale tags from the reserved namespace.
    pub fn ensure_reserved_backup_profile_tags(&mut self) -> Result<()> {
        let runnable_profiles =
            crate::config::profile_resolver::ProfileResolver::resolve_all_active(self)?;
        for profile in runnable_profiles {
            let section = self
                .profiles
                .get_mut(&profile.name)
                .ok_or_else(|| anyhow::anyhow!("Unknown Backup Profile '{}'", profile.name))?;
            let tags = section
                .backup
                .as_ref()
                .and_then(|backup| backup.tag.as_ref())
                .cloned()
                .unwrap_or_default();
            let backup = section.backup.get_or_insert_with(Default::default);
            backup.tag = Some(normalize_backup_profile_tags(tags, &profile.name));
        }
        Ok(())
    }

    /// Verifies that an operational Backup Profile can only create snapshots carrying
    /// its exact CLI-owned identity tag. Existing legacy snapshots remain untouched;
    /// this guard only protects new backup executions from producing ambiguous snapshots.
    pub fn validate_reserved_backup_profile_tag(&self, profile: &str) -> Result<()> {
        let expected = backup_profile_snapshot_tag(profile);
        let tags =
            crate::config::profile_resolver::ProfileResolver::resolve_backup_tags(self, profile)?;
        if !tags.iter().any(|tag| tag == &expected) {
            anyhow::bail!(
                "Backup Profile '{profile}' must declare the exact reserved snapshot tag '{expected}'"
            );
        }
        Ok(())
    }

    /// Validates a concrete profiles file before an operational adapter is invoked.
    pub fn validate_reserved_backup_profile_tag_at_path(path: &Path, profile: &str) -> Result<()> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_file() => {
                Self::load_from_path(path)?.validate_reserved_backup_profile_tag(profile)
            }
            Ok(_) => anyhow::bail!(
                "profiles configuration '{}' is not a regular file",
                path.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => anyhow::bail!(
                "profiles configuration '{}' is not a regular file",
                path.display()
            ),
            Err(error) => Err(error.into()),
        }
    }

    /// Resolves the copy target declared by a Backup Profile through its inheritance chain.
    /// A copy command must name an existing Backend Profile before it reaches an external runner.
    pub fn effective_copy_profile(&self, profile: &str) -> Result<Option<String>> {
        for section in self.inherited_profile_chain(profile)? {
            if let Some(copy) = &section.copy {
                let target = copy.profile.as_deref().unwrap_or("secondary");
                if !self.profiles.contains_key(target) {
                    anyhow::bail!(
                        "Copy target Backend Profile '{target}' is not configured for '{profile}'"
                    );
                }
                return Ok(Some(target.into()));
            }
        }
        Ok(None)
    }

    fn inherited_profile_chain(&self, profile: &str) -> Result<Vec<&ProfileSection>> {
        let mut current = profile;
        let mut remaining = self.profiles.len() + 1;
        let mut chain = Vec::new();
        while remaining > 0 {
            remaining -= 1;
            let section = self.profiles.get(current).ok_or_else(|| {
                anyhow::anyhow!("Profile '{profile}' inherits unknown profile '{current}'")
            })?;
            chain.push(section);
            let Some(parent) = section.inherit.as_deref() else {
                return Ok(chain);
            };
            current = parent;
        }
        anyhow::bail!("Profile '{profile}' has a cyclic inheritance chain")
    }

    /// Resolves the repository URI for a Backend Profile through its inheritance chain.
    /// Diagnostics use this accessor so storage checks always follow the configured
    /// repository instead of inventing an adapter target.
    pub fn backend_repository(&self, profile: &str) -> Result<String> {
        self.inherited_profile_chain(profile)?
            .into_iter()
            .find_map(|section| section.repository.clone())
            .ok_or_else(|| anyhow::anyhow!("Profile '{profile}' has no repository"))
    }

    /// Returns Backend Adapter initialization targets in the contract-defined order.
    pub fn backend_initialization_targets(&self) -> Result<Vec<String>> {
        let mut targets = Vec::new();
        if self.profiles.contains_key("primary") {
            targets.push("primary".into());
        }
        let mut ordinary = self
            .profiles
            .keys()
            .filter(|name| {
                !matches!(name.as_str(), "primary" | "secondary")
                    && !(name.as_str() == "default"
                        && self
                            .profiles
                            .get(*name)
                            .is_some_and(|profile| profile.backup.is_none()))
            })
            .cloned()
            .collect::<Vec<_>>();
        ordinary.sort();
        targets.extend(ordinary);
        if self.profiles.contains_key("secondary") {
            targets.push("secondary".into());
        }
        if targets.is_empty() {
            anyhow::bail!("No Backup Profiles are configured for backend initialization");
        }
        Ok(targets)
    }

    /// Resolves a Backend Profile through its inheritance chain without a lossy
    /// projection into the retired operational configuration model.
    pub fn backend_credentials(
        &self,
        config_dir: &Path,
        profile: &str,
    ) -> Result<(String, String)> {
        let chain = self.inherited_profile_chain(profile)?;
        let repository = chain
            .iter()
            .find_map(|section| section.repository.clone())
            .ok_or_else(|| anyhow::anyhow!("Profile '{profile}' has no repository"))?;
        let password_file = chain
            .iter()
            .find_map(|section| section.password_file.clone())
            .ok_or_else(|| anyhow::anyhow!("Profile '{profile}' has no password-file"))?;
        let password_path = Path::new(&password_file);
        let password_path = if password_path.is_absolute() {
            password_path.to_path_buf()
        } else {
            config_dir.join(password_path)
        };
        Ok((repository, read_secure_sidecar(&password_path)?))
    }

    pub fn secure_sidecar_value(config_dir: &Path, name: &str) -> Result<String> {
        read_secure_sidecar(&config_dir.join(name))
    }

    /// Returns sidecar paths referenced by profile environment declarations.
    /// Environment-reference parsing belongs to the configuration model so cleanup code does
    /// not need to duplicate the resticprofile syntax or sidecar naming convention.
    pub fn environment_sidecar_paths(&self, config_dir: &Path) -> Vec<std::path::PathBuf> {
        self.profiles
            .values()
            .filter_map(|profile| profile.env.as_ref())
            .flat_map(|environment| environment.values())
            .filter_map(|value| env_reference(value))
            .filter_map(|variable| sidecar_file_name(&variable))
            .map(|name| config_dir.join(name))
            .collect()
    }
}

fn normalize_backup_profile_tags(tags: Vec<String>, profile: &str) -> Vec<String> {
    let mut normalized = tags
        .into_iter()
        .filter(|tag| !tag.starts_with(BACKUP_PROFILE_TAG_PREFIX))
        .collect::<Vec<_>>();
    if !normalized.iter().any(|tag| tag == profile) {
        normalized.push(profile.into());
    }
    let reserved = backup_profile_snapshot_tag(profile);
    normalized.push(reserved);
    normalized
}

fn read_secure_sidecar(path: &Path) -> Result<String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(path)?.permissions().mode() & 0o777;
        if mode != 0o600 {
            anyhow::bail!("credential sidecar {} must have mode 0600", path.display());
        }
    }
    let value = fs::read_to_string(path)?;
    if value.trim().is_empty() {
        anyhow::bail!("credential sidecar {} cannot be empty", path.display());
    }
    Ok(value)
}

fn aws_sidecar_references(
    environment: &std::collections::BTreeMap<String, String>,
) -> Option<(String, String)> {
    Some((
        environment
            .get("AWS_ACCESS_KEY_ID")
            .and_then(|value| env_reference(value))?,
        environment
            .get("AWS_SECRET_ACCESS_KEY")
            .and_then(|value| env_reference(value))?,
    ))
}

fn sidecar_file_name(variable: &str) -> Option<String> {
    variable
        .strip_prefix("BACKUP_")
        .map(|name| name.to_ascii_lowercase().replace('_', "-"))
}

fn env_reference(value: &str) -> Option<String> {
    let reference = value.split_once(".Env.")?.1;
    let end = reference
        .find(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .unwrap_or(reference.len());
    (!reference[..end].is_empty()).then(|| reference[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::ResticProfileConfig;

    fn config(yaml: &str) -> ResticProfileConfig {
        serde_yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn effective_copy_profile_resolves_inherited_target_and_rejects_unknown_target() {
        let profiles = config(
            r#"
version: "2"
profiles:
  primary:
    repository: /primary
  secondary:
    repository: /secondary
  default:
    copy:
      profile: secondary
  daily:
    inherit: default
    backup:
      source: [/data]
"#,
        );
        assert_eq!(
            profiles.effective_copy_profile("daily").unwrap(),
            Some("secondary".into())
        );

        let invalid = config(
            r#"
version: "2"
profiles:
  daily:
    copy:
      profile: missing
"#,
        );
        assert!(invalid.effective_copy_profile("daily").is_err());
    }

    #[test]
    fn backend_initialization_targets_are_deterministic_and_keep_real_default_profiles() {
        let profiles = config(
            r#"
version: "2"
profiles:
  secondary: {}
  zeta:
    backup:
      source: [/zeta]
  primary: {}
  alpha:
    backup:
      source: [/alpha]
  default: {}
"#,
        );

        assert_eq!(
            profiles.backend_initialization_targets().unwrap(),
            ["primary", "alpha", "zeta", "secondary"]
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GlobalSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scheduler: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct GroupSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_on_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ProfileSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub insecure_tls: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup: Option<BackupCommandSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retention: Option<RetentionSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forget: Option<ForgetSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<PruneCommandSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<CheckCommandSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<std::collections::BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copy: Option<CopyCommandSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CopyCommandSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initialize: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct BackupCommandSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_permission: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule_ignore_on_battery_less_than: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_finally: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_before: Option<Vec<HttpHook>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_after: Option<Vec<HttpHook>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send_after_fail: Option<HttpHook>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct RetentionSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after_backup: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before_backup: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compact: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_daily: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_weekly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_monthly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_yearly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_hourly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_last: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct ForgetSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prune: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_daily: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_weekly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_monthly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_yearly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_hourly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_last: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_tag: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct PruneCommandSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_daily: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_weekly: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keep_monthly: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct CheckCommandSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schedule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct HttpHook {
    pub method: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<HeaderEntry>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "kebab-case")]
pub struct HeaderEntry {
    pub name: String,
    pub value: String,
}

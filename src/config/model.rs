use anyhow::Result;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fs;
use std::path::Path;

pub const DEFAULT_PROFILES_FILENAME: &str = "profiles.yaml";
pub const DEFAULT_PROFILES_PATH: &str = "/etc/backup/profiles.yaml";

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
        primary_profile.inherit = Some("default".into());
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
                tag: Some(vec![self.profile.clone()]),
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
}

impl AuditConfig {
    pub fn system_manager_name<'a>(&'a self, default: &'a str) -> &'a str {
        self.system_manager.as_deref().unwrap_or(default)
    }

    pub fn security_officer_name<'a>(&'a self, default: &'a str) -> &'a str {
        self.security_officer.as_deref().unwrap_or(default)
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
    pub fn sidecar_environment(&self, config_dir: &Path) -> Result<Vec<(String, String)>> {
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
                environment.push((access_var.into(), read_secure_sidecar(&access_path)?));
                environment.push((secret_var.into(), read_secure_sidecar(&secret_path)?));
            } else {
                for (variable, path) in [(access_var, access_path), (secret_var, secret_path)] {
                    if path.exists() {
                        environment.push((variable.into(), read_secure_sidecar(&path)?));
                    }
                }
            }
        }
        Ok(environment)
    }

    /// Resolves the S3 credentials needed by a profile's copy target. The
    /// target profile may inherit its S3 environment from another profile;
    /// callers receive only child-process environment values, never config
    /// fields or process-global state.
    pub fn copy_sidecar_environment(
        &self,
        config_dir: &Path,
        profile: &str,
    ) -> Result<Vec<(String, String)>> {
        let Some(copy) = self
            .profiles
            .get(profile)
            .and_then(|profile| profile.copy.as_ref())
        else {
            return Ok(Vec::new());
        };
        let target_name = copy.profile.as_deref().unwrap_or("secondary");
        let Some((access_var, secret_var)) = self.s3_sidecar_references(target_name) else {
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
                read_secure_sidecar(&config_dir.join(access_file))?,
            ),
            (
                "AWS_SECRET_ACCESS_KEY".into(),
                read_secure_sidecar(&config_dir.join(secret_file))?,
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

    /// Resolves a Backend Profile through its inheritance chain without a lossy
    /// projection into the retired operational configuration model.
    pub fn backend_credentials(
        &self,
        config_dir: &Path,
        profile: &str,
    ) -> Result<(String, String)> {
        let mut current = self
            .profiles
            .get(profile)
            .ok_or_else(|| anyhow::anyhow!("Unknown profile '{profile}'"))?;
        let mut repository = current.repository.clone();
        let mut password_file = current.password_file.clone();
        let mut remaining = self.profiles.len() + 1;
        while (repository.is_none() || password_file.is_none()) && remaining > 0 {
            remaining -= 1;
            let Some(parent) = current.inherit.as_deref() else {
                break;
            };
            current = self.profiles.get(parent).ok_or_else(|| {
                anyhow::anyhow!("Profile '{profile}' inherits unknown profile '{parent}'")
            })?;
            repository = repository.or_else(|| current.repository.clone());
            password_file = password_file.or_else(|| current.password_file.clone());
        }
        if remaining == 0 {
            anyhow::bail!("Profile '{profile}' has a cyclic inheritance chain");
        }
        let repository =
            repository.ok_or_else(|| anyhow::anyhow!("Profile '{profile}' has no repository"))?;
        let password_file = password_file
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
    Ok(fs::read_to_string(path)?)
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

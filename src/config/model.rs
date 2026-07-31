use anyhow::Result;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};
use std::fs;
use std::path::Path;

pub const DEFAULT_PROFILES_FILENAME: &str = "profiles.yaml";
pub const DEFAULT_PROFILES_PATH: &str = "/etc/backup/profiles.yaml";

const APPLICATION_SECRET_PREFIX: &str = "${BACKUP_";

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
                    audit: None,
                    global: None,
                    groups: None,
                    profiles: std::collections::BTreeMap::new(),
                }
            })
        } else {
            ResticProfileConfig {
                version: "2".into(),
                application: None,
                audit: None,
                global: None,
                groups: None,
                profiles: std::collections::BTreeMap::new(),
            }
        };

        restic_config.version = "2".into();
        restic_config.audit = Some(self.audit.clone());
        restic_config.application =
            Some(self.application_config_with_secret_references(config_dir)?);

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
        if let Some(env) = primary_profile.env.as_mut() {
            env.remove("AWS_ACCESS_KEY_ID");
            env.remove("AWS_SECRET_ACCESS_KEY");
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
                if let Some(env) = secondary_profile.env.as_mut() {
                    env.remove("AWS_ACCESS_KEY_ID");
                    env.remove("AWS_SECRET_ACCESS_KEY");
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
        let content = fs::read_to_string(path)?;
        let profiles: ResticProfileConfig = serde_yaml::from_str(&content)?;
        let config = profiles.application.ok_or_else(|| {
            anyhow::anyhow!(
                "{} is missing its application configuration section",
                path.display()
            )
        })?;
        config.resolve_application_secret_references(path.parent().unwrap_or(Path::new(".")))
    }

    fn application_config_with_secret_references(&self, config_dir: &Path) -> Result<Self> {
        let mut application = self.clone();
        application.storage.primary.password =
            SecretString::new("${BACKUP_PRIMARY_PASSWORD}".into());
        if let Some(s3) = application.storage.primary.s3.as_mut() {
            save_secure_file(
                &config_dir.join("primary-aws-access-key-id"),
                s3.access_key_id.expose_secret(),
            )?;
            save_secure_file(
                &config_dir.join("primary-aws-secret-access-key"),
                s3.secret_access_key.expose_secret(),
            )?;
            s3.access_key_id = SecretString::new("${BACKUP_PRIMARY_AWS_ACCESS_KEY_ID}".into());
            s3.secret_access_key =
                SecretString::new("${BACKUP_PRIMARY_AWS_SECRET_ACCESS_KEY}".into());
        }
        if let Some(secondary) = application.storage.secondary.as_mut() {
            secondary.password = SecretString::new("${BACKUP_SECONDARY_PASSWORD}".into());
            if let Some(s3) = secondary.s3.as_mut() {
                save_secure_file(
                    &config_dir.join("secondary-aws-access-key-id"),
                    s3.access_key_id.expose_secret(),
                )?;
                save_secure_file(
                    &config_dir.join("secondary-aws-secret-access-key"),
                    s3.secret_access_key.expose_secret(),
                )?;
                s3.access_key_id =
                    SecretString::new("${BACKUP_SECONDARY_AWS_ACCESS_KEY_ID}".into());
                s3.secret_access_key =
                    SecretString::new("${BACKUP_SECONDARY_AWS_SECRET_ACCESS_KEY}".into());
            }
        }
        if let BackupType::DbStream {
            connection_url: Some(connection_url),
            ..
        } = &mut application.backup.backup_type
        {
            save_secure_file(&config_dir.join("database-connection-url"), connection_url)?;
            *connection_url = "${BACKUP_DATABASE_CONNECTION_URL}".into();
        }
        Ok(application)
    }

    fn resolve_application_secret_references(mut self, config_dir: &Path) -> Result<Self> {
        fn read_secret(config_dir: &Path, marker: &str, filename: &str) -> Result<String> {
            if marker.starts_with(APPLICATION_SECRET_PREFIX) {
                let path = config_dir.join(filename);
                if path.is_file() {
                    return Ok(fs::read_to_string(path)?);
                }
                let legacy_enc = config_dir.join("enc");
                if (filename == "primary-password" || filename == "secondary-password")
                    && legacy_enc.is_file()
                {
                    return Ok(fs::read_to_string(legacy_enc)?);
                }
                anyhow::bail!("Missing secret file for application setting {marker}");
            }
            Ok(marker.to_owned())
        }

        self.storage.primary.password = SecretString::new(read_secret(
            config_dir,
            self.storage.primary.password.expose_secret(),
            "primary-password",
        )?);
        if let Some(s3) = self.storage.primary.s3.as_mut() {
            s3.access_key_id = SecretString::new(read_secret(
                config_dir,
                s3.access_key_id.expose_secret(),
                "primary-aws-access-key-id",
            )?);
            s3.secret_access_key = SecretString::new(read_secret(
                config_dir,
                s3.secret_access_key.expose_secret(),
                "primary-aws-secret-access-key",
            )?);
        }
        if let Some(secondary) = self.storage.secondary.as_mut() {
            secondary.password = SecretString::new(read_secret(
                config_dir,
                secondary.password.expose_secret(),
                "secondary-password",
            )?);
            if let Some(s3) = secondary.s3.as_mut() {
                s3.access_key_id = SecretString::new(read_secret(
                    config_dir,
                    s3.access_key_id.expose_secret(),
                    "secondary-aws-access-key-id",
                )?);
                s3.secret_access_key = SecretString::new(read_secret(
                    config_dir,
                    s3.secret_access_key.expose_secret(),
                    "secondary-aws-secret-access-key",
                )?);
            }
        }
        if let BackupType::DbStream {
            connection_url: Some(connection_url),
            ..
        } = &mut self.backup.backup_type
        {
            *connection_url = read_secret(config_dir, connection_url, "database-connection-url")?;
        }
        Ok(self)
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub application: Option<BackupConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit: Option<AuditConfig>,
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
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
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

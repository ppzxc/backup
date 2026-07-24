use anyhow::Result;
use std::fs;
use std::path::Path;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, Serializer};

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
}

impl BackupConfig {
    pub fn redacted(&self) -> Self {
        let mut masked = self.clone();
        masked.storage.primary.password = SecretString::new("******".into());
        if let Some(ref mut s3) = masked.storage.primary.s3 {
            s3.secret_access_key = SecretString::new("******".into());
        }
        if let Some(ref mut sec) = masked.storage.secondary {
            sec.password = SecretString::new("******".into());
        }
        masked
    }

    pub fn save_to_path(&self, path: &Path) -> Result<()> {
        let yaml = serde_yaml::to_string(self)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(parent, fs::Permissions::from_mode(0o700))?;
            }
        }
        fs::write(path, yaml)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn save_and_sync(&self, config_dir: &Path) -> Result<()> {
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(config_dir, fs::Permissions::from_mode(0o700))?;
            }
        }

        let profiles_yaml_path = config_dir.join("profiles.yaml");
        let mut restic_config = if profiles_yaml_path.exists() {
            ResticProfileConfig::load_from_path(&profiles_yaml_path).unwrap_or_else(|_| ResticProfileConfig {
                version: "2".into(),
                global: None,
                groups: None,
                profiles: std::collections::BTreeMap::new(),
            })
        } else {
            ResticProfileConfig {
                version: "2".into(),
                global: None,
                groups: None,
                profiles: std::collections::BTreeMap::new(),
            }
        };

        restic_config.version = "2".into();

        // 1. Populate default profile (truly global options only)
        let mut default_profile = restic_config.profiles.remove("default").unwrap_or_default();
        if default_profile.description.is_none() {
            default_profile.description = Some("Global common options".into());
        }
        default_profile.insecure_tls = Some(true);
        restic_config.profiles.insert("default".into(), default_profile);

        // 2. Populate primary profile (1st storage configuration)
        let mut primary_profile = restic_config.profiles.remove("primary").unwrap_or_default();
        if primary_profile.description.is_none() {
            primary_profile.description = Some("Primary Storage configuration".into());
        }
        primary_profile.repository = Some(self.storage.primary.repository.clone());
        let enc_path = Path::new("/etc/backup/enc");
        if enc_path.is_file() {
            primary_profile.password_file = Some("/etc/backup/enc".into());
            primary_profile.password = None;
        } else {
            let pwd = self.storage.primary.password.expose_secret();
            if !pwd.trim().is_empty() {
                primary_profile.password = Some(pwd.to_string());
            } else {
                primary_profile.password = Some("default_secret_pass123".into());
            }
        }
        if let Some(ref s3) = self.storage.primary.s3 {
            let mut env_map = primary_profile.env.unwrap_or_default();
            env_map.insert("AWS_ACCESS_KEY_ID".into(), s3.access_key_id.clone());
            env_map.insert("AWS_SECRET_ACCESS_KEY".into(), s3.secret_access_key.expose_secret().to_string());
            primary_profile.env = Some(env_map);
        }
        restic_config.profiles.insert("primary".into(), primary_profile);

        // 3. Populate secondary profile (if enabled)
        if let Some(ref sec) = self.storage.secondary {
            if sec.enabled {
                let mut secondary_profile = restic_config.profiles.remove("secondary").unwrap_or_default();
                if secondary_profile.description.is_none() {
                    secondary_profile.description = Some("Secondary Storage configuration".into());
                }
                secondary_profile.repository = Some(sec.repository.clone());
                let pwd = sec.password.expose_secret();
                if !pwd.trim().is_empty() {
                    secondary_profile.password = Some(pwd.to_string());
                }
                restic_config.profiles.insert("secondary".into(), secondary_profile);
            }
        }

        // 4. Build target profile section
        let copy_section = if self.storage.secondary.as_ref().map_or(false, |s| s.enabled) {
            Some(CopyCommandSection {
                profile: Some("secondary".into()),
                initialize: Some(true),
                schedule: Some("*-*-* 04:00:00".into()),
                ..Default::default()
            })
        } else {
            None
        };

        let profile_section = ProfileSection {
            description: Some(format!("Backup profile for {}", self.profile)),
            inherit: Some(vec!["default".into(), "primary".into()]),
            initialize: Some(true),
            insecure_tls: None,
            backup: Some(BackupCommandSection {
                source: Some(self.backup.targets.clone()),
                exclude: if self.backup.excludes.is_empty() { None } else { Some(self.backup.excludes.clone()) },
                tag: Some(vec![self.profile.clone()]),
                schedule: Some("*-*-* 03:00:00".into()),
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
            copy: copy_section,
        };

        restic_config.profiles.insert(self.profile.clone(), profile_section);

        let yaml_content = serde_yaml::to_string(&restic_config)?;
        fs::write(&profiles_yaml_path, yaml_content)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&profiles_yaml_path, fs::Permissions::from_mode(0o600))?;
        }
        Ok(())
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn render(&self, format: &str, redacted: bool) -> Result<String> {
        let target = if redacted { self.redacted() } else { self.clone() };
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
                    backend: "sftp".into(),
                    repository: "rclone:syno_backup:/backup".into(),
                    password: SecretString::new("default_secret".into()),
                    sftp: None,
                    s3: None,
                },
                secondary: None,
            },
            reports: ReportsConfig::default(),
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum BackupType {
    Directory,
    DbStream {
        db_type: String, // "mysql" or "postgres"
        connection_url: Option<String>,
        dump_command: Option<String>,
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
    #[serde(default)]
    pub backup_type: BackupType,
    pub targets: Vec<String>,
    pub excludes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetentionPolicy {
    pub keep_daily: u32,
    pub keep_weekly: u32,
    pub keep_monthly: u32,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SftpConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub key_file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct S3Config {
    pub endpoint: String,
    pub access_key_id: String,
    #[serde(serialize_with = "serialize_secret_string")]
    pub secret_access_key: SecretString,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResticProfileConfig {
    pub version: String,
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
            .keys()
            .filter(|k| k.as_str() != "default" && k.as_str() != "primary" && k.as_str() != "secondary")
            .cloned()
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(with = "inherit_serde")]
    pub inherit: Option<Vec<String>>,
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
    pub copy: Option<CopyCommandSection>,
}

pub mod inherit_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Deserialize, Serialize)]
    #[serde(untagged)]
    enum StringOrVec {
        Single(String),
        Multiple(Vec<String>),
    }

    pub fn serialize<S>(val: &Option<Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match val {
            Some(v) => {
                if v.len() == 1 {
                    serializer.serialize_str(&v[0])
                } else {
                    v.serialize(serializer)
                }
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<StringOrVec> = Option::deserialize(deserializer)?;
        match opt {
            Some(StringOrVec::Single(s)) => Ok(Some(vec![s])),
            Some(StringOrVec::Multiple(v)) => Ok(Some(v)),
            None => Ok(None),
        }
    }
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


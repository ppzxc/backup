use crate::config::model::ResticProfileConfig;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedProfile {
    pub name: String,
    pub backend: String,
    pub repository: String,
    pub targets: Vec<String>,
}

pub struct ProfileResolver;

impl ProfileResolver {
    pub fn resolve(config: &ResticProfileConfig, profile_name: &str) -> Option<ResolvedProfile> {
        let profile_section = config.profiles.get(profile_name)?;

        let repository = profile_section
            .repository
            .as_deref()
            .or_else(|| {
                profile_section.inherit.as_ref().and_then(|p| {
                    config
                        .profiles
                        .get(p)
                        .and_then(|sec| sec.repository.as_deref())
                })
            })
            .or_else(|| {
                config
                    .profiles
                    .get("primary")
                    .and_then(|p| p.repository.as_deref())
            })
            .or_else(|| {
                config
                    .profiles
                    .get("default")
                    .and_then(|p| p.repository.as_deref())
            })
            .unwrap_or("unknown")
            .to_string();

        let backend = if repository.starts_with("s3:") {
            "s3"
        } else if repository.starts_with("rclone:") {
            "rclone"
        } else if repository.starts_with("sftp:") {
            "sftp"
        } else {
            "local"
        }
        .to_string();

        let targets = profile_section
            .backup
            .as_ref()
            .and_then(|b| b.source.as_ref())
            .or_else(|| {
                profile_section.inherit.as_ref().and_then(|p| {
                    config
                        .profiles
                        .get(p)
                        .and_then(|sec| sec.backup.as_ref().and_then(|b| b.source.as_ref()))
                })
            })
            .or_else(|| {
                config
                    .profiles
                    .get("default")
                    .and_then(|p| p.backup.as_ref().and_then(|b| b.source.as_ref()))
            })
            .cloned()
            .unwrap_or_default();

        Some(ResolvedProfile {
            name: profile_name.to_string(),
            backend,
            repository,
            targets,
        })
    }

    pub fn resolve_all_or_filtered(
        config: &ResticProfileConfig,
        profile_filter: Option<&str>,
    ) -> Vec<ResolvedProfile> {
        let target_profile_names: Vec<String> = if let Some(p) = profile_filter {
            vec![p.to_string()]
        } else {
            let names = config.profile_names();
            if names.is_empty() { Vec::new() } else { names }
        };

        target_profile_names
            .into_iter()
            .filter_map(|name| Self::resolve(config, &name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::model::ProfileSection;
    use std::collections::BTreeMap;

    #[test]
    fn test_profile_resolver_with_inheritance() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "default".to_string(),
            ProfileSection {
                repository: Some("/var/backup/repo".to_string()),
                ..Default::default()
            },
        );
        profiles.insert(
            "myprofile".to_string(),
            ProfileSection {
                inherit: Some("default".to_string()),
                backup: Some(crate::config::model::BackupCommandSection {
                    source: Some(vec!["/home/user".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );

        let config = ResticProfileConfig {
            version: "2".to_string(),
            application: None,
            global: None,
            groups: None,
            profiles,
        };

        let resolved = ProfileResolver::resolve(&config, "myprofile").unwrap();
        assert_eq!(resolved.name, "myprofile");
        assert_eq!(resolved.repository, "/var/backup/repo");
        assert_eq!(resolved.backend, "local");
        assert_eq!(resolved.targets, vec!["/home/user".to_string()]);
    }
}

use crate::config::model::{ProfileSection, ResticProfileConfig};
use anyhow::Result;

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
        let chain = inheritance_chain(config, profile_name)?;

        let repository = first_repository(&chain)
            .or_else(|| fallback_repository(config, "primary"))
            .or_else(|| fallback_repository(config, "default"))
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

        let targets = first_targets(&chain)
            .or_else(|| fallback_targets(config, "default"))
            .cloned()
            .unwrap_or_default();

        Some(ResolvedProfile {
            name: profile_name.to_string(),
            backend,
            repository,
            targets,
        })
    }

    /// Resolves an exact selector for a command contract. Selectors are never trimmed, folded,
    /// prefix-matched, or replaced with a fallback profile.
    pub fn resolve_exact(
        config: &ResticProfileConfig,
        profile_name: &str,
        command: &str,
    ) -> Result<ResolvedProfile> {
        if profile_name.trim().is_empty() || profile_name != profile_name.trim() {
            anyhow::bail!("{command} profile must be an exact, non-empty configured profile name");
        }
        if !config
            .profile_names()
            .iter()
            .any(|name| name == profile_name)
        {
            anyhow::bail!("{command} profile '{profile_name}' is not configured");
        }
        Self::resolve(config, profile_name).ok_or_else(|| {
            anyhow::anyhow!(
                "{command} profile '{profile_name}' has an invalid or cyclic inheritance chain"
            )
        })
    }

    /// Returns every configured active Backup Profile in stable configuration order.
    pub fn resolve_all_active(config: &ResticProfileConfig) -> Result<Vec<ResolvedProfile>> {
        resolve_all_runnable(config, "status")
    }

    /// Resolves all active profiles while retaining per-profile resolution failures so status can
    /// report healthy profiles and warnings together.
    pub fn resolve_all_active_with_failures(
        config: &ResticProfileConfig,
    ) -> (Vec<ResolvedProfile>, Vec<String>) {
        resolve_all_runnable_with_failures(config, "status")
    }

    /// Resolves the effective Backup Profile tag list through the same inheritance chain used by
    /// operational profile resolution.
    pub fn resolve_backup_tags(
        config: &ResticProfileConfig,
        profile_name: &str,
    ) -> Result<Vec<String>> {
        let chain = inheritance_chain(config, profile_name).ok_or_else(|| {
            anyhow::anyhow!("Backup Profile '{profile_name}' has an invalid inheritance chain")
        })?;
        let mut tags = Vec::new();
        for section in chain {
            if let Some(section_tags) = section
                .backup
                .as_ref()
                .and_then(|backup| backup.tag.as_ref())
            {
                for tag in section_tags {
                    if !tags.contains(tag) {
                        tags.push(tag.clone());
                    }
                }
            }
        }
        Ok(tags)
    }

    /// Resolves the command-specific run selection: all runnable profiles when omitted, or one
    /// exact profile when supplied.
    pub fn resolve_for_run(
        config: &ResticProfileConfig,
        profile_filter: Option<&str>,
    ) -> Result<Vec<ResolvedProfile>> {
        match profile_filter {
            Some(profile) => Ok(vec![Self::resolve_exact(config, profile, "run")?]),
            None => resolve_all_runnable(config, "run"),
        }
    }

    /// Resolves the command-specific status selection: all active profiles when omitted, or one
    /// exact profile when supplied.
    pub fn resolve_for_status(
        config: &ResticProfileConfig,
        profile_filter: Option<&str>,
    ) -> Result<Vec<ResolvedProfile>> {
        match profile_filter {
            Some(profile) => Ok(vec![Self::resolve_exact(config, profile, "status")?]),
            None => Self::resolve_all_active(config),
        }
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

fn inheritance_chain<'a>(
    config: &'a ResticProfileConfig,
    profile_name: &str,
) -> Option<Vec<&'a ProfileSection>> {
    let mut chain = Vec::new();
    let mut current = profile_name;
    let mut remaining = config.profiles.len() + 1;

    while remaining > 0 {
        remaining -= 1;
        let section = config.profiles.get(current)?;
        chain.push(section);
        let Some(parent) = section.inherit.as_deref() else {
            return Some(chain);
        };
        current = parent;
    }

    None
}

fn first_repository<'a>(chain: &[&'a ProfileSection]) -> Option<&'a str> {
    chain
        .iter()
        .find_map(|section| section.repository.as_deref())
}

fn first_targets<'a>(chain: &[&'a ProfileSection]) -> Option<&'a Vec<String>> {
    chain.iter().find_map(|section| {
        section
            .backup
            .as_ref()
            .and_then(|backup| backup.source.as_ref())
    })
}

fn fallback_repository<'a>(config: &'a ResticProfileConfig, profile_name: &str) -> Option<&'a str> {
    let chain = inheritance_chain(config, profile_name)?;
    first_repository(&chain)
}

fn fallback_targets<'a>(
    config: &'a ResticProfileConfig,
    profile_name: &str,
) -> Option<&'a Vec<String>> {
    let chain = inheritance_chain(config, profile_name)?;
    first_targets(&chain)
}

fn resolve_all_runnable(
    config: &ResticProfileConfig,
    command: &str,
) -> Result<Vec<ResolvedProfile>> {
    let (resolved_profiles, failures) = resolve_all_runnable_with_failures(config, command);
    if let Some(failure) = failures.first() {
        anyhow::bail!("{failure}");
    }
    Ok(resolved_profiles)
}

fn resolve_all_runnable_with_failures(
    config: &ResticProfileConfig,
    command: &str,
) -> (Vec<ResolvedProfile>, Vec<String>) {
    let database_profile = config
        .application
        .as_ref()
        .and_then(|application| application.database.as_ref())
        .map(|database| database.profile.as_str());

    let mut resolved_profiles = Vec::new();
    let mut failures = Vec::new();
    for name in config.profile_names() {
        match ProfileResolver::resolve_exact(config, &name, command) {
            Ok(profile)
                if !profile.targets.is_empty()
                    || Some(profile.name.as_str()) == database_profile =>
            {
                resolved_profiles.push(profile);
            }
            Ok(_) => {}
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    (resolved_profiles, failures)
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

    #[test]
    fn profile_resolver_walks_the_full_inheritance_chain_before_fallback() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "root".to_string(),
            ProfileSection {
                repository: Some("s3:https://root.example/backup".to_string()),
                backup: Some(crate::config::model::BackupCommandSection {
                    source: Some(vec!["/srv/data".to_string()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        profiles.insert(
            "middle".to_string(),
            ProfileSection {
                inherit: Some("root".to_string()),
                ..Default::default()
            },
        );
        profiles.insert(
            "child".to_string(),
            ProfileSection {
                inherit: Some("middle".to_string()),
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

        let resolved = ProfileResolver::resolve(&config, "child").unwrap();
        assert_eq!(resolved.repository, "s3:https://root.example/backup");
        assert_eq!(resolved.backend, "s3");
        assert_eq!(resolved.targets, vec!["/srv/data"]);
    }

    #[test]
    fn profile_resolver_keeps_profile_selectors_exact() {
        let mut profiles = BTreeMap::new();
        profiles.insert("default".to_string(), ProfileSection::default());
        profiles.insert("Unicode-프로필".to_string(), ProfileSection::default());
        let config = ResticProfileConfig {
            version: "2".to_string(),
            application: None,
            global: None,
            groups: None,
            profiles,
        };

        assert!(ProfileResolver::resolve(&config, "Unicode-프로필").is_some());
        assert!(ProfileResolver::resolve(&config, " unicode-프로필").is_none());
        assert!(ProfileResolver::resolve(&config, "unicode-프로필").is_none());
    }

    #[test]
    fn command_specific_all_resolution_excludes_profiles_without_effective_backup_targets() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "default".to_string(),
            ProfileSection {
                repository: Some("/repositories/default".to_string()),
                ..Default::default()
            },
        );
        profiles.insert(
            "archive-only".to_string(),
            ProfileSection {
                repository: Some("/repositories/archive".to_string()),
                ..Default::default()
            },
        );
        profiles.insert(
            "active".to_string(),
            ProfileSection {
                inherit: Some("default".to_string()),
                backup: Some(crate::config::model::BackupCommandSection {
                    source: Some(vec!["/srv/data".to_string()]),
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

        let run_profiles = ProfileResolver::resolve_for_run(&config, None).unwrap();
        let active_profiles = ProfileResolver::resolve_all_active(&config).unwrap();
        let status_profiles = ProfileResolver::resolve_for_status(&config, None).unwrap();
        assert_eq!(
            run_profiles
                .iter()
                .map(|profile| profile.name.as_str())
                .collect::<Vec<_>>(),
            vec!["active"]
        );
        assert_eq!(
            active_profiles
                .iter()
                .map(|profile| profile.name.as_str())
                .collect::<Vec<_>>(),
            vec!["active"]
        );
        assert_eq!(
            status_profiles
                .iter()
                .map(|profile| profile.name.as_str())
                .collect::<Vec<_>>(),
            vec!["active"]
        );
        assert!(ProfileResolver::resolve_exact(&config, " active", "run").is_err());
        assert!(ProfileResolver::resolve_exact(&config, "missing", "status").is_err());
    }
}

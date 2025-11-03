use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime};

use git2::{BranchType, Oid, Repository, Sort};
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

use crate::muxi::path;

use super::{PluginChange, PluginOptions, PluginStatus, PluginUpdateStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plugin {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<Url>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(skip)]
    pub name: String,
    #[serde(default)]
    pub options: PluginOptions,
}

impl Plugin {
    fn new_remote(url: Url) -> Self {
        let name = extract_repo_name(&url);
        Self {
            url: Some(url),
            path: None,
            name,
            options: PluginOptions::default(),
        }
    }

    fn new_local(path: PathBuf, url: Option<Url>) -> Self {
        let expanded = path::expand_tilde(path);
        let name = expanded
            .file_name()
            .and_then(|s| s.to_str())
            .map_or_else(|| expanded.display().to_string(), ToString::to_string);

        Self {
            url,
            path: Some(expanded),
            name,
            options: PluginOptions::default(),
        }
    }

    /// Parse a plugin from either "owner/repo" format or a full URL
    pub fn parse(s: &str) -> Result<Self, url::ParseError> {
        // Try parsing as a full URL first
        if let Ok(url) = Url::parse(s) {
            return Ok(Self::new_remote(url));
        }

        // If it contains a slash and doesn't look like a URL, treat it as owner/repo
        if s.contains('/') && !s.contains("://") {
            let github_url = format!("https://github.com/{s}");
            let url = Url::parse(&github_url)?;
            return Ok(Self::new_remote(url));
        }

        // Try parsing as URL one more time to get a proper error
        Url::parse(s).map(Self::new_remote)
    }

    pub fn status(&self) -> Result<PluginStatus> {
        if let Some(path) = &self.path {
            let exists = path.exists();
            return Ok(PluginStatus::Local {
                exists,
                path: display_path(path),
            });
        }

        if !self.is_installed() {
            return Ok(PluginStatus::Remote {
                installed: false,
                commit: None,
            });
        }

        let repo = Repository::open(self.install_path())
            .map_err(|e| miette::miette!("Failed to open repository: {}", e.dimmed()))?;

        let commit = repo
            .head()
            .and_then(|h| h.peel_to_commit())
            .map_err(|e| miette::miette!("Failed to read commit: {}", e.dimmed()))?;

        Ok(PluginStatus::Remote {
            installed: true,
            commit: Some(short_id(commit.id())),
        })
    }

    /// Check if plugin is installed
    pub fn is_installed(&self) -> bool {
        self.install_path().exists()
    }

    /// Sources the plugin
    pub fn source(&self) -> Result<()> {
        let root = self.install_path();

        if !root.exists() {
            return Err(miette::miette!(
                "Plugin path {} does not exist",
                display_path(&root)
            ));
        }

        self.apply_options()?;

        let entries = std::fs::read_dir(&root).into_diagnostic()?;

        for entry in entries.flatten() {
            let path = entry.path();

            // Execute all "*.tmux" files in the plugin directory
            if path.extension().is_some_and(|ext| ext == "tmux") {
                let status = Command::new("tmux")
                    .arg("run")
                    .arg("-b")
                    .arg(&path)
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .into_diagnostic()?;

                if !status.success() {
                    return Err(miette::miette!(
                        "Failed to execute {}: script exited with {}",
                        path.display(),
                        status
                    ));
                }
            }
        }

        Ok(())
    }

    fn apply_options(&self) -> Result<()> {
        for (key, value) in &self.options {
            let status = Command::new("tmux")
                .arg("set")
                .arg("-g")
                .arg(format!("@{key}"))
                .arg(value)
                .status()
                .into_diagnostic()?;

            if !status.success() {
                return Err(miette::miette!(
                    "Failed to configure option @{key} for {}",
                    self.name
                ));
            }
        }

        Ok(())
    }

    /// Install this plugin to the plugins directory
    pub fn install(&self) -> Result<bool> {
        if let Some(path) = &self.path {
            if path.exists() {
                return Ok(false);
            }

            return Err(miette::miette!(
                "Local plugin path {} does not exist",
                display_path(path)
            ));
        }

        if self.is_installed() {
            return Ok(false);
        }

        self.create_plugins_dir()?;

        let url = self
            .url
            .as_ref()
            .expect("remote plugin must have url")
            .as_str()
            .to_string();
        let target = self.install_path();
        Repository::clone(&url, target)
            .map_err(|e| miette::miette!("Failed to clone repository: {}", e.dimmed()))?;

        Ok(true)
    }

    /// Update this plugin to the latest commit on the default branch
    pub fn update(&self) -> Result<PluginUpdateStatus> {
        if let Some(path) = &self.path {
            ensure_exists(path)?;
            return Ok(PluginUpdateStatus::Local {
                path: display_path(path),
            });
        }

        let freshly_installed = if self.is_installed() {
            false
        } else {
            self.install()?;
            true
        };

        let repo = Repository::open(self.install_path())
            .map_err(|e| miette::miette!("Failed to open repository: {}", e.dimmed()))?;

        let head = repo
            .head()
            .map_err(|e| miette::miette!("Failed to get HEAD: {}", e.dimmed()))?;

        let commit_base_url = self.commit_base_url();

        if !head.is_branch() {
            let commit = head
                .peel_to_commit()
                .map_err(|e| miette::miette!("Failed to read commit: {}", e.dimmed()))?;

            return Ok(PluginUpdateStatus::Updated {
                from: None,
                to: short_id(commit.id()),
                changes: Vec::new(),
                range_url: None,
            });
        }

        let branch_name = head
            .shorthand()
            .ok_or_else(|| miette::miette!("Could not determine current branch"))?
            .to_string();

        let local_commit = head
            .peel_to_commit()
            .map_err(|e| miette::miette!("Failed to read commit: {}", e.dimmed()))?
            .id();

        if freshly_installed {
            return Ok(PluginUpdateStatus::Updated {
                from: None,
                to: short_id(local_commit),
                changes: Vec::new(),
                range_url: None,
            });
        }

        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| miette::miette!("Failed to find remote 'origin': {}", e.dimmed()))?;
        let remote_name = remote.name().unwrap_or("origin").to_string();

        remote
            .fetch::<&str>(&[], None, None)
            .map_err(|e| miette::miette!("Failed to fetch from remote: {}", e.dimmed()))?;

        let branch = repo
            .find_branch(&branch_name, BranchType::Local)
            .map_err(|e| miette::miette!("Failed to open branch {branch_name}: {}", e.dimmed()))?;
        let branch_ref = branch
            .get()
            .name()
            .map_or_else(|| format!("refs/heads/{branch_name}"), ToString::to_string);

        let upstream_commit = match branch.upstream() {
            Ok(upstream) => upstream
                .get()
                .peel_to_commit()
                .map_err(|e| miette::miette!("Failed to read upstream commit: {}", e.dimmed()))?
                .id(),
            Err(_) => repo
                .find_reference(&format!("refs/remotes/{remote_name}/{branch_name}"))
                .and_then(|r| r.peel_to_commit())
                .map_err(|e| miette::miette!("Failed to read remote commit: {}", e.dimmed()))?
                .id(),
        };

        if upstream_commit == local_commit {
            return Ok(PluginUpdateStatus::UpToDate {
                commit: short_id(local_commit),
            });
        }

        let changes = collect_changes(
            &repo,
            Some(local_commit),
            upstream_commit,
            commit_base_url.as_deref(),
        )?;

        let mut reference = branch.into_reference();
        reference
            .set_target(upstream_commit, "fast-forward")
            .map_err(|e| miette::miette!("Failed to update branch: {}", e.dimmed()))?;

        repo.set_head(&branch_ref)
            .map_err(|e| miette::miette!("Failed to set HEAD: {}", e.dimmed()))?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| miette::miette!("Failed to checkout: {}", e.dimmed()))?;

        Ok(PluginUpdateStatus::Updated {
            from: Some(short_id(local_commit)),
            to: short_id(upstream_commit),
            changes,
            range_url: self.compare_url(local_commit, upstream_commit),
        })
    }

    /// Get the installation path for this plugin
    fn install_path(&self) -> PathBuf {
        if let Some(path) = &self.path {
            path.clone()
        } else {
            path::plugins_dir().join(&self.name)
        }
    }

    fn commit_base_url(&self) -> Option<String> {
        let mut base = self.url.as_ref()?.to_string();

        if base.len() >= 4 && base[base.len() - 4..].eq_ignore_ascii_case(".git") {
            base.truncate(base.len() - 4);
        }

        if !base.ends_with('/') {
            base.push('/');
        }

        base.push_str("commit/");
        Some(base)
    }

    fn compare_url(&self, from: Oid, to: Oid) -> Option<String> {
        let url = self.url.as_ref()?;
        let host = url.host_str()?.to_ascii_lowercase();

        let mut base = url.to_string();
        if base.len() >= 4 && base[base.len() - 4..].eq_ignore_ascii_case(".git") {
            base.truncate(base.len() - 4);
        }
        if !base.ends_with('/') {
            base.push('/');
        }

        let suffix = match host.as_str() {
            "github.com" | "www.github.com" => "compare/",
            "gitlab.com" | "www.gitlab.com" => "-/compare/",
            _ => return None,
        };

        base.push_str(suffix);
        Some(format!("{base}{from}...{to}"))
    }

    fn create_plugins_dir(&self) -> Result<()> {
        if self.path.is_some() {
            return Ok(());
        }

        let plugins_dir = path::plugins_dir();

        std::fs::create_dir_all(&plugins_dir).map_err(|e| {
            miette::miette!(
                "Failed to create plugins directory at {}: {}",
                plugins_dir.display(),
                e
            )
        })?;

        Ok(())
    }

    fn with_options(mut self, options: PluginOptions) -> Self {
        self.options = options;
        self
    }
}

impl<'de> Deserialize<'de> for Plugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PluginVisitor;

        impl<'de> serde::de::Visitor<'de> for PluginVisitor {
            type Value = Plugin;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a plugin string or table")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Plugin::parse(value).map_err(serde::de::Error::custom)
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut url: Option<String> = None;
                let mut path_value: Option<PathBuf> = None;
                let mut options: Option<PluginOptions> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "url" => {
                            if url.is_some() {
                                return Err(serde::de::Error::duplicate_field("url"));
                            }
                            url = Some(map.next_value()?);
                        }
                        "path" => {
                            if path_value.is_some() {
                                return Err(serde::de::Error::duplicate_field("path"));
                            }
                            path_value = Some(map.next_value()?);
                        }
                        "opts" => {
                            if options.is_some() {
                                return Err(serde::de::Error::duplicate_field("opts"));
                            }
                            options = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let options = options.unwrap_or_default();

                if let Some(path) = path_value {
                    let url_value = match url {
                        Some(u) => Some(
                            Plugin::parse(&u)
                                .map_err(serde::de::Error::custom)?
                                .url
                                .unwrap(),
                        ),
                        None => None,
                    };

                    return Ok(Plugin::new_local(path, url_value).with_options(options));
                }

                if let Some(url_string) = url {
                    return Plugin::parse(&url_string)
                        .map_err(serde::de::Error::custom)
                        .map(|plugin| plugin.with_options(options));
                }

                Err(serde::de::Error::custom(
                    "plugin table must include either `url` or `path`",
                ))
            }
        }

        deserializer.deserialize_any(PluginVisitor)
    }
}

impl Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}", path.display())
        } else if let Some(url) = &self.url {
            write!(f, "{url}")
        } else {
            write!(f, "unknown")
        }
    }
}

fn extract_repo_name(url: &Url) -> String {
    url.path_segments()
        .and_then(std::iter::Iterator::last)
        .unwrap_or_else(|| url.as_str())
        .trim_end_matches(".git")
        .to_string()
}

fn short_id(oid: Oid) -> String {
    let id = oid.to_string();
    id.chars().take(7).collect()
}

fn display_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(stripped) = path.strip_prefix(&home)
    {
        return format!("~/{}", stripped.display());
    }

    path.display().to_string()
}

fn ensure_exists(path: &Path) -> Result<()> {
    if path.exists() {
        Ok(())
    } else {
        Err(miette::miette!(
            "Plugin path {} does not exist",
            display_path(path)
        ))
    }
}

fn collect_changes(
    repo: &Repository,
    from: Option<Oid>,
    to: Oid,
    commit_base_url: Option<&str>,
) -> Result<Vec<PluginChange>> {
    let Some(from) = from else {
        return Ok(Vec::new());
    };

    if from == to {
        return Ok(Vec::new());
    }

    let mut revwalk = repo
        .revwalk()
        .map_err(|e| miette::miette!("Failed to create revwalk: {}", e.dimmed()))?;
    revwalk
        .set_sorting(Sort::TOPOLOGICAL | Sort::TIME)
        .map_err(|e| miette::miette!("Failed to configure revwalk: {}", e.dimmed()))?;
    revwalk
        .push(to)
        .map_err(|e| miette::miette!("Failed to add target commit: {}", e.dimmed()))?;
    revwalk
        .hide(from)
        .map_err(|e| miette::miette!("Failed to hide previous commit: {}", e.dimmed()))?;

    let mut changes = Vec::new();

    for oid in revwalk {
        let oid = oid.map_err(|e| miette::miette!("Failed to walk commits: {}", e.dimmed()))?;
        let commit = repo
            .find_commit(oid)
            .map_err(|e| miette::miette!("Failed to read commit {oid}: {}", e.dimmed()))?;

        let summary = commit
            .summary()
            .map_or_else(|| "(no commit message)".to_string(), ToString::to_string);

        let full_id = commit.id().to_string();
        let short = short_id(commit.id());
        let url = commit_base_url.map(|base| format!("{base}{full_id}"));

        changes.push(PluginChange {
            id: short,
            full_id,
            summary,
            time: commit_time(&commit),
            url,
        });
    }

    Ok(changes)
}

fn commit_time(commit: &git2::Commit<'_>) -> SystemTime {
    let seconds = commit.time().seconds();

    if seconds >= 0 {
        let secs = u64::try_from(seconds).unwrap_or(0);
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    } else {
        let seconds = u64::try_from(seconds.saturating_neg()).unwrap_or(0);
        SystemTime::UNIX_EPOCH
            .checked_sub(Duration::from_secs(seconds))
            .unwrap_or(SystemTime::UNIX_EPOCH)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Oid;
    use mlua::{Lua, LuaSerdeExt, Value as LuaValue};

    #[test]
    fn test_plugin_parse_short_form() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();
        assert_eq!(
            plugin.url.unwrap().as_str(),
            "https://github.com/tmux-plugins/tmux-continuum"
        );
    }

    #[test]
    fn test_plugin_parse_full_url() {
        let plugin = Plugin::parse("https://github.com/tmux-plugins/tmux-continuum").unwrap();
        assert_eq!(
            plugin.url.unwrap().as_str(),
            "https://github.com/tmux-plugins/tmux-continuum"
        );
    }

    #[test]
    fn test_plugin_parse_custom_git_url() {
        let plugin = Plugin::parse("https://gitlab.com/user/repo").unwrap();
        assert_eq!(plugin.url.unwrap().as_str(), "https://gitlab.com/user/repo");
    }

    #[test]
    fn test_plugin_repo_name() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();
        assert_eq!(plugin.name, "tmux-continuum");
    }

    #[test]
    fn test_plugin_repo_name_with_git_suffix() {
        let plugin = Plugin::parse("https://github.com/user/repo.git").unwrap();
        assert_eq!(plugin.name, "repo");
    }

    #[test]
    fn test_plugin_parse_with_options() {
        let lua = Lua::new();
        let value = lua
            .load(
                r#"
                return {
                    url = "tmux-plugins/tmux-yank",
                    opts = {
                        copy_mode_put = "Space",
                        yank_selection_mouse = "clipboard",
                    },
                }
                "#,
            )
            .eval::<LuaValue>()
            .unwrap();
        let plugin: Plugin = lua.from_value(value).unwrap();

        assert_eq!(plugin.name, "tmux-yank");

        let expected: PluginOptions = [
            ("copy_mode_put".to_string(), "Space".to_string()),
            ("yank_selection_mouse".to_string(), "clipboard".to_string()),
        ]
        .into_iter()
        .collect();

        assert_eq!(plugin.options, expected);
    }

    #[test]
    fn test_plugin_install_path() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();

        assert_eq!(
            plugin.install_path(),
            path::plugins_dir().join("tmux-continuum")
        );
    }

    #[test]
    fn test_plugin_local_path_only() {
        let plugin = Plugin::new_local(PathBuf::from("~/dev/my-plugin"), None);
        assert!(plugin.url.is_none());
        assert!(plugin.path.is_some());
        assert_eq!(plugin.name, "my-plugin");
    }

    #[test]
    fn test_install_local_plugin_missing_path_errors() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        let plugin = Plugin::new_local(temp.clone(), None);

        if temp.exists() {
            std::fs::remove_dir_all(&temp).unwrap();
        }

        let error = plugin.install().expect_err("expected install to fail");
        assert!(format!("{error}").contains("does not exist"));
    }

    #[test]
    fn test_install_local_plugin_existing_path_is_noop() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();

        let plugin = Plugin::new_local(temp.clone(), None);
        let result = plugin.install().unwrap();

        assert!(!result);
        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn test_update_local_plugin_missing_path_errors() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        let plugin = Plugin::new_local(temp.clone(), None);

        if temp.exists() {
            std::fs::remove_dir_all(&temp).unwrap();
        }

        let error = plugin.update().expect_err("expected update to fail");
        assert!(format!("{error}").contains("does not exist"));
    }

    #[test]
    fn test_update_local_plugin_existing_path_returns_status() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();

        let plugin = Plugin::new_local(temp.clone(), None);
        let status = plugin.update().unwrap();

        match status {
            PluginUpdateStatus::Local { path } => {
                assert!(path.contains(temp.file_name().unwrap().to_str().unwrap()));
            }
            other => panic!("unexpected status: {other:?}"),
        }

        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn test_compare_url_github() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();
        let from = Oid::from_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        let to = Oid::from_str("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();

        let url = plugin.compare_url(from, to).expect("expected github url");

        assert_eq!(
            url,
            "https://github.com/tmux-plugins/tmux-continuum/compare/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
    }

    #[test]
    fn test_compare_url_gitlab() {
        let plugin = Plugin::parse("https://gitlab.com/user/repo").unwrap();
        let from = Oid::from_str("1234567890abcdef1234567890abcdef12345678").unwrap();
        let to = Oid::from_str("87654321fedcba0987654321fedcba0987654321").unwrap();

        let url = plugin.compare_url(from, to).expect("expected gitlab url");

        assert_eq!(
            url,
            "https://gitlab.com/user/repo/-/compare/1234567890abcdef1234567890abcdef12345678...87654321fedcba0987654321fedcba0987654321"
        );
    }

    #[test]
    fn test_compare_url_unknown_host_returns_none() {
        let plugin = Plugin::parse("https://example.com/user/repo").unwrap();
        let from = Oid::from_str("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
        let to = Oid::from_str("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();

        assert!(plugin.compare_url(from, to).is_none());
    }
}

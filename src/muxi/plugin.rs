use std::fmt::Display;
use std::path::PathBuf;

use git2::{BranchType, Repository};
use miette::Result;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

use crate::muxi::path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Plugin {
    pub url: Url,
}

impl Plugin {
    /// Parse a plugin from either "owner/repo" format or a full URL
    pub fn parse(s: &str) -> Result<Self, url::ParseError> {
        // Try parsing as a full URL first
        if let Ok(url) = Url::parse(s) {
            return Ok(Plugin { url });
        }

        // If it contains a slash and doesn't look like a URL, treat it as owner/repo
        if s.contains('/') && !s.contains("://") {
            let github_url = format!("https://github.com/{s}");
            let url = Url::parse(&github_url)?;
            return Ok(Plugin { url });
        }

        // Try parsing as URL one more time to get a proper error
        Url::parse(s).map(|url| Plugin { url })
    }

    /// Extract the repository name from the URL
    pub fn repo_name(&self) -> &str {
        self.url
            .path_segments()
            .and_then(std::iter::Iterator::last)
            .unwrap_or(self.url.as_str())
            .trim_end_matches(".git")
    }

    /// Get the installation path for this plugin
    pub fn install_path(&self) -> PathBuf {
        path::plugins_dir().join(self.repo_name())
    }

    /// Check if this plugin is installed
    pub fn is_installed(&self) -> bool {
        self.install_path().exists()
    }

    /// Install this plugin to the plugins directory
    pub fn install(&self) -> Result<bool> {
        if self.is_installed() {
            return Ok(false);
        }

        Repository::clone(self.url.as_str(), self.install_path())
            .map_err(|e| miette::miette!("Failed to clone repository: {e}"))?;

        Ok(true)
    }

    /// Update this plugin to the latest commit on the default branch
    /// Returns true if updated, false if already up to date
    pub fn update(&self) -> Result<bool> {
        // If not installed, install it
        if !self.is_installed() {
            self.install()?;
            return Ok(true);
        }

        let repo = Repository::open(self.install_path())
            .map_err(|e| miette::miette!("Failed to open repository: {e}"))?;

        let head = repo
            .head()
            .map_err(|e| miette::miette!("Failed to get HEAD: {e}"))?;

        let old_commit = head
            .peel_to_commit()
            .map_err(|e| miette::miette!("Failed to get commit: {e}"))?;

        let mut remote = repo
            .find_remote("origin")
            .map_err(|e| miette::miette!("Failed to find remote 'origin': {e}"))?;

        remote
            .fetch(&["HEAD"], None, None)
            .map_err(|e| miette::miette!("Failed to fetch from remote: {e}"))?;

        // Get the default branch name
        let default_branch = repo
            .find_branch("origin/HEAD", BranchType::Remote)
            .and_then(|branch| {
                branch
                    .get()
                    .symbolic_target()
                    .and_then(|target| target.strip_prefix("refs/remotes/origin/"))
                    .map(String::from)
                    .ok_or_else(|| git2::Error::from_str("Could not determine default branch"))
            })
            .or_else(|_| {
                // Fallback to common default branches
                for branch_name in ["main", "master"] {
                    if repo
                        .find_branch(&format!("origin/{branch_name}"), BranchType::Remote)
                        .is_ok()
                    {
                        return Ok(branch_name.to_string());
                    }
                }
                Err(git2::Error::from_str("Could not find default branch"))
            })
            .map_err(|e| miette::miette!("Failed to determine default branch: {e}"))?;

        // Get the latest commit from the default branch
        let remote_branch = repo
            .find_branch(&format!("origin/{default_branch}"), BranchType::Remote)
            .map_err(|e| miette::miette!("Failed to find remote branch: {e}"))?;

        let remote_commit = remote_branch
            .get()
            .peel_to_commit()
            .map_err(|e| miette::miette!("Failed to get remote commit: {e}"))?;

        // Check if already up to date
        if old_commit.id() == remote_commit.id() {
            return Ok(false);
        }

        // Update to the latest commit
        repo.set_head_detached(remote_commit.id())
            .map_err(|e| miette::miette!("Failed to update HEAD: {e}"))?;

        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
            .map_err(|e| miette::miette!("Failed to checkout: {e}"))?;

        Ok(true)
    }
}

impl<'de> Deserialize<'de> for Plugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Plugin::parse(&s).map_err(serde::de::Error::custom)
    }
}

impl Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_parse_short_form() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();
        assert_eq!(
            plugin.url.as_str(),
            "https://github.com/tmux-plugins/tmux-continuum"
        );
    }

    #[test]
    fn test_plugin_parse_full_url() {
        let plugin = Plugin::parse("https://github.com/tmux-plugins/tmux-continuum").unwrap();
        assert_eq!(
            plugin.url.as_str(),
            "https://github.com/tmux-plugins/tmux-continuum"
        );
    }

    #[test]
    fn test_plugin_parse_custom_git_url() {
        let plugin = Plugin::parse("https://gitlab.com/user/repo").unwrap();
        assert_eq!(plugin.url.as_str(), "https://gitlab.com/user/repo");
    }

    #[test]
    fn test_plugin_repo_name() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();
        assert_eq!(plugin.repo_name(), "tmux-continuum");
    }

    #[test]
    fn test_plugin_repo_name_with_git_suffix() {
        let plugin = Plugin::parse("https://github.com/user/repo.git").unwrap();
        assert_eq!(plugin.repo_name(), "repo");
    }

    #[test]
    fn test_plugin_install_path() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();

        assert_eq!(
            plugin.install_path(),
            path::plugins_dir().join("tmux-continuum")
        );
    }
}

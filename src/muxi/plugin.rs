use std::fmt::Display;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

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
            .unwrap_or("plugin")
            .trim_end_matches(".git")
    }

    /// Get the installation path for this plugin
    pub fn install_path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.repo_name())
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
        let base = PathBuf::from("/some/path");
        assert_eq!(
            plugin.install_path(&base),
            PathBuf::from("/some/path/tmux-continuum")
        );
    }
}

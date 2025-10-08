use std::fmt::Display;

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
}

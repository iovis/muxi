use std::fmt::Display;
use std::path::PathBuf;

use miette::Result;
use serde::de::IgnoredAny;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

use crate::muxi::path;

use super::PluginOptions;

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

    pub(super) fn new_local(path: PathBuf, url: Option<Url>) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_plugin_local_path_only() {
        let plugin = Plugin::new_local(PathBuf::from("~/dev/my-plugin"), None);
        assert!(plugin.url.is_none());
        assert!(plugin.path.is_some());
        assert_eq!(plugin.name, "my-plugin");
    }
}

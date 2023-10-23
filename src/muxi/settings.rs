use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::Path;

use config::{Config, ConfigError, File};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::tmux::{self, Key, Popup};

type Bindings = BTreeMap<Key, Binding>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Settings {
    pub muxi_prefix: Key,
    pub tmux_prefix: bool,
    pub uppercase_overrides: bool,
    #[serde(default)]
    pub bindings: Bindings,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Binding {
    // TODO: `prefix: bool` or `table: String`?
    pub command: String,
    #[serde(default)]
    pub popup: Option<Popup>,
}

impl Settings {
    pub fn new(path: &Path, tmux_settings: tmux::Settings) -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("muxi_prefix", "g")?
            .set_default("tmux_prefix", true)?
            .set_default("uppercase_overrides", false)?
            .add_source(File::from(path).required(false))
            .set_override_option("muxi_prefix", tmux_settings.muxi_prefix)?
            .set_override_option("tmux_prefix", tmux_settings.tmux_prefix)?
            .set_override_option("uppercase_overrides", tmux_settings.uppercase_overrides)?
            .build()?
            .try_deserialize()
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "    {} {}", "muxi_prefix:".green(), self.muxi_prefix)?;
        writeln!(f, "    {} {}", "tmux_prefix:".green(), self.tmux_prefix)?;
        writeln!(
            f,
            "    {} {}",
            "uppercase_overrides:".green(),
            self.uppercase_overrides
        )
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            muxi_prefix: Key::parse("g").unwrap(),
            tmux_prefix: true,
            uppercase_overrides: false,
            bindings: BTreeMap::default(),
        }
    }
}

pub struct SettingsBuilder {
    settings: Settings,
}

impl SettingsBuilder {
    pub fn new() -> Self {
        Self {
            settings: Settings::default(),
        }
    }

    pub fn set(mut self, settings: Settings) -> Self {
        self.settings = settings;
        self
    }

    pub fn merge_tmux_settings(mut self, tmux_settings: &tmux::Settings) -> Self {
        if let Some(muxi_prefix) = &tmux_settings.muxi_prefix {
            self.settings.muxi_prefix = Key::parse(muxi_prefix).unwrap();
        }

        if let Some(tmux_prefix) = &tmux_settings.tmux_prefix {
            self.settings.tmux_prefix = *tmux_prefix;
        }

        if let Some(uppercase_overrides) = &tmux_settings.uppercase_overrides {
            self.settings.uppercase_overrides = *uppercase_overrides;
        }

        self
    }

    pub fn build(self) -> Settings {
        self.settings
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::io::Write;

    use uuid::Uuid;

    use super::*;

    fn with_config<F>(config: &str, test: F)
    where
        F: Fn(Settings) + std::panic::RefUnwindSafe,
    {
        // Create tmp folder
        let pwd = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&pwd).unwrap();

        // Create settings file
        let path = pwd.join("settings.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(config.as_bytes()).unwrap();

        // Set $MUXI_CONFIG_PATH to current folder and load config
        temp_env::with_var("MUXI_CONFIG_PATH", Some(pwd.clone()), || {
            let settings = Settings::new(&path, tmux::Settings::default());

            // Cleanup before test, in case of panic
            std::fs::remove_dir_all(&pwd).unwrap();

            test(settings.unwrap());
        });
    }

    #[test]
    fn test_parse_valid_muxi_prefix() {
        let config = r#"
        "#;

        let expected_settings = Settings::default();

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_valid_muxi_prefix_root() {
        let config = r#"
            muxi_prefix = "M-Space"
            tmux_prefix = false
        "#;

        let expected_settings = Settings {
            tmux_prefix: false,
            muxi_prefix: "M-Space".try_into().unwrap(),
            uppercase_overrides: false,
            bindings: BTreeMap::new(),
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_no_popup() {
        let config = r#"
            muxi_prefix = "g"

            [bindings]
            j = { command = "tmux switch-client -l" }
        "#;

        let mut bindings: Bindings = BTreeMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "tmux switch-client -l".into(),
                popup: None,
            },
        );

        let expected_settings = Settings {
            bindings,
            ..Default::default()
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_default_height() {
        let config = r#"
            muxi_prefix = "g"

            [bindings]
            j = { popup = { width = "60%" }, command = "muxi sessions edit" }
        "#;

        let mut bindings: Bindings = BTreeMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Some(Popup {
                    title: None,
                    width: "60%".into(),
                    height: "75%".into(),
                }),
            },
        );

        let expected_settings = Settings {
            bindings,
            ..Default::default()
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_default_all() {
        let config = r#"
            muxi_prefix = "g"

            [bindings]
            j = { popup = {}, command = "muxi sessions edit" }
        "#;

        let mut bindings: Bindings = BTreeMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Some(Popup {
                    title: None,
                    width: "75%".into(),
                    height: "75%".into(),
                }),
            },
        );

        let expected_settings = Settings {
            bindings,
            ..Default::default()
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_struct() {
        let config = r#"
            muxi_prefix = "g"

            [bindings.j]
            popup = { title = "my title", width = "75%", height = "60%" }
            command = "muxi sessions edit"
        "#;

        let mut bindings: Bindings = BTreeMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Some(Popup {
                    title: Some("my title".into()),
                    width: "75%".into(),
                    height: "60%".into(),
                }),
            },
        );

        let expected_settings = Settings {
            bindings,
            ..Default::default()
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }
}

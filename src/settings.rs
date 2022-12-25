use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use config::{Config, ConfigError, File};
use serde::Deserialize;

use crate::tmux::TmuxKey;

type Bindings = HashMap<TmuxKey, Binding>;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub muxi_prefix: TmuxKey,
    pub tmux_prefix: bool,
    #[serde(default)]
    pub bindings: Bindings,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Binding {
    pub command: String,
    #[serde(default)]
    pub popup: Popup,
}

impl Binding {
    pub fn has_popup(&self) -> bool {
        matches!(self.popup, Popup::Bool(true) | Popup::Options { .. })
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Popup {
    Bool(bool),
    Options { width: String, height: String }, // TODO: Make Option, add title
}

impl Default for Popup {
    fn default() -> Self {
        Self::Bool(false)
    }
}

impl Settings {
    pub fn new(path: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .set_default("tmux_prefix", true)?
            .add_source(File::from(path).required(false))
            .build()?
            .try_deserialize()
    }
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "muxi_prefix: {}", self.muxi_prefix)?;
        writeln!(f, "tmux_prefix: {}", self.tmux_prefix)
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
            let settings = Settings::new(&path);

            // Cleanup before test, in case of panic
            std::fs::remove_dir_all(&pwd).unwrap();

            test(settings.unwrap());
        });
    }

    fn default_settings(bindings: Bindings) -> Settings {
        Settings {
            tmux_prefix: true,
            muxi_prefix: "g".try_into().unwrap(),
            bindings,
        }
    }

    #[test]
    fn test_parse_valid_muxi_prefix() {
        let config = r#"
            muxi_prefix = "g"
        "#;

        let expected_settings = default_settings(HashMap::new());

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
            bindings: HashMap::new(),
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_normal() {
        // Create configuration file
        let config = r#"
            muxi_prefix = "g"

            [bindings]
            j = { command = "tmux switch-client -l" }
        "#;

        let mut bindings: Bindings = HashMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "tmux switch-client -l".into(),
                popup: Popup::Bool(false),
            },
        );

        let expected_settings = default_settings(bindings);

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_boolean() {
        // Create configuration file
        let config = r#"
            muxi_prefix = "g"

            [bindings]
            j = { popup = true, command = "muxi sessions edit" }
        "#;

        let mut bindings: Bindings = HashMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Popup::Bool(true),
            },
        );

        let expected_settings = default_settings(bindings);

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_struct() {
        // Create configuration file
        let config = r#"
            muxi_prefix = "g"

            [bindings.j]
            popup = { width = "75%", height = "60%"}
            command = "muxi sessions edit"
        "#;

        let mut bindings: Bindings = HashMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Popup::Options {
                    width: "75%".into(),
                    height: "60%".into(),
                },
            },
        );

        let expected_settings = default_settings(bindings);

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }
}

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use config::{Config, ConfigError, File};
use owo_colors::OwoColorize;
use serde::Deserialize;

use crate::tmux::{PopupOptions, TmuxKey};

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
    pub popup: Option<PopupOptions>,
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
        writeln!(f, "    {} {}", "muxi_prefix:".green(), self.muxi_prefix)?;
        writeln!(f, "    {} {}", "tmux_prefix:".green(), self.tmux_prefix)
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
    fn test_parse_binding_no_popup() {
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
                popup: None,
            },
        );

        let expected_settings = default_settings(bindings);

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

        let mut bindings: Bindings = HashMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Some(PopupOptions {
                    title: None,
                    width: "60%".into(),
                    height: "75%".into(),
                }),
            },
        );

        let expected_settings = default_settings(bindings);

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

        let mut bindings: Bindings = HashMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Some(PopupOptions {
                    title: None,
                    width: "75%".into(),
                    height: "75%".into(),
                }),
            },
        );

        let expected_settings = default_settings(bindings);

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

        let mut bindings: Bindings = HashMap::new();
        bindings.insert(
            "j".try_into().unwrap(),
            Binding {
                command: "muxi sessions edit".into(),
                popup: Some(PopupOptions {
                    title: Some("my title".into()),
                    width: "75%".into(),
                    height: "60%".into(),
                }),
            },
        );

        let expected_settings = default_settings(bindings);

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }
}

use std::collections::HashMap;
use std::fmt::Display;
use std::path::Path;

use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub muxi_prefix: String, // TODO: validate no spaces
    pub tmux_prefix: bool,
    #[serde(default)]
    pub bindings: HashMap<String, Binding>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Binding {
    pub command: String,
    #[serde(default)]
    pub popup: bool,
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
    use std::io::Write;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn test_parse_valid_binding_prefix() {
        // Create configuration file
        let config = r#"
            muxi_prefix = "g"
        "#;

        let expected_settings = Settings {
            tmux_prefix: true,
            muxi_prefix: "g".into(),
            bindings: HashMap::new(),
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_valid_binding_root() {
        // Create configuration file
        let config = r#"
            muxi_prefix = "M-Space"
            tmux_prefix = false
        "#;

        let expected_settings = Settings {
            tmux_prefix: false,
            muxi_prefix: "M-Space".into(),
            bindings: HashMap::new(),
        };

        with_config(config, |settings| {
            assert_eq!(settings, expected_settings);
        });
    }

    fn with_config<F>(config: &str, test: F)
    where
        F: Fn(Settings) + std::panic::RefUnwindSafe,
    {
        let settings_file = "settings.toml";
        let mut file = std::fs::File::create(settings_file).unwrap();
        file.write_all(config.as_bytes()).unwrap();

        // Set $MUXI_CONFIG_PATH to current folder and load config
        let pwd = std::env::var("PWD").unwrap();
        let path = PathBuf::from(&pwd).join(settings_file);

        temp_env::with_var("MUXI_CONFIG_PATH", Some(pwd), || {
            let settings = Settings::new(&path).unwrap();

            // Cleanup before test, in case of panic
            std::fs::remove_file(settings_file).unwrap();

            test(settings);
        });
    }
}

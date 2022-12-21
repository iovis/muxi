use std::fmt::Display;

use config::{Config, ConfigError, File};
use serde::Deserialize;

use crate::path::muxi_path;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub muxi_prefix: String, // TODO: validate no spaces
    pub tmux_prefix: bool,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let path = muxi_path().join("settings");

        Config::builder()
            .set_default("muxi_prefix", "g".to_string())?
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

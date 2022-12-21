use std::fmt::Display;
use std::path::Path;

use config::{Config, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub muxi_prefix: String, // TODO: validate no spaces
    pub tmux_prefix: bool,
}

impl Settings {
    pub fn new(path: &Path) -> Result<Self, ConfigError> {
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

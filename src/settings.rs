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

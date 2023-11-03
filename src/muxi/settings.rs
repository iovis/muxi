use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::Path;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::tmux::{self, Key, Popup};

use super::lua;

#[allow(clippy::module_name_repetitions)]
pub fn parse_settings(path: &Path) -> color_eyre::Result<Settings> {
    let mut settings_builder = SettingsBuilder::new();

    match lua::parse_settings(path, &settings_builder.settings) {
        Ok(settings) => {
            settings_builder = settings_builder.set(settings);
        }
        Err(lua::Error::NotFound(_)) => (),
        Err(error) => return Err(error)?,
    };

    let tmux_settings = tmux::Settings::new();
    Ok(settings_builder.merge_tmux_settings(&tmux_settings).build())
}

pub type Bindings = BTreeMap<Key, Binding>;

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

#[allow(clippy::module_name_repetitions)]
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

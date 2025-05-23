use std::collections::BTreeMap;
use std::fmt::Display;
use std::path::Path;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::tmux::{Key, Popup};

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
    }

    Ok(settings_builder.build())
}

pub type Bindings = BTreeMap<Key, Binding>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Binding {
    // TODO: `prefix: bool` or `table: String`?
    pub command: String,
    #[serde(default)]
    pub popup: Option<Popup>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
pub struct EditorSettings {
    pub args: Vec<String>,
    pub command: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct FzfSettings {
    pub args: Vec<String>,
    pub bind_sessions: bool,
    pub input: bool,
}

impl Default for FzfSettings {
    fn default() -> Self {
        Self {
            input: true,
            bind_sessions: false,
            args: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Settings {
    pub muxi_prefix: Key,
    pub tmux_prefix: bool,
    pub uppercase_overrides: bool,
    pub use_current_pane_path: bool,
    pub editor: EditorSettings,
    pub fzf: FzfSettings,
    #[serde(default)]
    pub bindings: Bindings,
}

impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "    {} {}",
            "muxi_prefix:".green(),
            self.muxi_prefix.yellow()
        )?;

        writeln!(
            f,
            "    {} {}",
            "tmux_prefix:".green(),
            self.tmux_prefix.blue()
        )?;

        writeln!(
            f,
            "    {} {}",
            "uppercase_overrides:".green(),
            self.uppercase_overrides.blue()
        )?;

        writeln!(
            f,
            "    {} {}",
            "use_current_pane_path:".green(),
            self.use_current_pane_path.blue()
        )?;

        // Editor
        writeln!(f, "\n{}", "editor:".yellow())?;
        writeln!(
            f,
            "    {} {}",
            "command:".green(),
            self.editor
                .command
                .clone()
                .unwrap_or_else(|| "$EDITOR".to_string())
                .blue()
        )?;
        writeln!(
            f,
            "    {} {}",
            "args:".green(),
            self.editor.args.join(" ").blue()
        )?;

        // FZF
        writeln!(f, "\n{}", "fzf:".yellow())?;
        writeln!(f, "    {} {}", "input:".green(), self.fzf.input.blue())?;
        writeln!(
            f,
            "    {} {}",
            "bind_sessions:".green(),
            self.fzf.bind_sessions.blue()
        )?;
        writeln!(
            f,
            "    {} {}",
            "args:".green(),
            self.fzf.args.join(" ").blue()
        )?;

        // Bindings
        if !self.bindings.is_empty() {
            writeln!(f, "\n{}", "bindings:".yellow())?;

            let max_width_key = self
                .bindings
                .keys()
                .map(|key| key.as_ref().len())
                .max()
                .unwrap();

            for (key, binding) in &self.bindings {
                write!(
                    f,
                    "    {:<max_width_key$}  {}",
                    key.green(),
                    binding.command
                )?;

                if binding.popup.is_some() {
                    write!(f, "{}", " (popup)".yellow())?;
                }

                writeln!(f)?;
            }
        }

        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            muxi_prefix: Key::parse("g").unwrap(),
            tmux_prefix: true,
            uppercase_overrides: false,
            use_current_pane_path: false,
            editor: EditorSettings::default(),
            fzf: FzfSettings::default(),
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

    pub fn build(self) -> Settings {
        self.settings
    }
}

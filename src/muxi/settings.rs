use std::collections::BTreeMap;
use std::fmt::Display;

use miette::Result;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::tmux::{Key, Popup};

use super::{Plugin, lua};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Settings {
    pub muxi_prefix: Key,
    pub tmux_prefix: bool,
    pub uppercase_overrides: bool,
    pub use_current_pane_path: bool,
    pub plugins: Vec<Plugin>,
    pub editor: EditorSettings,
    pub fzf: FzfSettings,
    #[serde(default)]
    pub bindings: Bindings,
}

impl Settings {
    pub fn from_lua() -> Result<Settings> {
        let path = super::path::muxi_dir();
        let mut settings = Settings::default();

        match lua::parse_settings(&path, &settings) {
            Ok(user_settings) => settings = user_settings,
            Err(lua::Error::NotFound(_)) => (),
            Err(error) => return Err(error)?,
        }

        Ok(settings)
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            muxi_prefix: Key::new("g"),
            tmux_prefix: true,
            uppercase_overrides: true,
            use_current_pane_path: false,
            plugins: vec![],
            editor: EditorSettings::default(),
            fzf: FzfSettings::default(),
            bindings: BTreeMap::default(),
        }
    }
}

#[allow(clippy::too_many_lines)]
impl Display for Settings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {}",
            "muxi_prefix".dimmed(),
            self.muxi_prefix.bold().green()
        )?;
        writeln!(
            f,
            "{} {}",
            "tmux_prefix".dimmed(),
            self.tmux_prefix.bold().green()
        )?;
        writeln!(
            f,
            "{} {}",
            "uppercase_overrides".dimmed(),
            self.uppercase_overrides.bold().green()
        )?;

        writeln!(
            f,
            "{} {}",
            "use_current_pane_path".dimmed(),
            self.use_current_pane_path.bold().green()
        )?;

        // Plugins
        writeln!(f, "\n{}", "Plugins".bold().underline())?;
        if self.plugins.is_empty() {
            writeln!(f, "{}", "(none)".dimmed())?;
        } else {
            for plugin in &self.plugins {
                let source = plugin
                    .path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .or_else(|| plugin.url.as_ref().map(ToString::to_string))
                    .unwrap_or_else(|| "unknown".into());

                writeln!(f, "{} {}", plugin.name.bold().green(), source.dimmed())?;

                if !plugin.options.is_empty() {
                    writeln!(f, "{}", plugin.options)?;
                }
            }
        }

        // Editor
        writeln!(f, "\n{}", "Editor".bold().underline())?;
        writeln!(
            f,
            "{} {}",
            "command".dimmed(),
            self.editor
                .command
                .clone()
                .unwrap_or_else(|| format!(
                    "{} {}",
                    std::env::var("EDITOR")
                        .unwrap_or_else(|_| "vim".to_string())
                        .bold()
                        .green(),
                    "($EDITOR)".dimmed()
                ))
                .bold()
                .green()
        )?;
        writeln!(
            f,
            "{} {}",
            "args".dimmed(),
            self.editor.args.join(" ").bold().green()
        )?;

        // FZF
        writeln!(f, "\n{}", "FZF".bold().underline())?;
        writeln!(f, "{} {}", "input".dimmed(), self.fzf.input.bold().green())?;
        writeln!(
            f,
            "{} {}",
            "bind_sessions".dimmed(),
            self.fzf.bind_sessions.bold().green()
        )?;
        writeln!(
            f,
            "{} {}",
            "args".dimmed(),
            self.fzf.args.join(" ").bold().green()
        )?;

        // Bindings
        if !self.bindings.is_empty() {
            writeln!(f, "\n{}", "Bindings".bold().underline())?;

            let max_width_key = self
                .bindings
                .keys()
                .map(|key| key.as_ref().len())
                .max()
                .unwrap();

            for (key, binding) in &self.bindings {
                write!(
                    f,
                    "{:<max_width_key$} {}",
                    key.bold().green(),
                    binding.command.dimmed()
                )?;

                if binding.popup.is_some() {
                    write!(f, "{}", " (popup)".cyan())?;
                }

                writeln!(f)?;
            }
        }

        Ok(())
    }
}

pub type Bindings = BTreeMap<Key, Binding>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Binding {
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

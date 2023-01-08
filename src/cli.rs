use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

use crate::tmux::Key;

#[derive(Debug, Parser)]
#[command(name = "muxi")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
    #[arg(
        long,
        global = true,
        require_equals = true,
        value_name = "WHEN",
        num_args = 0..=1,
        default_value_t = ColorWhen::Auto,
        default_missing_value = "always",
        value_enum
    )]
    pub color: ColorWhen,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum ColorWhen {
    Always,
    Auto,
    Never,
}

impl ColorWhen {
    pub fn init(self) {
        // Set a supports-color override based on the variable passed in.
        match self {
            ColorWhen::Always => owo_colors::set_override(true),
            ColorWhen::Auto => {}
            ColorWhen::Never => owo_colors::set_override(false),
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Register within Tmux and add bindings
    Init,
    /// See and edit your settings
    Config(Config),
    /// See and manage your muxi sessions
    Sessions(Sessions),
    /// Generate completions for your shell
    Completions { shell: Shell },
}

#[derive(Debug, Args)]
pub struct Config {
    #[command(subcommand)]
    pub command: Option<ConfigCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Open your editor to change your settings
    Edit,
    /// See your current settings
    List,
}

#[derive(Debug, Args)]
pub struct Sessions {
    #[command(subcommand)]
    pub command: Option<SessionCommands>,
}

#[derive(Debug, Subcommand)]
pub enum SessionCommands {
    /// Remove binding to a current muxi session
    Delete(SessionDeleteArgs),
    /// Open your editor to change your muxi sessions
    Edit,
    /// Print your current muxi sessions
    List,
    /// Set a binding for a new muxi session
    Set(SessionSetArgs),
}

#[derive(Debug, Args)]
pub struct SessionSetArgs {
    /// Tmux key binding
    pub key: Key,
    #[arg(short, long)]
    /// Name of the session (default: current session's name)
    pub name: Option<String>,
    #[arg(short, long)]
    /// Path of the session (default: current session's path)
    pub path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct SessionDeleteArgs {
    /// Tmux key binding
    pub key: Key,
}

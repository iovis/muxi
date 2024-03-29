use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use clap_complete::Shell;

use crate::tmux::Key;

#[derive(Debug, Parser)]
#[command(name = "muxi")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
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

    /// Spawn a FZF popup to manage your muxi sessions
    Fzf {
        /// Args forwarded to `fzf`
        #[arg(last = true)]
        fzf_args: Vec<String>,
    },
}

#[derive(Debug, Args)]
pub struct Config {
    #[command(subcommand)]
    pub command: Option<ConfigCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    /// Open your editor to change your settings
    Edit {
        /// Args forwarded to your $EDITOR
        #[arg(last = true)]
        editor_args: Vec<String>,
    },

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
    Edit {
        /// Args forwarded to your $EDITOR
        #[arg(last = true)]
        editor_args: Vec<String>,
    },

    /// Print your current muxi sessions
    List,

    /// Set a binding for a new muxi session
    Set(SessionSetArgs),

    /// Go to session
    Switch {
        /// Tmux key binding
        #[arg(required_unless_present_any = ["interactive", "tmux_menu"])]
        key: Option<Key>,

        /// Choose session from a list
        #[arg(short, long, exclusive = true)]
        interactive: bool,

        /// Choose session from a native tmux menu (display-menu)
        #[arg(short, long, exclusive = true)]
        tmux_menu: bool,
    },
}

#[derive(Debug, Args)]
pub struct SessionSetArgs {
    /// Tmux key binding
    pub key: Key,

    /// Name of the session (default: current session's name)
    #[arg(short, long)]
    pub name: Option<String>,

    /// Path of the session (default: current session's path)
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct SessionDeleteArgs {
    /// Tmux key binding
    pub key: Key,
}

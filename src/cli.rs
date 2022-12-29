use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::tmux::TmuxKey;

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
    Init,
    Config(Config),
    Sessions(Sessions),
}

#[derive(Debug, Args)]
pub struct Config {
    #[command(subcommand)]
    pub command: Option<ConfigCommands>,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommands {
    Edit,
    List,
}

#[derive(Debug, Args)]
pub struct Sessions {
    #[command(subcommand)]
    pub command: Option<SessionCommands>,
}

#[derive(Debug, Subcommand)]
pub enum SessionCommands {
    Delete(SessionDeleteOptions),
    Edit,
    List,
    Set(SessionSetOptions),
}

#[derive(Debug, Args)]
pub struct SessionSetOptions {
    pub key: TmuxKey,
    #[arg(short, long)]
    pub name: Option<String>,
    #[arg(short, long)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct SessionDeleteOptions {
    pub key: TmuxKey,
}

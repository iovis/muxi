use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::tmux::TmuxKey;

#[derive(Debug, Parser)]
#[command(name = "muxi")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
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

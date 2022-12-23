use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::sessions::TmuxKey;

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
    Edit,
    List,
    Set {
        key: TmuxKey,
        #[arg(short, long)]
        name: Option<String>,
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
}
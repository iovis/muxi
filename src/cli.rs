use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "muxi")]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
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
    List,
}

#[derive(Debug, Args)]
pub struct Sessions {
    #[command(subcommand)]
    pub command: Option<SessionCommands>,
}

#[derive(Debug, Subcommand)]
pub enum SessionCommands {
    Init,
    Edit,
    List,
}

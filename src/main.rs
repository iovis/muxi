use clap::Parser;
use muxi::cli::{self, Cli};
use muxi::commands;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        cli::Command::Init => commands::init(),
        cli::Command::Sessions(sessions_command) => {
            // Default to `list` if no command given
            let command = sessions_command
                .command
                .unwrap_or(cli::SessionCommands::List);

            match command {
                cli::SessionCommands::Edit => commands::sessions::edit(),
                cli::SessionCommands::List => commands::sessions::list(),
                cli::SessionCommands::Delete { key } => commands::sessions::delete(key),
                cli::SessionCommands::Set { key, name, path } => {
                    commands::sessions::set(key, name, path)
                }
            }
        }
        cli::Command::Config(config_command) => {
            // Default to `list` if no command given
            let command = config_command.command.unwrap_or(cli::ConfigCommands::List);

            match command {
                cli::ConfigCommands::List => commands::config::list(),
                cli::ConfigCommands::Edit => commands::config::edit(),
            }
        }
    }
}

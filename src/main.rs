use clap::Parser;
use muxi::cli::{self, Cli};
use muxi::commands;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        cli::Command::Sessions(sessions_command) => {
            // Default to `list` if no command given
            let command = sessions_command
                .command
                .unwrap_or(cli::SessionCommands::List);

            match command {
                cli::SessionCommands::Init => commands::sessions::init(),
                cli::SessionCommands::Edit => commands::sessions::edit(),
                cli::SessionCommands::List => commands::sessions::list(),
            }
        }
        cli::Command::Config(config_command) => {
            // Default to `list` if no command given
            let command = config_command.command.unwrap_or(cli::ConfigCommands::List);

            match command {
                cli::ConfigCommands::List => commands::config::list(),
            }
        }
    }
}

use clap::Parser;
use muxi::cli::{Cli, Command, ConfigCommands, SessionCommands};
use muxi::commands::{self, config, sessions};

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Init => commands::init(),
        Command::Sessions(sessions_command) => {
            // Default to `list` if no command given
            let command = sessions_command.command.unwrap_or(SessionCommands::List);

            match command {
                SessionCommands::Edit => sessions::edit(),
                SessionCommands::List => sessions::list(),
                SessionCommands::Delete(options) => sessions::delete(options),
                SessionCommands::Set(options) => sessions::set(options),
            }
        }
        Command::Config(config_command) => {
            // Default to `list` if no command given
            let command = config_command.command.unwrap_or(ConfigCommands::List);

            match command {
                ConfigCommands::List => config::list(),
                ConfigCommands::Edit => config::edit(),
            }
        }
    }
}

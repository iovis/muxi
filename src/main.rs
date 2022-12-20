use clap::Parser;

use self::muxi::cli::{self, Cli};
use self::muxi::commands;

mod muxi;

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
    }
}

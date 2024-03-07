use clap::{CommandFactory, Parser};
use clap_complete::generate;
use color_eyre::Result;
use muxi::cli::{Cli, Command, ConfigCommands, SessionCommands};
use muxi::commands::{self, config, fzf, sessions};

fn main() -> Result<()> {
    color_eyre::install()?;
    let app = Cli::parse();

    match app.command {
        Command::Init => commands::init(),
        Command::Sessions(sessions_command) => {
            // Default to `list` if no command given
            let command = sessions_command.command.unwrap_or(SessionCommands::List);

            match command {
                SessionCommands::Edit { editor_args } => sessions::edit(&editor_args),
                SessionCommands::List => sessions::list(),
                SessionCommands::Delete(options) => sessions::delete(options),
                SessionCommands::Set(options) => sessions::set(options),
                SessionCommands::Switch {
                    key,
                    interactive,
                    tmux_menu,
                } => {
                    if interactive {
                        sessions::picker()
                    } else if tmux_menu {
                        sessions::tmux_menu()
                    } else {
                        // Clap will validate that key exists
                        sessions::switch(&key.unwrap())
                    }
                }
            }
        }
        Command::Config(config_command) => {
            // Default to `list` if no command given
            let command = config_command.command.unwrap_or(ConfigCommands::List);

            match command {
                ConfigCommands::List => config::list(),
                ConfigCommands::Edit { editor_args } => config::edit(&editor_args),
            }
        }
        Command::Completions { shell } => {
            generate(
                shell,
                &mut Cli::command(),
                "muxi",
                &mut std::io::stdout().lock(),
            );
            Ok(())
        }
        Command::Fzf { fzf_args } => fzf::spawn(&fzf_args),
    }
}

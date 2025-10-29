use clap::{CommandFactory, Parser};
use clap_complete::generate;
use miette::Result;
use muxi::cli::{Cli, Command, ConfigCommands, PluginCommands, SessionCommands};
use muxi::commands::{self, config, fzf, plugins, sessions};

fn main() -> Result<()> {
    let app = Cli::parse();

    match app.command {
        Command::Init => commands::init(),
        Command::Ls => sessions::list(),
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
        Command::Plugins(plugins_command) => {
            // Default to `list` if no command given
            let command = plugins_command.command.unwrap_or(PluginCommands::List);

            match command {
                PluginCommands::Init => plugins::init(),
                PluginCommands::List => plugins::list(),
                PluginCommands::Install => plugins::install(),
                PluginCommands::Update => plugins::update(),
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
        Command::FzfKeybindings => fzf::keybindings::show(),
    }
}

use clap::Parser;

use self::muxi::cli::{self, Cli};
use self::muxi::commands;

mod muxi;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        cli::Command::Init => commands::init(),
        cli::Command::Edit => commands::edit(),
        cli::Command::List => commands::list(),
    }
}

use clap::Parser;

use self::muxi::{Cli, Commands};

mod muxi;

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Init => muxi::init(),
        Commands::Edit => muxi::edit(),
    }
}

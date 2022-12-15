mod cli;
pub use cli::*;

mod config;
mod sessions;
mod tmux;

use self::config::Config;
use self::tmux::Tmux;

pub fn init() -> anyhow::Result<()> {
    let config = Config::new();
    let sessions = config.sessions()?;
    let tmux = Tmux::new()?;

    tmux.bind_sessions(&sessions)?;

    Ok(())
}

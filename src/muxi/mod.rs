mod config;
mod sessions;
mod tmux;

use self::config::Config;
use self::tmux::Tmux;

pub fn init() -> anyhow::Result<()> {
    let config = Config::new()?;
    let tmux = Tmux::new()?;

    tmux.bind_sessions(&config.sessions)?;

    Ok(())
}

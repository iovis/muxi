use crate::config::Config;
use crate::tmux::Tmux;

mod config;
mod sessions;
mod tmux;

fn main() -> anyhow::Result<()> {
    // Init
    let config = Config::new()?;
    // dbg!(&config);

    let tmux = Tmux::new()?;
    // dbg!(&tmux);

    tmux.bind_sessions(&config.sessions)?;

    Ok(())
}

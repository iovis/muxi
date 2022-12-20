use crate::muxi::config::Config;
use crate::muxi::tmux::Tmux;

pub fn init() -> anyhow::Result<()> {
    let config = Config::new();
    let sessions = config.sessions()?;
    let tmux = Tmux::new()?;

    tmux.bind_sessions(&sessions)?;

    Ok(())
}

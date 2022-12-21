use crate::muxi::Muxi;
use crate::tmux::Tmux;

pub fn init() -> anyhow::Result<()> {
    let config = Muxi::new();
    let sessions = config.sessions()?;
    let tmux = Tmux::new(config.settings)?;

    tmux.bind_sessions(&sessions)?;

    Ok(())
}

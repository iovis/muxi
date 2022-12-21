use crate::muxi::Tmux;
use crate::muxi::Muxi;

pub fn init() -> anyhow::Result<()> {
    let config = Muxi::new();
    let sessions = config.sessions()?;
    let tmux = Tmux::new()?;

    tmux.bind_sessions(&sessions)?;

    Ok(())
}

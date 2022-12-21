use crate::muxi::Muxi;
use crate::path;
use crate::settings::Settings;
use crate::tmux::Tmux;

pub fn init() -> anyhow::Result<()> {
    let settings = Settings::new(&path::settings_file())?;
    let tmux = Tmux::new(settings)?;

    let config = Muxi::new();
    let sessions = config.sessions()?;

    tmux.bind_sessions(&sessions)?;

    Ok(())
}

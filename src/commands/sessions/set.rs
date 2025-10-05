use color_eyre::Result;
use color_eyre::eyre::ContextCompat;

use crate::cli::SessionSetArgs;
use crate::muxi::{Muxi, Session, Settings};
use crate::tmux;
use crate::commands;

pub fn set(SessionSetArgs { key, name, path }: SessionSetArgs) -> Result<()> {
    let settings = Settings::from_lua()?;

    let name = name
        .or_else(tmux::current_session_name)
        .context("Couldn't find current session name")?;

    let path = path
        .or_else(|| {
            if settings.use_current_pane_path {
                tmux::current_pane_path()
            } else {
                tmux::current_session_path()
            }
        })
        .context("Couldn't find current path")?;

    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.0.insert(key, Session { name, path });
    sessions.save()?;

    // Reload
    commands::init()
}

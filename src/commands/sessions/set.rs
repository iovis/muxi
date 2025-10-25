use miette::Result;
use miette::miette;

use crate::cli::SessionSetArgs;
use crate::commands;
use crate::muxi::{Muxi, Session, Settings};
use crate::tmux;

pub fn set(SessionSetArgs { key, name, path }: SessionSetArgs) -> Result<()> {
    let settings = Settings::from_lua()?;

    let name = name
        .or_else(tmux::current_session_name)
        .ok_or_else(|| miette!("Couldn't find current session name"))?;

    let path = path
        .or_else(|| {
            if settings.use_current_pane_path {
                tmux::current_pane_path()
            } else {
                tmux::current_session_path()
            }
        })
        .ok_or_else(|| miette!("Couldn't find current path"))?;

    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.0.insert(key, Session { name, path });
    sessions.save()?;

    // Reload
    commands::init()
}

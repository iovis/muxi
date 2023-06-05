use color_eyre::eyre::ContextCompat;
use color_eyre::Result;

use crate::cli::SessionSetArgs;
use crate::commands;
use crate::muxi::{sessions, Muxi, Session};
use crate::tmux;

pub fn set(SessionSetArgs { key, name, path }: SessionSetArgs) -> Result<()> {
    let name = name
        .or_else(tmux::current_session_name)
        .context("Couldn't find current session name")?;

    let path = path
        .or_else(tmux::current_session_path)
        .context("Couldn't find current session path")?;

    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.insert(key, Session { name, path });
    sessions::save(&sessions)?;

    // Reload
    commands::init()
}

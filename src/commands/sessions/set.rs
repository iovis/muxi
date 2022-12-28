use anyhow::Context;

use crate::cli::SessionSetOptions;
use crate::commands;
use crate::muxi::Muxi;
use crate::sessions::{self, Session};
use crate::tmux;

pub fn set(SessionSetOptions { key, name, path }: SessionSetOptions) -> anyhow::Result<()> {
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

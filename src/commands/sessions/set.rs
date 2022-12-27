use std::path::PathBuf;

use anyhow::Context;

use crate::commands;
use crate::muxi::Muxi;
use crate::sessions::{self, Session};
use crate::tmux::{self, TmuxKey};

pub fn set(key: TmuxKey, name: Option<String>, path: Option<PathBuf>) -> anyhow::Result<()> {
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

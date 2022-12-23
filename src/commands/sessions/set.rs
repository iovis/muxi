use std::path::PathBuf;

use crate::muxi::Muxi;
use crate::sessions::{Session, TmuxKey, self};
use crate::{tmux, commands};

pub fn set(key: TmuxKey, name: Option<String>, path: Option<PathBuf>) -> anyhow::Result<()> {
    // Get current session name if not given
    let name = if let Some(name) = name {
        name
    } else {
        tmux::current_session_name()?
    };

    // Get current session path if not given
    let path = if let Some(path) = path {
        path
    } else {
        tmux::current_session_path()?
    };

    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.insert(key, Session { name, path });
    sessions::save(&sessions)?;

    // Reload
    commands::init()
}

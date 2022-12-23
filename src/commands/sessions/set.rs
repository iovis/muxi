use std::path::PathBuf;

use crate::sessions::TmuxKey;
use crate::tmux;

pub fn set(key: TmuxKey, name: Option<String>, path: Option<PathBuf>) -> anyhow::Result<()> {
    println!("{:?}", key);

    // Get current session name if not given
    let name = if let Some(name) = name {
        name
    } else {
        tmux::current_session_name()?
    };

    println!("name: {}", name);

    // Get current session path if not given
    let path = if let Some(path) = path {
        path
    } else {
        tmux::current_session_path()?
    };

    println!("path: {}", path.display());

    // TODO: update sessions.toml
    // - Do I need to parse it first and write the end result?

    // TODO: call init() to reset
    todo!()
}

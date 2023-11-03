use color_eyre::eyre::ContextCompat;
use color_eyre::Result;

use crate::cli::SessionSetArgs;
use crate::muxi::{path, Muxi, Session};
use crate::tmux;
use crate::{commands, muxi};

pub fn set(SessionSetArgs { key, name, path }: SessionSetArgs) -> Result<()> {
    let settings = muxi::parse_settings(&path::muxi_dir())?;

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

use crate::muxi::Muxi;
use crate::{sessions, commands};
use crate::tmux::TmuxKey;

pub fn delete(key: TmuxKey) -> anyhow::Result<()> {
    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.remove(&key);

    sessions::save(&sessions)?;

    // Reload
    commands::init()
}

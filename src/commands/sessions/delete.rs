use color_eyre::Result;

use crate::cli::SessionDeleteArgs;
use crate::commands;
use crate::muxi::{sessions, Muxi};

pub fn delete(SessionDeleteArgs { key }: SessionDeleteArgs) -> Result<()> {
    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.remove(&key);

    sessions::save(&sessions)?;

    // Reload
    commands::init()
}

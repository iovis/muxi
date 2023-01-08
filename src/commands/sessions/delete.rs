use color_eyre::Result;

use crate::cli::SessionDeleteArgs;
use crate::muxi::Muxi;
use crate::{commands, sessions};

pub fn delete(SessionDeleteArgs { key }: SessionDeleteArgs) -> Result<()> {
    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.remove(&key);

    sessions::save(&sessions)?;

    // Reload
    commands::init()
}

use crate::cli::SessionDeleteOptions;
use crate::muxi::Muxi;
use crate::{commands, sessions};

pub fn delete(SessionDeleteOptions { key }: SessionDeleteOptions) -> anyhow::Result<()> {
    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;
    sessions.remove(&key);

    sessions::save(&sessions)?;

    // Reload
    commands::init()
}

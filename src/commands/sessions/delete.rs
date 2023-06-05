use color_eyre::Result;

use crate::cli::SessionDeleteArgs;
use crate::commands;
use crate::muxi::Muxi;

pub fn delete(SessionDeleteArgs { key }: SessionDeleteArgs) -> Result<()> {
    // Update sessions.toml
    let mut sessions = Muxi::new()?.sessions;

    sessions.0.remove(&key);
    sessions.save()?;

    // Reload
    commands::init()
}

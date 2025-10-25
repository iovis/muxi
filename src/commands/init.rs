use miette::Result;

use crate::muxi::{Muxi, Settings};
use crate::tmux;

pub fn init() -> Result<()> {
    let muxi = Muxi::new()?;
    let settings = Settings::from_lua()?;

    tmux::init(&settings, &muxi.sessions)?;

    Ok(())
}

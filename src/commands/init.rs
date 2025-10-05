use color_eyre::Result;

use crate::muxi::{Muxi, Settings};
use crate::tmux;

pub fn init() -> Result<()> {
    let muxi = Muxi::new()?;
    let settings = Settings::from_lua()?;

    tmux::create_muxi_bindings(&settings, &muxi.sessions)?;

    Ok(())
}

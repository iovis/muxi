use color_eyre::Result;

use crate::muxi::{self, path, Muxi};
use crate::tmux;

pub fn init() -> Result<()> {
    let muxi = Muxi::new()?;
    let settings = muxi::parse_settings(&path::muxi_dir())?;

    tmux::create_muxi_bindings(&settings, &muxi.sessions)?;

    Ok(())
}

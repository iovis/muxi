use color_eyre::Result;

use crate::muxi::{self, path, Muxi};
use crate::tmux;

pub fn init() -> Result<()> {
    let muxi = Muxi::new()?;
    let tmux_settings = tmux::Settings::new();
    let settings = muxi::Settings::new(&path::settings_file(), tmux_settings)?;

    tmux::create_muxi_bindings(&settings, &muxi.sessions)?;

    Ok(())
}

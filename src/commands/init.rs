use color_eyre::Result;

use crate::muxi::Muxi;
use crate::path;
use crate::settings::Settings;
use crate::tmux;

pub fn init() -> Result<()> {
    let muxi = Muxi::new()?;
    let settings = Settings::new(&path::settings_file())?;

    tmux::create_muxi_bindings(&settings, &muxi.sessions)?;

    Ok(())
}

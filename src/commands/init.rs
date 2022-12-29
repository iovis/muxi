use color_eyre::Result;

use crate::muxi::Muxi;
use crate::path;
use crate::settings::Settings;
use crate::tmux::Tmux;

pub fn init() -> Result<()> {
    let muxi = Muxi::new()?;

    let settings = Settings::new(&path::settings_file())?;
    let tmux = Tmux::new(settings)?;

    tmux.bind_sessions(&muxi.sessions)?;

    Ok(())
}

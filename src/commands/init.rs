use color_eyre::Result;

use crate::muxi::{lua, Muxi, SettingsBuilder};
use crate::tmux;

pub fn init() -> Result<()> {
    let muxi = Muxi::new()?;
    let mut settings_builder = SettingsBuilder::new();

    match lua::settings() {
        Ok(settings) => {
            settings_builder = settings_builder.set(settings);
        }
        Err(lua::Error::NotFound(_)) => (),
        Err(error) => return Err(error)?,
    };

    let tmux_settings = tmux::Settings::new();
    let settings = settings_builder.merge_tmux_settings(&tmux_settings).build();

    tmux::create_muxi_bindings(&settings, &muxi.sessions)?;

    Ok(())
}

use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::muxi::Settings;

pub fn list() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        println!("{}", "No plugins defined!".red());
    }

    for plugin in plugins {
        // TODO: show if installed
        println!("{}", plugin.green());
    }

    Ok(())
}

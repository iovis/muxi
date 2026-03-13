use super::ui;
use crate::muxi::Settings;
use miette::Result;

pub fn init() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        return Ok(());
    }

    let mut errors = Vec::new();

    for plugin in plugins {
        if let Err(error) = plugin.source() {
            errors.push((plugin, error));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ui::format_plugin_errors(&errors, "source"))
    }
}

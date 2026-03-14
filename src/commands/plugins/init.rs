use std::sync::Mutex;
use std::thread;

use super::ui;
use crate::muxi::Settings;
use miette::Result;

pub fn init() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        return Ok(());
    }

    let errors = Mutex::new(Vec::new());

    thread::scope(|s| {
        for plugin in plugins {
            s.spawn(|| {
                if let Err(error) = plugin.source() {
                    errors.lock().unwrap().push((plugin, error));
                }
            });
        }
    });

    let errors = errors.into_inner().unwrap();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ui::format_plugin_errors(&errors, "source"))
    }
}

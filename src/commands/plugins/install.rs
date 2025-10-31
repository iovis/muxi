use std::sync::Mutex;
use std::thread;

use indicatif::MultiProgress;
use miette::Result;
use owo_colors::OwoColorize;

use super::ui::{self, PluginSpinner};
use crate::muxi::Settings;

pub fn install() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        println!("{}", "No plugins defined!".red());
        return Ok(());
    }

    let multi = MultiProgress::new();
    let errors = Mutex::new(Vec::new());

    thread::scope(|s| {
        for plugin in plugins {
            s.spawn(|| {
                let spinner = PluginSpinner::new(&multi, &plugin.name);

                match plugin.install() {
                    Ok(true) => spinner.finish_success(None),
                    Ok(false) => spinner.finish_already_installed(),
                    Err(error) => {
                        spinner.finish_error();
                        errors.lock().unwrap().push((plugin, error));
                    }
                }
            });
        }
    });

    let errors = errors.into_inner().unwrap();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ui::format_plugin_errors(&errors, "install"))
    }
}

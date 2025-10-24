use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

use indicatif::MultiProgress;
use miette::Result;
use owo_colors::OwoColorize;

use super::helpers;
use crate::commands::plugins::ui::PluginSpinner;
use crate::muxi::{Settings, path};

pub fn install() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        println!("{}", "No plugins defined!".red());
        return Ok(());
    }

    let plugins_dir = path::plugins_dir();

    // Create plugins directory if it doesn't exist
    fs::create_dir_all(&plugins_dir).map_err(|e| {
        miette::miette!(
            "Failed to create plugins directory at {}: {}",
            plugins_dir.display(),
            e
        )
    })?;

    let multi = Arc::new(MultiProgress::new());
    let errors = Arc::new(Mutex::new(Vec::new()));

    let handles: Vec<_> = plugins
        .into_iter()
        .map(|plugin| {
            let multi = Arc::clone(&multi);
            let errors = Arc::clone(&errors);

            thread::spawn(move || {
                let spinner = PluginSpinner::new(&multi, plugin.repo_name());

                match plugin.install() {
                    Ok(true) => spinner.finish_success(),
                    Ok(false) => spinner.finish_already_installed(),
                    Err(error) => {
                        spinner.finish_error();
                        errors.lock().unwrap().push((plugin, error));
                    }
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Report any errors at the end
    let errors = errors.lock().unwrap();
    if !errors.is_empty() {
        return Err(helpers::format_plugin_errors(&errors, "install"));
    }

    Ok(())
}

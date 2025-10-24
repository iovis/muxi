use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

use indicatif::MultiProgress;
use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::{Settings, path};

use super::helpers::{format_plugin_errors, install_plugin};

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
            let plugins_dir = plugins_dir.clone();
            let multi = Arc::clone(&multi);
            let errors = Arc::clone(&errors);

            thread::spawn(move || {
                let result = install_plugin(&plugin, &plugins_dir, &multi);
                if let Err(e) = result {
                    errors.lock().unwrap().push((plugin, e));
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
        return Err(format_plugin_errors(&errors, "install"));
    }

    Ok(())
}

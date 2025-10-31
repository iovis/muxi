use std::sync::Mutex;
use std::thread;

use indicatif::MultiProgress;
use miette::Result;
use owo_colors::OwoColorize;

use super::ui::{self, PluginSpinner};
use crate::muxi::{PluginUpdateStatus, Settings};

pub fn update() -> Result<()> {
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

                match plugin.update() {
                    Ok(PluginUpdateStatus::Updated { from, to }) => {
                        let detail = match from {
                            Some(from) => format!("{from}..{to}"),
                            None => to,
                        };
                        spinner.finish_success(Some(&detail));
                    }
                    Ok(PluginUpdateStatus::UpToDate { commit }) => {
                        spinner.finish_up_to_date(Some(&commit));
                    }
                    Ok(PluginUpdateStatus::Local { path }) => {
                        spinner.finish_up_to_date(Some(&path));
                    }
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
        Err(ui::format_plugin_errors(&errors, "update"))
    }
}

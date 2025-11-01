use std::sync::Mutex;
use std::thread;
use std::time::{Duration, SystemTime};

use indicatif::MultiProgress;
use miette::Result;
use owo_colors::OwoColorize;
use timeago::Formatter;

use super::ui::{self, PluginSpinner};
use crate::muxi::{PluginChange, PluginUpdateStatus, Settings};

pub fn update() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        println!("{}", "No plugins defined!".red());
        return Ok(());
    }

    let multi = MultiProgress::new();
    let errors = Mutex::new(Vec::new());
    let change_logs = Mutex::new(Vec::new());

    thread::scope(|s| {
        for (index, plugin) in plugins.into_iter().enumerate() {
            let progress = &multi;
            let errors_ref = &errors;
            let change_logs_ref = &change_logs;

            s.spawn(move || {
                let spinner = PluginSpinner::new(progress, &plugin.name);
                let plugin_name = plugin.name.clone();

                match plugin.update() {
                    Ok(PluginUpdateStatus::Updated {
                        from,
                        to,
                        changes,
                        range_url,
                    }) => {
                        let display = match from {
                            Some(from) => format!("{from}..{to}"),
                            None => to,
                        };
                        let detail = if let Some(url) = range_url.as_ref() {
                            ui::hyperlink(&display, url)
                        } else {
                            display
                        };
                        spinner.finish_success(Some(&detail));

                        if !changes.is_empty() {
                            let log = format_plugin_changes(&plugin_name, &changes);
                            change_logs_ref.lock().unwrap().push((index, log));
                        }
                    }
                    Ok(PluginUpdateStatus::UpToDate { commit }) => {
                        spinner.finish_up_to_date(Some(&commit));
                    }
                    Ok(PluginUpdateStatus::Local { path }) => {
                        spinner.finish_up_to_date(Some(&path));
                    }
                    Err(error) => {
                        spinner.finish_error();
                        errors_ref.lock().unwrap().push((plugin.clone(), error));
                    }
                }
            });
        }
    });

    drop(multi);

    let mut change_logs = change_logs.into_inner().unwrap();
    change_logs.sort_by_key(|(index, _)| *index);
    if !change_logs.is_empty() {
        println!();
        for (_, log) in change_logs {
            println!("{log}");
        }
    }

    let errors = errors.into_inner().unwrap();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ui::format_plugin_errors(&errors, "update"))
    }
}

fn format_plugin_changes(plugin_name: &str, changes: &[PluginChange]) -> String {
    let header = plugin_name.bold().to_string();

    let body = changes
        .iter()
        .map(|change| {
            let id_colored = change.id.green().bold().to_string();
            let id_formatted = change
                .url
                .as_ref()
                .map_or_else(|| id_colored.clone(), |url| ui::hyperlink(&id_colored, url));

            format!(
                "  {} {} {}",
                id_formatted,
                change.summary.trim(),
                format!("({})", format_relative_time(change.time)).dimmed()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("{header}\n{body}")
}

fn format_relative_time(time: SystemTime) -> String {
    let now = SystemTime::now();
    let duration = now
        .duration_since(time)
        .unwrap_or_else(|_| Duration::from_secs(0));

    Formatter::new().convert(duration)
}

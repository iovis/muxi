use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::{PluginStatus, Settings};

pub fn list() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        println!("{}", "No plugins defined!".red());
        return Ok(());
    }

    for plugin in plugins {
        print_plugin_status(&plugin)?;

        if !plugin.options.is_empty() {
            println!("{}", plugin.options);
        }
    }

    Ok(())
}

fn print_plugin_status(plugin: &crate::muxi::Plugin) -> Result<()> {
    let status = plugin.status()?;

    match status {
        PluginStatus::Remote {
            installed: true,
            commit: Some(commit),
        } => println!(
            "{} {} {}",
            "✔".green().bold(),
            plugin.name,
            format!("({commit})").dimmed()
        ),
        PluginStatus::Remote {
            installed: true,
            commit: None,
        } => println!("{} {}", "✔".green().bold(), plugin.name),
        PluginStatus::Remote {
            installed: false, ..
        } => println!(
            "{} {} {}",
            "○".dimmed(),
            plugin.name.dimmed(),
            "(not installed)".dimmed()
        ),
        PluginStatus::Local { exists: true, path } => println!(
            "{} {} {}",
            "✔".green().bold(),
            plugin.name,
            format!("({path})").dimmed()
        ),
        PluginStatus::Local {
            exists: false,
            path,
        } => println!(
            "{} {} {}",
            "○".dimmed(),
            plugin.name.dimmed(),
            format!("({path})").dimmed()
        ),
    }

    Ok(())
}

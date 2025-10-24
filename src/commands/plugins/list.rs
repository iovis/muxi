use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::Settings;

pub fn list() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        println!("{}", "No plugins defined!".red());
        return Ok(());
    }

    for plugin in plugins {
        if plugin.is_installed() {
            println!("{} {}", "✔".green().bold(), plugin.repo_name());
        } else {
            println!("{} {}", "○".dimmed(), plugin.repo_name().dimmed());
        }
    }

    Ok(())
}

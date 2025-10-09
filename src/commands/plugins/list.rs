use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::{Settings, path};

pub fn list() -> Result<()> {
    let plugins = Settings::from_lua()?.plugins;

    if plugins.is_empty() {
        println!("{}", "No plugins defined!".red());
        return Ok(());
    }

    let plugins_dir = path::plugins_dir();

    for plugin in plugins {
        let install_path = plugin.install_path(&plugins_dir);
        let repo_name = plugin.repo_name();

        if install_path.exists() {
            println!("{} {}", "✔".green().bold(), repo_name);
        } else {
            println!("{} {}", "○".dimmed(), repo_name.dimmed());
        }
    }

    Ok(())
}

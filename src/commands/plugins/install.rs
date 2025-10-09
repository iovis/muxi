use std::fs;

use git2::Repository;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::{Plugin, Settings, path};

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

    let multi = MultiProgress::new();
    let mut errors = Vec::new();

    for plugin in plugins {
        let result = install_plugin(&plugin, &plugins_dir, &multi);
        if let Err(e) = result {
            errors.push((plugin, e));
        }
    }

    // Report any errors at the end
    if !errors.is_empty() {
        eprintln!();
        for (plugin, error) in errors {
            eprintln!("{} {}: {}", "✗".red(), plugin, error);
        }
        return Err(miette::miette!("Some plugins failed to install"));
    }

    Ok(())
}

fn install_plugin(
    plugin: &Plugin,
    plugins_dir: &std::path::Path,
    multi: &MultiProgress,
) -> Result<()> {
    let repo_name = plugin.repo_name();
    let install_path = plugin.install_path(plugins_dir);

    // Check if already installed
    if install_path.exists() {
        let pb = multi.add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold} {msg}")
                .unwrap(),
        );
        pb.set_prefix("⊙");
        pb.set_message(repo_name.to_string());
        pb.finish_with_message(format!("{} (already installed)", repo_name.dimmed()));
        return Ok(());
    }

    // Create spinner for this plugin
    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(repo_name.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));

    // Clone the repository
    let result = Repository::clone(plugin.url.as_str(), &install_path);

    match result {
        Ok(_) => {
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:.bold.green} {msg}")
                    .unwrap(),
            );
            pb.set_prefix("✔");
            pb.finish_with_message(repo_name.to_string());
            Ok(())
        }
        Err(e) => {
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:.bold.red} {msg}")
                    .unwrap(),
            );
            pb.set_prefix("✗");
            pb.finish_with_message(repo_name.to_string());
            Err(miette::miette!("Failed to clone repository: {}", e))
        }
    }
}

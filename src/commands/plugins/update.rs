use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

use git2::{BranchType, Repository};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::{Plugin, Settings, path};

pub fn update() -> Result<()> {
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
                let result = update_plugin(&plugin, &plugins_dir, &multi);
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
        eprintln!();
        for (plugin, error) in errors.iter() {
            eprintln!("{} {}: {}", "✗".red(), plugin, error);
        }
        return Err(miette::miette!("Some plugins failed to update"));
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
fn update_plugin(
    plugin: &Plugin,
    plugins_dir: &std::path::Path,
    multi: &MultiProgress,
) -> Result<()> {
    let repo_name = plugin.repo_name();
    let install_path = plugin.install_path(plugins_dir);

    // If not installed, install it
    if !install_path.exists() {
        return install_plugin(plugin, plugins_dir, multi);
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

    // Open the repository
    let repo = Repository::open(&install_path).map_err(|e| {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.red} {msg}")
                .unwrap(),
        );
        pb.set_prefix("✗");
        pb.finish_with_message(repo_name.to_string());
        miette::miette!("Failed to open repository: {}", e)
    })?;

    // Get current commit
    let head = repo.head().map_err(|e| {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.red} {msg}")
                .unwrap(),
        );
        pb.set_prefix("✗");
        pb.finish_with_message(repo_name.to_string());
        miette::miette!("Failed to get HEAD: {}", e)
    })?;

    let old_commit = head.peel_to_commit().map_err(|e| {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.red} {msg}")
                .unwrap(),
        );
        pb.set_prefix("✗");
        pb.finish_with_message(repo_name.to_string());
        miette::miette!("Failed to get commit: {}", e)
    })?;

    let old_hash = format!("{:.7}", old_commit.id());

    // Fetch from origin
    let mut remote = repo.find_remote("origin").map_err(|e| {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.red} {msg}")
                .unwrap(),
        );
        pb.set_prefix("✗");
        pb.finish_with_message(repo_name.to_string());
        miette::miette!("Failed to find remote 'origin': {}", e)
    })?;

    remote.fetch(&["HEAD"], None, None).map_err(|e| {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.red} {msg}")
                .unwrap(),
        );
        pb.set_prefix("✗");
        pb.finish_with_message(repo_name.to_string());
        miette::miette!("Failed to fetch from remote: {}", e)
    })?;

    // Get the default branch name
    let default_branch = repo
        .find_branch("origin/HEAD", BranchType::Remote)
        .and_then(|branch| {
            branch
                .get()
                .symbolic_target()
                .and_then(|target| target.strip_prefix("refs/remotes/origin/"))
                .map(String::from)
                .ok_or_else(|| git2::Error::from_str("Could not determine default branch"))
        })
        .or_else(|_| {
            // Fallback to common default branches
            for branch_name in ["main", "master"] {
                if repo
                    .find_branch(&format!("origin/{branch_name}"), BranchType::Remote)
                    .is_ok()
                {
                    return Ok(branch_name.to_string());
                }
            }
            Err(git2::Error::from_str("Could not find default branch"))
        })
        .map_err(|e| {
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:.bold.red} {msg}")
                    .unwrap(),
            );
            pb.set_prefix("✗");
            pb.finish_with_message(repo_name.to_string());
            miette::miette!("Failed to determine default branch: {}", e)
        })?;

    // Get the latest commit from the default branch
    let remote_branch = repo
        .find_branch(&format!("origin/{default_branch}"), BranchType::Remote)
        .map_err(|e| {
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:.bold.red} {msg}")
                    .unwrap(),
            );
            pb.set_prefix("✗");
            pb.finish_with_message(repo_name.to_string());
            miette::miette!("Failed to find remote branch: {}", e)
        })?;

    let remote_commit = remote_branch.get().peel_to_commit().map_err(|e| {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.red} {msg}")
                .unwrap(),
        );
        pb.set_prefix("✗");
        pb.finish_with_message(repo_name.to_string());
        miette::miette!("Failed to get remote commit: {}", e)
    })?;

    let new_hash = format!("{:.7}", remote_commit.id());

    // Check if already up to date
    if old_commit.id() == remote_commit.id() {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.blue} {msg}")
                .unwrap(),
        );
        pb.set_prefix("≡");
        pb.finish_with_message(repo_name.to_string());
        return Ok(());
    }

    // Update to the latest commit
    repo.set_head_detached(remote_commit.id()).map_err(|e| {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{prefix:.bold.red} {msg}")
                .unwrap(),
        );
        pb.set_prefix("✗");
        pb.finish_with_message(repo_name.to_string());
        miette::miette!("Failed to update HEAD: {}", e)
    })?;

    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .map_err(|e| {
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{prefix:.bold.red} {msg}")
                    .unwrap(),
            );
            pb.set_prefix("✗");
            pb.finish_with_message(repo_name.to_string());
            miette::miette!("Failed to checkout: {}", e)
        })?;

    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{prefix:.bold.green} {msg}")
            .unwrap(),
    );
    pb.set_prefix("✔");
    pb.finish_with_message(format!(
        "{} {}..{}",
        repo_name,
        old_hash.dimmed(),
        new_hash.dimmed()
    ));

    Ok(())
}

fn install_plugin(
    plugin: &Plugin,
    plugins_dir: &std::path::Path,
    multi: &MultiProgress,
) -> Result<()> {
    let repo_name = plugin.repo_name();
    let install_path = plugin.install_path(plugins_dir);

    // Create spinner for this plugin
    let pb = multi.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("{repo_name} (installing)"));
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
            pb.finish_with_message(format!("{repo_name} (installed)"));
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

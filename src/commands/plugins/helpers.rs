use git2::{BranchType, Repository};
use indicatif::MultiProgress;
use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::Plugin;

use super::ui::PluginSpinner;

/// Install a single plugin from its git repository
pub fn install_plugin(
    plugin: &Plugin,
    plugins_dir: &std::path::Path,
    multi: &MultiProgress,
) -> Result<()> {
    let repo_name = plugin.repo_name();
    let install_path = plugin.install_path(plugins_dir);

    if install_path.exists() {
        let spinner = PluginSpinner::new(multi, repo_name);
        spinner.finish_already_installed();
        return Ok(());
    }

    let spinner = PluginSpinner::new(multi, repo_name);
    let result = Repository::clone(plugin.url.as_str(), &install_path);

    match result {
        Ok(_) => {
            spinner.finish_success();
            Ok(())
        }
        Err(e) => {
            spinner.finish_error();
            Err(miette::miette!("Failed to clone repository: {e}"))
        }
    }
}

/// Update a single plugin to the latest commit on the default branch
pub fn update_plugin(
    plugin: &Plugin,
    plugins_dir: &std::path::Path,
    multi: &MultiProgress,
) -> Result<()> {
    let repo_name = plugin.repo_name();
    let install_path = plugin.install_path(plugins_dir);

    if !install_path.exists() {
        return install_plugin(plugin, plugins_dir, multi);
    }

    let spinner = PluginSpinner::new(multi, repo_name);

    // Helper to finish with error
    let finish_error = |error_msg: String| {
        spinner.finish_error();
        miette::miette!(error_msg)
    };

    let repo = Repository::open(&install_path)
        .map_err(|e| finish_error(format!("Failed to open repository: {e}")))?;

    // Get current commit
    let head = repo
        .head()
        .map_err(|e| finish_error(format!("Failed to get HEAD: {e}")))?;

    let old_commit = head
        .peel_to_commit()
        .map_err(|e| finish_error(format!("Failed to get commit: {e}")))?;

    let old_hash = format!("{:.7}", old_commit.id());

    // Fetch from origin
    let mut remote = repo
        .find_remote("origin")
        .map_err(|e| finish_error(format!("Failed to find remote 'origin': {e}")))?;

    remote
        .fetch(&["HEAD"], None, None)
        .map_err(|e| finish_error(format!("Failed to fetch from remote: {e}")))?;

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
        .map_err(|e| finish_error(format!("Failed to determine default branch: {e}")))?;

    // Get the latest commit from the default branch
    let remote_branch = repo
        .find_branch(&format!("origin/{default_branch}"), BranchType::Remote)
        .map_err(|e| finish_error(format!("Failed to find remote branch: {e}")))?;

    let remote_commit = remote_branch
        .get()
        .peel_to_commit()
        .map_err(|e| finish_error(format!("Failed to get remote commit: {e}")))?;

    let new_hash = format!("{:.7}", remote_commit.id());

    // Check if already up to date
    if old_commit.id() == remote_commit.id() {
        spinner.finish_up_to_date();
        return Ok(());
    }

    // Update to the latest commit
    repo.set_head_detached(remote_commit.id())
        .map_err(|e| finish_error(format!("Failed to update HEAD: {e}")))?;

    repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))
        .map_err(|e| finish_error(format!("Failed to checkout: {e}")))?;

    spinner.finish_success_with_details(&format!("{}..{}", old_hash.dimmed(), new_hash.dimmed()));

    Ok(())
}

/// Format plugin errors for display
pub fn format_plugin_errors(
    errors: &[(Plugin, miette::Report)],
    operation: &str,
) -> miette::Report {
    let error_messages: String = errors
        .iter()
        .map(|(plugin, error)| format!("- {plugin}: {error}"))
        .collect::<Vec<_>>()
        .join("\n");

    miette::miette!("Some plugins failed to {operation}\n{error_messages}")
}

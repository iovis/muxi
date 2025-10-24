use owo_colors::OwoColorize;

use crate::muxi::Plugin;

pub fn format_plugin_errors(
    errors: &[(Plugin, miette::Report)],
    operation: &str,
) -> miette::Report {
    let error_messages = errors
        .iter()
        .map(|(plugin, error)| format!("{} {error}", plugin.repo_name().red()))
        .collect::<Vec<_>>()
        .join("\n");

    miette::miette!("Some plugins failed to {operation}\n{error_messages}")
}

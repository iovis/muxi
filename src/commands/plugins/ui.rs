use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

use crate::muxi::Plugin;

pub fn format_plugin_errors(
    errors: &[(Plugin, miette::Report)],
    operation: &str,
) -> miette::Report {
    let error_messages = errors
        .iter()
        .map(|(plugin, error)| format!("{} {error}", plugin.name.red()))
        .collect::<Vec<_>>()
        .join("\n");

    miette::miette!("Some plugins failed to {operation}\n{error_messages}")
}

const OSC8_PREFIX: &str = "\u{1b}]8;;";
const OSC8_SUFFIX: &str = "\u{1b}]8;;\u{1b}\\";

/// Wraps `message` in an OSC 8 hyperlink sequence pointing to `url`.
pub fn hyperlink(message: &str, url: &str) -> String {
    format!("{OSC8_PREFIX}{url}\u{1b}\\{message}{OSC8_SUFFIX}")
}

#[derive(Debug, Clone, Copy)]
enum PluginSpinnerResult {
    AlreadyInstalled,
    Error,
    Success,
    UpToDate,
}

pub struct PluginSpinner {
    pb: ProgressBar,
    finish_style: ProgressStyle,
    repo_name: String,
}

impl PluginSpinner {
    pub fn new(multi: &MultiProgress, repo_name: &str) -> Self {
        let pb = multi.add(ProgressBar::new_spinner());
        let spinner_style = ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.black} {msg}")
            .unwrap();
        let finish_style = ProgressStyle::default_spinner()
            .template("{prefix} {msg}")
            .unwrap();

        pb.set_style(spinner_style);
        pb.set_message(repo_name.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(80));

        Self {
            pb,
            finish_style,
            repo_name: repo_name.to_string(),
        }
    }

    pub fn finish_success(&self, detail: Option<&str>) {
        self.finish_with(PluginSpinnerResult::Success, detail);
    }

    pub fn finish_error(&self) {
        self.finish_with(PluginSpinnerResult::Error, None);
    }

    pub fn finish_up_to_date(&self, detail: Option<&str>) {
        self.finish_with(PluginSpinnerResult::UpToDate, detail);
    }

    pub fn finish_already_installed(&self) {
        self.finish_with(
            PluginSpinnerResult::AlreadyInstalled,
            Some("already installed"),
        );
    }

    fn finish_with(&self, result: PluginSpinnerResult, detail: Option<&str>) {
        self.pb.set_style(self.finish_style.clone());

        let prefix = match result {
            PluginSpinnerResult::AlreadyInstalled => "⊙".blue().bold().to_string(),
            PluginSpinnerResult::Error => "✗".red().bold().to_string(),
            PluginSpinnerResult::Success => "✔".green().bold().to_string(),
            PluginSpinnerResult::UpToDate => "≡".blue().bold().to_string(),
        };

        self.pb.set_prefix(prefix);

        let message = match detail {
            Some(detail) => format!("{} {}", self.repo_name, format!("({detail})").dimmed()),
            None => self.repo_name.clone(),
        };

        self.pb.finish_with_message(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hyperlink_wraps_message_with_osc8_sequence() {
        let message = "hash";
        let url = "https://example.com/commit";
        let expected = format!("{OSC8_PREFIX}{url}\u{1b}\\{message}{OSC8_SUFFIX}");

        assert_eq!(hyperlink(message, url), expected);
    }

    #[test]
    fn hyperlink_allows_empty_message() {
        let url = "https://example.com";
        let expected = format!("{OSC8_PREFIX}{url}\u{1b}\\{OSC8_SUFFIX}");

        assert_eq!(hyperlink("", url), expected);
    }
}

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

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

    pub fn finish_success(&self) {
        self.pb.set_style(self.finish_style.clone());
        self.pb.set_prefix(format!("{}", "✔".green().bold()));
        self.pb.finish_with_message(self.repo_name.clone());
    }

    pub fn finish_success_with_details(&self, details: &str) {
        self.pb.set_style(self.finish_style.clone());
        self.pb.set_prefix(format!("{}", "✔".green().bold()));
        self.pb
            .finish_with_message(format!("{} {}", self.repo_name, details));
    }

    pub fn finish_error(&self) {
        self.pb.set_style(self.finish_style.clone());
        self.pb.set_prefix(format!("{}", "✗".red().bold()));
        self.pb.finish_with_message(self.repo_name.clone());
    }

    pub fn finish_up_to_date(&self) {
        self.pb.set_style(self.finish_style.clone());
        self.pb.set_prefix(format!("{}", "≡".blue().bold()));
        self.pb.finish_with_message(self.repo_name.clone());
    }

    pub fn finish_already_installed(&self) {
        self.pb.set_style(self.finish_style.clone());
        self.pb.set_prefix(format!("{}", "⊙".blue().bold()));
        self.pb.finish_with_message(format!(
            "{} {}",
            self.repo_name,
            "(already installed)".dimmed()
        ));
    }
}

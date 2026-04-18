use std::process::{Command, Stdio};

use miette::{IntoDiagnostic, Result};

use super::Plugin;
use super::install::install_path;
use super::shared::ensure_exists;

impl Plugin {
    /// Sources the plugin
    pub fn source(&self) -> Result<()> {
        let root = install_path(self);
        ensure_exists(&root)?;

        self.apply_options()?;

        let entries = std::fs::read_dir(&root).into_diagnostic()?;

        for entry in entries.flatten() {
            let path = entry.path();

            // Execute all "*.tmux" files in the plugin directory
            if path.extension().is_some_and(|ext| ext == "tmux") {
                let status = Command::new(&path)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .into_diagnostic()?;

                if !status.success() {
                    return Err(miette::miette!(
                        "Failed to execute {}: script exited with {}",
                        path.display(),
                        status
                    ));
                }
            }
        }

        Ok(())
    }

    fn apply_options(&self) -> Result<()> {
        for (key, value) in &self.options {
            let status = Command::new("tmux")
                .arg("set")
                .arg("-g")
                .arg(format!("@{key}"))
                .arg(value)
                .status()
                .into_diagnostic()?;

            if !status.success() {
                return Err(miette::miette!(
                    "Failed to configure option @{key} for {}",
                    self.name
                ));
            }
        }

        Ok(())
    }
}

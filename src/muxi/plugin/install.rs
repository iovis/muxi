use std::path::PathBuf;
use std::process::{Command, Stdio};

use miette::{IntoDiagnostic, Result};

use crate::muxi::path;

use super::Plugin;
use super::shared::display_path;

impl Plugin {
    pub fn is_installed(&self) -> bool {
        install_path(self).exists()
    }

    /// Install this plugin to the plugins directory
    pub fn install(&self) -> Result<bool> {
        if let Some(path) = &self.path {
            if path.exists() {
                return Ok(false);
            }

            return Err(miette::miette!(
                "Local plugin path {} does not exist",
                display_path(path)
            ));
        }

        if self.is_installed() {
            return Ok(false);
        }

        create_plugins_dir(self)?;

        let url = self
            .url
            .as_ref()
            .expect("remote plugin must have url")
            .as_str();
        let target = install_path(self);
        let target_str = target.to_string_lossy();

        let status = Command::new("git")
            .args(["clone", url, &target_str])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .into_diagnostic()?;

        if !status.success() {
            return Err(miette::miette!("Failed to clone repository"));
        }

        Ok(true)
    }
}

pub(super) fn install_path(plugin: &Plugin) -> PathBuf {
    if let Some(path) = &plugin.path {
        path.clone()
    } else {
        path::plugins_dir().join(&plugin.name)
    }
}

fn create_plugins_dir(plugin: &Plugin) -> Result<()> {
    if plugin.path.is_some() {
        return Ok(());
    }

    let plugins_dir = path::plugins_dir();

    std::fs::create_dir_all(&plugins_dir).map_err(|error| {
        miette::miette!(
            "Failed to create plugins directory at {}: {}",
            plugins_dir.display(),
            error
        )
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_install_path() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();

        assert_eq!(
            install_path(&plugin),
            path::plugins_dir().join("tmux-continuum")
        );
    }

    #[test]
    fn test_install_local_plugin_missing_path_errors() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        let plugin = Plugin::new_local(temp.clone(), None);

        if temp.exists() {
            std::fs::remove_dir_all(&temp).unwrap();
        }

        let error = plugin.install().expect_err("expected install to fail");
        assert!(format!("{error}").contains("does not exist"));
    }

    #[test]
    fn test_install_local_plugin_existing_path_is_noop() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();

        let plugin = Plugin::new_local(temp.clone(), None);
        let result = plugin.install().unwrap();

        assert!(!result);
        std::fs::remove_dir_all(&temp).unwrap();
    }
}

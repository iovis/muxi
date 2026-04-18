use miette::Result;

use super::Plugin;
use super::install::install_path;
use super::shared::{display_path, git};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Remote {
        installed: bool,
        commit: Option<String>,
    },
    Local {
        exists: bool,
        path: String,
    },
}

impl Plugin {
    pub fn status(&self) -> Result<PluginStatus> {
        if let Some(path) = &self.path {
            let exists = path.exists();
            return Ok(PluginStatus::Local {
                exists,
                path: display_path(path),
            });
        }

        if !self.is_installed() {
            return Ok(PluginStatus::Remote {
                installed: false,
                commit: None,
            });
        }

        let commit = git(&["rev-parse", "--short", "HEAD"], &install_path(self))?;

        Ok(PluginStatus::Remote {
            installed: true,
            commit: Some(commit),
        })
    }
}

use std::path::Path;
use std::time::{Duration, SystemTime};

use miette::Result;

use super::Plugin;
use super::install::install_path;
use super::shared::{display_path, ensure_exists, git};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginChange {
    pub id: String,
    pub full_id: String,
    pub summary: String,
    pub time: SystemTime,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginUpdateStatus {
    Updated {
        from: Option<String>,
        to: String,
        changes: Vec<PluginChange>,
        range_url: Option<String>,
    },
    UpToDate {
        commit: String,
    },
    Local {
        path: String,
    },
}

impl Plugin {
    /// Update this plugin to the latest commit on the default branch
    pub fn update(&self) -> Result<PluginUpdateStatus> {
        if let Some(path) = &self.path {
            ensure_exists(path)?;
            return Ok(PluginUpdateStatus::Local {
                path: display_path(path),
            });
        }

        if !self.is_installed() {
            self.install()?;
            let dir = install_path(self);
            let to = git(&["rev-parse", "--short", "HEAD"], &dir)?;
            return Ok(PluginUpdateStatus::Updated {
                from: None,
                to,
                changes: Vec::new(),
                range_url: None,
            });
        }

        let dir = install_path(self);

        let before_short = git(&["rev-parse", "--short", "HEAD"], &dir)?;
        let before_full = git(&["rev-parse", "HEAD"], &dir)?;

        git(&["pull", "--ff-only"], &dir)?;

        let after_short = git(&["rev-parse", "--short", "HEAD"], &dir)?;
        let after_full = git(&["rev-parse", "HEAD"], &dir)?;

        if before_full == after_full {
            return Ok(PluginUpdateStatus::UpToDate {
                commit: before_short,
            });
        }

        let changes = collect_changes(
            &dir,
            &before_full,
            &after_full,
            self.commit_base_url().as_deref(),
        )?;

        Ok(PluginUpdateStatus::Updated {
            from: Some(before_short),
            to: after_short,
            changes,
            range_url: self.compare_url(&before_full, &after_full),
        })
    }

    fn commit_base_url(&self) -> Option<String> {
        let mut base = self.url.as_ref()?.to_string();

        if base.len() >= 4 && base[base.len() - 4..].eq_ignore_ascii_case(".git") {
            base.truncate(base.len() - 4);
        }

        if !base.ends_with('/') {
            base.push('/');
        }

        base.push_str("commit/");
        Some(base)
    }

    fn compare_url(&self, from: &str, to: &str) -> Option<String> {
        let url = self.url.as_ref()?;
        let host = url.host_str()?.to_ascii_lowercase();

        let mut base = url.to_string();
        if base.len() >= 4 && base[base.len() - 4..].eq_ignore_ascii_case(".git") {
            base.truncate(base.len() - 4);
        }
        if !base.ends_with('/') {
            base.push('/');
        }

        let suffix = match host.as_str() {
            "github.com" | "www.github.com" => "compare/",
            "gitlab.com" | "www.gitlab.com" => "-/compare/",
            _ => return None,
        };

        base.push_str(suffix);
        Some(format!("{base}{from}...{to}"))
    }
}

fn collect_changes(
    dir: &Path,
    from: &str,
    to: &str,
    commit_base_url: Option<&str>,
) -> Result<Vec<PluginChange>> {
    if from == to {
        return Ok(Vec::new());
    }

    let range = format!("{from}..{to}");
    let output = git(&["log", "--format=%H%x00%h%x00%s%x00%at", &range], dir)?;

    if output.is_empty() {
        return Ok(Vec::new());
    }

    output
        .lines()
        .map(|line| {
            let parts: Vec<&str> = line.splitn(4, '\0').collect();
            if parts.len() < 4 {
                return Err(miette::miette!("Failed to parse git log output: {line}"));
            }

            let full_id = parts[0].to_string();
            let id = parts[1].to_string();
            let summary = parts[2].to_string();
            let timestamp: u64 = parts[3]
                .parse()
                .map_err(|_| miette::miette!("Failed to parse commit timestamp: {}", parts[3]))?;

            let url = commit_base_url.map(|base| format!("{base}{full_id}"));

            Ok(PluginChange {
                id,
                full_id,
                summary,
                time: SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp),
                url,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_local_plugin_missing_path_errors() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        let plugin = Plugin::new_local(temp.clone(), None);

        if temp.exists() {
            std::fs::remove_dir_all(&temp).unwrap();
        }

        let error = plugin.update().expect_err("expected update to fail");
        assert!(format!("{error}").contains("does not exist"));
    }

    #[test]
    fn test_update_local_plugin_existing_path_returns_status() {
        let temp = std::env::temp_dir().join(format!("muxi-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp).unwrap();

        let plugin = Plugin::new_local(temp.clone(), None);
        let status = plugin.update().unwrap();

        match status {
            PluginUpdateStatus::Local { path } => {
                assert!(path.contains(temp.file_name().unwrap().to_str().unwrap()));
            }
            other => panic!("unexpected status: {other:?}"),
        }

        std::fs::remove_dir_all(&temp).unwrap();
    }

    #[test]
    fn test_compare_url_github() {
        let plugin = Plugin::parse("tmux-plugins/tmux-continuum").unwrap();
        let from = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let to = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

        let url = plugin.compare_url(from, to).expect("expected github url");

        assert_eq!(
            url,
            "https://github.com/tmux-plugins/tmux-continuum/compare/aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa...bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
        );
    }

    #[test]
    fn test_compare_url_gitlab() {
        let plugin = Plugin::parse("https://gitlab.com/user/repo").unwrap();
        let from = "1234567890abcdef1234567890abcdef12345678";
        let to = "87654321fedcba0987654321fedcba0987654321";

        let url = plugin.compare_url(from, to).expect("expected gitlab url");

        assert_eq!(
            url,
            "https://gitlab.com/user/repo/-/compare/1234567890abcdef1234567890abcdef12345678...87654321fedcba0987654321fedcba0987654321"
        );
    }

    #[test]
    fn test_compare_url_unknown_host_returns_none() {
        let plugin = Plugin::parse("https://example.com/user/repo").unwrap();
        let from = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let to = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

        assert!(plugin.compare_url(from, to).is_none());
    }
}

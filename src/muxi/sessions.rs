use std::collections::BTreeMap;
use std::fmt;
use std::path::Path;
use std::path::PathBuf;

use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;
use serde::{Deserialize, Deserializer, Serialize};

use crate::tmux::Key;

use super::path;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Session {
    pub name: String,
    #[serde(deserialize_with = "expand_tilde")]
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub on_create: Vec<OnCreateAction>,
}

impl Session {
    pub fn display_path(&self) -> String {
        if let Some(home_dir) = dirs::home_dir()
            && self.path.starts_with(&home_dir)
        {
            let stripped = self.path.strip_prefix(&home_dir).unwrap();
            return format!("~/{}", stripped.display());
        }

        self.path.display().to_string()
    }

    pub fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_relative() {
            self.path.join(path)
        } else {
            path.to_path_buf()
        }
    }

    pub fn on_create_path(&self, path: Option<&Path>) -> PathBuf {
        path.map_or_else(|| self.path.clone(), |path| self.resolve_path(path))
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
#[serde(rename_all = "snake_case")]
pub enum OnCreateAction {
    NewWindow(NewWindow),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord, Clone, Default)]
pub struct NewWindow {
    #[serde(
        default,
        deserialize_with = "expand_tilde_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub path: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Sessions(pub BTreeMap<Key, Session>);

impl Sessions {
    pub fn save(&self) -> Result<()> {
        let toml = toml_edit::ser::to_string(&self.0).into_diagnostic()?;
        let sessions_file = path::sessions_file();

        std::fs::write(sessions_file, toml).into_diagnostic()?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for Sessions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let max_width_key = self
            .0
            .keys()
            .map(|key| key.as_ref().len())
            .max()
            .unwrap_or(0);

        let max_width_name = self
            .0
            .values()
            .map(|session| session.name.len())
            .max()
            .unwrap_or(0);

        for (i, (key, session)) in self.0.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }

            write!(
                f,
                "{:<max_width_key$}  {:<max_width_name$}  {}",
                key.green(),
                session.name.blue(),
                session.display_path().dimmed(),
            )?;
        }

        Ok(())
    }
}

fn expand_tilde<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    Ok(path::expand_tilde(s.into()))
}

fn expand_tilde_option<'de, D>(deserializer: D) -> Result<Option<PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let path = Option::<PathBuf>::deserialize(deserializer)?;
    Ok(path.map(path::expand_tilde))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_session() {
        let toml_string = r#"
            d = { name = "dotfiles", path = "~/.dotfiles" }
        "#;

        let mut expected = BTreeMap::new();
        expected.insert(
            "d".into(),
            Session {
                name: "dotfiles".into(),
                path: path::expand_tilde("~/.dotfiles".into()),
                on_create: Vec::new(),
            },
        );

        let session = Sessions(toml_edit::de::from_str(toml_string).unwrap());

        assert_eq!(session, Sessions(expected));
    }

    #[test]
    fn test_display_sessions() {
        let mut sessions_map = BTreeMap::new();
        sessions_map.insert(
            "d".into(),
            Session {
                name: "dotfiles".into(),
                path: "/home/user/.dotfiles".into(),
                on_create: Vec::new(),
            },
        );
        sessions_map.insert(
            "p".into(),
            Session {
                name: "project".into(),
                path: "/home/user/projects/myproject".into(),
                on_create: Vec::new(),
            },
        );

        let sessions = Sessions(sessions_map);
        let output = format!("{sessions}");

        assert_eq!(
            output.as_str(),
            format!(
                "{}  {}  {}\n{}  {}  {}",
                "d".green(),
                "dotfiles".blue(),
                "/home/user/.dotfiles".dimmed(),
                "p".green(),
                "project ".blue(),
                "/home/user/projects/myproject".dimmed(),
            )
        );
    }

    #[test]
    fn test_display_empty_sessions() {
        let sessions = Sessions(BTreeMap::new());
        let output = format!("{sessions}");

        assert_eq!(output, "");
    }

    #[test]
    fn test_valid_session_with_on_create() {
        let toml_string = r#"
            q = { name = "qmk", path = "~/qmk_userspace", on_create = [
                { new_window = { path = "../qmk_firmware", name = "firmware" } }
            ] }
        "#;

        let mut expected = BTreeMap::new();
        expected.insert(
            "q".into(),
            Session {
                name: "qmk".into(),
                path: path::expand_tilde("~/qmk_userspace".into()),
                on_create: vec![OnCreateAction::NewWindow(NewWindow {
                    path: Some(PathBuf::from("../qmk_firmware")),
                    name: Some("firmware".into()),
                    command: None,
                })],
            },
        );

        let session = Sessions(toml_edit::de::from_str(toml_string).unwrap());

        assert_eq!(session, Sessions(expected));
    }

    #[test]
    fn test_resolve_relative_on_create_path() {
        let session = Session {
            name: "qmk".into(),
            path: PathBuf::from("/tmp/qmk_userspace"),
            on_create: Vec::new(),
        };

        assert_eq!(
            session.resolve_path(&PathBuf::from("../qmk_firmware")),
            PathBuf::from("/tmp/qmk_userspace/../qmk_firmware")
        );
    }

    #[test]
    fn test_on_create_path_defaults_to_session_path() {
        let session = Session {
            name: "qmk".into(),
            path: PathBuf::from("/tmp/qmk_userspace"),
            on_create: Vec::new(),
        };

        assert_eq!(
            session.on_create_path(None),
            PathBuf::from("/tmp/qmk_userspace")
        );
    }
}

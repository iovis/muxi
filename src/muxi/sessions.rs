use std::collections::BTreeMap;
use std::fmt;
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
            Key::parse("d").unwrap(),
            Session {
                name: "dotfiles".into(),
                path: path::expand_tilde("~/.dotfiles".into()),
            },
        );

        let session = Sessions(toml_edit::de::from_str(toml_string).unwrap());

        assert_eq!(session, Sessions(expected));
    }

    #[test]
    fn test_display_sessions() {
        let mut sessions_map = BTreeMap::new();
        sessions_map.insert(
            Key::parse("d").unwrap(),
            Session {
                name: "dotfiles".into(),
                path: "/home/user/.dotfiles".into(),
            },
        );
        sessions_map.insert(
            Key::parse("p").unwrap(),
            Session {
                name: "project".into(),
                path: "/home/user/projects/myproject".into(),
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
}

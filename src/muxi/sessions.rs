use std::collections::BTreeMap;
use std::path::PathBuf;

use color_eyre::Result;
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
        if let Some(home_dir) = std::env::home_dir()
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
        let toml = toml_edit::ser::to_string(&self.0)?;
        let sessions_file = path::sessions_file();

        std::fs::write(sessions_file, toml)?;

        Ok(())
    }

    pub fn to_list(&self) -> Vec<String> {
        let max_width_key = self.0.keys().map(|key| key.as_ref().len()).max().unwrap();

        let max_width_name = self
            .0
            .values()
            .map(|session| session.name.len())
            .max()
            .unwrap();

        let mut sessions_list: Vec<String> = Vec::with_capacity(self.0.len());

        for (key, session) in &self.0 {
            sessions_list.push(format!(
                "{:<max_width_key$}  {:<max_width_name$}  {}",
                key.green(),
                session.name.blue(),
                session.display_path().dimmed(),
            ));
        }

        sessions_list
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// Thank you ChatGPT
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
}

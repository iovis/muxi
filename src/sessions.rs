use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Deserializer, Serialize};

use crate::path;
use crate::tmux::TmuxKey;

pub type Sessions = HashMap<TmuxKey, Session>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Session {
    pub name: String,
    #[serde(deserialize_with = "expand_tilde")]
    pub path: PathBuf,
}

pub fn save(sessions: &Sessions) -> anyhow::Result<()> {
    let toml = toml_edit::easy::to_string(&sessions)?;
    let sessions_file = path::sessions_file();

    std::fs::write(sessions_file, toml)?;

    Ok(())
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

        let mut expected: Sessions = HashMap::new();
        expected.insert(
            TmuxKey::parse("d").unwrap(),
            Session {
                name: "dotfiles".into(),
                path: path::expand_tilde("~/.dotfiles".into()),
            },
        );

        let session: Sessions = toml::from_str(toml_string).unwrap();

        assert_eq!(session, expected);
    }
}

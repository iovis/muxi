use std::io;

use thiserror::Error;

use crate::settings::Settings;

use super::path;
use super::sessions::{self, Session, SessionParseError};

#[derive(Debug, Error)]
pub enum MuxiError {
    #[error("failed to open sessions file")]
    MissingSessionsFile(#[from] io::Error),
    #[error("failed to parse sessions file")]
    SessionParseError(#[from] SessionParseError),
}

#[derive(Debug)]
pub struct Muxi {
    pub settings: Settings,
}

impl Muxi {
    pub fn new() -> Self {
        let path = path::muxi_dir();
        let settings = Settings::new(&path).unwrap();

        Self { settings }
    }

    pub fn sessions(&self) -> Result<Vec<Session>, MuxiError> {
        let sessions_data = self.read_or_create_sessions_file()?;
        let sessions = sessions::from_config(sessions_data)?;

        Ok(sessions)
    }

    #[cfg(not(test))]
    fn read_or_create_sessions_file(&self) -> Result<String, MuxiError> {
        use std::fs::OpenOptions;
        use std::io::{BufReader, Read};

        std::fs::create_dir_all(path::muxi_dir())?;

        let sessions_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path::sessions_file())?;

        let mut reader = BufReader::new(sessions_file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents)?;

        Ok(contents)
    }

    // Mocks

    #[cfg(test)]
    #[allow(unused_variables)]
    fn read_or_create_sessions_file(&self) -> Result<String, MuxiError> {
        let sessions_data = r#"
            d dotfiles ~/.dotfiles

            k muxi ~/Sites/rust/muxi/
            Space tmux ~/Sites/tmux/
            M-n notes ~/Library/Mobile Documents/com~apple~CloudDocs/notes
        "#
        .to_string();

        Ok(sessions_data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expected_sessions_data() -> Vec<Session> {
        let home_dir = dirs::home_dir().unwrap();

        vec![
            Session {
                key: "d".into(),
                name: "dotfiles".into(),
                path: home_dir.join(".dotfiles"),
            },
            Session {
                key: "k".into(),
                name: "muxi".into(),
                path: home_dir.join("Sites/rust/muxi/"),
            },
            Session {
                key: "Space".into(),
                name: "tmux".into(),
                path: home_dir.join("Sites/tmux/"),
            },
            Session {
                key: "M-n".into(),
                name: "notes".into(),
                path: home_dir.join("Library/Mobile Documents/com~apple~CloudDocs/notes"),
            },
        ]
    }

    #[test]
    fn test_sessions() {
        let expected_sessions = expected_sessions_data();
        let sessions = Muxi::new().sessions().unwrap();

        assert_eq!(sessions, expected_sessions);
    }
}

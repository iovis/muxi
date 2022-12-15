use std::path::PathBuf;
use std::{env, io};

use thiserror::Error;

use super::sessions::{self, Session, SessionParseError};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to open sessions file")]
    MissingSessionsFile(#[from] io::Error),
    #[error("failed to parse sessions file")]
    SessionParseError(#[from] SessionParseError),
}

#[derive(Debug)]
pub struct Config {
    pub path: PathBuf,
    // TODO: settings
}

impl Config {
    pub fn new() -> Self {
        let path = Self::path_from_env();

        Self { path }
    }

    pub fn sessions(&self) -> Result<Vec<Session>, ConfigError>  {
        let sessions_data = self.read_or_create_sessions_file()?;
        let sessions = sessions::from_config(sessions_data)?;

        Ok(sessions)
    }

    pub fn sessions_path(&self) -> PathBuf {
        self.path.join("sessions.muxi")
    }

    fn path_from_env() -> PathBuf {
        let path = if let Ok(env_path) = env::var("MUXI_CONFIG_PATH") {
            PathBuf::from(env_path)
        } else if let Ok(env_path) = env::var("XDG_CONFIG_HOME") {
            PathBuf::from(env_path).join("muxi/")
        } else {
            PathBuf::from("~/.config/muxi/")
        };

        Self::handle_tilde_in_path(path)
    }

    fn handle_tilde_in_path(path: PathBuf) -> PathBuf {
        if !path.starts_with("~") {
            return path;
        }

        let path = path.strip_prefix("~").unwrap();

        let home_str = env::var("HOME").expect("$HOME is not defined");
        let home = PathBuf::from(home_str);

        home.join(path)
    }

    #[cfg(not(test))]
    fn read_or_create_sessions_file(&self) -> Result<String, ConfigError> {
        use std::fs::OpenOptions;
        use std::io::{BufReader, Read};

        std::fs::create_dir_all(&self.path)?;

        let sessions_file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(self.sessions_path())?;

        let mut reader = BufReader::new(sessions_file);
        let mut contents = String::new();

        reader.read_to_string(&mut contents)?;

        Ok(contents)
    }

    // Mocks

    #[cfg(test)]
    #[allow(unused_variables)]
    fn read_or_create_sessions_file(&self) -> Result<String, ConfigError> {
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

    #[test]
    fn test_path_muxi_env_set() {
        env::set_var("MUXI_CONFIG_PATH", "~/my/path");
        let home_dir = PathBuf::from(env::var("HOME").unwrap());

        let config = Config::new();

        assert_eq!(config.path, home_dir.join("my/path"));
    }

    #[test]
    fn test_path_xdg_home_env_set() {
        env::set_var("XDG_CONFIG_HOME", "~/xdg/path");
        let home_dir = PathBuf::from(env::var("HOME").unwrap());

        let config = Config::new();

        assert_eq!(config.path, home_dir.join("xdg/path/muxi/"));
    }

    #[test]
    fn test_path_fallback() {
        let home_dir = PathBuf::from(env::var("HOME").unwrap());

        let config = Config::new();

        assert_eq!(config.path, home_dir.join(".config/muxi/"));
    }

    fn expected_sessions_data() -> Vec<Session> {
        vec![
            Session {
                key: "d".into(),
                name: "dotfiles".into(),
                path: "~/.dotfiles".into(),
            },
            Session {
                key: "k".into(),
                name: "muxi".into(),
                path: "~/Sites/rust/muxi/".into(),
            },
            Session {
                key: "Space".into(),
                name: "tmux".into(),
                path: "~/Sites/tmux/".into(),
            },
            Session {
                key: "M-n".into(),
                name: "notes".into(),
                path: "~/Library/Mobile Documents/com~apple~CloudDocs/notes".into(),
            },
        ]
    }

    #[test]
    fn test_sessions() {
        let expected_sessions = expected_sessions_data();
        let sessions = Config::new().sessions().unwrap();

        assert_eq!(sessions, expected_sessions);
    }
}

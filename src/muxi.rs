use thiserror::Error;

use crate::path;
use crate::sessions::Sessions;

#[derive(Debug, Error)]
pub enum MuxiError {
    #[error("Error reading your sessions file")]
    IoError(#[from] std::io::Error),
    #[error("Error parsing your sessions file")]
    ParseError(#[from] toml::de::Error),
}

#[derive(Debug)]
pub struct Muxi {
    pub sessions: Sessions,
}

impl Muxi {
    pub fn new() -> Result<Self, MuxiError> {
        let sessions_file = path::sessions_file();

        if std::fs::metadata(&sessions_file).is_err() {
            std::fs::create_dir_all(path::muxi_dir())?;
            std::fs::File::create(&sessions_file)?;
        }

        let sessions_data = std::fs::read_to_string(sessions_file)?;
        let sessions = toml::from_str(&sessions_data)?;

        Ok(Self { sessions })
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::io::Write;

    use uuid::Uuid;

    use crate::sessions::Session;

    use super::*;

    fn with_config<F>(config: &str, test: F)
    where
        F: Fn(Sessions) + std::panic::RefUnwindSafe,
    {
        // Create tmp folder
        let pwd = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&pwd).unwrap();

        // Create sessions file
        let path = pwd.join("sessions.toml");
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(config.as_bytes()).unwrap();

        // Set $MUXI_CONFIG_PATH to current folder and load config
        temp_env::with_var("MUXI_CONFIG_PATH", Some(pwd.clone()), || {
            let muxi = Muxi::new();

            // Cleanup before test, in case of panic
            std::fs::remove_dir_all(&pwd).unwrap();

            test(muxi.unwrap().sessions);
        });
    }

    fn expected_sessions() -> Sessions {
        vec![
            (
                "d".try_into().unwrap(),
                Session {
                    name: "dotfiles".into(),
                    path: path::expand_tilde("~/.dotfiles".into()),
                },
            ),
            (
                "k".try_into().unwrap(),
                Session {
                    name: "muxi".into(),
                    path: path::expand_tilde("/home/user/muxi/".into()),
                },
            ),
            (
                "Space".try_into().unwrap(),
                Session {
                    name: "tmux".into(),
                    path: path::expand_tilde("~/Sites/tmux/".into()),
                },
            ),
            (
                "M-n".try_into().unwrap(),
                Session {
                    name: "notes".into(),
                    path: path::expand_tilde(
                        "~/Library/Mobile Documents/com~apple~CloudDocs/notes".into(),
                    ),
                },
            ),
        ]
        .into_iter()
        .collect()
    }

    #[test]
    fn test_normal_sessions() {
        let config = r#"
            d = { name = "dotfiles", path = "~/.dotfiles" }
            k = { name = "muxi", path = "/home/user/muxi/" }
            Space = { name = "tmux", path = "~/Sites/tmux/" }
            M-n = { name = "notes", path = "~/Library/Mobile Documents/com~apple~CloudDocs/notes" }
        "#;

        let expected_sessions = expected_sessions();

        with_config(config, |sessions| {
            assert_eq!(sessions, expected_sessions);
        });
    }
}

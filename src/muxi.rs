use config::{Config, ConfigError, File};

use crate::path;
use crate::sessions::Sessions;

#[derive(Debug)]
pub struct Muxi {
    pub sessions: Sessions,
}

impl Muxi {
    pub fn new() -> Result<Self, ConfigError> {
        let sessions_file = path::sessions_file();

        if std::fs::metadata(&sessions_file).is_err() {
            std::fs::create_dir_all(path::muxi_dir()).unwrap();
            std::fs::File::create(&sessions_file).unwrap();
        }

        let sessions: Sessions = Config::builder()
            .add_source(File::from(sessions_file).required(true))
            .build()?
            .try_deserialize()?;

        Ok(Self { sessions })
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::sessions::Session;

    use super::*;

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
        // Create configuration file
        let config = r#"
            d = { name = "dotfiles", path = "~/.dotfiles" }
            k = { name = "muxi", path = "/home/user/muxi/" }
            Space = { name = "tmux", path = "~/Sites/tmux/" }
            M-n = { name = "notes", path = "~/Library/Mobile Documents/com~apple~CloudDocs/notes" }
        "#;

        let mut file = std::fs::File::create("sessions.toml").unwrap();
        file.write_all(config.as_bytes()).unwrap();

        // Set $MUXI_CONFIG_PATH to current folder and load config
        let pwd = std::env::var("PWD").unwrap();

        temp_env::with_var("MUXI_CONFIG_PATH", Some(pwd), || {
            let expected_sessions = expected_sessions();
            let sessions = Muxi::new().unwrap().sessions;

            // Cleanup before test, in case of panic
            std::fs::remove_file("sessions.toml").unwrap();

            assert_eq!(sessions, expected_sessions);
        });
    }
}

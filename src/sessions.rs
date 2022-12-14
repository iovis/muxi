use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, PartialEq, Eq)]
pub struct Session {
    pub key: String,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum SessionParseError {
    #[error("empty line")]
    EmptyLine,
    #[error("missing session name")]
    MissingName,
    #[error("missing session path")]
    MissingPath,
}

impl TryFrom<String> for Session {
    type Error = SessionParseError;

    // TODO: Allow comments with '#'
    // TODO: Use `nom`?
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut columns = value.split_whitespace();

        let key = columns.next().ok_or(SessionParseError::EmptyLine)?.into();
        let name = columns.next().ok_or(SessionParseError::MissingName)?.into();

        let path: PathBuf = columns.collect::<Vec<_>>().join(" ").into();

        if path.as_os_str().is_empty() {
            return Err(SessionParseError::MissingPath);
        }

        Ok(Self { key, name, path })
    }
}

pub fn from_config<T: AsRef<str>>(data: T) -> Result<Vec<Session>, SessionParseError> {
    let mut sessions = vec![];

    for (i, line) in data.as_ref().lines().enumerate() {
        match line.to_owned().try_into() {
            Ok(session) => sessions.push(session),
            Err(SessionParseError::EmptyLine) => continue,
            Err(e) => {
                eprintln!("Failed to parse line {}: \"{}\"", i, line);
                return Err(e);
            }
        }
    }

    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parses_normal_line() {
        let line = "d dotfiles ~/.dotfiles".to_string();
        let session: Session = line.try_into().unwrap();

        assert_eq!(
            session,
            Session {
                key: "d".into(),
                name: "dotfiles".into(),
                path: "~/.dotfiles".into(),
            }
        );
    }

    #[test]
    fn test_parses_line_with_multichar_binding() {
        let line = "Space tmux ~/Sites/rust/tmux/".to_string();
        let session: Session = line.try_into().unwrap();

        assert_eq!(
            session,
            Session {
                key: "Space".into(),
                name: "tmux".into(),
                path: "~/Sites/rust/tmux/".into(),
            }
        );
    }

    #[test]
    fn test_parses_line_with_multiple_spaces() {
        let line = "Space     tmux     ~/Sites/rust/tmux/".to_string();
        let session: Session = line.try_into().unwrap();

        assert_eq!(
            session,
            Session {
                key: "Space".into(),
                name: "tmux".into(),
                path: "~/Sites/rust/tmux/".into(),
            }
        );
    }

    #[test]
    fn test_parses_line_with_spaces_in_path() {
        let line = "M-n notes ~/Library/Mobile Documents/com~apple~CloudDocs/notes".to_string();
        let session: Session = line.try_into().unwrap();

        assert_eq!(
            session,
            Session {
                key: "M-n".into(),
                name: "notes".into(),
                path: "~/Library/Mobile Documents/com~apple~CloudDocs/notes".into(),
            }
        );
    }

    type SessionResult = Result<Session, SessionParseError>;

    #[test]
    fn test_errors_on_missing_key() {
        let line = "".to_string();
        let result: SessionResult = line.try_into();

        assert_eq!(result, Err(SessionParseError::EmptyLine));
    }

    #[test]
    fn test_errors_on_missing_session_name() {
        let line = "d".to_string();
        let result: SessionResult = line.try_into();

        assert_eq!(result, Err(SessionParseError::MissingName));
    }

    #[test]
    fn test_errors_on_missing_session_path() {
        let line = "d dotfiles".to_string();
        let result: SessionResult = line.try_into();

        assert_eq!(result, Err(SessionParseError::MissingPath));
    }

    fn raw_sessions_data() -> String {
        r#"
        d dotfiles ~/.dotfiles
        k muxi ~/Sites/rust/muxi/
        Space tmux ~/Sites/tmux/
        M-n notes ~/Library/Mobile Documents/com~apple~CloudDocs/notes
        "#
        .to_string()
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
    fn test_from_config() {
        let expected_sessions = expected_sessions_data();

        let config = raw_sessions_data();
        let sessions = from_config(config).unwrap();

        assert_eq!(expected_sessions, sessions);
    }
}

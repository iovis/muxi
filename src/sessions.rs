use std::path::PathBuf;

use anyhow::{bail, Context};

#[derive(Debug, PartialEq)]
struct Session {
    key: String,
    name: String,
    path: PathBuf,
}

impl TryFrom<String> for Session {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let mut columns = value.split_whitespace();

        let key: String = columns.next().context("Failed to parse key")?.into();
        let name: String = columns.next().context("Failed to parse name")?.into();
        let path: PathBuf = columns.collect::<Vec<_>>().join(" ").into();

        if path.as_os_str().is_empty() {
            bail!("Failed to parse path");
        }

        Ok(Self { key, name, path })
    }
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

    #[test]
    fn test_errors_on_missing_key() {
        let line = "".to_string();
        let result: anyhow::Result<Session> = line.try_into();
        let error_msg = format!("{}", result.unwrap_err());

        assert_eq!(error_msg, "Failed to parse key");
    }

    #[test]
    fn test_errors_on_missing_session_name() {
        let line = "d".to_string();
        let result: anyhow::Result<Session> = line.try_into();
        let error_msg = format!("{}", result.unwrap_err());

        assert_eq!(error_msg, "Failed to parse name");
    }

    #[test]
    fn test_errors_on_missing_session_path() {
        let line = "d dotfiles".to_string();
        let result: anyhow::Result<Session> = line.try_into();
        let error_msg = format!("{}", result.unwrap_err());

        assert_eq!(error_msg, "Failed to parse path");
    }

    fn settings_data() -> String {
        r#"
            d dotfiles ~/.dotfiles
            k muxi ~/Sites/rust/muxi/
            Space tmux ~/Sites/rust/tmux/
            M-n notes ~/Library/Mobile Documents/com~apple~CloudDocs/notes
            "#
        .to_string()
    }
}

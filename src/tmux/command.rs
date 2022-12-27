use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::sessions::{Session, Sessions};
use crate::settings::Settings;

use super::PopupOptions;

type TmuxResult<T> = Result<T, TmuxError>;

#[derive(Debug, Error)]
pub enum TmuxError {
    #[error("muxi needs to be executed within a tmux session")]
    NotInTmux(#[from] std::env::VarError),
    #[error("Failed to run command")]
    CommandError(#[from] io::Error),
    #[error("Failed to clear muxi table: `{0}`")]
    ClearTable(String),
    #[error("{0}\nin: {1}")]
    BindKey(String, String),
    #[error("Failed to parse tmux output: `{0}`")]
    ParseError(#[from] FromUtf8Error),
}

#[derive(Debug)]
pub struct Tmux {
    settings: Settings,
}

impl Tmux {
    pub fn new(settings: Settings) -> TmuxResult<Self> {
        std::env::var("TMUX")?;

        Ok(Self { settings })
    }

    pub fn bind_sessions(&self, sessions: &Sessions) -> TmuxResult<()> {
        self.clear_table()?;
        self.bind_table_prefix()?;
        self.settings_bindings()?;

        for (key, session) in sessions {
            self.bind_session(key.as_ref(), session)?;
        }

        Ok(())
    }

    fn clear_table(&self) -> TmuxResult<()> {
        // tmux unbind -aq -T muxi
        let output = Command::new("tmux")
            .arg("unbind")
            .arg("-aq")
            .arg("-T")
            .arg("muxi")
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(TmuxError::ClearTable(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }

    fn bind_table_prefix(&self) -> TmuxResult<()> {
        // tmux bind <settings.prefix> switch-client -T muxi
        let mut command = Command::new("tmux");
        command.arg("bind");

        // Bind at root table if no tmux prefix
        if !&self.settings.tmux_prefix {
            command.arg("-n");
        }

        command
            .arg(self.settings.muxi_prefix.as_ref())
            .arg("switch-client")
            .arg("-T")
            .arg("muxi");

        let output = command.output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(TmuxError::BindKey(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
                format!("muxi_prefix: {}", self.settings.muxi_prefix),
            ))
        }
    }

    fn settings_bindings(&self) -> TmuxResult<()> {
        for (key, binding) in &self.settings.bindings {
            let mut command = Command::new("tmux");

            command.arg("bind").arg("-T").arg("muxi").arg(key.as_ref());

            if let Some(PopupOptions {
                title,
                width,
                height,
            }) = &binding.popup
            {
                command
                    .arg("popup")
                    .arg("-w")
                    .arg(width)
                    .arg("-h")
                    .arg(height)
                    .arg("-b")
                    .arg("rounded")
                    .arg("-E");

                if let Some(title) = title {
                    command.arg("-T").arg(title);
                }
            } else {
                command.arg("run");
            }

            command.arg(&binding.command);

            let output = command.output()?;

            if !output.status.success() {
                return Err(TmuxError::BindKey(
                    String::from_utf8_lossy(&output.stderr).trim().to_string(),
                    format!("{} = {:?}", key, binding),
                ));
            }
        }

        Ok(())
    }

    fn bind_session(&self, key: &str, session: &Session) -> TmuxResult<()> {
        // tmux bind -T muxi <session_key> new-session -A -s <name> -c "<path>"
        let output = Command::new("tmux")
            .arg("bind")
            .arg("-T")
            .arg("muxi")
            .arg(key)
            .arg("new-session")
            .arg("-A")
            .arg("-s")
            .arg(&session.name)
            .arg("-c")
            .arg(&session.path)
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(TmuxError::BindKey(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
                format!("{} = {:?}", key, session),
            ))
        }
    }
}

pub fn current_session_name() -> Option<String> {
    // tmux display-message -p '#S'
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("#S")
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8(output.stdout).ok()?.trim().into())
    } else {
        None
    }
}

pub fn current_session_path() -> Option<PathBuf> {
    // tmux display-message -p '#{session_path}'
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("#{session_path}")
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8(output.stdout).ok()?.trim().into())
    } else {
        None
    }
}

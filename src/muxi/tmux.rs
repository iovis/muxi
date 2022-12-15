use std::io;
use std::process::Command;

use thiserror::Error;

use super::sessions::Session;

type TmuxResult<T> = Result<T, TmuxError>;

#[derive(Debug, Error)]
pub enum TmuxError {
    #[error("muxi needs to be executed within a tmux session")]
    NotInTmux(#[from] std::env::VarError),
    #[error("Failed to run command")]
    CommandError(#[from] io::Error),
    #[error("Failed to clear muxi table: `{0}`")]
    ClearTable(String),
    #[error("Failed to bind key: `{0}`")]
    BindKey(String),
}

#[derive(Debug)]
pub struct Tmux {}

impl Tmux {
    pub fn new() -> TmuxResult<Self> {
        std::env::var("TMUX")?;

        Ok(Self {})
    }

    pub fn bind_sessions(&self, sessions: &[Session]) -> TmuxResult<()> {
        self.clear_table()?;
        self.bind_table_prefix()?;
        self.default_bindings()?;

        for session in sessions {
            self.bind_session(session)?;
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
        let output = Command::new("tmux")
            .arg("bind")
            .arg("g")
            .arg("switch-client")
            .arg("-T")
            .arg("muxi")
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(TmuxError::BindKey(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }

    fn default_bindings(&self) -> TmuxResult<()> {
        // bind -T muxi e popup -w 80% -h 80% -b rounded -E "muxi edit"
        let output = Command::new("tmux")
            .arg("bind")
            .arg("-T")
            .arg("muxi")
            .arg("e")
            .arg("popup")
            .arg("-w")
            .arg("80%")
            .arg("-h")
            .arg("80%")
            .arg("-b")
            .arg("rounded")
            .arg("-T")
            .arg(" muxi ")
            .arg("-E")
            .arg("muxi edit")
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(TmuxError::BindKey(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ))
        }
    }

    fn bind_session(&self, session: &Session) -> TmuxResult<()> {
        // tmux bind -T muxi <session_key> new-session -A -s <name> -c "<path>"
        let output = Command::new("tmux")
            .arg("bind")
            .arg("-T")
            .arg("muxi")
            .arg(&session.key)
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
            ))
        }
    }
}

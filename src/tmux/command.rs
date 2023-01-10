use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::string::FromUtf8Error;

use thiserror::Error;

use crate::sessions::{Session, Sessions};
use crate::settings::Settings;

use super::Popup;

type TmuxResult<T> = Result<T, TmuxError>;

#[derive(Debug, Error)]
pub enum TmuxError {
    #[error("muxi needs to be executed within a tmux session")]
    NotInTmux(#[from] std::env::VarError),
    #[error("failed to run command")]
    CommandError(#[from] io::Error),
    #[error("failed to clear muxi table: `{0}`")]
    ClearTable(String),
    #[error("{0}\nin: {1}")]
    BindKey(String, String),
    #[error("failed to parse tmux output: `{0}`")]
    ParseError(#[from] FromUtf8Error),
    #[error("failed to switch to sesion {0}: `{1}`")]
    SwitchError(String, String),
}

/// Checks if it's run within a tmux session
#[inline]
pub fn within_tmux() -> TmuxResult<()> {
    // TODO: attribute macro?
    std::env::var("TMUX")?;

    Ok(())
}

/// Generate all muxi bindings
pub fn create_muxi_bindings(settings: &Settings, sessions: &Sessions) -> TmuxResult<()> {
    within_tmux()?;

    clear_muxi_table()?;
    bind_table_prefix(settings)?;

    if settings.uppercase_overrides {
        bind_uppercase_overrides()?;
    }

    settings_bindings(settings)?;
    bind_sessions(sessions)?;

    Ok(())
}

/// Runs `tmux unbind -aq -T muxi`
fn clear_muxi_table() -> TmuxResult<()> {
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

/// tmux bind <settings.prefix> switch-client -T muxi
fn bind_table_prefix(settings: &Settings) -> TmuxResult<()> {
    let mut command = Command::new("tmux");
    command.arg("bind");

    // Bind at root table if no tmux prefix
    if !settings.tmux_prefix {
        command.arg("-n");
    }

    command
        .arg(settings.muxi_prefix.as_ref())
        .arg("switch-client")
        .arg("-T")
        .arg("muxi");

    let output = command.output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(TmuxError::BindKey(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
            format!("muxi_prefix: {}", settings.muxi_prefix),
        ))
    }
}

/// Generates bindings defined in the settings
fn settings_bindings(settings: &Settings) -> TmuxResult<()> {
    for (key, binding) in &settings.bindings {
        let mut command = Command::new("tmux");

        command.arg("bind").arg("-T").arg("muxi").arg(key.as_ref());

        if let Some(Popup {
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
                format!("{key} = {binding:?}"),
            ));
        }
    }

    Ok(())
}

/// Generates bindings for all the muxi sessions
/// Equivalent to: `tmux bind -T muxi <session_key> new-session -A -s <name> -c "<path>"`
fn bind_sessions(sessions: &Sessions) -> TmuxResult<()> {
    let mut tmux_command = Command::new("tmux");

    for (key, session) in sessions {
        tmux_command
            .arg("bind")
            .arg("-T")
            .arg("muxi")
            .arg(key.as_ref())
            .arg("new-session")
            .arg("-A")
            .arg("-s")
            .arg(&session.name)
            .arg("-c")
            .arg(&session.path)
            .arg(";");
    }

    let output = tmux_command.output()?;

    if !output.status.success() {
        return Err(TmuxError::BindKey(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
            "Error binding sessions".to_string(),
        ));
    }

    Ok(())
}

/// Generates uppercase overrides
/// Equivalent to: `tmux bind -T muxi <uppercase_letter> run-shell "muxi sessions set j && tmux display 'bound current session to j'"`
fn bind_uppercase_overrides() -> TmuxResult<()> {
    let mut tmux_command = Command::new("tmux");

    for key in 'A'..='Z' {
        let command = format!(
            "muxi sessions set {} && tmux display 'bound current session to {}'",
            key.to_lowercase(),
            key.to_lowercase()
        );

        tmux_command
            .arg("bind")
            .arg("-T")
            .arg("muxi")
            .arg(key.to_string())
            .arg("run")
            .arg(&command)
            .arg(";");
    }

    let output = tmux_command.output()?;

    if !output.status.success() {
        return Err(TmuxError::BindKey(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
            "Error binding uppercase overrides".to_string(),
        ));
    }

    Ok(())
}

/// Captures de current session's name
/// Equivalent to: `tmux display-message -p '#S'`
pub fn current_session_name() -> Option<String> {
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

/// Captures de current session's path
/// Equivalent to: `tmux display-message -p '#{session_path}'`
pub fn current_session_path() -> Option<PathBuf> {
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

/// Check if a tmux session exists
/// Equivalent to: `tmux has-session -t <session_name>`
pub fn has_session(session: &Session) -> bool {
    let output = Command::new("tmux")
        .arg("has-session")
        .arg("-t")
        .arg(&session.name)
        .output();

    if let Ok(output) = output {
        output.status.success()
    } else {
        false
    }
}

/// Create tmux session
/// Equivalent to: `tmux new-session -d -s <session_name> -c <session_path>`
pub fn create_session(session: &Session) -> bool {
    let output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(&session.name)
        .arg("-c")
        .arg(&session.path)
        .output();

    if let Ok(output) = output {
        output.status.success()
    } else {
        false
    }
}

/// Switch to tmux session
/// Equivalent to: `tmux switch-client -t <session_name>`
pub fn switch_to(session: &Session) -> TmuxResult<()> {
    let output = Command::new("tmux")
        .arg("switch-client")
        .arg("-t")
        .arg(&session.name)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(TmuxError::SwitchError(
            session.name.to_string(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

use std::path::PathBuf;
use std::process::Command;

use crate::sessions::{Session, Sessions};

use super::{Error, TmuxResult};

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
        Err(Error::Switch(
            session.name.to_string(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

/// Tmux session menu picker
/// Equivalent to: `tmux display-menu -T ' muxi ' <session_name> <key> "run {switch_to_session}"`
pub fn sessions_menu(sessions: &Sessions) -> TmuxResult<()> {
    let mut tmux = Command::new("tmux"); // Prevent 'temporary value dropped while borrowed'
    let tmux_command = tmux
        .arg("display-menu")
        .arg("-T")
        .arg("#[align=left fg=green] muxi ");

    for (key, session) in sessions {
        tmux_command
            .arg(format!("#[fg=blue]{}", &session.name))
            .arg(key.as_ref())
            .arg(format!("run -b 'muxi sessions switch {key}'"));
    }

    let output = tmux_command.output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::DisplayMenu(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

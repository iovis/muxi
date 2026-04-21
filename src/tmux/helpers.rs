use std::path::PathBuf;
use std::process::Command;

use crate::muxi::{NewWindow, OnCreateAction, Session, Sessions};

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

/// Captures de current pane's path
/// Equivalent to: `tmux display-message -p '#{pane_current_path}'`
pub fn current_pane_path() -> Option<PathBuf> {
    let output = Command::new("tmux")
        .arg("display-message")
        .arg("-p")
        .arg("#{pane_current_path}")
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
pub fn create_session(session: &Session) -> TmuxResult<()> {
    let output = Command::new("tmux")
        .arg("new-session")
        .arg("-d")
        .arg("-s")
        .arg(&session.name)
        .arg("-c")
        .arg(&session.path)
        .output()?;

    if output.status.success() {
        run_on_create(session)?;
        Ok(())
    } else {
        Err(Error::Create(
            session.name.clone(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
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
            session.name.clone(),
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

    // Define tmux menu items: {session_name} {key} {command}
    // Ex: "#[blue]dotfiles" "d" "run -b 'muxi sessions switch d'"
    for (key, session) in &sessions.0 {
        tmux_command
            .arg(format!("#[fg=blue]{}", &session.name))
            .arg(key.as_ref())
            .arg(format!("run -b '{}'", switch_session_command(key.as_ref())));
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

pub fn switch_session_command(key: &str) -> String {
    format!("muxi sessions switch {key}")
}

fn run_on_create(session: &Session) -> TmuxResult<()> {
    for action in &session.on_create {
        match action {
            OnCreateAction::NewWindow(new_window) => create_window(session, new_window)?,
        }
    }

    Ok(())
}

fn create_window(session: &Session, new_window: &NewWindow) -> TmuxResult<()> {
    let mut command = Command::new("tmux");
    command
        .arg("new-window")
        .arg("-d")
        .arg("-t")
        .arg(format!("{}:", session.name));

    if let Some(name) = &new_window.name {
        command.arg("-n").arg(name);
    }

    command
        .arg("-c")
        .arg(session.on_create_path(new_window.path.as_deref()));

    if let Some(command_value) = &new_window.command {
        command.arg(command_value);
    }

    let output = command.output()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(Error::NewWindow(
            session.name.clone(),
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

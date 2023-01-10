use std::process::Command;

use crate::sessions::Sessions;
use crate::settings::Settings;

use super::{Error, Popup, TmuxResult};

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

    let mut tmux_command = Command::new("tmux");

    bind_table_prefix(&mut tmux_command, settings);

    if settings.uppercase_overrides {
        bind_uppercase_overrides(&mut tmux_command);
    }

    bind_settings(&mut tmux_command, settings);
    bind_sessions(&mut tmux_command, sessions);

    let output = tmux_command.output()?;

    if !output.status.success() {
        return Err(Error::Init(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(())
}

/// Runs `tmux unbind -aq -T muxi`
/// Cannot be ran alongside binding creation because if fails to bind anything
#[inline]
fn clear_muxi_table() -> TmuxResult<()> {
    let output = Command::new("tmux")
        .arg("unbind")
        .arg("-aq")
        .arg("-T")
        .arg("muxi")
        .output()?;

    if !output.status.success() {
        return Err(Error::Init(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ));
    }

    Ok(())
}

/// tmux bind <settings.prefix> switch-client -T muxi
#[inline]
fn bind_table_prefix(tmux_command: &mut Command, settings: &Settings) {
    tmux_command.arg("bind");

    // Bind at root table if no tmux prefix
    if !settings.tmux_prefix {
        tmux_command.arg("-n");
    }

    tmux_command
        .arg(settings.muxi_prefix.as_ref())
        .arg("switch-client")
        .arg("-T")
        .arg("muxi")
        .arg(";");
}

/// Generates bindings defined in the settings
#[inline]
fn bind_settings(tmux_command: &mut Command, settings: &Settings) {
    for (key, binding) in &settings.bindings {
        tmux_command
            .arg("bind")
            .arg("-T")
            .arg("muxi")
            .arg(key.as_ref());

        if let Some(Popup {
            title,
            width,
            height,
        }) = &binding.popup
        {
            tmux_command
                .arg("popup")
                .arg("-w")
                .arg(width)
                .arg("-h")
                .arg(height)
                .arg("-b")
                .arg("rounded")
                .arg("-E");

            if let Some(title) = title {
                tmux_command.arg("-T").arg(title);
            }
        } else {
            tmux_command.arg("run");
        }

        tmux_command.arg(&binding.command).arg(";");
    }
}

/// Generates bindings for all the muxi sessions
/// Equivalent to: `tmux bind -T muxi <session_key> new-session -A -s <name> -c "<path>"`
#[inline]
fn bind_sessions(tmux_command: &mut Command, sessions: &Sessions) {
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
}

/// Generates uppercase overrides
/// Equivalent to: `tmux bind -T muxi <uppercase_letter> run-shell "muxi sessions set j && tmux display 'bound current session to j'"`
#[inline]
fn bind_uppercase_overrides(tmux_command: &mut Command) {
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
}

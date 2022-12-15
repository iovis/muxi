mod cli;
pub use cli::*;

use std::process::{Command, Stdio};

use anyhow::bail;

mod config;
mod sessions;
mod tmux;

use self::config::Config;
use self::tmux::Tmux;

pub fn init() -> anyhow::Result<()> {
    let config = Config::new();
    let sessions = config.sessions()?;
    let tmux = Tmux::new()?;

    tmux.bind_sessions(&sessions)?;

    Ok(())
}

pub fn edit() -> anyhow::Result<()> {
    let sessions_file = Config::new().sessions_path();

    // Get the value of the $EDITOR environment variable
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let mut command = Command::new(editor);
    command.arg(sessions_file);

    // Set stdin, stdout, and stderr to be the same as the current process
    command
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    // Spawn the editor process and wait for it to finish
    let status = command
        .spawn()
        .expect("Failed to spawn editor process")
        .wait()
        .expect("Failed to wait for editor process");

    // Check the exit status of the editor process
    if status.success() {
        init()
    } else {
        bail!("Edit failed")
    }
}

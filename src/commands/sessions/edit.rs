use std::process::{Command, Stdio};

use anyhow::bail;

use crate::muxi::Muxi;

pub fn edit() -> anyhow::Result<()> {
    let sessions_file = Muxi::new().sessions_path();

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
        super::init()
    } else {
        bail!("Edit failed")
    }
}

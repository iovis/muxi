use std::path::Path;
use std::process::{Command, Stdio};

use color_eyre::eyre::bail;
use color_eyre::Result;

pub fn open_editor_for(path: &Path) -> Result<()> {
    // Get the value of the $EDITOR environment variable
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    let mut command = Command::new(editor);
    command.arg(path);

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

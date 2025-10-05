use std::path::Path;
use std::process::{Command, Stdio};

use color_eyre::Result;
use color_eyre::eyre::bail;

use crate::muxi::Settings;

pub fn open_editor_for(path: &Path, editor_args: &[String]) -> Result<()> {
    let settings = Settings::from_lua()?;
    let editor = settings
        .editor
        .command
        .unwrap_or_else(|| std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string()));

    let mut command = Command::new(editor);

    command
        .args(settings.editor.args)
        .args(editor_args)
        .arg(path);

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

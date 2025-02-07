use std::process::{Command, Stdio};

use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::muxi::Muxi;
use crate::tmux::Key;

pub fn spawn(fzf_args: &[String]) -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    if sessions.is_empty() {
        println!("{}", "No sessions defined!".red());
        return Ok(());
    }

    let mut fzf_command = Command::new("tmux");

    fzf_command
        .arg("popup")
        .arg("-w")
        .arg("80%")
        .arg("-h")
        .arg("80%")
        .arg("-b")
        .arg("rounded")
        .arg("-E")
        .arg("-S")
        .arg("fg=#414559")
        .arg("fzf")
        .arg("--reverse")
        .arg("--info")
        .arg("hidden")
        .arg("--header")
        .arg(format!(
            "  <{}> to {} | <{}> to {} | <{}> to {}",
            "ctrl-x".yellow(),
            "delete".red(),
            "ctrl-r".yellow(),
            "rename".red(),
            "ctrl-g".yellow(),
            "config".red()
        ))
        .arg("--prompt")
        .arg("muxi> ")
        .arg("--bind")
        .arg("start:reload:muxi sessions list")
        .arg("--bind")
        .arg("change:first")
        .arg("--bind")
        .arg("enter:execute(muxi sessions switch {1})+abort")
        .arg("--bind")
        .arg("x,ctrl-x:execute-silent(muxi sessions delete {1})+reload(muxi sessions list)")
        .arg("--bind")
        .arg("r,ctrl-r:execute(muxi sessions edit)+reload(muxi sessions list)")
        .arg("--bind")
        .arg("g,ctrl-g:execute(muxi config edit)+reload(muxi sessions list)")
        .arg("--preview")
        .arg("tmux capture-pane -ep -t '{2}:'")
        .arg("--preview-window")
        .arg("down,60%")
        .arg("--bind")
        .arg("alt-p:toggle-preview")
        .arg("--bind")
        .arg("alt-r:change-preview-window(right|down)");

    // Bind muxi keys to fzf
    let keys = sessions
        .0
        .keys()
        .map(Key::to_string)
        .collect::<Vec<_>>()
        .join(",");

    fzf_command.arg("--no-input").arg("--bind").arg(format!(
        "j:down,k:up,q:abort,i,/:show-input+unbind(j,k,q,i,/,x,{keys})"
    ));

    for key in sessions.0.keys() {
        fzf_command
            .arg("--bind")
            .arg(format!("{key}:execute(muxi sessions switch {key})+abort"));
    }

    fzf_command
        .args(fzf_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(())
}

use std::process::{Command, Stdio};

use color_eyre::Result;
use owo_colors::OwoColorize;

pub fn spawn() -> Result<()> {
    Command::new("fzf-tmux")
        .arg("-p80%,80%")
        .arg("--reverse")
        .arg("--ansi")
        .arg("--info")
        .arg("hidden")
        .arg("--header")
        .arg(format!(":: <{}> to {}", "ctrl-x".yellow(), "delete".red()))
        .arg("--prompt")
        .arg("muxi> ")
        .arg("--bind")
        .arg("start:reload:muxi sessions list")
        .arg("--bind")
        .arg("change:first")
        .arg("--bind")
        .arg("enter:execute(muxi sessions switch {1})+abort")
        .arg("--bind")
        .arg("ctrl-x:execute-silent(muxi sessions delete {1})+reload(muxi sessions list)")
        .arg("--preview")
        .arg("tmux capture-pane -ep -t '{2}:'")
        .arg("--preview-window")
        .arg("down,60%")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(())
}

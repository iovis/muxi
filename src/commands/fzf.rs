use std::process::{Command, Stdio};

use color_eyre::Result;
use owo_colors::OwoColorize;

pub fn spawn(fzf_args: &[String]) -> Result<()> {
    Command::new("tmux")
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
            ":: <{}> to {} | <{}> to {}",
            "ctrl-x".yellow(),
            "delete".red(),
            "ctrl-r".yellow(),
            "edit".red()
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
        .arg("ctrl-x:execute-silent(muxi sessions delete {1})+reload(muxi sessions list)")
        .arg("--bind")
        .arg("ctrl-r:execute(muxi sessions edit)+reload(muxi sessions list)")
        .arg("--preview")
        .arg("tmux capture-pane -ep -t '{2}:'")
        .arg("--preview-window")
        .arg("down,60%")
        .args(fzf_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(())
}

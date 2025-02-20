use std::process::{Command, Stdio};

use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::muxi::{self, Muxi, path};
use crate::tmux::Key;

pub fn spawn(fzf_args: &[String]) -> Result<()> {
    let sessions = Muxi::new()?.sessions;
    let settings = muxi::parse_settings(&path::muxi_dir())?;

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
            "  <{}> to {} | <{}> to {} | <{}> to {}\n ",
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
        .arg("ctrl-x:execute-silent(muxi sessions delete {1})+reload(muxi sessions list)")
        .arg("--bind")
        .arg("ctrl-r:execute(muxi sessions edit)+reload(muxi sessions list)")
        .arg("--bind")
        .arg("ctrl-g:execute(muxi config edit)+reload(muxi sessions list)")
        .arg("--preview")
        .arg("tmux capture-pane -ep -t '{2}:'")
        .arg("--preview-window")
        .arg("down,60%")
        .arg("--bind")
        .arg("alt-p:toggle-preview")
        .arg("--bind")
        .arg("alt-r:change-preview-window(right|down)");

    // Hide fuzzy prompt
    if !settings.fzf.input {
        fzf_command.arg("--no-input");

        // vim bindings
        fzf_command
            .arg("--bind")
            .arg("j:down,k:up,q:abort")
            .arg("--bind")
            .arg("x:execute-silent(muxi sessions delete {1})+reload(muxi sessions list)")
            .arg("--bind")
            .arg("e:execute(muxi sessions edit)+reload(muxi sessions list)")
            .arg("--bind")
            .arg("c:execute(muxi config edit)+reload(muxi sessions list)")
            .arg("--bind")
            .arg("p:toggle-preview")
            .arg("--bind")
            .arg("r:change-preview-window(right|down)")
            .arg("--bind")
            .arg("i,/:show-input+unbind(j,k,q,x,e,c,p,r,i,/)");

        // Bind muxi keys to fzf
        let muxi_session_keys = sessions.0.keys().map(Key::to_string).collect::<Vec<_>>();

        for key in &muxi_session_keys {
            fzf_command.arg("--bind").arg(format!(
                "alt-{key}:execute(muxi sessions switch {key})+abort"
            ));
        }

        if settings.fzf.bind_sessions {
            fzf_command.arg("--bind").arg(format!(
                "i,/:show-input+unbind(j,k,q,x,e,c,p,r,i,/,{})",
                muxi_session_keys.join(",")
            ));

            for key in &muxi_session_keys {
                fzf_command
                    .arg("--bind")
                    .arg(format!("{key}:execute(muxi sessions switch {key})+abort"));
            }
        }
    }

    // Append user provided args
    fzf_command.args(settings.fzf.args).args(fzf_args);

    // Execute
    fzf_command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(())
}

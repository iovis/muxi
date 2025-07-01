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
        .arg("none")
        .arg("-E")
        .arg("fzf")
        .arg("--reverse")
        .arg("--info")
        .arg("inline-right")
        .arg("--list-border")
        .arg("--list-label")
        .arg(" muxi sessions ")
        .arg("--input-border")
        .arg("--color")
        .arg("list-label:green")
        .arg("--color")
        .arg("preview-label:black")
        .arg("--color")
        .arg("list-border:black")
        .arg("--color")
        .arg("preview-border:black")
        .arg("--color")
        .arg("input-border:black")
        .arg("--prompt")
        .arg("❯ ")
        .arg("--ghost")
        .arg("muxi sessions")
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
        .arg("right,60%")
        .arg("--bind")
        .arg("focus:transform-preview-label:echo ' {2} '")
        .arg("--bind")
        .arg("alt-p:toggle-preview")
        .arg("--bind")
        .arg("alt-r:change-preview-window(down|right)");

    // Hide fuzzy prompt
    if !settings.fzf.input {
        fzf_command.arg("--no-input");

        bind_vim_keys(&mut fzf_command);

        let muxi_session_keys = sessions.0.keys().map(Key::to_string).collect::<Vec<_>>();
        bind_alt_session_keys(&mut fzf_command, &muxi_session_keys);

        // Allow to set current session to key with alt-<uppercase_letter>
        if settings.uppercase_overrides {
            bind_session_overrides(&mut fzf_command);
        }

        if settings.fzf.bind_sessions {
            bind_raw_session_keys(&mut fzf_command, &muxi_session_keys);
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

fn bind_vim_keys(fzf_command: &mut Command) {
    fzf_command
        .arg("--bind")
        .arg("j:down,k:up,q:abort")
        .arg("--bind")
        .arg("d,x:execute-silent(muxi sessions delete {1})+reload(muxi sessions list)")
        .arg("--bind")
        .arg("e:execute(muxi sessions edit)+reload(muxi sessions list)")
        .arg("--bind")
        .arg("c:execute(muxi config edit)+reload(muxi sessions list)")
        .arg("--bind")
        .arg("p:toggle-preview")
        .arg("--bind")
        .arg("r:change-preview-window(down|right)")
        .arg("--bind")
        .arg("i,/:show-input+unbind(j,k,q,d,x,e,c,p,r,i,/)");
}

fn bind_alt_session_keys(fzf_command: &mut Command, muxi_session_keys: &[String]) {
    for key in muxi_session_keys {
        fzf_command.arg("--bind").arg(format!(
            "alt-{key}:execute(muxi sessions switch {key})+abort"
        ));
    }
}

fn bind_raw_session_keys(fzf_command: &mut Command, muxi_session_keys: &[String]) {
    fzf_command.arg("--bind").arg(format!(
        "i,/:show-input+unbind(j,k,q,d,x,e,c,p,r,i,/,{})",
        muxi_session_keys.join(",")
    ));

    for key in muxi_session_keys {
        fzf_command
            .arg("--bind")
            .arg(format!("{key}:execute(muxi sessions switch {key})+abort"));
    }
}

fn bind_session_overrides(fzf_command: &mut Command) {
    for key in 'A'..='Z' {
        fzf_command.arg("--bind").arg(format!(
            "alt-{key}:execute-silent(muxi sessions set {})+reload(muxi sessions list)",
            key.to_lowercase()
        ));
    }
}

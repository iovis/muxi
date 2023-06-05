use std::io::{stdout, IsTerminal};
use std::process::{Command, Stdio};

use color_eyre::Result;

use crate::muxi::Muxi;
use crate::sessions::Sessions;

pub fn spawn() -> Result<()> {
    if stdout().is_terminal() {
        spawn_fzf_popup()?;
    } else {
        let sessions = Muxi::new()?.sessions;

        for session in sessions_for_display(&sessions) {
            println!("{session}");
        }
    }

    Ok(())
}

fn spawn_fzf_popup() -> Result<()> {
    // --delimiter : \
    // --preview 'bat --color=always {1} --highlight-line {2}' \
    // --preview-window 'up,60%,border-bottom,+{2}+3/3,~3' \
    let fzf_tmux_command = Command::new("fzf-tmux")
        .arg("-p80%,80%")
        .arg("--reverse")
        // .arg("--ansi") // TODO: necessary or should I highlight with fzf?
        .arg("--print-query") // TODO: remove
        // .arg("--exit-0")
        .arg("--info")
        .arg("inline")
        .arg("--header")
        .arg(":: some boring header")
        .arg("--prompt")
        .arg("muxi> ")
        .arg("--bind")
        .arg("start:reload:cargo run -q -- fzf") // TODO: change to `muxi`
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let output = fzf_tmux_command.wait_with_output()?;
    let selected_option = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let error = String::from_utf8_lossy(&output.stderr).trim().to_string();

    // Use the selected option as needed
    println!("Selected Option: {selected_option}");
    println!("Error: {error}");

    Ok(())
}

fn sessions_for_display(sessions: &Sessions) -> Vec<String> {
    let max_width_key = sessions.keys().map(|key| key.as_ref().len()).max().unwrap();

    let max_width_name = sessions
        .values()
        .map(|session| session.name.len())
        .max()
        .unwrap();

    let mut sessions_list: Vec<String> = Vec::with_capacity(sessions.len());

    for (key, session) in sessions {
        sessions_list.push(format!(
            "{:<max_width_key$}  {:<max_width_name$}  {}",
            key,
            session.name,
            session.path.display(),
        ));
    }

    sessions_list
}

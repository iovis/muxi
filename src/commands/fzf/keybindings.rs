use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::{Muxi, Settings};
use crate::tmux::Key;

pub fn show() -> Result<()> {
    let sessions = Muxi::new()?.sessions;
    let settings = Settings::from_lua()?;

    let muxi_session_keys = sessions.0.keys().map(Key::to_string).collect::<Vec<_>>();

    show_default_keys();

    if !settings.fzf.input {
        if settings.fzf.bind_sessions {
            show_raw_session_keys(&muxi_session_keys);
        } else {
            show_vim_keys();
        }
    }

    show_alt_session_keys(&muxi_session_keys);

    if settings.uppercase_overrides {
        show_session_overrides();
    }

    Ok(())
}

fn show_default_keys() {
    println!("{}", "Keybindings".bold().underline());

    println!(
        "{} {}",
        "Enter ".bold().cyan(),
        "switch to session".dimmed()
    );
    println!("{} {}", "ctrl-x".bold().cyan(), "delete session".dimmed());
    println!("{} {}", "ctrl-r".bold().cyan(), "edit sessions".dimmed());
    println!("{} {}", "ctrl-g".bold().cyan(), "edit config".dimmed());
    println!("{} {}", "alt-p ".bold().cyan(), "toggle preview".dimmed());
    println!("{} {}", "alt-r ".bold().cyan(), "rotate preview".dimmed());
}

fn show_raw_session_keys(muxi_session_keys: &[String]) {
    println!("\n{}", "Session Keybindings".bold().underline());

    for key in muxi_session_keys {
        println!(
            "{} {} {}",
            key.bold().cyan(),
            "switch to session".dimmed(),
            key.green()
        );
    }
}

fn show_vim_keys() {
    println!("\n{}", "Vim Keybindings".bold().underline());

    println!("{} {}", "j k".bold().cyan(), "move".dimmed());
    println!("{} {}", "d x".bold().cyan(), "delete session".dimmed());
    println!("{} {}", "e  ".bold().cyan(), "edit sessions".dimmed());
    println!("{} {}", "c  ".bold().cyan(), "edit config".dimmed());
    println!("{} {}", "p  ".bold().cyan(), "toggle preview".dimmed());
    println!("{} {}", "r  ".bold().cyan(), "rotate preview".dimmed());
    println!("{} {}", "i /".bold().cyan(), "fuzzy find sessions".dimmed());
    println!("{} {}", "q  ".bold().cyan(), "quit".dimmed());
}

fn show_alt_session_keys(muxi_session_keys: &[String]) {
    println!("\n{}", "Session Keybindings".bold().underline());

    for key in muxi_session_keys {
        println!(
            "{} {} {}",
            format!("alt+{key}").bold().cyan(),
            "switch to session".dimmed(),
            key.green()
        );
    }
}

fn show_session_overrides() {
    println!("\n{}", "Session Override Keybindings".bold().underline());

    println!(
        "{} {} {}",
        "alt+A..Z".bold().cyan(),
        "set this session to".dimmed(),
        "a..z".green()
    );
}

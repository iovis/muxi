use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::Muxi;
use crate::tmux;

pub fn switch(key: &tmux::Key) -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    let Some(session) = sessions.0.get(key) else {
        println!("{}", "Session not found!".red());
        return Ok(());
    };

    if !tmux::has_session(session) {
        tmux::create_session(session);
    }

    tmux::switch_to(session)?;

    Ok(())
}

pub fn tmux_menu() -> Result<()> {
    let sessions = Muxi::new()?.sessions;
    tmux::sessions_menu(&sessions)?;

    Ok(())
}

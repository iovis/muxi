use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::cli::SessionSwitchArgs;
use crate::muxi::Muxi;
use crate::tmux;

pub fn switch(SessionSwitchArgs { key }: SessionSwitchArgs) -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    let Some(session) = sessions.get(&key) else {
        println!("{}", "Session not found!".red());
        return Ok(());
    };

    if !tmux::has_session(session) {
        tmux::create_session(session);
    }

    tmux::switch_to(session)?;

    Ok(())
}

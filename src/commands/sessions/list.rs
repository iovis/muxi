use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::muxi::Muxi;
use crate::sessions::sessions_for_display;

pub fn list() -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    if sessions.is_empty() {
        println!("{}", "No sessions defined!".red());
        return Ok(());
    }

    for session in sessions_for_display(&sessions) {
        println!("{session}");
    }

    Ok(())
}

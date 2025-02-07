use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::muxi::Muxi;

pub fn list() -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    if sessions.is_empty() {
        println!("{}", "No sessions defined!".red());
        return Ok(());
    }

    for session in sessions.to_list() {
        println!("{session}");
    }

    Ok(())
}

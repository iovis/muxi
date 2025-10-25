use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::Muxi;

pub fn list() -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    if sessions.is_empty() {
        println!("{}", "No sessions defined!".red());
        return Ok(());
    }

    println!("{sessions}");

    Ok(())
}

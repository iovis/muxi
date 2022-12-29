use color_eyre::Result;
use itertools::Itertools;
use owo_colors::OwoColorize;

use crate::muxi::Muxi;

pub fn list() -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    if sessions.is_empty() {
        println!("{}", "No sessions defined!".red());
        return Ok(());
    }

    let max_width_key = sessions.keys().map(|key| key.as_ref().len()).max().unwrap();

    let max_width_name = sessions
        .values()
        .map(|session| session.name.len())
        .max()
        .unwrap();

    for (key, session) in sessions.iter().sorted_by_key(|key| key.0) {
        println!(
            "{:<max_width_key$}  {:<max_width_name$}  {}",
            key.green(),
            session.name.blue(),
            session.path.display().dimmed(),
        );
    }

    Ok(())
}

use miette::Result;
use owo_colors::OwoColorize;

use crate::muxi::{Settings, path};

pub fn list() -> Result<()> {
    let settings = Settings::from_lua()?;

    println!(
        "{} {}",
        "settings:".yellow(),
        path::settings_file().to_string_lossy().dimmed()
    );

    println!("{settings}");

    Ok(())
}

use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::muxi::{self, path};

pub fn list() -> Result<()> {
    let path = path::muxi_dir();
    let settings = muxi::parse_settings(&path)?;

    println!(
        "{} {}",
        "settings:".yellow(),
        path::settings_file().to_string_lossy().dimmed()
    );

    println!("{settings}");

    Ok(())
}

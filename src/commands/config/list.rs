use color_eyre::Result;
use owo_colors::OwoColorize;

use crate::settings::Settings;
use crate::{path, tmux};

pub fn list() -> Result<()> {
    let tmux_settings = tmux::Settings::new();
    let settings = Settings::new(&path::settings_file(), tmux_settings)?;

    // Settings
    println!(
        "{} {}",
        "SETTINGS:".yellow(),
        path::settings_file().to_string_lossy().dimmed()
    );
    println!("{settings}");

    // Bindings
    if !settings.bindings.is_empty() {
        println!("{}", "BINDINGS:".yellow());

        let max_width_key = settings
            .bindings
            .keys()
            .map(|key| key.as_ref().len())
            .max()
            .unwrap();

        for (key, binding) in settings.bindings {
            print!("    {:<max_width_key$}  {}", key.green(), binding.command);

            if binding.popup.is_some() {
                print!("{}", " (popup)".yellow());
            }

            println!();
        }
    }

    Ok(())
}

use itertools::Itertools;

use crate::path;
use crate::settings::Settings;

pub fn list() -> anyhow::Result<()> {
    let settings = Settings::new(&path::settings_file())?;

    println!(
        "Change your settings in {}\n",
        path::settings_file().to_string_lossy()
    );

    println!("Settings");
    println!("========");
    println!("{}", settings);

    if !settings.bindings.is_empty() {
        println!("Bindings");
        println!("========");

        for (key, binding) in settings.bindings.iter().sorted_by_key(|key| key.0) {
            print!("[{}]: {}", key, binding.command);

            if binding.popup.is_some() {
                print!(" (popup)");
            }

            println!();
        }
    }

    Ok(())
}

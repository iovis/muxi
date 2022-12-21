use crate::path;
use crate::settings::Settings;

pub fn list() -> anyhow::Result<()> {
    let settings = Settings::new(&path::settings_file())?;

    println!("Settings");
    println!("========");
    println!("{}", settings);

    println!(
        "Change your settings in {}",
        path::settings_file().to_string_lossy()
    );

    Ok(())
}

use crate::muxi::Muxi;
use crate::path;

pub fn list() -> anyhow::Result<()> {
    let config = Muxi::new();

    println!("Settings");
    println!("========");
    println!("{}", config.settings);

    println!(
        "Change your settings in {}",
        path::settings_file().to_string_lossy()
    );

    Ok(())
}

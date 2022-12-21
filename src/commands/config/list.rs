use crate::muxi::Muxi;

pub fn list() -> anyhow::Result<()> {
    let config = Muxi::new();

    println!("Settings");
    println!("========");
    println!("{}", config.settings);

    println!(
        "Change your settings in {}",
        config.path.join("settings.toml").to_string_lossy()
    );

    Ok(())
}

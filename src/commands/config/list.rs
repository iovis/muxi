use crate::muxi::Muxi;

pub fn list() -> anyhow::Result<()> {
    let config = Muxi::new();

    println!("{:?}", config.settings);

    Ok(())
}

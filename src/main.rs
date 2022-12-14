use crate::config::Config;

mod config;
mod sessions;

fn main() -> anyhow::Result<()> {
    let config = Config::new()?;

    dbg!(config);

    Ok(())
}

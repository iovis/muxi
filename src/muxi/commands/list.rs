use crate::muxi::config::Config;

pub fn list() -> anyhow::Result<()> {
    let config = Config::new();
    let sessions = config.sessions()?;

    if sessions.is_empty() {
        println!("No sessions defined!");
        return Ok(());
    }

    let max_width_key = sessions
        .iter()
        .map(|session| session.key.len())
        .max()
        .unwrap();

    let max_width_name = sessions
        .iter()
        .map(|session| session.name.len())
        .max()
        .unwrap();

    for session in sessions {
        println!(
            "[{:<max_width_key$}]: {:<max_width_name$}  ({})",
            session.key,
            session.name,
            session.path.display(),
        );
    }

    Ok(())
}

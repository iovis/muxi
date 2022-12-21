use crate::muxi::Muxi;

pub fn list() -> anyhow::Result<()> {
    let sessions = Muxi::new()?.sessions;

    if sessions.is_empty() {
        println!("No sessions defined!");
        return Ok(());
    }

    let max_width_key = sessions.keys().map(|key| key.as_ref().len()).max().unwrap();

    let max_width_name = sessions
        .values()
        .map(|session| session.name.len())
        .max()
        .unwrap();

    for (key, session) in sessions {
        println!(
            "[{:<max_width_key$}]: {:<max_width_name$}  ({})",
            key,
            session.name,
            session.path.display(),
        );
    }

    Ok(())
}

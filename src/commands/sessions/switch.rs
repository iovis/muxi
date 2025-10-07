use std::fmt::Display;

use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::muxi::{Muxi, Session, Sessions};
use crate::tmux;

pub fn switch(key: &tmux::Key) -> Result<()> {
    let sessions = Muxi::new()?.sessions;

    let Some(session) = sessions.0.get(key) else {
        println!("{}", "Session not found!".red());
        return Ok(());
    };

    if !tmux::has_session(session) {
        tmux::create_session(session);
    }

    tmux::switch_to(session)?;

    Ok(())
}

pub fn tmux_menu() -> Result<()> {
    let sessions = Muxi::new()?.sessions;
    tmux::sessions_menu(&sessions)?;

    Ok(())
}

pub fn picker() -> Result<()> {
    let sessions = Muxi::new()?.sessions;
    let choices = SessionChoice::from(sessions);

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&choices)
        .default(0)
        .interact_opt()
        .into_diagnostic()?;

    let Some(index) = selection else {
        return Ok(());
    };

    let session = &choices[index];

    switch(&session.key)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct SessionChoice {
    pub key: tmux::Key,
    pub session: Session,
}

impl SessionChoice {
    pub fn from(sessions: Sessions) -> Vec<Self> {
        let mut choices = Vec::with_capacity(sessions.0.len());

        for (key, session) in sessions.0 {
            choices.push(Self { key, session });
        }

        choices.sort();
        choices
    }
}

impl Display for SessionChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {} {}",
            self.key.green(),
            self.session.name.blue(),
            self.session.path.display().dimmed()
        )
    }
}

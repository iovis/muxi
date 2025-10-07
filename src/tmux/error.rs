use std::io;
use std::string::FromUtf8Error;

use miette::Diagnostic;
use thiserror::Error;

pub type TmuxResult<T> = Result<T, Error>;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("muxi needs to be executed within a tmux session")]
    #[diagnostic(
        code(muxi::tmux::not_in_tmux),
        help("Run muxi from within a tmux session")
    )]
    NotInTmux(#[from] std::env::VarError),

    #[error("failed to run command")]
    #[diagnostic(code(muxi::tmux::command_failed))]
    Command(#[from] io::Error),

    #[error("failed to parse tmux output: `{0}`")]
    #[diagnostic(
        code(muxi::tmux::parse_error),
        help("This might be a bug - please report it with your tmux version")
    )]
    Parse(#[from] FromUtf8Error),

    #[error("failed to switch to session {0}: `{1}`")]
    #[diagnostic(code(muxi::tmux::switch_failed))]
    Switch(String, String),

    #[error("failed to generate tmux menu: `{0}`")]
    #[diagnostic(code(muxi::tmux::menu_failed))]
    DisplayMenu(String),

    #[error("failed to initialize muxi: `{0}`")]
    #[diagnostic(
        code(muxi::tmux::init_failed),
        help("Check your tmux configuration for any conflicting bindings")
    )]
    Init(String),
}

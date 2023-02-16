use std::io;
use std::string::FromUtf8Error;

use thiserror::Error;

pub type TmuxResult<T> = Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("muxi needs to be executed within a tmux session")]
    NotInTmux(#[from] std::env::VarError),
    #[error("failed to run command")]
    Command(#[from] io::Error),
    #[error("failed to parse tmux output: `{0}`")]
    Parse(#[from] FromUtf8Error),
    #[error("failed to switch to session {0}: `{1}`")]
    Switch(String, String),
    #[error("failed to generate tmux menu: `{0}`")]
    DisplayMenu(String),
    #[error("failed to initialize muxi: `{0}`")]
    Init(String),
}

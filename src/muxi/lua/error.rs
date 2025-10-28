use miette::{Diagnostic, NamedSource, SourceSpan};
use mlua::prelude::LuaError;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("{0} not found")]
    #[diagnostic(code(muxi::lua::not_found))]
    NotFound(#[from] std::io::Error),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LuaParse(#[from] Box<LuaParseDiagnostic>),

    #[error("failed to execute embedded Lua code")]
    #[diagnostic(code(muxi::lua::runtime_error))]
    Lua(#[from] LuaError),

    #[error("failed to deserialize Lua config at {path}: {message}")]
    #[diagnostic(
        code(muxi::lua::deserialize_error),
        help("Check the value assigned to {path} in ~/.config/muxi/init.lua")
    )]
    LuaDeserialize {
        #[source]
        source: LuaError,
        path: String,
        message: String,
    },
}

#[derive(Debug, Error, Diagnostic)]
#[error("failed to parse Lua config: {label}")]
#[diagnostic(
    code(muxi::lua::parse_error),
    help(
        "Check the syntax in ~/.config/muxi/init.lua\nMake sure it returns a valid configuration table"
    )
)]
pub(crate) struct LuaParseDiagnostic {
    #[source]
    pub(super) source: LuaError,
    #[source_code]
    pub(super) src: NamedSource<String>,
    #[label("{label}")]
    pub(super) span: Option<SourceSpan>,
    pub(super) label: String,
}

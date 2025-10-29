use miette::{Diagnostic, NamedSource, SourceSpan};
use mlua::prelude::LuaError;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum Error {
    #[error("{0} not found")]
    #[diagnostic(code(muxi::lua::not_found))]
    NotFound(#[from] std::io::Error),

    #[error("failed to execute embedded Lua code")]
    #[diagnostic(code(muxi::lua::runtime_error))]
    Lua(#[from] LuaError),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LuaParse(#[from] Box<LuaParseDiagnostic>),

    #[error(transparent)]
    #[diagnostic(transparent)]
    LuaDeserialize(#[from] Box<LuaDeserializeDiagnostic>),
}

#[derive(Debug, Error, Diagnostic)]
#[error("failed to parse Lua config: {label}")]
#[diagnostic(code(muxi::lua::parse_error))]
pub struct LuaParseDiagnostic {
    #[source]
    pub source: LuaError,
    #[source_code]
    pub src: NamedSource<String>,
    #[label("{label}")]
    pub span: Option<SourceSpan>,
    pub label: String,
}

#[derive(Debug, Error, Diagnostic)]
#[error("failed to deserialize Lua config at {path}")]
#[diagnostic(
    code(muxi::lua::deserialize_error),
    help("Check the value assigned to {path} in {file}")
)]
pub struct LuaDeserializeDiagnostic {
    #[source]
    pub source: LuaError,
    pub path: String,
    pub message: String,
    pub file: String,
}

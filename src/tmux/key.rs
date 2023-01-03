use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct TmuxKey(String);

impl TmuxKey {
    pub fn parse<T: AsRef<str>>(value: T) -> Result<Self, String> {
        let value = value.as_ref();

        if is_valid_tmux_key(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(format!("{value} is not a valid tmux binding"))
        }
    }
}

impl TryFrom<String> for TmuxKey {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for TmuxKey {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

// For some reason if I do From<String> it conflicts with something
impl FromStr for TmuxKey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

impl std::fmt::Display for TmuxKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for TmuxKey {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

fn is_valid_tmux_key(_s: &str) -> bool {
    true
}

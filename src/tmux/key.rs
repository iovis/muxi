use std::str::FromStr;

use regex::Regex;
use serde::{Deserialize, Serialize};
use lazy_static::lazy_static;

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
#[serde(try_from = "String")]
pub struct TmuxKey(String);

impl TmuxKey {
    pub fn parse<T: AsRef<str>>(value: T) -> Result<Self, String> {
        let value = value.as_ref();

        if is_valid_tmux_key(value) {
            Ok(Self(value.to_string()))
        } else {
            Err(format!("{} is not a valid tmux binding", value))
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

fn is_valid_tmux_key(s: &str) -> bool {
    // Whatever ChatGPT says
    lazy_static! {
        static ref RE: Regex = Regex::new(
            r"(^[A-Za-z]$)|(^C-[A-Za-z]$)|(^\^[A-Za-z]$)|(^S-[A-Za-z]$)|(^M-[A-Za-z]$)|(^Up$)|(^Down$)|(^Left$)|(^Right$)|(^BSpace$)|(^BTab$)|(^DC$)|(^End$)|(^Enter$)|(^Escape$)|(^F[1-9]$)|(^F1[0-2]$)|(^Home$)|(^IC$)|(^NPage$)|(^PageDown$)|(^PgDn$)|(^PPage$)|(^PageUp$)|(^PgUp$)|(^Space$)|(^Tab$)"
            )
            .unwrap();
    }

    RE.is_match(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_valid_tmux_key() {
        let valid_keys = vec![
            "A", "a", "C-A", "C-a", "^a", "^A", "S-a", "S-A", "M-a", "M-A", "Up", "Down", "Left",
            "Right", "BSpace", "BTab", "DC", "End", "Enter", "Escape", "F1", "F2",
        ];

        for key in valid_keys {
            assert!(is_valid_tmux_key(key), "Expected `{}` to be valid", key);
        }

        let invalid_keys = vec!["", "C-", "^", "S-", "M-", "F13", "invalid"];

        for key in invalid_keys {
            assert!(
                !is_valid_tmux_key(key),
                "Expected `{}` to not be valid",
                key
            );
        }
    }
}

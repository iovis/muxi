use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Key(String);

impl Key {
    pub fn new<T: AsRef<str>>(value: T) -> Self {
        Self(value.as_ref().to_string())
    }
}

impl From<&str> for Key {
    fn from(s: &str) -> Self {
        Key::new(s)
    }
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for Key {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

use std::collections::BTreeMap;
use std::fmt::Display;

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PluginOptions(BTreeMap<String, String>);

impl PluginOptions {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl std::ops::Deref for PluginOptions {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for PluginOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a PluginOptions {
    type Item = (&'a String, &'a String);
    type IntoIter = std::collections::btree_map::Iter<'a, String, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<(String, String)> for PluginOptions {
    fn from_iter<T: IntoIterator<Item = (String, String)>>(iter: T) -> Self {
        Self(BTreeMap::from_iter(iter))
    }
}

impl Display for PluginOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            return Ok(());
        }

        let total = self.len();

        for (index, (key, value)) in self.0.iter().enumerate() {
            if index > 0 {
                writeln!(f)?;
            }

            let connector = if index + 1 == total {
                "└──"
            } else {
                "├──"
            };

            write!(
                f,
                "{} {} {}",
                connector.dimmed(),
                format!("@{key}").dimmed(),
                value.green().bold()
            )?;
        }

        Ok(())
    }
}

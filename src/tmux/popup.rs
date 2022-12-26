use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct PopupOptions {
    pub title: Option<String>,
    #[serde(default = "default_popup_dimension")]
    pub width: String,
    #[serde(default = "default_popup_dimension")]
    pub height: String,
}

fn default_popup_dimension() -> String {
    "75%".to_string()
}

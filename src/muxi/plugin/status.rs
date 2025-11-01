use std::time::SystemTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginStatus {
    Remote {
        installed: bool,
        commit: Option<String>,
    },
    Local {
        exists: bool,
        path: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginChange {
    pub id: String,
    pub full_id: String,
    pub summary: String,
    pub time: SystemTime,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PluginUpdateStatus {
    Updated {
        from: Option<String>,
        to: String,
        changes: Vec<PluginChange>,
        range_url: Option<String>,
    },
    UpToDate { commit: String },
    Local { path: String },
}

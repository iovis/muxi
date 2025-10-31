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
pub enum PluginUpdateStatus {
    Updated { from: Option<String>, to: String },
    UpToDate { commit: String },
    Local { path: String },
}

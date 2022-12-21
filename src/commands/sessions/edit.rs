use crate::commands::helpers::open_editor_for;
use crate::path;

pub fn edit() -> anyhow::Result<()> {
    open_editor_for(&path::sessions_file())
}

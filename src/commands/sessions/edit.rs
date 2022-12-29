use color_eyre::Result;

use crate::commands::helpers::open_editor_for;
use crate::path;

pub fn edit() -> Result<()> {
    open_editor_for(&path::sessions_file())
}

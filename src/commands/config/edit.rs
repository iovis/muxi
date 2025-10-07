use miette::Result;

use crate::commands::helpers::open_editor_for;
use crate::muxi::path;

pub fn edit(editor_args: &[String]) -> Result<()> {
    open_editor_for(&path::settings_file(), editor_args)
}

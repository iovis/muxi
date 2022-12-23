use std::path::PathBuf;

use crate::sessions::TmuxKey;

pub fn set(key: TmuxKey, name: Option<String>, path: Option<PathBuf>) -> anyhow::Result<()> {
    println!("{:?}", key);

    // TODO: get current session name if not given
    println!("{:?}", name);

    // TODO: get current session path if not given
    println!("{:?}", path);

    todo!()
}

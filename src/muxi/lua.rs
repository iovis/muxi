use std::path::Path;

use mlua::prelude::{Lua, LuaError, LuaSerdeExt};
use thiserror::Error;

use crate::muxi::{path, Settings};

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0} not found")]
    NotFound(#[from] std::io::Error),
    #[error("failed to parse tmux output: `{0}`")]
    LuaError(#[from] LuaError),
}

/// Get `muxi::Settings` from muxi/init.lua
pub fn settings() -> Result<Settings, Error> {
    let muxi_path = path::muxi_dir();
    let code = std::fs::read_to_string(muxi_path.join("init.lua"))?;

    let lua = lua_init(&muxi_path)?;

    lua.load(code).exec()?;

    let muxi_table = lua.globals().get("muxi")?;

    Ok(lua.from_value(muxi_table)?)
}

fn lua_init(path: &Path) -> Result<Lua, Error> {
    let lua = Lua::new();

    {
        // `globals` is a borrow of lua
        let globals = lua.globals();

        // package.path (allow requires)
        let package: mlua::Table = globals.get("package")?;
        let mut package_path: Vec<String> = package
            .get::<_, String>("path")?
            .split(';')
            .map(ToOwned::to_owned)
            .collect();

        package_path.insert(0, format!("{}/?.lua", path.display()));
        package_path.insert(1, format!("{}/?/init.lua", path.display()));

        package.set("path", package_path.join(";"))?;

        // muxi table
        globals.set("muxi", lua.to_value(&Settings::default())?)?;
    }

    Ok(lua)
}

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result};
use owo_colors::OwoColorize;

use crate::muxi::path;

const INIT_LUA: &str = include_str!("templates/init.lua");
const LUARC_JSON: &str = include_str!("templates/luarc.json");
const LUA_LS_MUXI: &str = include_str!("templates/lua_ls_muxi.lua");

pub fn init() -> Result<()> {
    let paths = InitPaths::current();
    let summary = init_config_dir(&paths)?;

    print_summary(&summary, &paths);

    Ok(())
}

#[derive(Debug)]
struct InitPaths {
    config_dir: PathBuf,
    lua_ls_dir: PathBuf,
    init_lua: PathBuf,
    luarc_json: PathBuf,
    lua_ls_muxi: PathBuf,
}

impl InitPaths {
    fn current() -> Self {
        Self {
            config_dir: path::muxi_dir(),
            lua_ls_dir: path::lua_ls_dir(),
            init_lua: path::settings_file(),
            luarc_json: path::luarc_file(),
            lua_ls_muxi: path::lua_ls_file(),
        }
    }

    #[cfg(test)]
    fn new(config_dir: PathBuf) -> Self {
        let lua_ls_dir = config_dir.join(".lua_ls");

        Self {
            init_lua: config_dir.join("init.lua"),
            luarc_json: config_dir.join(".luarc.json"),
            lua_ls_muxi: lua_ls_dir.join("muxi.lua"),
            lua_ls_dir,
            config_dir,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct InitSummary {
    init_lua: FileAction,
    luarc_json: FileAction,
    lua_ls_muxi: FileAction,
}

#[derive(Debug, PartialEq, Eq)]
enum FileAction {
    Created,
    Skipped,
    Updated,
}

fn init_config_dir(paths: &InitPaths) -> Result<InitSummary> {
    fs::create_dir_all(&paths.config_dir).into_diagnostic()?;
    fs::create_dir_all(&paths.lua_ls_dir).into_diagnostic()?;

    let init_lua = write_if_missing(&paths.init_lua, INIT_LUA)?;
    let luarc_json = write_if_missing(&paths.luarc_json, LUARC_JSON)?;
    let lua_ls_muxi = write_generated(&paths.lua_ls_muxi, LUA_LS_MUXI)?;

    Ok(InitSummary {
        init_lua,
        luarc_json,
        lua_ls_muxi,
    })
}

fn write_if_missing(file: &Path, contents: &str) -> Result<FileAction> {
    let mut file = match OpenOptions::new().write(true).create_new(true).open(file) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::AlreadyExists => return Ok(FileAction::Skipped),
        Err(error) => return Err(error).into_diagnostic(),
    };

    file.write_all(contents.as_bytes()).into_diagnostic()?;

    Ok(FileAction::Created)
}

fn write_generated(file: &Path, contents: &str) -> Result<FileAction> {
    let action = if file.exists() {
        FileAction::Updated
    } else {
        FileAction::Created
    };

    fs::write(file, contents).into_diagnostic()?;

    Ok(action)
}

fn print_summary(summary: &InitSummary, paths: &InitPaths) {
    print_file_action(&summary.init_lua, &paths.init_lua);
    print_file_action(&summary.luarc_json, &paths.luarc_json);
    print_file_action(&summary.lua_ls_muxi, &paths.lua_ls_muxi);
}

fn print_file_action(action: &FileAction, file: &Path) {
    let file = file.display().to_string();
    let file = file.dimmed();

    match action {
        FileAction::Created => println!("{} {file}", "created".green().bold()),
        FileAction::Skipped => println!("{} {file}", "skipped".yellow().bold()),
        FileAction::Updated => println!("{} {file}", "updated".cyan().bold()),
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;

    use uuid::Uuid;

    use super::*;

    fn temp_paths() -> InitPaths {
        InitPaths::new(temp_dir().join(Uuid::new_v4().to_string()))
    }

    fn cleanup(paths: &InitPaths) {
        if paths.config_dir.exists() {
            fs::remove_dir_all(&paths.config_dir).unwrap();
        }
    }

    #[test]
    fn creates_config_files_for_fresh_directory() {
        let paths = temp_paths();

        let summary = init_config_dir(&paths).unwrap();

        assert_eq!(
            summary,
            InitSummary {
                init_lua: FileAction::Created,
                luarc_json: FileAction::Created,
                lua_ls_muxi: FileAction::Created,
            }
        );
        assert_eq!(fs::read_to_string(&paths.init_lua).unwrap(), INIT_LUA);
        assert_eq!(fs::read_to_string(&paths.luarc_json).unwrap(), LUARC_JSON);
        assert_eq!(fs::read_to_string(&paths.lua_ls_muxi).unwrap(), LUA_LS_MUXI);

        cleanup(&paths);
    }

    #[test]
    fn preserves_existing_init_lua() {
        let paths = temp_paths();
        fs::create_dir_all(&paths.config_dir).unwrap();
        fs::write(&paths.init_lua, "-- user config\n").unwrap();

        let summary = init_config_dir(&paths).unwrap();

        assert_eq!(summary.init_lua, FileAction::Skipped);
        assert_eq!(
            fs::read_to_string(&paths.init_lua).unwrap(),
            "-- user config\n"
        );

        cleanup(&paths);
    }

    #[test]
    fn preserves_existing_luarc_json() {
        let paths = temp_paths();
        fs::create_dir_all(&paths.config_dir).unwrap();
        fs::write(&paths.luarc_json, "{\"custom\":true}\n").unwrap();

        let summary = init_config_dir(&paths).unwrap();

        assert_eq!(summary.luarc_json, FileAction::Skipped);
        assert_eq!(
            fs::read_to_string(&paths.luarc_json).unwrap(),
            "{\"custom\":true}\n"
        );

        cleanup(&paths);
    }

    #[test]
    fn overwrites_existing_lua_ls_muxi_file() {
        let paths = temp_paths();
        fs::create_dir_all(&paths.lua_ls_dir).unwrap();
        fs::write(&paths.lua_ls_muxi, "-- stale\n").unwrap();

        let summary = init_config_dir(&paths).unwrap();

        assert_eq!(summary.lua_ls_muxi, FileAction::Updated);
        assert_eq!(fs::read_to_string(&paths.lua_ls_muxi).unwrap(), LUA_LS_MUXI);

        cleanup(&paths);
    }

    #[test]
    fn lua_ls_snapshot_describes_muxi_config() {
        assert!(LUA_LS_MUXI.contains("---@class (exact) muxi.Config"));
        assert!(LUA_LS_MUXI.contains("---@class (exact) muxi.FzfSettings"));
        assert!(LUA_LS_MUXI.contains("---@class (exact) muxi.PluginSpec"));
        assert!(LUA_LS_MUXI.contains("---@field plugins? muxi.Plugin[]"));
        assert!(LUA_LS_MUXI.contains("---@alias muxi.Plugin string|muxi.PluginSpec"));
        assert!(LUA_LS_MUXI.contains("---@type muxi.Api"));
        assert!(LUA_LS_MUXI.contains("muxi = muxi"));
    }
}

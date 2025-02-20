use std::path::Path;

use mlua::prelude::{Lua, LuaError, LuaSerdeExt};
use thiserror::Error;

use crate::muxi::Settings;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0} not found")]
    NotFound(#[from] std::io::Error),
    #[error("failed to parse tmux output: `{0}`")]
    LuaError(#[from] LuaError),
}

/// Get `muxi::Settings` from muxi/init.lua
pub fn parse_settings(path: &Path, settings: &Settings) -> Result<Settings, Error> {
    let code = std::fs::read_to_string(path.join("init.lua"))?;
    let lua = lua_init(path, settings)?;

    lua.load(code).exec()?;

    let muxi_table = lua.globals().get("muxi")?;

    Ok(lua.from_value(muxi_table)?)
}

fn lua_init(path: &Path, settings: &Settings) -> Result<Lua, Error> {
    let lua = Lua::new();

    {
        // `globals` is a borrow of `lua`
        let globals = lua.globals();

        // package.path (allow requires)
        let package: mlua::Table = globals.get("package")?;
        let mut package_path: Vec<String> = package
            .get::<String>("path")?
            .split(';')
            .map(ToOwned::to_owned)
            .collect();

        package_path.insert(0, format!("{}/?.lua", path.display()));
        package_path.insert(1, format!("{}/?/init.lua", path.display()));

        package.set("path", package_path.join(";"))?;

        // Expose muxi settings table to lua
        globals.set("muxi", lua.to_value(settings)?)?;
    }

    Ok(lua)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env::temp_dir;
    use std::io::Write;

    use uuid::Uuid;

    use crate::muxi::{Binding, Bindings, FzfSettings};
    use crate::tmux::Popup;

    use super::*;

    fn with_config<F>(config: &str, test: F)
    where
        F: Fn(Settings) + std::panic::RefUnwindSafe,
    {
        // Create tmp folder
        let pwd = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&pwd).unwrap();

        // Create settings file
        let mut file = std::fs::File::create(pwd.join("init.lua")).unwrap();
        file.write_all(config.as_bytes()).unwrap();

        // Set $MUXI_CONFIG_PATH to current folder and load config
        temp_env::with_var("MUXI_CONFIG_PATH", Some(pwd.clone()), || {
            let settings = parse_settings(&pwd, &Settings::default()).unwrap();

            // Cleanup before test, in case of panic
            std::fs::remove_dir_all(&pwd).unwrap();

            test(settings);
        });
    }

    #[test]
    fn test_parse_valid_muxi_prefix() {
        let config = "";

        with_config(config, |settings| {
            let expected_settings = Settings::default();
            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_valid_muxi_prefix_root() {
        let config = r#"
          muxi.muxi_prefix = "M-Space"
          muxi.tmux_prefix = false
        "#;

        with_config(config, |settings| {
            let expected_settings = Settings {
                tmux_prefix: false,
                muxi_prefix: "M-Space".try_into().unwrap(),
                uppercase_overrides: false,
                use_current_pane_path: false,
                fzf: FzfSettings::default(),
                bindings: BTreeMap::new(),
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_valid_fzf_options() {
        let config = r#"
          muxi.fzf.input = false
          muxi.fzf.bind_sessions = true
          muxi.fzf.args = "--bind=d:toggle-preview"
        "#;

        with_config(config, |settings| {
            let expected_settings = Settings {
                fzf: FzfSettings {
                    input: false,
                    bind_sessions: true,
                    args: Some("--bind=d:toggle-preview".to_string()),
                },
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_valid_fzf_options_table() {
        let config = "
          muxi.fzf = {
            input = false,
            bind_sessions = false,
          }
        ";

        with_config(config, |settings| {
            let expected_settings = Settings {
                fzf: FzfSettings {
                    input: false,
                    bind_sessions: false,
                    args: None,
                },
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_no_popup() {
        let config = r#"
          muxi.muxi_prefix = "g"

          muxi.bindings = {
            j = { command = "tmux switch-client -l" }
          }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".try_into().unwrap(),
                Binding {
                    command: "tmux switch-client -l".into(),
                    popup: None,
                },
            );

            let expected_settings = Settings {
                bindings,
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_default_height() {
        let config = r#"
          muxi.muxi_prefix = "g"

          muxi.bindings = {
            j = { popup = { width = "60%" }, command = "muxi sessions edit" }
          }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".try_into().unwrap(),
                Binding {
                    command: "muxi sessions edit".into(),
                    popup: Some(Popup {
                        title: None,
                        width: "60%".into(),
                        height: "75%".into(),
                    }),
                },
            );

            let expected_settings = Settings {
                bindings,
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_default_all() {
        let config = r#"
            muxi.muxi_prefix = "g"

            muxi.bindings = {
                j = { popup = {}, command = "muxi sessions edit" }
            }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".try_into().unwrap(),
                Binding {
                    command: "muxi sessions edit".into(),
                    popup: Some(Popup {
                        title: None,
                        width: "75%".into(),
                        height: "75%".into(),
                    }),
                },
            );

            let expected_settings = Settings {
                bindings,
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_popup_struct() {
        let config = r#"
            muxi.muxi_prefix = "g"
            muxi.use_current_pane_path = true

            muxi.bindings = {
                j = {
                    popup = { title = "my title", width = "75%", height = "60%" },
                    command = "muxi sessions edit"
                }
            }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".try_into().unwrap(),
                Binding {
                    command: "muxi sessions edit".into(),
                    popup: Some(Popup {
                        title: Some("my title".into()),
                        width: "75%".into(),
                        height: "60%".into(),
                    }),
                },
            );

            let expected_settings = Settings {
                bindings,
                use_current_pane_path: true,
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }
}

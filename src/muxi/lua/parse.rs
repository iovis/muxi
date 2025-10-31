use miette::{NamedSource, SourceSpan};
use mlua::LuaSerdeExt;
use mlua::Value as LuaValue;
use mlua::prelude::{Lua, LuaError, LuaTable};
use std::path::Path;

use crate::muxi::Settings;
use crate::muxi::path;

use super::error::LuaDeserializeDiagnostic;
use super::error::{Error, LuaParseDiagnostic};

pub fn parse_settings(path: &Path, settings: &Settings) -> Result<Settings, Error> {
    let lua = lua_init(path, settings)?;

    let init_path = path.join("init.lua");
    let code = std::fs::read_to_string(&init_path)?;
    let chunk = lua
        .load(&code)
        .set_name(format!("@{}", init_path.display()));

    let user_config = chunk
        .eval::<Option<LuaTable>>()
        .map_err(|error| enrich_lua_error(error, &code, &init_path))?
        .unwrap_or_else(|| lua.create_table().unwrap());

    lua.globals().set("muxi_user_config", user_config)?;
    lua.load("muxi.merge(muxi.config, muxi_user_config)")
        .exec()?;
    let muxi_config = lua.globals().get::<LuaTable>("muxi")?.get("config")?;

    deserialize_settings(LuaValue::Table(muxi_config)).map_err(enrich_deserialize_error)
}

fn lua_init(path: &Path, settings: &Settings) -> Result<Lua, Error> {
    let lua = Lua::new();

    {
        let globals = lua.globals();

        let package: mlua::Table = globals.get("package")?;
        let mut package_path: Vec<String> = package
            .get::<String>("path")?
            .split(';')
            .map(ToOwned::to_owned)
            .collect();

        package_path.insert(0, format!("{}/?.lua", path.display()));
        package_path.insert(1, format!("{}/?/init.lua", path.display()));

        package.set("path", package_path.join(";"))?;

        let muxi_table = lua.create_table_from([
            ("config", lua.to_value(settings)?),
            ("inspect", lua.load(include_str!("inspect.lua")).eval()?),
            ("merge", lua.load(include_str!("table_merge.lua")).eval()?),
            ("print", lua.load(include_str!("print.lua")).eval()?),
        ])?;

        globals.set("muxi", muxi_table)?;
    }

    Ok(lua)
}

fn deserialize_settings(value: LuaValue) -> Result<Settings, serde_path_to_error::Error<LuaError>> {
    let deserializer = mlua::serde::Deserializer::new(value);
    serde_path_to_error::deserialize(deserializer)
}

fn enrich_lua_error(error: LuaError, code: &str, path: &Path) -> Error {
    let fallback_label = error.to_string();
    let (span, label) = extract_span(&error, code).map_or_else(
        || (None, fallback_label.clone()),
        |(span, label)| (Some(span), label),
    );

    Error::LuaParse(Box::new(LuaParseDiagnostic {
        source: error,
        src: NamedSource::new(path.display().to_string(), code.to_string()),
        span,
        label,
    }))
}

fn enrich_deserialize_error(error: serde_path_to_error::Error<LuaError>) -> Error {
    let path_string = error.path().to_string();
    let source = error.into_inner();
    let message = match &source {
        LuaError::FromLuaConversionError {
            message: Some(msg), ..
        } => msg.clone(),
        _ => source.to_string(),
    };

    Error::LuaDeserialize(Box::new(LuaDeserializeDiagnostic {
        source,
        message,
        file: path::settings_file().display().to_string(),
        path: if path_string.is_empty() {
            "root".into()
        } else {
            path_string
        },
    }))
}

fn extract_span(error: &LuaError, code: &str) -> Option<(SourceSpan, String)> {
    let LuaError::SyntaxError { message, .. } = error else {
        return None;
    };

    let line = parse_line_number(message)?;
    let (offset, len) = line_range(code, line)?;
    let span: SourceSpan = (offset, len).into();

    Some((span, format!("Lua error at line {line}")))
}

fn parse_line_number(message: &str) -> Option<usize> {
    if let Some((_, remainder)) = message.split_once("]:") {
        let line_part = remainder.split(':').next()?;
        return line_part.trim().parse().ok();
    }

    let last_colon = message.rfind(':')?;
    let before = &message[..last_colon];
    let second_last_colon = before.rfind(':')?;
    let line_part = &message[second_last_colon + 1..last_colon];

    line_part.trim().parse().ok()
}

fn line_range(code: &str, line: usize) -> Option<(usize, usize)> {
    if line == 0 {
        return None;
    }

    if code.is_empty() {
        return Some((0, 0));
    }

    let mut offset = 0usize;
    for (idx, segment) in code.split_inclusive('\n').enumerate() {
        let current_line = idx + 1;
        let line_without_newline = segment.trim_end_matches('\n').trim_end_matches('\r');
        let len = line_without_newline.len();

        if current_line == line {
            return Some((offset, len));
        }

        offset += segment.len();
    }

    None
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::env::temp_dir;
    use std::io::Write;

    use url::Url;
    use uuid::Uuid;

    use crate::muxi::lua::Error;
    use crate::muxi::{Binding, Bindings, EditorSettings, FzfSettings, Settings};
    use crate::tmux::Popup;

    use super::parse_settings;

    fn with_config<F>(config: &str, test: F)
    where
        F: Fn(Settings) + std::panic::RefUnwindSafe,
    {
        let pwd = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&pwd).unwrap();

        let mut file = std::fs::File::create(pwd.join("init.lua")).unwrap();
        file.write_all(config.as_bytes()).unwrap();

        temp_env::with_var("MUXI_CONFIG_PATH", Some(pwd.clone()), || {
            let settings = parse_settings(&pwd, &Settings::default()).unwrap();

            std::fs::remove_dir_all(&pwd).unwrap();

            test(settings);
        });
    }

    fn with_config_error<F>(config: &str, test: F)
    where
        F: Fn(Error) + std::panic::RefUnwindSafe,
    {
        let pwd = temp_dir().join(Uuid::new_v4().to_string());
        std::fs::create_dir_all(&pwd).unwrap();

        let mut file = std::fs::File::create(pwd.join("init.lua")).unwrap();
        file.write_all(config.as_bytes()).unwrap();

        temp_env::with_var("MUXI_CONFIG_PATH", Some(pwd.clone()), || {
            let result = parse_settings(&pwd, &Settings::default());

            std::fs::remove_dir_all(&pwd).unwrap();

            test(result.expect_err("expected parse_settings to fail"));
        });
    }

    #[test]
    fn test_parse_valid_muxi_prefix() {
        with_config("", |settings| {
            assert_eq!(settings, Settings::default());
        });
    }

    #[test]
    fn test_parse_valid_muxi_prefix_root() {
        let config = r#"
            muxi.config.muxi_prefix = "M-Space"
            muxi.config.tmux_prefix = false
        "#;

        with_config(config, |settings| {
            let expected_settings = Settings {
                tmux_prefix: false,
                muxi_prefix: "M-Space".into(),
                uppercase_overrides: true,
                use_current_pane_path: false,
                plugins: vec![],
                editor: EditorSettings::default(),
                fzf: FzfSettings::default(),
                bindings: BTreeMap::new(),
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_valid_fzf_options() {
        let config = r#"
            muxi.config.fzf.input = false
            muxi.config.fzf.bind_sessions = true
            muxi.config.fzf.args = { "--bind", "d:toggle-preview" }
        "#;

        with_config(config, |settings| {
            let expected_settings = Settings {
                fzf: FzfSettings {
                    input: false,
                    bind_sessions: true,
                    args: vec!["--bind".to_string(), "d:toggle-preview".to_string()],
                },
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_valid_fzf_options_table() {
        let config = r"
            muxi.config.fzf = {
                input = false,
                bind_sessions = false,
                args = {},
            }
        ";

        with_config(config, |settings| {
            let expected_settings = Settings {
                fzf: FzfSettings {
                    input: false,
                    bind_sessions: false,
                    args: vec![],
                },
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_and_merge_config() {
        let config = r#"
            return {
              uppercase_overrides = true,
              use_current_pane_path = true,
              editor = {
                args = {
                  "+ZenMode",
                  "-c",
                  "nmap q <cmd>silent wqa<cr>",
                },
              },
              fzf = {
                input = false,
                bind_sessions = true,
              },
            }
        "#;

        with_config(config, |settings| {
            let expected_settings = Settings {
                uppercase_overrides: true,
                use_current_pane_path: true,
                editor: EditorSettings {
                    args: vec![
                        "+ZenMode".into(),
                        "-c".into(),
                        "nmap q <cmd>silent wqa<cr>".into(),
                    ],
                    ..Default::default()
                },
                fzf: FzfSettings {
                    input: false,
                    bind_sessions: true,
                    ..Default::default()
                },
                ..Default::default()
            };

            assert_eq!(settings, expected_settings);
        });
    }

    #[test]
    fn test_parse_binding_no_popup() {
        let config = r#"
            muxi.config.muxi_prefix = "g"

            muxi.config.bindings = {
              j = { command = "tmux switch-client -l" }
            }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".into(),
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
            muxi.config.muxi_prefix = "g"

            muxi.config.bindings = {
              j = { popup = { width = "60%" }, command = "muxi sessions edit" }
            }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".into(),
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
                muxi.config.muxi_prefix = "g"

                muxi.config.bindings = {
                    j = { popup = {}, command = "muxi sessions edit" }
                }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".into(),
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
                muxi.config.muxi_prefix = "g"
                muxi.config.use_current_pane_path = true

                muxi.config.bindings = {
                    j = {
                        popup = { title = "my title", width = "75%", height = "60%" },
                        command = "muxi sessions edit"
                    }
                }
        "#;

        with_config(config, |settings| {
            let mut bindings: Bindings = BTreeMap::new();
            bindings.insert(
                "j".into(),
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

    #[test]
    fn test_parse_plugins() {
        let config = r#"
            return {
              plugins = {
                  "tmux-plugins/tmux-continuum",
                  "tmux-plugins/tmux-resurrect",
                  "https://gitlab.com/user/custom-plugin",
              }
            }
        "#;

        with_config(config, |settings| {
            assert_eq!(settings.plugins.len(), 3);
            assert_eq!(
                settings.plugins[0].url.as_ref().map(Url::as_str).unwrap(),
                "https://github.com/tmux-plugins/tmux-continuum"
            );
            assert_eq!(
                settings.plugins[1].url.as_ref().map(Url::as_str).unwrap(),
                "https://github.com/tmux-plugins/tmux-resurrect"
            );
            assert_eq!(
                settings.plugins[2].url.as_ref().map(Url::as_str).unwrap(),
                "https://gitlab.com/user/custom-plugin"
            );
        });
    }

    #[test]
    fn test_parse_plugins_table() {
        let config = r#"
            return {
              plugins = {
                  { url = "tmux-plugins/tmux-yank" },
                  { path = "/tmp/local-plugin" },
              }
            }
        "#;

        with_config(config, |settings| {
            assert_eq!(settings.plugins.len(), 2);
            assert!(settings.plugins[0].url.is_some());
            assert!(settings.plugins[0].path.is_none());
            assert!(settings.plugins[1].url.is_none());
            assert!(settings.plugins[1].path.is_some());
        });
    }

    #[test]
    fn test_parse_invalid_lua_reports_span() {
        let config = r#"
            return {
                plugins = {
                    ["a"] =
                }
            }
        "#;

        with_config_error(config, |error| match error {
            Error::LuaParse(diag) => {
                assert_eq!(diag.label, "Lua error at line 5");
                let source = diag.src.inner();
                let lines: Vec<&str> = source.lines().collect();
                assert_eq!(lines[4].trim(), "}");
                let span = diag.span.expect("expected span");
                assert_eq!(span.len(), lines[4].len());
            }
            other => panic!("expected LuaParse error, got {other:?}"),
        });
    }

    #[test]
    fn test_parse_invalid_type_reports_path() {
        let config = r#"
            return {
                tmux_prefix = "nope"
            }
        "#;

        with_config_error(config, |error| match error {
            Error::LuaDeserialize(diag) => {
                assert!(
                    diag.path.ends_with("tmux_prefix"),
                    "unexpected path: {}",
                    diag.path
                );
                assert!(
                    diag.message.contains("boolean"),
                    "unexpected message: {}",
                    diag.message
                );
                assert!(
                    diag.file.contains("init.lua"),
                    "unexpected file: {}",
                    diag.file
                );
            }
            other => panic!("expected LuaDeserialize error, got {other:?}"),
        });
    }
}

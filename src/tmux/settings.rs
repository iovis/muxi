use std::str::ParseBoolError;

#[derive(Debug, Default)]
pub struct Settings {
    pub muxi_prefix: Option<String>,
    pub tmux_prefix: Option<bool>,
    pub uppercase_overrides: Option<bool>,
    pub use_current_pane_path: Option<bool>,
}

impl Settings {
    pub fn new() -> Self {
        let muxi_prefix = super::get_option("@muxi-prefix");

        let tmux_prefix = super::get_option("@muxi-use-tmux-prefix")
            .map(|opt| as_bool(&opt).expect("`@muxi-use-tmux-prefix` should be true|on|false|off"));

        let uppercase_overrides = super::get_option("@muxi-uppercase-overrides").map(|opt| {
            as_bool(&opt).expect("`@muxi-uppercase-overrides` should be true|on|false|off")
        });

        let use_current_pane_path = super::get_option("@muxi-use-current-pane-path").map(|opt| {
            as_bool(&opt).expect("`@muxi-use-current-pane-path` should be true|on|false|off")
        });

        Self {
            muxi_prefix,
            tmux_prefix,
            uppercase_overrides,
            use_current_pane_path,
        }
    }
}

fn as_bool(value: &str) -> Result<bool, ParseBoolError> {
    match value {
        "on" => Ok(true),
        "off" => Ok(false),
        _ => value.parse(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod as_bool {
        use super::*;

        #[test]
        fn parses_true() {
            assert_eq!(as_bool("true"), Ok(true));
            assert_eq!(as_bool("on"), Ok(true));
        }

        #[test]
        fn parses_false() {
            assert_eq!(as_bool("false"), Ok(false));
            assert_eq!(as_bool("off"), Ok(false));
        }

        #[test]
        fn errors_otherwise() {
            assert!(as_bool("asdfas").is_err());
        }
    }
}

use std::path::PathBuf;

pub fn muxi_dir() -> PathBuf {
    let path = if let Ok(env_path) = std::env::var("MUXI_CONFIG_PATH") {
        PathBuf::from(env_path)
    } else if let Ok(env_path) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(env_path).join("muxi/")
    } else {
        PathBuf::from("~/.config/muxi/")
    };

    expand_tilde(path)
}

pub fn settings_file() -> PathBuf {
    muxi_dir().join("settings.toml")
}

pub fn sessions_file() -> PathBuf {
    muxi_dir().join("sessions.toml")
}

pub fn expand_tilde(path: PathBuf) -> PathBuf {
    if !path.starts_with("~") {
        return path;
    }

    let home_dir = dirs::home_dir().unwrap();
    let relative_path = path.strip_prefix("~").unwrap();

    home_dir.join(relative_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_env<F: FnOnce(PathBuf)>(name: &str, value: &str, test: F) {
        let home_dir = dirs::home_dir().unwrap();
        std::env::set_var(name, value);

        test(home_dir);

        std::env::remove_var(name);
    }

    #[test]
    fn test_path_muxi_env_set() {
        with_env("MUXI_CONFIG_PATH", "~/my/path", |home_dir: PathBuf| {
            assert_eq!(muxi_dir(), home_dir.join("my/path"));
        })
    }

    #[test]
    fn test_path_xdg_home_env_set() {
        with_env("XDG_CONFIG_HOME", "~/xdg/path", |home_dir: PathBuf| {
            assert_eq!(muxi_dir(), home_dir.join("xdg/path/muxi/"));
        })
    }

    #[test]
    fn test_path_fallback() {
        let home_dir = dirs::home_dir().unwrap();

        assert_eq!(muxi_dir(), home_dir.join(".config/muxi/"));
    }

    #[test]
    fn test_path_with_tilde() {
        let home_dir = dirs::home_dir().unwrap();

        let path = PathBuf::from("~/some/path");
        let expanded_path = expand_tilde(path);

        assert_eq!(expanded_path, home_dir.join("some/path"));
    }

    #[test]
    fn test_path_without_tilde() {
        let path = PathBuf::from("/some/path");
        let expanded_path = expand_tilde(path);

        assert_eq!(expanded_path, PathBuf::from("/some/path"));
    }
}

// TODO:
// - Read or create $MUXI_CONFIG_PATH/sessions.muxi

use std::env;
use std::path::{Path, PathBuf};

pub struct Config {
    path: PathBuf,
    // TODO: settings
}

impl Config {
    pub fn new() -> Self {
        let path = Self::path_from_env();

        Self::create_config_folder(&path);

        Self { path }
    }

    fn path_from_env() -> PathBuf {
        let path = if let Ok(env_path) = env::var("MUXI_CONFIG_PATH") {
            PathBuf::from(env_path)
        } else if let Ok(env_path) = env::var("XDG_CONFIG_HOME") {
            PathBuf::from(env_path).join("muxi/")
        } else {
            PathBuf::from("~/.config/muxi/")
        };

        Self::handle_tilde_in_path(path)
    }

    fn handle_tilde_in_path(path: PathBuf) -> PathBuf {
        if !path.starts_with("~") {
            return path;
        }

        let path = path.strip_prefix("~").unwrap();

        let home_str = env::var("HOME").expect("$HOME is not defined");
        let home = PathBuf::from(home_str);

        home.join(path)
    }

    #[cfg(not(test))]
    fn create_config_folder<P: AsRef<Path>>(path: P) {
        std::fs::create_dir_all(&path).expect("Couldn't create settings path");
    }

    #[cfg(test)]
    fn create_config_folder<P: AsRef<Path>>(path: P) {
        println!("Create {}", path.as_ref().to_string_lossy());
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_path_muxi_env_set() {
        env::set_var("MUXI_CONFIG_PATH", "~/my/path");
        let home_dir = PathBuf::from(env::var("HOME").unwrap());

        let config = Config::new();

        assert_eq!(config.path, home_dir.join("my/path"));
    }

    #[test]
    fn test_path_xdg_home_env_set() {
        env::set_var("XDG_CONFIG_HOME", "~/xdg/path");
        let home_dir = PathBuf::from(env::var("HOME").unwrap());

        let config = Config::new();

        assert_eq!(config.path, home_dir.join("xdg/path/muxi/"));
    }

    #[test]
    fn test_path_fallback() {
        let home_dir = PathBuf::from(env::var("HOME").unwrap());

        let config = Config::new();

        assert_eq!(config.path, home_dir.join(".config/muxi/"));
    }
}

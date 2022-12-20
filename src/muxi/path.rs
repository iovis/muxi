use std::path::PathBuf;

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

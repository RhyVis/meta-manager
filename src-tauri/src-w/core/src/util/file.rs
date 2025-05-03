use std::env::current_dir;
use std::path::PathBuf;

/// Get the current working directory
pub fn cd() -> PathBuf {
    current_dir().unwrap_or(PathBuf::from("."))
}

/// Get the current working directory and append a path
pub fn cd_with(append: &str) -> PathBuf {
    cd().join(append)
}

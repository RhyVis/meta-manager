use std::env::current_dir;
use std::fs;
use std::path::{Path, PathBuf};

/// Get the current working directory, appending a subdirectory in test, debug
pub fn cd() -> PathBuf {
    let base = current_dir().unwrap_or(PathBuf::from("."));

    #[cfg(any(debug_assertions, test))]
    {
        const SUB_DIR: &str = ".run";
        let workspace = base.join(SUB_DIR);
        if !workspace.exists() {
            fs::create_dir_all(&workspace).unwrap();
        }

        workspace
    }

    #[cfg(not(any(debug_assertions, test)))]
    {
        base
    }
}

/// Get the current working directory and append a path
pub fn cd_with(append: &str) -> PathBuf {
    cd().join(append)
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> anyhow::Result<()> {
    if !dst.as_ref().exists() {
        fs::create_dir_all(&dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.as_ref().join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

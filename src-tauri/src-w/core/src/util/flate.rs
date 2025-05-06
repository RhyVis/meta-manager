use std::path::Path;
use std::process::Command;
use std::{fs, io};
use tracing::info;
use unrar::error::UnrarError;

/// Extract a zip file to a specified directory
pub fn extract_zip(zip: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), io::Error> {
    let file = fs::File::open(&zip)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if !dst.as_ref().exists() {
        fs::create_dir_all(&dst)?;
    }

    for cur in 0..archive.len() {
        let mut file = archive.by_index(cur)?;

        let out = dst.as_ref().join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            let mut out_file = fs::File::create(&out)?;
            io::copy(&mut file, &mut out_file)?;
        }
    }

    Ok(())
}

pub fn extract_rar(
    rar: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    password: Option<&str>,
) -> Result<(), UnrarError> {
    let mut archive = match password {
        Some(pwd) => unrar::Archive::with_password(&rar, pwd.as_bytes()).open_for_processing()?,
        None => unrar::Archive::new(&rar).open_for_processing()?,
    };

    while let Some(header) = archive.read_header()? {
        archive = if header.entry().is_file() {
            header.extract_with_base(&dst)?
        } else {
            header.skip()?
        }
    }

    Ok(())
}

/// Extract a zip file with decryption to a specified directory
pub fn extract_zip_decrypt(
    zip: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    password: &str,
) -> Result<(), io::Error> {
    let file = fs::File::open(&zip)?;
    let mut archive = zip::ZipArchive::new(file)?;

    if !dst.as_ref().exists() {
        fs::create_dir_all(&dst)?;
    }

    for cur in 0..archive.len() {
        let mut file = archive.by_index_decrypt(cur, password.as_bytes())?;

        let out = dst.as_ref().join(file.mangled_name());

        if file.name().ends_with('/') {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }

            let mut out_file = fs::File::create(&out)?;
            io::copy(&mut file, &mut out_file)?;
        }
    }

    Ok(())
}

/// Compress a directory to a 7z file, optionally with a password and compression level
pub fn compress_dir_to_7z(
    input_dir: impl AsRef<Path>,
    output_file_path: impl AsRef<Path>,
    password: Option<&str>,
    compression_level: Option<u32>,
) -> Result<(), String> {
    if is_7z_in_path() {
        info!(
            "Using external 7z command in compressing {}",
            input_dir.as_ref().display()
        );
        compress_7z_dir_external(input_dir, output_file_path, password, compression_level)
            .map_err(|e| e.to_string())
    } else {
        info!(
            "Using internal 7z library in compressing {}",
            input_dir.as_ref().display()
        );
        compress_7z_dir_internal(input_dir, output_file_path, password, compression_level)
            .map_err(|e| e.to_string())
    }
}

fn is_7z_in_path() -> bool {
    match Command::new("7z").arg("--help").output() {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "windows")]
fn create_hidden_command(cmd: &str) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let mut command = Command::new(cmd);
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

#[cfg(not(target_os = "windows"))]
fn create_hidden_command(cmd: &str) -> Command {
    Command::new(cmd)
}

fn compress_7z_dir_external(
    input_dir: impl AsRef<Path>,
    output_file_path: impl AsRef<Path>,
    password: Option<&str>,
    compression_level: Option<u32>,
) -> Result<(), io::Error> {
    let mut command = create_hidden_command("7z");

    command.arg("a");

    let level = compression_level.unwrap_or(5);
    command.arg(format!("-mx={level}"));

    if let Some(pwd) = password {
        command.arg(format!("-p{pwd}"));
    }

    command.arg(output_file_path.as_ref());
    command.current_dir(input_dir.as_ref());
    command.arg("*");

    let output = command.output()?;
    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Fail in 7z command exec: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    Ok(())
}

fn compress_7z_dir_internal(
    input_dir: impl AsRef<Path>,
    output_file_path: impl AsRef<Path>,
    password: Option<&str>,
    compression_level: Option<u32>,
) -> Result<(), sevenz_rust2::Error> {
    use sevenz_rust2::lzma::LZMA2Options;
    use sevenz_rust2::{AesEncoderOptions, SevenZWriter};

    let mut writer = SevenZWriter::create(output_file_path)?;
    let compression_level = compression_level.unwrap_or(5);

    if let Some(pwd) = password {
        writer.set_content_methods(vec![
            AesEncoderOptions::new(pwd.into()).into(),
            LZMA2Options::with_preset(compression_level).into(),
        ]);
    } else {
        writer.set_content_methods(vec![LZMA2Options::with_preset(compression_level).into()]);
    }

    writer.push_source_path(input_dir, |_| true)?;
    writer.finish()?;

    Ok(())
}

pub fn decompress_7z_to_dir(
    input_file_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    password: Option<&str>,
) -> Result<(), String> {
    if is_7z_in_path() {
        info!(
            "Using external 7z command in decompressing {}",
            input_file_path.as_ref().display()
        );
        decompress_7z_to_dir_external(input_file_path, output_dir, password)
            .map_err(|e| e.to_string())
    } else {
        info!(
            "Using internal 7z library in decompressing {}",
            input_file_path.as_ref().display()
        );
        if let Some(pwd) = password {
            sevenz_rust2::decompress_file_with_password(input_file_path, output_dir, pwd.into())
                .map_err(|e| e.to_string())
        } else {
            sevenz_rust2::decompress_file(input_file_path, output_dir).map_err(|e| e.to_string())
        }
    }
}

fn decompress_7z_to_dir_external(
    input_file_path: impl AsRef<Path>,
    output_dir: impl AsRef<Path>,
    password: Option<&str>,
) -> Result<(), io::Error> {
    if !output_dir.as_ref().exists() {
        fs::create_dir_all(output_dir.as_ref())?;
    }

    let mut command = create_hidden_command("7z");
    command.arg("x");
    command.arg(input_file_path.as_ref());
    command.arg(format!("-o{}", output_dir.as_ref().display()));
    command.arg("-aoa");

    if let Some(pwd) = password {
        command.arg(format!("-p{pwd}"));
    }

    let output = command.output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Fail in 7z command exec: {}",
                String::from_utf8_lossy(&output.stderr)
            ),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::util::file::{cd_test, cd_test_clear};
    use std::fs;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;

    fn create_test_files(base_path: &Path) -> std::io::Result<()> {
        // 创建多层嵌套目录
        fs::create_dir_all(base_path.join("dir1/sub_dir1"))?;
        fs::create_dir_all(base_path.join("dir2/sub_dir2/sub_sub_dir"))?;

        // 创建不同大小的文本文件
        let mut file1 = File::create(base_path.join("file1.txt"))?;
        file1.write_all(b"TestFile1")?;

        let mut file2 = File::create(base_path.join("dir1/file2.txt"))?;
        file2.write_all(b"TestFile2 in dir1")?;

        // 1KB的文件
        let mut file3 = File::create(base_path.join("dir1/sub_dir1/file3.txt"))?;
        file3.write_all(&vec![b'A'; 1024])?;

        // 5KB的文件
        let mut file4 = File::create(base_path.join("dir2/sub_dir2/file4.txt"))?;
        file4.write_all(&vec![b'B'; 1024 * 5])?;

        // 20KB的二进制文件
        let mut bin_file = File::create(base_path.join("dir2/sub_dir2/sub_sub_dir/data.bin"))?;
        bin_file.write_all(&vec![0u8; 1024 * 20])?;

        Ok(())
    }

    #[test]
    fn test_7z() {
        let origin_path = cd_test().join("7z_test");
        if origin_path.exists() {
            fs::remove_dir_all(&origin_path).expect("Failed to remove origin path");
        }
        fs::create_dir_all(&origin_path).expect("Error creating origin path");

        create_test_files(&origin_path).unwrap();
        let out = cd_test().join("7z_test.7z");
        if out.exists() {
            fs::remove_file(&out).expect("Failed to remove output file");
        }

        println!("Test no password");
        let result = super::compress_dir_to_7z(&origin_path, &out, None, None);
        assert!(result.is_ok(), "Failed to compress directory: {:?}", result);

        let out_pwd = cd_test().join("7z_test_pwd.7z");
        if out_pwd.exists() {
            fs::remove_file(&out_pwd).expect("Failed to remove output file");
        }

        println!("Test with password");
        let result = super::compress_dir_to_7z(&origin_path, &out_pwd, Some("中文密码"), Some(9));
        assert!(
            result.is_ok(),
            "Failed to compress directory with password: {:?}",
            result
        );

        cd_test_clear();
    }
}

use crate::util::file;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("Failed to build game metadata: {0}")]
    BuilderError(#[from] GameMetadataBuilderError),

    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("FileSystem failure: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed in decompressing: {0}")]
    DecompressionError(String),

    #[error("Failed in decompressing RAR: {0}")]
    DecompressionRARError(#[from] unrar::error::UnrarError),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(tag = "platform", content = "id")]
pub enum GamePlatform {
    Unknown,
    Steam,
    DLSite,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum DeployType {
    Directory,
    CopyFile,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct GameTag {
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Builder)]
#[builder(
    try_setter,
    setter(into, strip_option),
    derive(Debug, Eq, PartialEq),
    pattern = "owned"
)]
pub struct GameMetadata {
    pub id: String,
    pub title: String,
    #[builder(default)]
    pub original_title: Option<String>,
    #[builder(default = "GamePlatform::Unknown")]
    pub platform: GamePlatform,
    #[builder(default)]
    pub platform_id: Option<String>,

    #[builder(default)]
    pub description: Option<String>,
    #[builder(default)]
    pub version: Option<String>,
    #[builder(default)]
    pub developer: Option<String>,
    #[builder(default)]
    pub publisher: Option<String>,
    #[builder(default)]
    pub release_date: Option<String>,

    #[builder(default)]
    pub archive_path: Option<String>,
    #[builder(default)]
    pub archive_password: Option<String>,
    #[builder(default)]
    pub deployed_path: Option<String>,
    #[builder(default)]
    pub deployed_type: Option<DeployType>,
    #[builder(default)]
    pub size_bytes: Option<u64>,

    #[builder(default)]
    pub tags: Vec<GameTag>,

    #[builder(default = "Utc::now()")]
    pub date_created: DateTime<Utc>,
    #[builder(default = "Utc::now()")]
    pub date_updated: DateTime<Utc>,
}

impl GameMetadata {
    fn initial_builder(title: &str) -> GameMetadataBuilder {
        GameMetadataBuilder::default()
            .id(Uuid::new_v4().to_string())
            .title(title)
    }

    pub fn from_steam(
        title: &str,
        app_id: &str,
        archive_path: &str,
    ) -> Result<GameMetadata, MetadataError> {
        GameMetadata::initial_builder(title)
            .platform(GamePlatform::Steam)
            .platform_id(app_id)
            .archive_path(archive_path)
            .build()
            .map_err(|e| e.into())
    }

    pub fn from_dl(
        title: &str,
        dl_id: &str,
        archive_path: &str,
    ) -> Result<GameMetadata, MetadataError> {
        GameMetadata::initial_builder(title)
            .platform(GamePlatform::DLSite)
            .platform_id(dl_id)
            .archive_path(archive_path)
            .build()
            .map_err(|e| e.into())
    }

    pub fn form_unknown(title: &str, archive_path: &str) -> Result<GameMetadata, MetadataError> {
        GameMetadata::initial_builder(title)
            .platform(GamePlatform::Unknown)
            .archive_path(archive_path)
            .build()
            .map_err(|e| e.into())
    }

    /// Calculate the size of the archive
    pub fn calculate_size(&mut self) -> Result<u64, MetadataError> {
        match self.archive_path.as_ref() {
            None => {
                warn!(
                    "Trying to calculate size of '{}' without an archive path",
                    &self.title
                );
                Ok(0)
            }
            Some(archive_path) => {
                let path = Path::new(archive_path);
                if !path.exists() {
                    let err = format!(
                        "Trying to calculate size of '{}' without a valid archive path",
                        &self.title
                    );
                    error!(err);
                    Err(MetadataError::InvalidMetadata(err))
                } else if path.is_file() {
                    match path.metadata() {
                        Ok(meta) => {
                            let size = meta.len();
                            self.size_bytes = Some(size);
                            info!("Calculated size of file {}: {}", path.display(), size);
                            Ok(size)
                        }
                        Err(err) => {
                            warn!(
                                "Failed to get metadata for file {}: {}",
                                path.display(),
                                err
                            );
                            Err(MetadataError::FileError(err))
                        }
                    }
                } else if path.is_dir() {
                    let calculated_size = walkdir::WalkDir::new(path)
                        .into_iter()
                        .filter_map(|entry| entry.ok())
                        .filter(|entry| entry.file_type().is_file())
                        .map(|entry| entry.metadata().ok().map(|meta| meta.len()).unwrap_or(0))
                        .sum::<u64>();
                    self.size_bytes = Some(calculated_size);
                    info!(
                        "Calculated size of directory {}: {}",
                        path.display(),
                        calculated_size
                    );
                    Ok(calculated_size)
                } else {
                    let err = format!(
                        "Unexpected path type for archive {}: {}",
                        archive_path,
                        path.display()
                    );
                    warn!(err);
                    Err(MetadataError::InvalidMetadata(err))
                }
            }
        }
    }

    pub fn remove_deploy_info(&mut self) {
        self.deployed_path = None;
        self.deployed_type = None;
    }

    pub fn mark_updated(&mut self) {
        self.date_updated = Utc::now();
    }

    pub fn deploy(&mut self, path: &str) -> Result<(), MetadataError> {
        // Check if the archive path is valid
        let archive_path = match self.archive_path.as_ref() {
            Some(archive_path) => {
                let path = Path::new(archive_path);
                if !path.exists() {
                    let err = format!(
                        "Trying to deploy '{}' without a valid archive path",
                        &self.title
                    );
                    warn!(err);
                    return Err(MetadataError::InvalidMetadata(err));
                }
                path
            }
            None => {
                let err = format!("Trying to deploy '{}' without an archive path", &self.title);
                warn!(err);
                return Err(MetadataError::InvalidOperation(err));
            }
        };

        // Check if the deployment path is valid
        let deploy_path = Path::new(path);
        if deploy_path.exists() {
            // Selected a file target, invalid
            if deploy_path.is_file() {
                let err = format!(
                    "Trying to deploy '{}' to an existing file path: {}",
                    &self.title,
                    deploy_path.display()
                );
                warn!(err);
                return Err(MetadataError::InvalidOperation(err));
            }
        } else {
            // Selected a directory target, create it
            warn!(
                "The selected path {} does not exist, creating it",
                deploy_path.display()
            );
            fs::create_dir_all(deploy_path)?;
        }

        fn check_target_empty(path: &Path, title: &str) -> Result<(), MetadataError> {
            if fs::read_dir(path)?.next().is_some() {
                let err = format!(
                    "Trying to deploy folder '{}' to a non-empty directory: {}",
                    title,
                    path.display()
                );
                warn!(err);
                Err(MetadataError::InvalidOperation(err))
            } else {
                Ok(())
            }
        }

        if archive_path.is_dir() {
            // Directory deployment
            check_target_empty(deploy_path, &self.title)?;

            info!(
                "Copying from {} to {}",
                archive_path.display(),
                deploy_path.display()
            );

            file::copy_dir_all(archive_path, deploy_path)?;

            self.deployed_path = Some(path.to_string());
            self.deployed_type = Some(DeployType::Directory);
            self.mark_updated();
        } else if archive_path.is_file() {
            // File deployment
            match archive_path
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_lowercase()
                .as_str()
            {
                "zip" => {
                    check_target_empty(deploy_path, &self.title)?;
                    info!(
                        "Decompressing ZIP {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    match &self.archive_password {
                        Some(password) => {
                            file::extract_zip_decrypt(archive_path, deploy_path, password)
                                .map_err(|e| MetadataError::DecompressionError(e.to_string()))?
                        }
                        None => file::extract_zip(archive_path, deploy_path)
                            .map_err(|e| MetadataError::DecompressionError(e.to_string()))?,
                    }
                    self.deployed_path = Some(path.to_string());
                    self.deployed_type = Some(DeployType::Directory);
                }
                "rar" => {
                    // Not fully implemented, but should work
                    check_target_empty(deploy_path, &self.title)?;
                    info!(
                        "Decompressing RAR {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    let mut archive = match &self.archive_password {
                        Some(pwd) => unrar::Archive::with_password(archive_path, pwd.as_bytes())
                            .open_for_processing()?,
                        None => unrar::Archive::new(archive_path).open_for_processing()?,
                    };

                    while let Some(header) = archive.read_header()? {
                        archive = if header.entry().is_file() {
                            header.extract_with_base(archive_path)?
                        } else {
                            header.skip()?
                        }
                    }

                    self.deployed_path = Some(path.to_string());
                    self.deployed_type = Some(DeployType::Directory);
                }
                "7z" => {
                    check_target_empty(deploy_path, &self.title)?;
                    info!(
                        "Decompressing 7z {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    if let Some(password) = &self.archive_password {
                        sevenz_rust2::decompress_file_with_password(
                            archive_path,
                            deploy_path,
                            password.as_str().into(),
                        )
                        .map_err(|e| MetadataError::DecompressionError(e.to_string()))?;
                    } else {
                        sevenz_rust2::decompress_file(archive_path, deploy_path)
                            .map_err(|e| MetadataError::DecompressionError(e.to_string()))?;
                    }
                    self.deployed_path = Some(path.to_string());
                    self.deployed_type = Some(DeployType::Directory);
                }
                _ => {
                    info!(
                        "Copying {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );

                    let target_file_path = deploy_path.join(archive_path.file_name().unwrap());
                    fs::copy(archive_path, &target_file_path)?;
                    self.deployed_path = Some(target_file_path.to_string_lossy().to_string());
                    self.deployed_type = Some(DeployType::CopyFile);
                }
            }
            self.mark_updated();
        } else {
            let err = format!(
                "Unexpected path type for archive {}: {}",
                self.archive_path
                    .as_ref()
                    .unwrap_or(&"!notfound!".to_string()),
                archive_path.display()
            );
            warn!(err);
            return Err(MetadataError::InvalidMetadata(err));
        }

        Ok(())
    }

    pub fn deploy_off(&mut self) -> Result<(), MetadataError> {
        fn err_invalid_path(m: &mut GameMetadata) -> Result<(), MetadataError> {
            m.remove_deploy_info();
            m.mark_updated();
            let err = format!(
                "Trying to deploy off '{}' without a valid deployed path",
                &m.title
            );
            error!(err);
            Err(MetadataError::InvalidOperation(err))
        }

        match (&self.deployed_path, &self.deployed_type) {
            (None, _) | (_, None) => {
                return err_invalid_path(self);
            }
            (Some(deploy_path), Some(deploy_type)) => {
                let deploy_path = Path::new(deploy_path);
                if !deploy_path.exists() {
                    return err_invalid_path(self);
                }

                match deploy_type {
                    DeployType::Directory => {
                        if !deploy_path.is_dir() {
                            return err_invalid_path(self);
                        }
                        info!("Clearing directory {}", deploy_path.display());

                        for entry in fs::read_dir(deploy_path)? {
                            let entry = entry?;
                            if entry.path().is_dir() {
                                fs::remove_dir_all(entry.path())?;
                            } else {
                                fs::remove_file(entry.path())?;
                            }
                        }

                        info!("Successfully cleared directory {}", &deploy_path.display());

                        self.remove_deploy_info();
                        self.mark_updated();
                    }
                    DeployType::CopyFile => {
                        if !deploy_path.is_file() {
                            return err_invalid_path(self);
                        }
                        info!("Deleting file {}", deploy_path.display());
                        fs::remove_file(deploy_path)?;
                        info!("Successfully deleted file {}", &deploy_path.display());

                        self.remove_deploy_info();
                        self.mark_updated();
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_game_metadata_build() {
        let m = GameMetadataBuilder::default()
            .id("12")
            .title("Test Game")
            .platform(GamePlatform::Steam)
            .build()
            .unwrap();
        assert_eq!(m.id, "12");
        assert_eq!(m.title, "Test Game");
        assert_eq!(m.platform, GamePlatform::Steam);
    }
}

use crate::util::{file, flate};
use bon::{Builder, builder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum ContentType {
    Unknown,
    Game,
    Comic,
    Novel,
    Music,
    Anime,
}

impl Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ContentType::Unknown => "Unknown".to_string(),
            ContentType::Game => "Game".to_string(),
            ContentType::Comic => "Comic".to_string(),
            ContentType::Novel => "Novel".to_string(),
            ContentType::Music => "Music".to_string(),
            ContentType::Anime => "Anime".to_string(),
        };
        write!(f, "{}", str)
    }
}

impl Default for ContentType {
    fn default() -> Self {
        ContentType::Unknown
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(tag = "platform", content = "id")]
pub enum Platform {
    Unknown,
    Steam,
    DLSite,
    Other(String),
}

impl Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Platform::Unknown => "Unknown".to_string(),
            Platform::Steam => "Steam".to_string(),
            Platform::DLSite => "DLSite".to_string(),
            Platform::Other(name) => name.clone(),
        };
        write!(f, "{}", str)
    }
}

impl Default for Platform {
    fn default() -> Self {
        Platform::Unknown
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum DeployType {
    Directory,
    CopyFile,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Tag {
    pub name: String,
    pub category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Builder)]
pub struct Metadata {
    #[serde(default = "metadata_default_id")]
    #[builder(default = metadata_default_id())]
    pub id: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub original_title: Option<String>,
    #[serde(default)]
    #[builder(default)]
    pub content_type: ContentType,
    #[serde(default)]
    pub platform: Platform,
    #[serde(default)]
    pub platform_id: Option<String>,

    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "metadata_default_version")]
    #[builder(default = metadata_default_version())]
    pub version: String,
    #[serde(default)]
    pub developer: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub release_date: Option<String>,

    #[serde(default)]
    pub archive_path: Option<String>,
    #[serde(default)]
    pub archive_password: Option<String>,
    #[serde(default)]
    pub deployed_path: Option<String>,
    #[serde(default)]
    pub deployed_type: Option<DeployType>,
    #[serde(default)]
    pub size_bytes: Option<u64>,

    #[serde(default)]
    #[builder(default)]
    pub tags: Vec<Tag>,

    #[serde(default = "Utc::now")]
    #[builder(default = Utc::now())]
    pub date_created: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    #[builder(default = Utc::now())]
    pub date_updated: DateTime<Utc>,
}

fn metadata_default_id() -> String {
    Uuid::new_v4().to_string()
}

fn metadata_default_version() -> String {
    const VERSION_DEFAULT: &str = "1.0";
    VERSION_DEFAULT.to_string()
}

impl Metadata {
    /// New on existing archive
    pub fn new(
        title: String,
        platform: Platform,
        platform_id: Option<String>,
        archive_path: String,
    ) -> Self {
        let build = Self::builder()
            .title(title)
            .platform(platform)
            .archive_path(archive_path);
        if let Some(platform_id) = platform_id {
            build.platform_id(platform_id).build()
        } else {
            build.build()
        }
    }

    pub fn new_on_create_archive(
        title: String,
        platform: Platform,
        platform_id: Option<String>,
        from_path: String,
        target_path: impl AsRef<Path>,
        password: Option<String>,
    ) -> Result<Self, MetadataError> {
        let from = Path::new(&from_path);
        if !from.is_dir() {
            warn!("Unexpected origin path: {}", from.display());
            return Err(MetadataError::InvalidOrigin(from_path));
        }

        flate::compress_7z(from_path, &target_path, password.as_deref(), Some(9))
            .map_err(MetadataError::CompressionError)?;

        let mut metadata = Self::new(
            title,
            platform,
            platform_id,
            target_path.as_ref().to_string_lossy().to_string(),
        );
        let _ = metadata.calculate_size();
        if let Some(pwd) = password {
            metadata.archive_password = Some(pwd);
        }

        Ok(metadata)
    }

    /// Calculate the size of the archive
    pub fn calculate_size(&mut self) -> Result<(), MetadataError> {
        match self.archive_path.as_ref() {
            None => {
                warn!(
                    "Trying to calculate size of '{}' without an archive path",
                    &self.title
                );
                Ok(())
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
                            info!("Calculated size of file {}: {}", path.display(), size);
                            self.size_bytes = Some(size);
                            Ok(())
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
                    info!(
                        "Calculated size of directory {}: {}",
                        path.display(),
                        calculated_size
                    );
                    self.size_bytes = Some(calculated_size);
                    Ok(())
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

    pub fn mark_updated(&mut self) {
        self.date_updated = Utc::now();
    }

    fn remove_deploy_info(&mut self) {
        self.deployed_path = None;
        self.deployed_type = None;
        self.mark_updated();
    }

    fn update_deployed_path(&mut self, path: String, deploy_type: DeployType) {
        self.deployed_path = Some(path);
        self.deployed_type = Some(deploy_type);
        self.mark_updated();
    }

    /// Check if the archive path is valid, if valid, return the [Path]
    fn validate_archive_path(&self) -> Result<&Path, MetadataError> {
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
        Ok(archive_path)
    }

    /// Check if the deployment target path is valid, if valid, return the [Path]
    fn validate_deploy_path<'a>(&self, path: &'a str) -> Result<&'a Path, MetadataError> {
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
        Ok(deploy_path)
    }

    /// Check if the target directory is empty, if not return [MetadataError::InvalidOperation]
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

    pub fn deploy(&mut self, path: &str) -> Result<(), MetadataError> {
        // Validation
        let archive_path = self.validate_archive_path()?;
        let deploy_path = self.validate_deploy_path(path)?;

        if archive_path.is_dir() {
            // Directory deployment
            Self::check_target_empty(deploy_path, &self.title)?;

            info!(
                "Copying from {} to {}",
                archive_path.display(),
                deploy_path.display()
            );

            file::copy_dir_all(archive_path, deploy_path)?;

            self.update_deployed_path(path.to_string(), DeployType::Directory);
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
                    Self::check_target_empty(deploy_path, &self.title)?;
                    info!(
                        "Decompressing ZIP {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    flate::decompress_zip(
                        archive_path,
                        deploy_path,
                        self.archive_password.as_deref(),
                    )
                    .map_err(|e| MetadataError::DecompressionError(e.to_string()))?;
                    self.update_deployed_path(path.to_string(), DeployType::Directory);
                }
                "rar" => {
                    // Not fully implemented, but should work
                    Self::check_target_empty(deploy_path, &self.title)?;
                    info!(
                        "Decompressing RAR {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    flate::decompress_rar(
                        archive_path,
                        deploy_path,
                        self.archive_password.as_deref(),
                    )?;
                    self.update_deployed_path(path.to_string(), DeployType::Directory);
                }
                "7z" => {
                    Self::check_target_empty(deploy_path, &self.title)?;
                    info!(
                        "Decompressing 7z {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    flate::decompress_7z(
                        archive_path,
                        deploy_path,
                        self.archive_password.as_deref(),
                    )
                    .map_err(MetadataError::DecompressionError)?;
                    self.update_deployed_path(path.to_string(), DeployType::Directory);
                }
                _ => {
                    info!(
                        "Copying {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );

                    let target_file_path = deploy_path.join(archive_path.file_name().unwrap());
                    fs::copy(archive_path, &target_file_path)?;
                    self.update_deployed_path(path.to_string(), DeployType::CopyFile);
                }
            }
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
        fn err_invalid_path(m: &mut Metadata) -> Result<(), MetadataError> {
            m.remove_deploy_info();
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
                    }
                    DeployType::CopyFile => {
                        if !deploy_path.is_file() {
                            return err_invalid_path(self);
                        }
                        info!("Deleting file {}", deploy_path.display());
                        fs::remove_file(deploy_path)?;
                        info!("Successfully deleted file {}", &deploy_path.display());

                        self.remove_deploy_info();
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum MetadataError {
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),

    #[error("The provided origin path is invalid: {0}")]
    InvalidOrigin(String),

    #[error("FileSystem failure: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to compress: {0}")]
    CompressionError(String),

    #[error("Failed in decompressing: {0}")]
    DecompressionError(String),

    #[error("Failed in decompressing RAR: {0}")]
    DecompressionRARError(#[from] unrar::error::UnrarError),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

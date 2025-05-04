use crate::util::file;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Cursor;
use std::path::Path;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(tag = "platform", content = "id")]
pub enum GamePlatform {
    Steam,
    DLSite,
    Other(String),
    Unknown,
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
    pub release_date: Option<DateTime<Utc>>,

    #[builder(default)]
    pub archive_path: Option<String>,
    #[builder(default)]
    pub archive_path_password: Option<String>,
    #[builder(default)]
    pub deployed_path: Option<String>,
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
    ) -> Result<GameMetadata, GameMetadataBuilderError> {
        GameMetadata::initial_builder(title)
            .platform(GamePlatform::Steam)
            .platform_id(app_id)
            .archive_path(archive_path)
            .build()
    }

    pub fn from_dl(
        title: &str,
        dl_id: &str,
        archive_path: &str,
    ) -> Result<GameMetadata, GameMetadataBuilderError> {
        GameMetadata::initial_builder(title)
            .platform(GamePlatform::DLSite)
            .platform_id(dl_id)
            .archive_path(archive_path)
            .build()
    }

    pub fn form_unknown(
        title: &str,
        archive_path: &str,
    ) -> Result<GameMetadata, GameMetadataBuilderError> {
        GameMetadata::initial_builder(title)
            .platform(GamePlatform::Unknown)
            .archive_path(archive_path)
            .build()
    }

    /// Calculate the size of the archive
    pub fn calculate_size(&mut self) -> anyhow::Result<u64> {
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
                    Err(anyhow::anyhow!("{:?} does not exist", path))
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
                            Err(anyhow::anyhow!(
                                "Failed to get metadata for file {}: {}",
                                path.display(),
                                err
                            ))
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
                    warn!("Unexpected path type: {}", archive_path);
                    Err(anyhow::anyhow!("Unexpected path type: {}", archive_path))
                }
            }
        }
    }

    pub fn mark_updated(&mut self) {
        self.date_updated = Utc::now();
    }

    pub fn deploy(&self, path: &str) -> anyhow::Result<()> {
        let archive_path = match self.archive_path.as_ref() {
            Some(archive_path) => {
                let path = Path::new(archive_path);
                if !path.exists() {
                    warn!(
                        "Trying to deploy '{}' without a valid archive path",
                        &self.title
                    );
                    return Err(anyhow::anyhow!(
                        "Archive path for '{}' not exists",
                        &self.title
                    ));
                }
                path
            }
            None => {
                warn!("Trying to deploy '{}' without an archive path", &self.title);
                return Err(anyhow::anyhow!("No archive path for '{}'", &self.title));
            }
        };
        let deploy_path = Path::new(path);
        if deploy_path.exists() {
            warn!("Trying to deploy to an existing path: {}", path);
            return Err(anyhow::anyhow!("{:?} already exists", path));
        }
        fs::create_dir_all(deploy_path.parent().unwrap())?;

        if archive_path.is_dir() {
            info!(
                "Copying {} to {}",
                archive_path.display(),
                deploy_path.display()
            );
            file::copy_dir_all(archive_path, deploy_path)?;
        } else if archive_path.is_file() {
            match archive_path
                .extension()
                .unwrap()
                .to_string_lossy()
                .to_lowercase()
                .as_str()
            {
                "zip" => {
                    info!(
                        "Decompressing ZIP {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    zip_extract::extract(Cursor::new(fs::read(archive_path)?), deploy_path, false)?;
                }
                "7z" => {
                    info!(
                        "Decompressing 7z {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    if let Some(password) = &self.archive_path_password {
                        sevenz_rust2::decompress_file_with_password(
                            archive_path,
                            deploy_path,
                            password.as_str().into(),
                        )?
                    } else {
                        sevenz_rust2::decompress_file(archive_path, deploy_path)?;
                    }
                }
                _ => {
                    info!(
                        "Copying {} to {}",
                        archive_path.display(),
                        deploy_path.display()
                    );
                    fs::copy(
                        archive_path,
                        deploy_path.join(archive_path.file_name().unwrap()),
                    )?;
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

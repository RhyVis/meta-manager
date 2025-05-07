use crate::data::metadata::{Metadata, MetadataError, Platform};
use crate::foundation::config::{get_clone as config_get_clone, get_data_dir};
use chrono::Utc;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use std::time::SystemTime;
use thiserror::Error;
use tracing::{error, info, warn};

const LIB_FILE_NAME: &str = "library.redb";
const LIB_FILE_EXPORT: &str = "library.json";
const LIB_FILE_STEM: &str = "library";
const LIB_FILE_EXT: &str = "redb";
const LIB_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("LIBRARY");

const ARCHIVE_DIR_NAME: &str = "archive";
const BACKUP_DIR_NAME: &str = "backup";

fn library() -> &'static Database {
    static LIBRARY: OnceLock<Database> = OnceLock::new();
    fn backup(source_path: impl AsRef<Path>, backup_dir: impl AsRef<Path>) {
        if source_path.as_ref().exists() {
            if !backup_dir.as_ref().exists() {
                fs::create_dir_all(&backup_dir).unwrap();
            }

            let mut backups: Vec<_> = fs::read_dir(&backup_dir)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .collect();

            if backups.len() >= 4 {
                backups.sort_by_key(|entry| {
                    entry
                        .metadata()
                        .and_then(|m| m.modified())
                        .unwrap_or_else(|_| SystemTime::UNIX_EPOCH)
                });
                if let Some(oldest) = backups.first() {
                    if let Err(e) = fs::remove_file(oldest.path()) {
                        error!("Failed to remove old backup: {}", e);
                    } else {
                        info!("Removed old backup: {}", oldest.path().display());
                    }
                }
            }

            let backup_path = backup_dir.as_ref().join(format!(
                "{LIB_FILE_STEM}-{}.{LIB_FILE_EXT}",
                Utc::now().format("%Y%m%d-%H%M%S"),
            ));
            if let Err(e) = fs::copy(&source_path, &backup_path) {
                error!("Failed to copy library to backup: {}", e);
            } else {
                info!("Library copied to backup: {}", backup_path.display());
            }
        }
    }
    LIBRARY.get_or_init(|| {
        let data_dir = config_get_clone().unwrap().data_dir();
        let path = data_dir.join(LIB_FILE_NAME);
        backup(&path, data_dir.join(BACKUP_DIR_NAME));
        Database::create(path).unwrap()
    })
}

fn lib_internal_add_nocheck(metadata: Metadata) -> Result<(), LibraryError> {
    let to_save = bson::to_vec(&metadata).map_err(LibraryError::SerializeError)?;
    let write = library().begin_write()?;
    {
        let mut table = write.open_table(LIB_TABLE)?;
        table.insert(metadata.id.as_str(), to_save)?;
    }
    write.commit()?;
    Ok(())
}

pub fn lib_fresh() -> Result<(), LibraryError> {
    let write = library().begin_write()?;
    {
        let table = write.open_table(LIB_TABLE)?;
        table.get("fresh!")?;
    }
    write.commit()?;
    Ok(())
}

/// Gets a [Metadata] copy with the given id, if it exists
pub fn lib_get(id: &str) -> Result<Metadata, LibraryError> {
    let read = library().begin_read()?;
    let table = read.open_table(LIB_TABLE)?;
    let raw = table.get(id)?;
    match raw {
        Some(raw) => {
            let metadata =
                bson::from_slice::<Metadata>(&raw.value()).map_err(LibraryError::ParseError)?;
            Ok(metadata)
        }
        None => Err(LibraryError::NotFound(id.to_string())),
    }
}

/// Gets all the entries in the library, for compatibility with the old JSON format
pub fn lib_get_all() -> Result<Library, LibraryError> {
    let read = library().begin_read()?;
    let table = read.open_table(LIB_TABLE)?;
    let mut entries = Vec::new();
    for entry in table.iter()? {
        let (_, raw) = entry?;
        let metadata =
            bson::from_slice::<Metadata>(&raw.value()).map_err(LibraryError::ParseError)?;
        entries.push(metadata);
    }
    Ok(Library { entries })
}

/// Adds a [Metadata] to the library
pub fn lib_add(mut data: Metadata) -> Result<(), LibraryError> {
    if let Ok(existed) = lib_get(&data.id) {
        // Update mode
        if data.archive_path != existed.archive_path {
            // Update size
            let _ = data.calculate_size();
        }
        data.mark_updated();
    }
    lib_internal_add_nocheck(data)
}

/// No difference from [lib_add], just migration
pub fn lib_rep(data: Metadata) -> Result<(), LibraryError> {
    lib_add(data)
}

/// Removes a [Metadata] from the library
pub fn lib_del(id: &str) -> Result<(), LibraryError> {
    let write = library().begin_write()?;
    {
        let mut table = write.open_table(LIB_TABLE)?;
        table.remove(id)?;
    }
    write.commit()?;
    Ok(())
}

pub fn lib_export() -> Result<(), LibraryError> {
    let all = lib_get_all()?;
    let serialized = serde_json::to_string_pretty(&all)?;
    let path = get_data_dir()?.join(LIB_FILE_EXPORT);
    fs::write(&path, serialized)?;
    info!("Library exported to: {}", path.display());
    Ok(())
}

pub fn lib_import() -> Result<bool, LibraryError> {
    let path = get_data_dir()?.join(LIB_FILE_EXPORT);
    if !path.exists() || !path.is_file() {
        warn!("Exported library file not exists: {}", path.display());
        return Ok(false);
    }
    let lib = serde_json::from_str::<Library>(fs::read_to_string(&path)?.as_str())?;
    info!(
        "Trying to import {} entries from {}",
        lib.entries.len(),
        path.display()
    );
    let write = library().begin_write()?;
    {
        let mut table = write.open_table(LIB_TABLE)?;
        for entry in lib.entries {
            table.insert(
                entry.id.as_str(),
                &bson::to_vec(&entry).map_err(LibraryError::SerializeError)?,
            )?;
        }
    }
    write.commit()?;
    info!("Library imported from: {}", path.display());

    Ok(true)
}

pub fn lib_delegate_create(
    title: String,
    platform: Platform,
    platform_id: Option<String>,
    from_path: String,
    password: Option<String>,
) -> Result<(), LibraryError> {
    let config = config_get_clone()?;
    let path_to_dir = config
        .data_dir()
        .join(ARCHIVE_DIR_NAME)
        .join(platform.to_string());
    if !path_to_dir.exists() {
        fs::create_dir_all(&path_to_dir)?;
    }
    let path_to_archive = path_to_dir.join(format!(
        "{}.7z",
        platform_id
            .clone()
            .take_if(|s| !s.is_empty())
            .unwrap_or(Utc::now().format("ANONYMOUS-%Y%m%d-%H%M%S").to_string()),
    ));

    info!("Creating new archive at: {}", path_to_archive.display());

    let metadata = Metadata::new_on_create_archive(
        title,
        platform,
        platform_id,
        from_path,
        path_to_archive,
        password,
    )
    .map_err(LibraryError::CreateError)?;

    lib_internal_add_nocheck(metadata)
}

/// Get [Metadata] from the library and deploy it
pub fn lib_delegate_deploy(id: &str, path: &str) -> Result<(), LibraryError> {
    let mut g = lib_get(id)?;
    g.deploy(path)
        .map_err(|e| LibraryError::DeploymentError(e, id.to_string()))?;
    lib_add(g)
}

/// Get [Metadata] from the library and deploy it off
pub fn lib_delegate_deploy_off(id: &str) -> Result<(), LibraryError> {
    let mut g = lib_get(id)?;
    match g.deploy_off() {
        Ok(_) => lib_add(g),
        Err(err) => match err {
            MetadataError::InvalidOperation(_) => {
                // Update the info
                lib_add(g)?;
                Err(LibraryError::DeploymentOffError(err, id.to_string()))
            }
            _ => Err(LibraryError::DeploymentOffError(err, id.to_string()))?,
        },
    }
}

/// Since the library has switched to [redb],
/// This struct should only be used when passing data to the UI
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
pub struct Library {
    entries: Vec<Metadata>,
}

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Metadata with id {0} not found")]
    NotFound(String),

    #[error("Failed to read/write library from file: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to serialize library into bson: {0}")]
    SerializeError(bson::ser::Error),

    #[error("Failed to parse library file: {0}")]
    ParseError(bson::de::Error),

    #[error("Failed to create metadata entry: {0}")]
    CreateError(MetadataError),

    #[error("Failed in deployment for {1}: {0}")]
    DeploymentError(MetadataError, String),

    #[error("Failed in deployment off for {1}: {0}")]
    DeploymentOffError(MetadataError, String),

    #[error("Failed with config: {0}")]
    ConfigError(#[from] crate::foundation::config::ConfigError),

    #[error("Failed in database opt: {0}")]
    TransactionError(#[from] redb::TransactionError),

    #[error("Failed in database opt: {0}")]
    TableError(#[from] redb::TableError),

    #[error("Failed to storage data: {0}")]
    StorageError(#[from] redb::StorageError),

    #[error("Failed to commit data: {0}")]
    CommitError(#[from] redb::CommitError),

    #[error("Failed to export/import as JSON: {0}")]
    JSONError(#[from] serde_json::Error),

    #[error("Failed to acquire lock")]
    LockError,
}

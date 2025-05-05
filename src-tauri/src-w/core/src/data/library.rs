use crate::data::unit::{GameMetadata, MetadataError};
use crate::foundation::config::get_clone as config_get_clone;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use thiserror::Error;
use tracing::error;

const LIB_FILE_NAME: &str = "library.redb";
const LIB_TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("LIBRARY");

fn library() -> &'static Database {
    static LIBRARY: OnceLock<Database> = OnceLock::new();
    LIBRARY.get_or_init(|| {
        let path = config_get_clone().unwrap().data_dir().join(LIB_FILE_NAME);
        Database::create(path).unwrap()
    })
}

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Metadata with id {0} not found")]
    NotFound(String),

    #[error("Failed to read/write library from file: {0}")]
    FileError(std::io::Error),

    #[error("Failed to serialize library into bson: {0}")]
    SerializeError(bson::ser::Error),

    #[error("Failed to parse library file: {0}")]
    ParseError(bson::de::Error),

    #[error("Failed in deployment for {1}: {0}")]
    DeploymentError(MetadataError, String),

    #[error("Failed in deployment off for {1}: {0}")]
    DeploymentOffError(MetadataError, String),

    #[error("Failed with config: {0}")]
    ConfigError(crate::foundation::config::ConfigError),

    #[error("Failed in database opt: {0}")]
    TransactionError(#[from] redb::TransactionError),

    #[error("Failed in database opt: {0}")]
    TableError(#[from] redb::TableError),

    #[error("Failed to storage data: {0}")]
    StorageError(#[from] redb::StorageError),

    #[error("Failed to commit data: {0}")]
    CommitError(#[from] redb::CommitError),

    #[error("Failed to acquire lock")]
    LockError,
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

/// Gets a [GameMetadata] copy with the given id, if it exists
pub fn lib_get(id: &str) -> Result<GameMetadata, LibraryError> {
    let read = library().begin_read()?;
    let table = read.open_table(LIB_TABLE)?;
    let raw = table.get(id)?;
    match raw {
        Some(raw) => {
            let metadata =
                bson::from_slice::<GameMetadata>(&raw.value()).map_err(LibraryError::ParseError)?;
            Ok(metadata)
        }
        None => Err(LibraryError::NotFound(id.to_string())),
    }
}

/// Gets all the entries in the library, for compatibility with the old JSON format
pub fn lib_get_all() -> Result<GameLibrary, LibraryError> {
    let read = library().begin_read()?;
    let table = read.open_table(LIB_TABLE)?;
    let mut entries = Vec::new();
    for entry in table.iter()? {
        let (_, raw) = entry?;
        let metadata =
            bson::from_slice::<GameMetadata>(&raw.value()).map_err(LibraryError::ParseError)?;
        entries.push(metadata);
    }
    Ok(GameLibrary { entries })
}

/// Adds a [GameMetadata] to the library
pub fn lib_add(mut game: GameMetadata) -> Result<(), LibraryError> {
    if let Ok(existed) = lib_get(&game.id) {
        // Update mode
        game.mark_updated();
        if game.archive_path != existed.archive_path {
            // Update size
            let _ = game.calculate_size();
        }
    }

    let to_save = bson::to_vec(&game).map_err(LibraryError::SerializeError)?;
    let write = library().begin_write()?;
    {
        let mut table = write.open_table(LIB_TABLE)?;
        table.insert(game.id.as_str(), to_save)?;
    }
    write.commit()?;
    Ok(())
}

/// No difference from [lib_add], just migration
pub fn lib_rep(game: GameMetadata) -> Result<(), LibraryError> {
    lib_add(game)
}

/// Removes a [GameMetadata] from the library
pub fn lib_del(id: &str) -> Result<(), LibraryError> {
    let write = library().begin_write()?;
    {
        let mut table = write.open_table(LIB_TABLE)?;
        table.remove(id)?;
    }
    write.commit()?;
    Ok(())
}

/// Get [GameMetadata] from the library and deploy it
pub fn lib_delegate_deploy(id: &str, path: &str) -> Result<(), LibraryError> {
    let mut g = lib_get(id)?;
    g.deploy(path)
        .map_err(|e| LibraryError::DeploymentError(e, id.to_string()))?;
    lib_add(g)
}

/// Get [GameMetadata] from the library and deploy it off
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
pub struct GameLibrary {
    entries: Vec<GameMetadata>,
}

use crate::data::unit::GameMetadata;
use crate::foundation::config::get_clone as config_get_clone;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{OnceLock, RwLock, RwLockWriteGuard};
use thiserror::Error;
use tracing::{error, info};

const LIB_FILE_NAME: &str = "library.json";

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("Metadata with id {0} not found")]
    NotFound(String),

    #[error("Failed to read/write library from file: {0}")]
    FileError(std::io::Error),

    #[error("Failed to serialize library into file: {0}")]
    SerializeError(serde_json::Error),

    #[error("Failed to parse library file: {0}")]
    ParseError(serde_json::Error),

    #[error("Failed with config: {0}")]
    ConfigError(crate::foundation::config::ConfigError),

    #[error("Failed to acquire lock")]
    LockError,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
pub struct GameLibrary {
    entries: Vec<GameMetadata>,
}

impl GameLibrary {
    fn save_internal(&self, path: &PathBuf) -> Result<(), LibraryError> {
        match serde_json::to_string_pretty(&self) {
            Ok(s) => {
                fs::write(path, s).map_err(LibraryError::FileError)?;
                Ok(())
            }
            Err(e) => {
                error!("Failed to serialize game library: {e}");
                Err(LibraryError::SerializeError(e))
            }
        }
    }

    pub fn save(&self) -> Result<(), LibraryError> {
        let path_lib = config_get_clone()
            .map_err(LibraryError::ConfigError)?
            .data_dir()
            .join(LIB_FILE_NAME);
        info!("Saving library file on {}", &path_lib.display());
        self.save_internal(&path_lib)
    }

    pub fn get_game(&self, id: &str) -> Option<&GameMetadata> {
        self.entries.iter().find(|game| game.id == id)
    }

    pub fn add_game(&mut self, game: GameMetadata) -> Result<(), LibraryError> {
        info!("Adding metadata for {}", game.id);
        self.entries.push(game);
        self.save()?;
        Ok(())
    }

    pub fn replace_game(&mut self, mut game: GameMetadata) -> Result<(), LibraryError> {
        if let Some(pos) = self.entries.iter().position(|g| g.id == game.id) {
            info!("Updating metadata for {}", game.id);
            game.mark_updated();
            self.entries[pos] = game;
            self.save()
        } else {
            Err(LibraryError::NotFound(game.id))
        }
    }

    pub fn del_game(&mut self, id: &str) -> Result<(), LibraryError> {
        if let Some(pos) = self.entries.iter().position(|game| game.id == id) {
            info!("Deleting metadata for {}", id);
            self.entries.remove(pos);
            self.save()?;
            Ok(())
        } else {
            Err(LibraryError::NotFound(id.to_string()))
        }
    }

    pub fn load_or_init() -> Result<Self, LibraryError> {
        let path_lib = config_get_clone()
            .map_err(LibraryError::ConfigError)?
            .data_dir()
            .join(LIB_FILE_NAME);
        info!("Reading library file on {}", &path_lib.display());

        fn save_default(path: &PathBuf) -> Result<GameLibrary, LibraryError> {
            info!("Generating default config at {}", &path.display());
            let default = GameLibrary::default();
            if let Err(err) = default.save_internal(path) {
                error!("Failed to generate config at {}: {}", &path.display(), err);
                return Err(err);
            }
            Ok(default)
        }

        if path_lib.exists() && path_lib.is_file() {
            match fs::read_to_string(&path_lib) {
                Ok(contents) => match serde_json::from_str(contents.as_str()) {
                    Ok(game_library) => Ok(game_library),
                    Err(e) => {
                        error!("Failed to parse library file: {e}");
                        Ok(save_default(&path_lib)?)
                    }
                },
                Err(e) => {
                    error!("Failed to read library file: {e}");
                    Ok(save_default(&path_lib)?)
                }
            }
        } else {
            info!("Default library file not found");
            Ok(save_default(&path_lib)?)
        }
    }
}

fn library() -> &'static RwLock<GameLibrary> {
    static LIBRARY: OnceLock<RwLock<GameLibrary>> = OnceLock::new();
    LIBRARY.get_or_init(|| RwLock::new(GameLibrary::default()))
}

/// Reloads the library from disk
pub fn reload() -> Result<(), LibraryError> {
    match library().write() {
        Ok(mut lib) => {
            *lib = GameLibrary::load_or_init()?;
            Ok(())
        }
        Err(e) => {
            error!("Failed to acquire write lock: {e}");
            Err(LibraryError::LockError)
        }
    }
}

/// Gets a write lock to the library
pub fn get_write() -> Result<RwLockWriteGuard<'static, GameLibrary>, LibraryError> {
    match library().write() {
        Ok(lib) => Ok(lib),
        Err(e) => {
            error!("Failed to acquire write lock: {e}");
            Err(LibraryError::LockError)
        }
    }
}

/// Gets a clone of the library, used for passing info to UI
pub fn get_clone() -> Result<GameLibrary, LibraryError> {
    match library().read() {
        Ok(lib) => Ok(lib.clone()),
        Err(e) => {
            error!("Failed to acquire write lock: {e}");
            Err(LibraryError::LockError)
        }
    }
}

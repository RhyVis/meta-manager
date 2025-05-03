use crate::util::file::cd_with;
use config::{Config, FileFormat};
use serde::{Deserialize, Serialize};
use std::sync::{OnceLock, RwLock, RwLockReadGuard};
use thiserror::Error;
use tracing::error;

/// Configuration for the application
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    #[serde(default)]
    pub data_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            data_dir: "data".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read/write config from file: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Failed to load config from file or env")]
    LoadError,

    #[error("Failed to parse config: {0}")]
    ParseError(#[from] config::ConfigError),

    #[error("Failed to serialize config: {0}")]
    SerializationError(#[from] toml::ser::Error),

    #[error("Failed to acquire lock")]
    LockError,
}

fn config() -> &'static RwLock<AppConfig> {
    static CONFIG: OnceLock<RwLock<AppConfig>> = OnceLock::new();
    CONFIG.get_or_init(|| RwLock::new(load().unwrap_or(AppConfig::default())))
}

fn load() -> Result<AppConfig, ConfigError> {
    match Config::builder()
        .add_source(
            config::File::with_name("config")
                .format(FileFormat::Toml)
                .required(false),
        )
        .add_source(
            config::Environment::with_prefix("META_CONF")
                .separator("__")
                .list_separator(" "),
        )
        .build()
    {
        Ok(raw) => match raw.try_deserialize::<AppConfig>() {
            Ok(config) => Ok(config),
            Err(err) => {
                error!("Failed to parse config: {}", err);
                Err(ConfigError::ParseError(err))
            }
        },
        Err(err) => {
            error!("Failed to load config: {}", err);
            Err(ConfigError::LoadError)
        }
    }
}

/// Reload the config from the file, env
pub fn reload() -> Result<(), ConfigError> {
    match config().write() {
        Ok(mut config) => match load() {
            Ok(new_config) => {
                *config = new_config;
                Ok(())
            }
            Err(err) => {
                error!("Failed to reload config: {}", err);
                Err(err)
            }
        },
        Err(err) => {
            error!("Failed to acquire write lock: {}", err);
            Err(ConfigError::LockError)
        }
    }
}

/// Save the config to the default file
#[allow(dead_code)]
fn save() -> Result<(), ConfigError> {
    let config_c = get()?.clone();
    let config_path = cd_with("config.toml");

    match toml::to_string_pretty(&config_c) {
        Ok(config) => std::fs::write(config_path, config).map_err(ConfigError::FileError),
        Err(err) => {
            error!("Failed to serialize config: {}", err);
            Err(ConfigError::SerializationError(err))
        }
    }
}

/// Get a read lock to the config
pub fn get() -> Result<RwLockReadGuard<'static, AppConfig>, ConfigError> {
    config().read().map_err(|e| {
        error!("Failed to acquire read lock: {e}");
        ConfigError::LockError
    })
}

/// Get a clone of the config
pub fn get_clone() -> Result<AppConfig, ConfigError> {
    config()
        .read()
        .map_err(|e| {
            error!("Failed to acquire read lock: {e}");
            ConfigError::LockError
        })
        .map(|config| config.clone())
}

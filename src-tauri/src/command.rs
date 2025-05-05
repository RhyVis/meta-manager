use m_core::data::library::{
    GameLibrary, lib_add, lib_del, lib_delegate_deploy, lib_delegate_deploy_off, lib_get_all,
};
use m_core::data::unit::GameMetadata;
use serde::{Deserialize, Serialize};
use tauri::command;
use tracing::error;

fn internal_library_add(game: GameMetadata) -> Result<(), String> {
    lib_add(game).map_err(|err| {
        let err_msg = format!("Failed to add game: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_get() -> Result<GameLibrary, String> {
    lib_get_all().map_err(|err| {
        let err_msg = format!("Failed to get library: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_set(data: GameMetadata) -> Result<(), String> {
    internal_library_add(data)
}

#[command]
pub fn library_del(id: String) -> Result<(), String> {
    lib_del(id.as_str()).map_err(|err| {
        let err_msg = format!("Failed to delete game: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_deploy(id: String, path: String) -> Result<(), String> {
    lib_delegate_deploy(id.as_str(), path.as_str()).map_err(|err| {
        let err_msg = format!("Failed to deploy game: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_deploy_off(id: String) -> Result<(), String> {
    lib_delegate_deploy_off(id.as_str()).map_err(|err| {
        let err_msg = format!("Failed to off deploy game: {err}");
        error!(err_msg);
        err_msg
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub name: PlatformLimited,
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlatformLimited {
    Unknown,
    Steam,
    DLSite,
}

#[command]
pub fn metadata_add(title: String, archive_path: String, info: PlatformInfo) -> Result<(), String> {
    let mut game = match info.name {
        PlatformLimited::Steam => GameMetadata::from_steam(
            title.as_str(),
            info.id.unwrap().as_str(),
            archive_path.as_str(),
        )
        .map_err(|err| err.to_string())?,
        PlatformLimited::DLSite => GameMetadata::from_dl(
            title.as_str(),
            info.id.unwrap().as_str(),
            archive_path.as_str(),
        )
        .map_err(|err| err.to_string())?,
        PlatformLimited::Unknown => {
            GameMetadata::form_unknown(title.as_str(), archive_path.as_str())
                .map_err(|err| err.to_string())?
        }
    };
    let _ = game.calculate_size();
    internal_library_add(game)
}

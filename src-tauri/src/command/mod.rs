mod bridge;

use crate::command::bridge::PlatformInfo;
use m_core::data::library::{
    Library, lib_add, lib_del, lib_delegate_deploy, lib_delegate_deploy_off, lib_get_all,
};
use m_core::data::metadata::Metadata;
use tauri::command;
use tracing::error;

fn internal_library_add(game: Metadata) -> Result<(), String> {
    lib_add(game).map_err(|err| {
        let err_msg = format!("Failed to add game: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_get() -> Result<Library, String> {
    lib_get_all().map_err(|err| {
        let err_msg = format!("Failed to get library: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_set(data: Metadata) -> Result<(), String> {
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

#[command]
pub fn metadata_add(title: String, archive_path: String, info: PlatformInfo) -> Result<(), String> {
    let mut game = Metadata::new(title, info.name.into(), info.id, archive_path);
    let _ = game.calculate_size();
    internal_library_add(game)
}

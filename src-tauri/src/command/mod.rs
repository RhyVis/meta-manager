mod bridge;

use crate::command::bridge::PlatformInfo;
use m_core::data::library::{
    Library, lib_add, lib_del, lib_delegate_create, lib_delegate_deploy, lib_delegate_deploy_off,
    lib_export, lib_get_all, lib_import,
};
use m_core::data::metadata::Metadata;
use tauri::command;
use tracing::error;

fn internal_library_add(data: Metadata) -> Result<(), String> {
    lib_add(data).map_err(|err| {
        let err_msg = format!("Failed to add metadata: {err}");
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
        let err_msg = format!("Failed to delete metadata: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_export() -> Result<(), String> {
    lib_export().map_err(|err| err.to_string())
}

#[command]
pub fn library_import() -> Result<bool, String> {
    lib_import().map_err(|err| err.to_string())
}

#[command]
pub fn library_deploy(id: String, path: String) -> Result<(), String> {
    lib_delegate_deploy(id.as_str(), path.as_str()).map_err(|err| {
        let err_msg = format!("Failed to deploy metadata: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn library_deploy_off(id: String) -> Result<(), String> {
    lib_delegate_deploy_off(id.as_str()).map_err(|err| {
        let err_msg = format!("Failed to off deploy data: {err}");
        error!(err_msg);
        err_msg
    })
}

#[command]
pub fn metadata_add(title: String, archive_path: String, info: PlatformInfo) -> Result<(), String> {
    let mut metadata = Metadata::new(title, info.name.into(), info.id, archive_path);
    let _ = metadata.calculate_size();
    internal_library_add(metadata)
}

#[command]
pub fn metadata_create(
    title: String,
    from_path: String,
    info: PlatformInfo,
    password: Option<String>,
) -> Result<(), String> {
    lib_delegate_create(title, info.name.into(), info.id, from_path, password).map_err(|err| {
        let err_msg = format!("Failed to create archive: {err}");
        error!(err_msg);
        err_msg
    })
}

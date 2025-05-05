use m_core::data::library::{
    GameLibrary, lib_add, lib_del, lib_delegate_deploy, lib_delegate_deploy_off, lib_get_all,
};
use m_core::data::unit::GameMetadata;
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
pub fn library_reload() -> Result<GameLibrary, String> {
    // Reload is now useless when using database
    library_get()
}

#[command]
pub fn library_replace(replacer: GameMetadata) -> Result<(), String> {
    internal_library_add(replacer)
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
pub fn metadata_add_steam(title: String, id: String, archive_path: String) -> Result<(), String> {
    let mut game = GameMetadata::from_steam(title.as_str(), id.as_str(), archive_path.as_str())
        .map_err(|e| e.to_string())?;
    let _ = game.calculate_size();
    internal_library_add(game)
}

#[command]
pub fn metadata_add_dl(title: String, id: String, archive_path: String) -> Result<(), String> {
    let mut game = GameMetadata::from_dl(title.as_str(), id.as_str(), archive_path.as_str())
        .map_err(|e| e.to_string())?;
    let _ = game.calculate_size();
    internal_library_add(game)
}

#[command]
pub fn metadata_add_unknown(title: String, archive_path: String) -> Result<(), String> {
    let mut game = GameMetadata::form_unknown(title.as_str(), archive_path.as_str())
        .map_err(|e| e.to_string())?;
    let _ = game.calculate_size();
    internal_library_add(game)
}

use m_core::data::library;
use m_core::data::library::{GameLibrary, LibraryError};
use m_core::data::unit::GameMetadata;
use tauri::command;
use tracing::error;

#[command]
pub fn library_reload() -> Result<GameLibrary, String> {
    if let Err(err) = library::reload() {
        let err_msg = format!("Failed to reload library: {err}");
        error!(err_msg);
        return Err(err_msg);
    }
    library_get()
}

#[command]
pub fn library_replace(replacer: GameMetadata) -> Result<(), String> {
    let mut lib = library::get_write().map_err(|e| e.to_string())?;
    match lib.replace_game(replacer) {
        Ok(_) => {
            lib.save().map_err(|e| e.to_string())?;
            Ok(())
        }
        Err(err) => {
            let err_msg = format!("Failed to replace game: {err}");
            error!(err_msg);
            Err(err_msg)
        }
    }
}

#[command]
pub fn library_get() -> Result<GameLibrary, String> {
    match library::get_clone() {
        Ok(library) => Ok(library),
        Err(err) => {
            let err_msg = format!("Failed to get library: {err}");
            error!(err_msg);
            Err(err_msg)
        }
    }
}

#[command]
pub fn library_del(id: String) -> Result<bool, String> {
    let mut lib = library::get_write().map_err(|e| e.to_string())?;
    match lib.del_game(id.as_str()) {
        Ok(_) => Ok(true),
        Err(err) => match err {
            LibraryError::NotFound(_) => Ok(false),
            _ => {
                let err_msg = format!("Failed to delete game: {err}");
                error!(err_msg);
                Err(err_msg)
            }
        },
    }
}

#[command]
pub fn library_deploy(id: String, path: String) -> Result<bool, String> {
    let mut lib = library::get_write().map_err(|e| e.to_string())?;
    lib.deploy_game(id.as_str(), path.as_str())
        .map_err(|e| e.to_string())?;
    Ok(true)
}

#[command]
pub fn library_deploy_off(id: String) -> Result<(), String> {
    let mut lib = library::get_write().map_err(|e| e.to_string())?;
    lib.deploy_off_game(id.as_str())
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn metadata_add_steam(title: String, id: String, archive_path: String) -> Result<bool, String> {
    let mut lib = library::get_write().map_err(|e| e.to_string())?;
    let mut game = GameMetadata::from_steam(title.as_str(), id.as_str(), archive_path.as_str())
        .map_err(|e| e.to_string())?;
    let _ = game.calculate_size();
    lib.add_game(game).map_err(|e| e.to_string())?;
    Ok(true)
}

#[command]
pub fn metadata_add_dl(title: String, id: String, archive_path: String) -> Result<bool, String> {
    let mut lib = library::get_write().map_err(|e| e.to_string())?;
    let mut game = GameMetadata::from_dl(title.as_str(), id.as_str(), archive_path.as_str())
        .map_err(|e| e.to_string())?;
    let _ = game.calculate_size();
    lib.add_game(game).map_err(|e| e.to_string())?;
    Ok(true)
}

#[command]
pub fn metadata_add_unknown(title: String, archive_path: String) -> Result<bool, String> {
    let mut lib = library::get_write().map_err(|e| e.to_string())?;
    let mut game = GameMetadata::form_unknown(title.as_str(), archive_path.as_str())
        .map_err(|e| e.to_string())?;
    let _ = game.calculate_size();
    lib.add_game(game).map_err(|e| e.to_string())?;
    Ok(true)
}

use crate::command::*;
use tracing::info;

mod command;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init().expect("Unable to initialize the application");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            library_get,
            library_del,
            library_reload,
            library_replace,
            metadata_add_steam,
            metadata_add_dl,
            metadata_add_unknown
        ])
        .run(tauri::generate_context!())
        .expect("Initialization failed");
}

fn init() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("Initializing config...");
    m_core::foundation::config::init_once_only()?;
    info!("Initializing library...");
    m_core::data::library::reload()?;
    Ok(())
}

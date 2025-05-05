use crate::command::*;
use tracing::info;

mod command;
mod logger;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _guard = logger::setup_logger().expect("Unable to setup logger");
    init().expect("Unable to initialize the application");
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            library_get,
            library_del,
            library_reload,
            library_replace,
            library_deploy,
            library_deploy_off,
            metadata_add_steam,
            metadata_add_dl,
            metadata_add_unknown
        ])
        .run(tauri::generate_context!())
        .expect("Initialization failed");
}

fn init() -> anyhow::Result<()> {
    info!("Initializing config...");
    m_core::foundation::config::init_once_only()?;
    info!("Initializing library...");
    m_core::data::library::lib_fresh()?;
    Ok(())
}

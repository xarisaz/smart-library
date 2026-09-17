mod app_paths;
mod classifier;
mod commands;
mod credential_store;
mod database;
mod extractor;
mod gmail;
mod gmail_imap;
mod models;
mod scanner;

use std::path::PathBuf;
use tauri::Manager;

#[derive(Clone)]
pub struct AppState {
    pub db_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;

            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("smart-library.sqlite3");
            database::initialize(&db_path).map_err(std::io::Error::other)?;

            app.manage(AppState { db_path });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::initialize_app,
            commands::get_file_filter_settings,
            commands::save_file_filter_settings,
            commands::gmail_status,
            commands::select_gmail_oauth_file,
            commands::connect_gmail,
            commands::connect_gmail_imap,
            commands::import_gmail_attachments,
            commands::disconnect_gmail,
            commands::pick_folder,
            commands::scan_folder,
            commands::list_files,
            commands::preview_file,
            commands::get_file_provenance,
            commands::open_indexed_file,
            commands::reveal_indexed_file,
            commands::set_review_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running Smart Library");
}

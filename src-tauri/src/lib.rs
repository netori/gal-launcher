#[cfg(target_os = "android")]
mod android;
mod asset;
mod backup;
mod commands;
mod db;
mod esc;
mod ge;
#[cfg(target_os = "windows")]
mod launcher;
mod models;
mod nsa;
mod pac;
mod patcher;
mod platform_fs;
mod scanner;
mod util;
mod vndb;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let dir = app
                .path()
                .app_data_dir()
                .expect("无法解析应用数据目录");
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("gal_launcher.db");
            let conn = db::init(&db_path)?;
            app.manage(commands::AppState {
                db: Mutex::new(conn),
                db_path,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_directory,
            commands::import_games,
            commands::list_games,
            commands::check_missing,
            commands::get_game_files,
            commands::toggle_favorite,
            commands::set_hidden,
            commands::remove_from_library,
            commands::delete_game,
            commands::set_hidden_attr,
            commands::read_image,
            commands::read_cover,
            commands::set_cover,
            commands::set_launch_file,
            commands::launch_game,
            commands::save_setting,
            commands::get_settings,
            commands::search_vndb,
            commands::apply_vndb_metadata,
            commands::set_game_title,
            commands::set_status,
            commands::fetch_missing_covers,
            commands::reveal_in_explorer,
            commands::list_dir,
            commands::list_drives,
            commands::create_dir,
            commands::add_patch,
            commands::list_patches,
            commands::install_patch,
            commands::uninstall_patch,
            commands::remove_patch,
            commands::list_asset_archives,
            commands::extract_assets,
            commands::list_extracted_assets,
            commands::export_assets,
            commands::clear_asset_cache,
            commands::search_unpack_tools,
            commands::export_backup,
            commands::import_backup,
            commands::check_update,
            commands::dismiss_update,
            commands::check_files_access,
            commands::request_all_files_access,
            commands::get_authorized_roots,
            commands::add_authorized_root,
            commands::remove_authorized_root,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
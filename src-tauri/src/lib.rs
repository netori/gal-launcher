#[cfg(target_os = "android")]
mod android;
mod asset;
mod backup;
mod bgm;
mod commands;
mod db;
mod esc;
mod ge;
mod hikarinagi;
mod kun;
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

/// 窗口状态持久化插件（记住位置/大小，下次启动恢复）。
/// 与 single-instance 一样，crate 顶层 `#![cfg(not(any(android, ios)))]`——移动端为空 crate，
/// 因此按平台提供无操作占位版本。
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn window_state_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_window_state::Builder::default().build()
}

/// 移动端空占位插件：仅保持调用点一致（不注册任何能力）。
#[cfg(any(target_os = "android", target_os = "ios"))]
fn window_state_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::<tauri::Wry>::new("window-state").build()
}

/// 单实例插件（避免双开：双进程各持写连接是 SQLITE_BUSY/数据竞争的高发场景）。
#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn single_instance_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri_plugin_single_instance::init(|app, _argv, _cwd| {
        // 二次启动时把已有主窗口带到前台
        if let Some(w) = app.get_webview_window("main") {
            let _ = w.unminimize();
            let _ = w.set_focus();
        }
    })
}

#[cfg(any(target_os = "android", target_os = "ios"))]
fn single_instance_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    tauri::plugin::Builder::<tauri::Wry>::new("single-instance").build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // 窗口状态持久化：记住位置/大小，下次启动恢复
        .plugin(window_state_plugin())
        .plugin(single_instance_plugin())
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
                fetch_cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            });

            // 启动后后台清理过期封面缩略图缓存（孤儿按需重新生成，避免 covers/ 无限膨胀）
            let thumbs_dir = dir.join("covers").join("thumbs");
            std::thread::spawn(move || {
                let removed = util::cleanup_old_thumbs(&thumbs_dir, 30 * 24 * 3600);
                if removed > 0 {
                    eprintln!("[gal-launcher] 已清理 {removed} 个过期封面缩略图缓存");
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::scan_directory,
            commands::import_games,
            commands::list_games,
            commands::check_missing,
            commands::get_game_files,
            commands::backup_savedata,
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
            commands::search_bgm,
            commands::apply_bgm_metadata,
            commands::search_kun,
            commands::apply_kun_metadata,
            commands::search_hikarinagi,
            commands::apply_hikarinagi_metadata,
            commands::set_game_title,
            commands::set_status,
            commands::list_collections,
            commands::create_collection,
            commands::delete_collection,
            commands::add_to_collection,
            commands::remove_from_collection,
            commands::list_collection_games,
            commands::fetch_missing_covers,
            commands::cancel_fetch_covers,
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
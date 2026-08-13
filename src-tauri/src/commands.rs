//! Tauri 命令层：把数据库与扫描/启动能力暴露给前端。

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tauri::{AppHandle, State};

use rusqlite::Connection;

use crate::db;
#[cfg(target_os = "windows")]
use crate::launcher;
use crate::models::{Candidate, FileInfo, Game, Patch, PatchInput, Settings};
use crate::patcher;
use crate::scanner;
use crate::util;
use crate::vndb;

/// 全局状态：SQLite 连接与其文件路径（后台线程会再开一条连接以避开锁）。
pub struct AppState {
    pub db: Mutex<Connection>,
    pub db_path: std::path::PathBuf,
}

type CmdResult<T> = Result<T, String>;

fn lock<'a>(state: &'a State<'_, AppState>) -> std::sync::MutexGuard<'a, Connection> {
    state.db.lock().unwrap()
}

/// 扫描一个根目录，返回疑似游戏目录候选列表。
/// 已收录集合用一次短查询取回，之后释放数据库锁——重点扫描期间不会阻塞其它命令。
#[tauri::command]
pub fn scan_directory(root: String, state: State<AppState>) -> CmdResult<Vec<Candidate>> {
    let path = Path::new(&root);
    if !path.is_dir() {
        return Err("目录不存在或无法访问".into());
    }
    let imported: std::collections::HashSet<String> = {
        let db = lock(&state);
        let mut stmt = db
            .prepare("SELECT source_dir FROM games")
            .map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        let raw: std::collections::HashSet<String> =
            rows.collect::<Result<_, _>>().map_err(|e| e.to_string())?;
        // 归一化后比较，避免大小写/分隔符/盘符差异导致误判为「未收录」
        raw.into_iter().map(|d| util::norm_path(&d)).collect()
    };
    Ok(scanner::scan_directory(path, &imported))
}

/// 将选中的候选游戏写入库（跳过已收录的），返回实际新增数量。
/// 文件遍历在锁外完成，DB 只在插入/写画像时短暂上锁。
#[tauri::command]
pub fn import_games(candidates: Vec<Candidate>, state: State<AppState>) -> CmdResult<usize> {
    // 锁外预计算每个候选的磁盘画像（这一步最慢，不能握着库锁）
    let mut payloads = Vec::new();
    for c in candidates {
        if c.already_imported {
            continue;
        }
        let dir = Path::new(&c.source_dir);
        if !dir.is_dir() {
            continue;
        }
        let files = scanner::collect_game_files(dir);
        payloads.push((
            c.title,
            c.source_dir,
            c.launch_path,
            c.launch_candidates,
            c.engine,
            c.cover_path,
            files,
        ));
    }

    let mut db = lock(&state);
    // 一次性取回已收录目录的归一化键，导入时做「相等 + 子目录包含」双重去重。
    let existing: Vec<String> = db::list_source_dirs(&db)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|d| util::norm_path(&d))
        .collect();
    let mut imported = 0usize;
    for (title, src, launch, candidates, engine, cover, files) in payloads {
        let src_norm = util::norm_path(&src);
        if existing.iter().any(|e| util::is_same_or_descendant(&src_norm, e)) {
            continue; // 已收录（或已在某已收录目录内部）
        }
        let id = match db::insert_game(
            &mut db,
            &title,
            &src,
            &launch,
            &candidates,
            &engine,
            cover.as_deref(),
            util::now_secs(),
        ) {
            Ok(id) => id,
            Err(_) => continue,
        };
        let _ = db::replace_files(&mut db, id, &files);
        imported += 1;
    }
    Ok(imported)
}

/// 列出游戏库。show_hidden 为 true 时连隐藏的一起返回（排在后面）。
#[tauri::command]
pub fn list_games(show_hidden: Option<bool>, state: State<AppState>) -> CmdResult<Vec<Game>> {
    let db = lock(&state);
    db::list_games(&db, show_hidden.unwrap_or(false)).map_err(|e| e.to_string())
}

/// 检测库内失效游戏：source_dir 已不存在（目录被移动/删除），启动会失败。
#[tauri::command]
pub fn check_missing(state: State<AppState>) -> CmdResult<Vec<Game>> {
    let db = lock(&state);
    let games = db::list_games(&db, true).map_err(|e| e.to_string())?;
    Ok(games
        .into_iter()
        .filter(|g| !Path::new(&g.source_dir).is_dir())
        .collect())
}

/// 拉取某个游戏的目录文件画像。
#[tauri::command]
pub fn get_game_files(game_id: i64, state: State<AppState>) -> CmdResult<Vec<FileInfo>> {
    let db = lock(&state);
    db::list_files(&db, game_id).map_err(|e| e.to_string())
}

/// 收藏/取消收藏。
#[tauri::command]
pub fn toggle_favorite(game_id: i64, state: State<AppState>) -> CmdResult<Game> {
    let db = lock(&state);
    db::toggle_favorite(&db, game_id).map_err(|e| e.to_string())
}

/// 应用内隐藏（主列表不显示，可从隐藏抽屉恢复）。
#[tauri::command]
pub fn set_hidden(game_id: i64, hidden: bool, state: State<AppState>) -> CmdResult<Game> {
    let db = lock(&state);
    db::set_hidden(&db, game_id, hidden).map_err(|e| e.to_string())
}

/// 从库中移除记录，**不删除磁盘文件**。
#[tauri::command]
pub fn remove_from_library(game_id: i64, state: State<AppState>) -> CmdResult<()> {
    let db = lock(&state);
    db::delete_from_library(&db, game_id).map_err(|e| e.to_string())
}

/// 把游戏整个目录送进回收站（可恢复），同时从库中移除。
/// Android 无回收站概念：桌面走 trash，移动端直接删除（All-Files-Access 下可用）。
#[tauri::command]
pub fn delete_game(game_id: i64, state: State<AppState>) -> CmdResult<()> {
    let db = lock(&state);
    let game = db::get_game(&db, game_id).map_err(|e| e.to_string())?;
    let target = game.source_dir.clone();
    db::delete_from_library(&db, game_id).map_err(|e| e.to_string())?;
    drop(db);

    delete_on_disk(&target)?;
    Ok(())
}

/// Windows：送进系统回收站（可恢复）。失败时返回带「游戏文件仍在」的提示。
#[cfg(windows)]
fn delete_on_disk(target: &str) -> Result<(), String> {
    trash::delete(target).map_err(|e| {
        format!("已从库中移除，但送入回收站失败（{e}）。游戏文件仍在：{target}")
    })
}

/// Android（及未来其它移动平台）：直接删除目录，无回收站。
#[cfg(not(windows))]
fn delete_on_disk(target: &str) -> Result<(), String> {
    std::fs::remove_dir_all(target).map_err(|e| {
        format!("已从库中移除，但删除游戏目录失败（{e}）。游戏文件仍在：{target}")
    })
}

/// 对某个目录设置/取消 Windows 隐藏+系统属性（文件系统级改动）。
#[tauri::command]
pub fn set_hidden_attr(path: String, hidden: bool) -> CmdResult<()> {
    util::set_hidden_attr(&path, hidden)
}

/// 读取一张本地图片，转成 data URI 供前端展示。
#[tauri::command]
pub fn read_image(path: String) -> CmdResult<String> {
    util::read_image_data_uri(&path)
}

/// 读取封面缩略图（data URI），带磁盘缓存。封面墙用，避免整张原图 base64 进内存。
#[tauri::command]
pub fn read_cover(path: String, size: Option<u32>, state: State<AppState>) -> CmdResult<String> {
    let thumbs_dir = state
        .db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("covers")
        .join("thumbs");
    util::read_cover_thumb(&path, &thumbs_dir, size.unwrap_or(400))
}

/// 手动更换本地封面（封面图片文件保留在原处，只把路径写入库）。
#[tauri::command]
pub fn set_cover(game_id: i64, cover_path: String, state: State<AppState>) -> CmdResult<Game> {
    if !Path::new(&cover_path).is_file() {
        return Err("封面图片不存在".into());
    }
    let db = lock(&state);
    db::update_cover(&db, game_id, Some(&cover_path)).map_err(|e| e.to_string())?;
    db::get_game(&db, game_id).map_err(|e| e.to_string())
}

/// 把某游戏的默认启动文件改成指定的可执行文件（不启动）。
/// 供「更换启动文件」入口使用；首次启动选文件走 launch_game + launch_path。
#[tauri::command]
pub fn set_launch_file(game_id: i64, launch_path: String, state: State<AppState>) -> CmdResult<Game> {
    let db = lock(&state);
    if !Path::new(&launch_path).is_file() {
        return Err("指定的启动文件不存在".into());
    }
    db::set_launch_file(&db, game_id, &launch_path).map_err(|e| e.to_string())
}

/// 启动游戏。桌面版：exe + LE 转区 + 后台时长统计；移动版：M2 引擎→运行时适配。
#[tauri::command]
pub fn launch_game(
    game_id: i64,
    use_locale: Option<bool>,
    launch_path: Option<String>,
    state: State<AppState>,
) -> CmdResult<Game> {
    launch_game_impl(game_id, use_locale, launch_path, state)
}

#[cfg(target_os = "windows")]
fn launch_game_impl(
    game_id: i64,
    use_locale: Option<bool>,
    launch_path: Option<String>,
    state: State<AppState>,
) -> CmdResult<Game> {
    let use_locale = use_locale.unwrap_or(false);
    let (game, resolved, le_path, db_path) = {
        let db = lock(&state);
        let game = db::get_game(&db, game_id).map_err(|e| e.to_string())?;
        // 解析用户指定的启动文件（绝对路径直接使用，相对路径拼到游戏目录）。
        let resolved = match launch_path {
            Some(p) if !p.trim().is_empty() => {
                let full = if Path::new(&p).is_absolute() {
                    p
                } else {
                    Path::new(&game.source_dir).join(&p).to_string_lossy().to_string()
                };
                if !Path::new(&full).is_file() {
                    return Err(format!("指定的启动文件不存在：{full}"));
                }
                db::set_launch_file(&db, game_id, &full)
                    .map_err(|e| e.to_string())?;
                full
            }
            _ => game.launch_path.clone(),
        };
        let le_path = db::get_setting(&db, "locale_emulator_path");
        (game, resolved, le_path, state.db_path.clone())
    };

    let mut game = game;
    game.launch_path = resolved;
    let child = launcher::spawn_child(&game, le_path.as_deref(), use_locale)?;

    let started_at = util::now_secs();
    let session_id = {
        let db = lock(&state);
        db.execute(
            "INSERT INTO play_sessions (game_id, started_at) VALUES (?1, ?2)",
            rusqlite::params![game_id, started_at],
        )
        .map_err(|e| e.to_string())?;
        db.last_insert_rowid()
    };
    launcher::watch_session(db_path, session_id, game_id, started_at, child);
    Ok(game)
}

#[cfg(not(target_os = "windows"))]
fn launch_game_impl(
    _game_id: i64,
    _use_locale: Option<bool>,
    _launch_path: Option<String>,
    _state: State<AppState>,
) -> CmdResult<Game> {
    Err("移动端启动（引擎→运行时适配）将在后续版本提供".into())
}

/// 保存一项设置。
#[tauri::command]
pub fn save_setting(key: String, value: String, state: State<AppState>) -> CmdResult<()> {
    let db = lock(&state);
    db::set_setting(&db, &key, &value).map_err(|e| e.to_string())
}

/// 读取设置视图。
#[tauri::command]
pub fn get_settings(state: State<AppState>) -> CmdResult<Settings> {
    let db = lock(&state);
    Ok(Settings {
        locale_emulator_path: db::get_setting(&db, "locale_emulator_path"),
        game_root: db::get_setting(&db, "game_root"),
        unpack_tool: db::get_setting(&db, "unpack_tool"),
    })
}

// ---------------- 移动端文件访问权限 ----------------

/// 是否已获得「所有文件访问」权限（桌面端恒为 true，无此概念）。
#[tauri::command]
#[allow(unused_variables)]
pub fn check_files_access(app: tauri::AppHandle) -> bool {
    #[cfg(target_os = "android")]
    {
        crate::android::is_all_files_granted(&app)
    }
    #[cfg(not(target_os = "android"))]
    {
        true
    }
}

/// 请求「所有文件访问」权限（仅 Android；当前返回引导文案由前端展示）。
#[tauri::command]
#[allow(unused_variables)]
pub fn request_all_files_access(app: tauri::AppHandle) -> CmdResult<()> {
    #[cfg(target_os = "android")]
    {
        crate::android::request_all_files_access(&app)
    }
    #[cfg(not(target_os = "android"))]
    {
        Ok(())
    }
}

/// 列出已授权的游戏根目录（移动端扫描/导出选根用；桌面端为空）。
#[tauri::command]
pub fn get_authorized_roots(state: State<AppState>) -> CmdResult<Vec<String>> {
    let db = lock(&state);
    db::list_authorized_roots(&db).map_err(|e| e.to_string())
}

/// 登记一个已授权的游戏根目录。
#[tauri::command]
pub fn add_authorized_root(path: String, state: State<AppState>) -> CmdResult<()> {
    let p = path.trim();
    if p.is_empty() {
        return Err("路径不能为空".into());
    }
    let db = lock(&state);
    db::add_authorized_root(&db, p).map_err(|e| e.to_string())
}

/// 移除一个已授权根目录。
#[tauri::command]
pub fn remove_authorized_root(path: String, state: State<AppState>) -> CmdResult<()> {
    let db = lock(&state);
    db::remove_authorized_root(&db, &path).map_err(|e| e.to_string())
}

// ---------------- VNDB 元数据 ----------------

/// 按标题搜索 VNDB，返回候选条目。
#[tauri::command]
pub fn search_vndb(query: String) -> CmdResult<Vec<vndb::VnSearchHit>> {
    vndb::search_vn(&query)
}

/// 抓取某 VN 的元数据并应用到游戏（含封面下载）。
#[tauri::command]
pub fn apply_vndb_metadata(
    game_id: i64,
    vndb_id: String,
    use_vndb_title: Option<bool>,
    state: State<AppState>,
) -> CmdResult<Game> {
    let meta = vndb::fetch_vn(&vndb_id)?;

    let covers_dir = state.db_path.parent().unwrap_or(std::path::Path::new(".")).join("covers");
    std::fs::create_dir_all(&covers_dir).map_err(|e| format!("创建封面目录失败: {e}"))?;

    let cover_path = match &meta.cover_url {
        Some(url) => vndb::download_cover(&covers_dir, &vndb_id, url).ok(),
        None => None,
    };

    let db = lock(&state);
    let game = db::update_metadata(
        &db,
        game_id,
        meta.description.as_deref(),
        meta.rating.map(|r| r / 10.0), // VNDB 返回 0-100，转成 0-10 展示
        Some(&meta.vndb_id),
        meta.tags.clone(),
        meta.developers.first().map(|s| s.as_str()),
        meta.released.as_deref(),
        meta.length_minutes,
        cover_path.as_deref(),
    )
    .map_err(|e| e.to_string())?;

    if use_vndb_title.unwrap_or(false) && !meta.title.is_empty() {
        db::set_title(&db, game_id, &meta.title).map_err(|e| e.to_string())
    } else {
        Ok(game)
    }
}

/// 手动改游戏显示标题。
#[tauri::command]
pub fn set_game_title(game_id: i64, title: String, state: State<AppState>) -> CmdResult<Game> {
    let db = lock(&state);
    db::set_title(&db, game_id, title.trim()).map_err(|e| e.to_string())
}

/// 设置游玩状态（''/wishlist/playing/finished/dropped）。
#[tauri::command]
pub fn set_status(game_id: i64, status: String, state: State<AppState>) -> CmdResult<Game> {
    let db = lock(&state);
    db::set_status(&db, game_id, status.trim()).map_err(|e| e.to_string())
}

/// 批量补全结果。
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverBatch {
    pub updated: usize,
    pub failed: Vec<String>,
}

/// 为所有还没有封面的游戏从 VNDB 拉取封面 + 元数据（评分/简介/标签/厂商/时长）。
/// 命中策略：优先取有评分的首个结果，免得挂错封面；拿不准的进 failed 列表由用户手动处理。
#[tauri::command]
pub fn fetch_missing_covers(state: State<AppState>) -> CmdResult<CoverBatch> {
    let games = {
        let db = lock(&state);
        db::list_games(&db, true).map_err(|e| e.to_string())?
    };
    let covers_dir = state
        .db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("covers");
    std::fs::create_dir_all(&covers_dir).ok();

    let mut updated = 0usize;
    let mut failed = Vec::new();
    for g in games {
        if g.cover_path.is_some() {
            continue;
        }
        let result = (|| -> Result<(), String> {
            let hits = vndb::search_vn(&g.title)?;
            let pick = hits.iter().find(|h| h.rating.is_some()).or_else(|| hits.first());
            let hit = pick.ok_or_else(|| "VNDB 无结果".to_string())?;
            let url = hit
                .image_url
                .as_deref()
                .ok_or_else(|| "该条目无封面".to_string())?;
            let meta = vndb::fetch_vn(&hit.vndb_id)?;
            let cover = vndb::download_cover(&covers_dir, &hit.vndb_id, url)?;
            let db = lock(&state);
            db::update_metadata(
                &db,
                g.id,
                meta.description.as_deref(),
                meta.rating.map(|r| r / 10.0),
                Some(&meta.vndb_id),
                meta.tags.clone(),
                meta.developers.first().map(|s| s.as_str()),
                meta.released.as_deref(),
                meta.length_minutes,
                Some(&cover),
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => updated += 1,
            Err(e) => failed.push(format!("{}: {e}", g.title)),
        }
        // 对 VNDB 客气点：每请求间隔 250ms
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    Ok(CoverBatch { updated, failed })
}

/// 在资源管理器中打开（若是文件则定位选中它）。后台分离启动，应用不等待。
/// 仅 Windows 有意义；移动端返回不支持。
#[tauri::command]
pub fn reveal_in_explorer(path: String) -> CmdResult<()> {
    reveal_in_explorer_impl(&path)
}

#[cfg(target_os = "windows")]
fn reveal_in_explorer_impl(path: &str) -> CmdResult<()> {
    let p = Path::new(path);
    let (dir, select_file) = if p.is_file() {
        (p.parent().map(|d| d.to_path_buf()), Some(path.to_string()))
    } else if p.is_dir() {
        (Some(p.to_path_buf()), None)
    } else {
        (None, None)
    };
    match (dir, select_file) {
        (_, Some(file)) => {
            std::process::Command::new("explorer")
                .arg(format!("/select,{file}"))
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| format!("打开资源管理器失败: {e}"))?;
        }
        (Some(dir), None) => {
            std::process::Command::new("explorer")
                .arg(dir.to_string_lossy().into_owned())
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .map_err(|e| format!("打开资源管理器失败: {e}"))?;
        }
        _ => return Err("路径不存在：无法打开".into()),
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn reveal_in_explorer_impl(_path: &str) -> CmdResult<()> {
    Err("此平台不支持打开系统文件管理器".into())
}

/// 列出目录内容（子目录 + 文件），供内置文件选择器用。
/// `exts` 指定时只返回匹配扩展名的文件；不指定返回所有文件（最多 500 个，超出见 truncated）。
#[tauri::command]
pub fn list_dir(path: String, exts: Option<Vec<String>>) -> CmdResult<crate::models::DirListing> {
    util::list_dir(&path, exts.as_deref())
}

/// 列出可用的磁盘根目录（如 "C:\\"）。仅 Windows 有意义，其它平台返回空。
#[tauri::command]
pub fn list_drives() -> Vec<String> {
    util::list_drives()
}

/// 在指定路径下新建目录（供文件选择器的「新建文件夹」）。
#[tauri::command]
pub fn create_dir(path: String) -> CmdResult<()> {
    if path.trim().is_empty() {
        return Err("路径为空".into());
    }
    std::fs::create_dir_all(&path).map_err(|e| format!("创建目录失败: {e}"))
}

// ---------------- 补丁管理 ----------------

fn backup_base(state: &State<AppState>) -> std::path::PathBuf {
    state.db_path.parent().unwrap_or(std::path::Path::new(".")).join("backups")
}

/// 登记一个补丁（来源：文件夹 / zip / 安装器 exe）。
#[tauri::command]
pub fn add_patch(input: PatchInput, state: State<AppState>) -> CmdResult<Patch> {
    if input.name.trim().is_empty() {
        return Err("补丁名称不能为空".into());
    }
    if !Path::new(&input.source_path).exists() {
        return Err("补丁来源不存在".into());
    }
    let method = if input.install_method.is_empty() {
        patcher::guess_method(Path::new(&input.source_path))
    } else {
        &input.install_method
    };

    let db = lock(&state);
    let id = db::insert_patch(
        &db,
        input.game_id,
        input.name.trim(),
        input.kind.trim(),
        &input.source_path,
        method,
        None,
        "",
    )
    .map_err(|e| e.to_string())?;

    // 分配独立备份目录
    let backup_dir = backup_base(&state).join(id.to_string());
    std::fs::create_dir_all(&backup_dir).map_err(|e| format!("创建备份目录失败: {e}"))?;
    db::set_patch_backup_dir(&db, id, &backup_dir.to_string_lossy()).map_err(|e| e.to_string())?;
    db::get_patch(&db, id).map_err(|e| e.to_string())
}

/// 列出某游戏的补丁。
#[tauri::command]
pub fn list_patches(game_id: i64, state: State<AppState>) -> CmdResult<Vec<Patch>> {
    let db = lock(&state);
    db::list_patches(&db, game_id).map_err(|e| e.to_string())
}

/// 安装补丁（installer 运行安装程序；replace 覆盖式+预备份）。
#[tauri::command]
pub fn install_patch(patch_id: i64, state: State<AppState>) -> CmdResult<Patch> {
    let (patch, game) = {
        let db = lock(&state);
        let patch = db::get_patch(&db, patch_id).map_err(|e| e.to_string())?;
        let game = db::get_game(&db, patch.game_id).map_err(|e| e.to_string())?;
        (patch, game)
    };

    let game_dir = Path::new(&game.source_dir).to_path_buf();
    let method = patch.install_method.clone();
    let source = std::path::PathBuf::from(&patch.source_path);
    let backup_dir = patch.backup_dir.clone().map(std::path::PathBuf::from);

    match method.as_str() {
        "installer" => patcher::run_installer(&game_dir, &source)?,
        _ => {
            let bdir = backup_dir
                .unwrap_or_else(|| backup_base(&state).join(patch_id.to_string()));
            std::fs::create_dir_all(&bdir).map_err(|e| format!("创建备份目录失败: {e}"))?;
            // 记录备份用独立连接，避免握着 AppState 锁做 IO。
            let conn = Connection::open(state.db_path.as_path()).map_err(|e| e.to_string())?;
            patcher::install_replace(&game_dir, &source, &bdir, &conn, patch_id)?;
        }
    }

    let db = lock(&state);
    db::set_patch_installed(&db, patch_id, true).map_err(|e| e.to_string())
}

/// 回滚补丁：从备份恢复被覆盖的文件。
#[tauri::command]
pub fn uninstall_patch(patch_id: i64, state: State<AppState>) -> CmdResult<Patch> {
    let (patch, game) = {
        let db = lock(&state);
        let patch = db::get_patch(&db, patch_id).map_err(|e| e.to_string())?;
        let game = db::get_game(&db, patch.game_id).map_err(|e| e.to_string())?;
        (patch, game)
    };

    if !patch.installed {
        return Err("该补丁尚未安装".into());
    }
    if patch.install_method != "installer" {
        let conn = Connection::open(state.db_path.as_path()).map_err(|e| e.to_string())?;
        patcher::rollback(Path::new(&game.source_dir), &conn, patch_id)?;
    }

    let db = lock(&state);
    db::set_patch_installed(&db, patch_id, false).map_err(|e| e.to_string())
}

/// 删除补丁记录（清理备份目录，不做文件级回滚）。
#[tauri::command]
pub fn remove_patch(patch_id: i64, state: State<AppState>) -> CmdResult<()> {
    let backup_dir = {
        let db = lock(&state);
        let patch = db::get_patch(&db, patch_id).map_err(|e| e.to_string())?;
        db::delete_patch(&db, patch_id).map_err(|e| e.to_string())?;
        patch.backup_dir.clone()
    };
    if let Some(d) = backup_dir {
        patcher::remove_backup_dir(&std::path::PathBuf::from(d));
    }
    Ok(())
}

// ---------------- M3：资源解包 ----------------

fn asset_root(state: &State<AppState>, game_id: i64) -> std::path::PathBuf {
    state
        .db_path
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join("assets")
        .join(game_id.to_string())
}

/// 列出游戏目录里可解包的资源包（.xp3 / .pfs）。
#[tauri::command]
pub fn list_asset_archives(game_id: i64, state: State<AppState>) -> CmdResult<Vec<crate::asset::ArchiveInfo>> {
    let game = {
        let db = lock(&state);
        db::get_game(&db, game_id).map_err(|e| e.to_string())?
    };
    let root = asset_root(&state, game_id);
    std::fs::create_dir_all(&root).ok();
    Ok(crate::asset::list_archives(Path::new(&game.source_dir), &root))
}

/// 解包某个资源包到缓存，返回条目列表。
/// 内置不认识的格式（PAC/NSA/PKG 等）会交给设置里配置的外部解包工具。
#[tauri::command]
pub fn extract_assets(game_id: i64, archive_rel: String, state: State<AppState>) -> CmdResult<Vec<crate::asset::AssetEntry>> {
    let (game, external_tool) = {
        let db = lock(&state);
        let game = db::get_game(&db, game_id).map_err(|e| e.to_string())?;
        let external_tool = db::get_setting(&db, "unpack_tool");
        (game, external_tool)
    };
    let rel = crate::asset::sanitize_rel(&archive_rel)
        .ok_or_else(|| "非法的资源包路径".to_string())?;
    let abs = Path::new(&game.source_dir).join(&rel);
    if !abs.is_file() {
        return Err("资源包文件不存在".into());
    }
    let stem = std::path::Path::new(&rel)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| rel.clone());
    let out_dir = asset_root(&state, game_id).join(stem);

    match crate::asset::detect_format(&abs) {
        Some(_) => crate::asset::extract(&abs, &out_dir),
        None => match external_tool.filter(|t| !t.trim().is_empty()) {
            Some(tool) => crate::asset::extract_external(&abs, &out_dir, &tool),
            None => Err(
                "内置不支持的格式（PAC / NSA / PKG 等）。请在「设置」里配置一个外部解包工具路径后重试。"
                    .into(),
            ),
        },
    }
}

/// 列出某游戏已解包的条目（含分类）。
#[tauri::command]
pub fn list_extracted_assets(game_id: i64, state: State<AppState>) -> CmdResult<Vec<crate::asset::AssetEntry>> {
    let root = asset_root(&state, game_id);
    Ok(crate::asset::list_extracted(&root))
}

/// 把某分类（None=全部）的已解包文件导出到指定目录。
#[tauri::command]
pub fn export_assets(
    game_id: i64,
    dest: String,
    category: Option<String>,
    state: State<AppState>,
) -> CmdResult<usize> {
    if dest.trim().is_empty() {
        return Err("请先选择导出目录".into());
    }
    let root = asset_root(&state, game_id);
    let dst = Path::new(&dest);
    std::fs::create_dir_all(dst).map_err(|e| format!("创建导出目录失败: {e}"))?;
    crate::asset::export_matching(&root, dst, category.as_deref())
}

/// 清空某游戏的解包缓存（回收磁盘空间），返回删除的文件数。
#[tauri::command]
pub fn clear_asset_cache(game_id: i64, state: State<AppState>) -> CmdResult<usize> {
    let root = asset_root(&state, game_id);
    if !root.exists() {
        return Ok(0);
    }
    let count = crate::asset::count_files(&root);
    std::fs::remove_dir_all(&root).map_err(|e| format!("清理失败: {e}"))?;
    Ok(count)
}

// ---------------- 整库备份 / 恢复 ----------------

/// 导出整库备份（db 一致快照 + covers/ + 补丁 backups）到指定目录，排除可再生的解包缓存。
#[tauri::command]
pub fn export_backup(dest: String, state: State<AppState>) -> CmdResult<crate::backup::BackupOut> {
    if dest.trim().is_empty() {
        return Err("请先选择导出目录".into());
    }
    crate::backup::export(&state.db_path, Path::new(&dest)).map_err(|e| e.to_string())
}

/// 从备份 zip 恢复（校验通过后热切换数据库连接，替换封面与补丁备份）。
#[tauri::command]
pub fn import_backup(src: String, state: State<AppState>) -> CmdResult<crate::backup::RestoreOut> {
    if src.trim().is_empty() {
        return Err("请先选择备份文件".into());
    }
    if !Path::new(&src).is_file() {
        return Err("备份文件不存在".into());
    }
    let mut guard = lock(&state);
    crate::backup::restore(&state.db_path, Path::new(&src), &mut guard).map_err(|e| e.to_string())
}

// ---------------- 更新检查 ----------------

/// 实际发布的 GitHub 仓库（owner/repo）。改仓库时同步这一处即可。
const UPDATE_REPO: &str = "netori/gal-launcher";

/// 检查 GitHub 最新 release：
/// - 无 release / 网络失败 / 请求被限流 → 静默返回 None（不打扰用户）
/// - 最新版本不高于当前版本 → None
/// - 已被用户「不再提示」过的版本 → None
#[tauri::command]
pub fn check_update(
    app: AppHandle,
    state: State<AppState>,
) -> CmdResult<Option<crate::models::UpdateInfo>> {
    let dismissed = {
        let db = lock(&state);
        db::get_setting(&db, "dismissed_update").unwrap_or_default()
    };
    let info = query_latest_release(&app);
    Ok(info.and_then(|u| (u.version != dismissed).then_some(u)))
}

/// 记录「不再提示」的版本；之后新版本发布时仍会提示。
#[tauri::command]
pub fn dismiss_update(version: String, state: State<AppState>) -> CmdResult<()> {
    let db = lock(&state);
    db::set_setting(&db, "dismissed_update", &version).map_err(|e| e.to_string())
}

fn query_latest_release(app: &AppHandle) -> Option<crate::models::UpdateInfo> {
    let current = app.package_info().version.to_string();
    let url = format!("https://api.github.com/repos/{UPDATE_REPO}/releases/latest");
    let resp = ureq::get(&url)
        .set("User-Agent", &format!("gal-launcher/{current}"))
        .set("Accept", "application/vnd.github+json")
        .timeout(std::time::Duration::from_secs(8))
        .call()
        .ok()?;
    let body = resp.into_string().ok()?;
    let v: serde_json::Value = serde_json::from_str(&body).ok()?;
    let tag = v.get("tag_name")?.as_str()?.to_string();
    if let Some(remote) = parse_version(&tag) {
        if let Some(cur) = parse_version(&current) {
            if remote <= cur {
                return None; // 不高于当前版本
            }
        }
    }
    let url = v.get("html_url").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let note = v.get("body").and_then(|x| x.as_str()).unwrap_or("").to_string();
    // 优先找一个安装包（exe/msi）资产作为下载直链，找不到就只给 release 页
    let download_url = v
        .get("assets")?
        .as_array()?
        .iter()
        .filter_map(|a| {
            let name = a.get("name")?.as_str()?;
            let dl = a.get("browser_download_url")?.as_str()?;
            Some((name, dl))
        })
        .find(|(name, _)| name.ends_with(".exe") || name.ends_with(".msi"))
        .map(|(_, dl)| dl.to_string());
    Some(crate::models::UpdateInfo {
        version: tag,
        url,
        note,
        download_url,
    })
}

/// 从 "v1.2.3" / "1.2.3" 解析出 (major, minor, patch)。
fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let t = s.trim().trim_start_matches(['v', 'V']);
    let mut it = t.split('.');
    let maj = it.next()?.parse().ok()?;
    let min = it.next()?.parse().ok()?;
    let patch = it.next()?.parse().ok()?;
    Some((maj, min, patch))
}

/// 在常见位置查找常用的外部解包工具（GARbro / GalArc / arc_unpacker 等）。
/// 前端传入候选 exe 文件名（小写），返回「exe 文件名 → 首次找到的完整路径」。
/// 仅桌面端（exe 工具在移动端无意义）。
#[tauri::command]
pub fn search_unpack_tools(exes: Vec<String>) -> HashMap<String, String> {
    detect_unpack_tools(&exes)
}

#[cfg(target_os = "windows")]
fn detect_unpack_tools(exes: &[String]) -> HashMap<String, String> {
    let mut wanted: std::collections::HashSet<String> = exes
        .iter()
        .map(|e| e.trim().to_lowercase())
        .filter(|e| e.ends_with(".exe") && !e.is_empty())
        .collect();
    let mut found: HashMap<String, String> = HashMap::new();
    if wanted.is_empty() {
        return found;
    }

    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(home) = std::env::var("USERPROFILE") {
        let home = std::path::PathBuf::from(home);
        for d in ["Downloads", "Desktop", "Documents", "tools", "工具"] {
            roots.push(home.join(d));
        }
    }
    for d in ["C:\\Program Files", "C:\\Program Files (x86)"] {
        roots.push(d.into());
    }
    roots.push(std::env::temp_dir());
    for letter in ['D', 'E', 'F', 'G'] {
        let r: std::path::PathBuf = format!("{letter}:\\").into();
        if r.exists() {
            roots.push(r);
        }
    }

    // 跳过常见的大目录/系统目录，避免扫描过慢
    const SKIP: &[&str] = &[
        "windows",
        "system32",
        "programdata",
        "$recycle.bin",
        "node_modules",
        ".git",
        "venv",
        "target",
        "dist",
        "appdata",
        "msys64",
        "program files",
        "回收站",
        "恢复",
    ];

    for root in roots {
        if wanted.is_empty() {
            break;
        }
        if !root.exists() {
            continue;
        }
        walk_for_tools(&root, &mut wanted, &mut found, SKIP, 0, 2);
    }
    found
}

/// 移动端无 exe 解包工具概念，直接返回空。
#[cfg(not(target_os = "windows"))]
fn detect_unpack_tools(_exes: &[String]) -> HashMap<String, String> {
    HashMap::new()
}

fn walk_for_tools(
    dir: &Path,
    wanted: &mut std::collections::HashSet<String>,
    found: &mut HashMap<String, String>,
    skip: &[&str],
    depth: usize,
    max_depth: usize,
) {
    if wanted.is_empty() || depth > max_depth {
        return;
    }
    let Ok(rd) = std::fs::read_dir(dir) else {
        return;
    };
    for ent in rd.filter_map(|e| e.ok()).take(3000) {
        if wanted.is_empty() {
            return;
        }
        let name = ent.file_name().to_string_lossy().to_lowercase();
        let Ok(ft) = ent.file_type() else {
            continue;
        };
        if ft.is_dir() && depth < max_depth {
            if skip.iter().any(|s| name.contains(s)) {
                continue;
            }
            walk_for_tools(&ent.path(), wanted, found, skip, depth + 1, max_depth);
        } else if ft.is_file() && wanted.contains(&name) {
            found.insert(name.clone(), ent.path().to_string_lossy().into_owned());
            wanted.remove(&name);
        }
    }
}
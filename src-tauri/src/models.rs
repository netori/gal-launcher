//! 前后端共享的数据结构。

use serde::{Deserialize, Serialize};

/// 扫描目录后产出的疑似游戏目录候选。
/// 作为 Tauri 命令入参，需要 Deserialize。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub title: String,
    pub source_dir: String,
    pub launch_path: String,
    /// 该目录内其它可启动 exe（相对路径），供首次启动时让用户挑选。
    pub launch_candidates: Vec<String>,
    pub engine: String,
    pub cover_path: Option<String>,
    pub file_count: usize,
    pub already_imported: bool,
    pub note: String,
}

/// 游戏库中的一条游戏记录。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    pub id: i64,
    pub title: String,
    pub source_dir: String,
    pub launch_path: String,
    /// 全部可启动 exe（相对路径）。首次启动前由前端让用户挑选。
    pub launch_candidates: Vec<String>,
    /// 用户是否已在多个启动文件里选定过（选定后默认用 launch_path）。
    pub launch_set: bool,
    pub engine: String,
    pub cover_path: Option<String>,
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub vndb_id: Option<String>,
    /// VNDB 标签名（JSON 数组，稳定顺序）
    pub tags: Vec<String>,
    pub developer: Option<String>,
    pub released: Option<String>,
    pub length_minutes: Option<i64>,
    pub added_at: i64,
    pub last_played: Option<i64>,
    pub total_seconds: i64,
    pub play_count: i64,
    pub hidden: bool,
    pub favorite: bool,
    /// 游玩状态：''=未分类 | wishlist=想玩 | playing=进行中 | finished=已通关 | dropped=搁置。
    pub status: String,
}

/// 游戏目录内的一个文件画像条目。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub id: i64,
    pub rel_path: String,
    pub kind: String,
    pub size: i64,
}

/// 前端用的设置对象。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub locale_emulator_path: Option<String>,
    pub game_root: Option<String>,
    /// 外部解包工具（如 GARbro 控制台 / GalArc / arc_unpacker），用于内置不支持的格式。
    pub unpack_tool: Option<String>,
}

/// 一条补丁记录（汉化 / R18 / 修正 / 其他）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub id: i64,
    pub game_id: i64,
    pub name: String,
    pub kind: String,
    pub source_path: String,
    /// replace（覆盖式） | installer（安装程序）
    pub install_method: String,
    pub installed: bool,
    pub installed_at: Option<i64>,
    pub backup_dir: Option<String>,
    pub note: String,
}

/// 添加补丁时前端传入的参数。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchInput {
    pub game_id: i64,
    pub name: String,
    pub kind: String,
    pub source_path: String,
    pub install_method: String,
}

/// GitHub 最新 release 的更新信息（供前端展示更新提示）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// release 标签名，如 "v0.2.0"
    pub version: String,
    /// release 页面链接
    pub url: String,
    /// release 发布说明（可能为空）
    pub note: String,
    /// 安装包（exe/msi）直链，可能为空
    pub download_url: Option<String>,
}
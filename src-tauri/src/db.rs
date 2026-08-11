//! SQLite 数据层：schema 迁移与仓储操作。

use rusqlite::{params, Connection, Result};
use std::path::Path;

use crate::models::{FileInfo, Game, Patch};

/// 一次性初始化数据库连接（建库 + 建表）。
pub fn init(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.execute_batch(SCHEMA)?;
    ensure_columns(&conn)?;
    Ok(conn)
}

/// 老库补充新列（CREATE TABLE IF NOT EXISTS 不会给已存在的表加列）。
/// 注意：新增 games 列时都要在这里补一条，否则老库会在 SELECT/INSERT 上直接报错。
fn ensure_columns(conn: &Connection) -> Result<()> {
    let existing: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(games)")?;
        let rows = stmt.query_map([], |r| r.get::<_, String>(1))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };
    let additions: &[(&str, &str)] = &[
        ("launch_candidates", "TEXT NOT NULL DEFAULT '[]'"),
        ("launch_set", "INTEGER NOT NULL DEFAULT 0"),
        ("tags", "TEXT NOT NULL DEFAULT '[]'"),
        ("developer", "TEXT"),
        ("released", "TEXT"),
        ("length_minutes", "INTEGER"),
    ];
    for (name, ddl) in additions {
        if !existing.iter().any(|c| c == name) {
            conn.execute(&format!("ALTER TABLE games ADD COLUMN {name} {ddl}"), [])?;
        }
    }
    Ok(())
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS games (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  title        TEXT NOT NULL,
  source_dir   TEXT NOT NULL UNIQUE,
  launch_path  TEXT NOT NULL,
  launch_candidates TEXT NOT NULL DEFAULT '[]',
  launch_set   INTEGER NOT NULL DEFAULT 0,
  engine       TEXT NOT NULL DEFAULT '',
  cover_path   TEXT,
  description  TEXT,
  rating       REAL,
  vndb_id      TEXT,
  tags         TEXT NOT NULL DEFAULT '[]',
  developer    TEXT,
  released     TEXT,
  length_minutes INTEGER,
  added_at     INTEGER NOT NULL,
  last_played  INTEGER,
  total_seconds INTEGER NOT NULL DEFAULT 0,
  play_count   INTEGER NOT NULL DEFAULT 0,
  hidden       INTEGER NOT NULL DEFAULT 0,
  favorite     INTEGER NOT NULL DEFAULT 0,
  scanned_at   INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS game_files (
  id       INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id  INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  rel_path TEXT NOT NULL,
  kind     TEXT NOT NULL,
  size     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS play_sessions (
  id         INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id    INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  started_at INTEGER NOT NULL,
  ended_at   INTEGER
);

CREATE TABLE IF NOT EXISTS settings (
  key   TEXT PRIMARY KEY,
  value TEXT
);

CREATE TABLE IF NOT EXISTS patches (
  id             INTEGER PRIMARY KEY AUTOINCREMENT,
  game_id        INTEGER NOT NULL REFERENCES games(id) ON DELETE CASCADE,
  name           TEXT NOT NULL,
  kind           TEXT NOT NULL DEFAULT '其他',
  source_path    TEXT NOT NULL,
  install_method TEXT NOT NULL DEFAULT 'replace',
  installed      INTEGER NOT NULL DEFAULT 0,
  installed_at   INTEGER,
  backup_dir     TEXT,
  note           TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS patch_backups (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  patch_id    INTEGER NOT NULL REFERENCES patches(id) ON DELETE CASCADE,
  target_rel  TEXT NOT NULL,
  backup_path TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_game_files_game ON game_files(game_id);
CREATE INDEX IF NOT EXISTS idx_sessions_game ON play_sessions(game_id);
CREATE INDEX IF NOT EXISTS idx_patches_game ON patches(game_id);
CREATE INDEX IF NOT EXISTS idx_pbackups_patch ON patch_backups(patch_id);
"#;

const GAME_COLS: &str = "id,title,source_dir,launch_path,launch_candidates,launch_set,engine,cover_path,description,rating,vndb_id,tags,developer,released,length_minutes,added_at,last_played,total_seconds,play_count,hidden,favorite";

fn parse_strings(raw: &str) -> Vec<String> {
    serde_json::from_str(raw).unwrap_or_default()
}

fn row_to_game(r: &rusqlite::Row) -> Result<Game> {
    Ok(Game {
        id: r.get(0)?,
        title: r.get(1)?,
        source_dir: r.get(2)?,
        launch_path: r.get(3)?,
        launch_candidates: parse_strings(&r.get::<_, String>(4)?),
        launch_set: r.get::<_, i64>(5)? != 0,
        engine: r.get(6)?,
        cover_path: r.get(7)?,
        description: r.get(8)?,
        rating: r.get(9)?,
        vndb_id: r.get(10)?,
        tags: parse_strings(&r.get::<_, String>(11)?),
        developer: r.get(12)?,
        released: r.get(13)?,
        length_minutes: r.get(14)?,
        added_at: r.get(15)?,
        last_played: r.get(16)?,
        total_seconds: r.get(17)?,
        play_count: r.get(18)?,
        hidden: r.get::<_, i64>(19)? != 0,
        favorite: r.get::<_, i64>(20)? != 0,
    })
}

/// 列出游戏。`show_hidden` 为 true 时同时返回已隐藏项（排在末尾）。
pub fn list_games(conn: &Connection, show_hidden: bool) -> Result<Vec<Game>> {
    let sql = if show_hidden {
        format!("SELECT {GAME_COLS} FROM games ORDER BY hidden, favorite DESC, last_played DESC")
    } else {
        format!(
            "SELECT {GAME_COLS} FROM games WHERE hidden = 0 ORDER BY favorite DESC, last_played DESC, id DESC"
        )
    };
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |r| row_to_game(r))?;
    rows.collect()
}

pub fn get_game(conn: &Connection, id: i64) -> Result<Game> {
    conn.query_row(
        &format!("SELECT {GAME_COLS} FROM games WHERE id = ?1"),
        params![id],
        row_to_game,
    )
}

#[allow(dead_code)] // M2+ 去重/合并会用
pub fn get_game_by_dir(conn: &Connection, dir: &str) -> Result<Option<Game>> {
    match conn.query_row(
        &format!("SELECT {GAME_COLS} FROM games WHERE source_dir = ?1"),
        params![dir],
        row_to_game,
    ) {
        Ok(g) => Ok(Some(g)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn insert_game(
    conn: &mut Connection,
    title: &str,
    src_dir: &str,
    launch: &str,
    candidates: &[String],
    engine: &str,
    cover: Option<&str>,
    scanned_at: i64,
) -> Result<i64> {
    let cand_json = serde_json::to_string(candidates).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "INSERT INTO games (title, source_dir, launch_path, launch_candidates, engine, cover_path, added_at, scanned_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![title, src_dir, launch, cand_json, engine, cover, scanned_at],
    )?;
    Ok(conn.last_insert_rowid())
}

/// 选定启动文件并持久化为默认。
pub fn set_launch_file(conn: &Connection, id: i64, launch_path: &str) -> Result<Game> {
    conn.execute(
        "UPDATE games SET launch_path = ?2, launch_set = 1 WHERE id = ?1",
        params![id, launch_path],
    )?;
    get_game(conn, id)
}

/// 把 VNDB 抓到的元数据合并进游戏（None 字段保留原值）。
pub fn update_metadata(
    conn: &Connection,
    id: i64,
    description: Option<&str>,
    rating: Option<f64>,
    vndb_id: Option<&str>,
    tags: Vec<String>,
    developer: Option<&str>,
    released: Option<&str>,
    length_minutes: Option<i64>,
    cover_path: Option<&str>,
) -> Result<Game> {
    let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".into());
    conn.execute(
        "UPDATE games SET
           description = COALESCE(?1, description),
           rating = COALESCE(?2, rating),
           vndb_id = COALESCE(?3, vndb_id),
           tags = ?4,
           developer = COALESCE(?5, developer),
           released = COALESCE(?6, released),
           length_minutes = COALESCE(?7, length_minutes),
           cover_path = COALESCE(?8, cover_path)
         WHERE id = ?9",
        params![
            description,
            rating,
            vndb_id,
            tags_json,
            developer,
            released,
            length_minutes,
            cover_path,
            id
        ],
    )?;
    get_game(conn, id)
}

pub fn set_title(conn: &Connection, id: i64, title: &str) -> Result<Game> {
    conn.execute("UPDATE games SET title = ?2 WHERE id = ?1", params![id, title])?;
    get_game(conn, id)
}

pub fn set_hidden(conn: &Connection, id: i64, hidden: bool) -> Result<Game> {
    conn.execute("UPDATE games SET hidden = ?2 WHERE id = ?1", params![id, hidden as i64])?;
    get_game(conn, id)
}

pub fn toggle_favorite(conn: &Connection, id: i64) -> Result<Game> {
    conn.execute(
        "UPDATE games SET favorite = CASE favorite WHEN 0 THEN 1 ELSE 0 END WHERE id = ?1",
        params![id],
    )?;
    get_game(conn, id)
}

#[allow(dead_code)] // 手动换本地封面会用
pub fn update_cover(conn: &Connection, id: i64, cover: Option<&str>) -> Result<()> {
    conn.execute("UPDATE games SET cover_path = ?2 WHERE id = ?1", params![id, cover])?;
    Ok(())
}

/// 从库中移除记录（不动磁盘文件）。
pub fn delete_from_library(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear_files(conn: &Connection, game_id: i64) -> Result<()> {
    conn.execute("DELETE FROM game_files WHERE game_id = ?1", params![game_id])?;
    Ok(())
}

/// 批量写入文件画像（先清后写）。
pub fn replace_files(conn: &mut Connection, game_id: i64, files: &[(String, String, i64)]) -> Result<()> {
    clear_files(conn, game_id)?;
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT INTO game_files (game_id, rel_path, kind, size) VALUES (?1, ?2, ?3, ?4)",
        )?;
        for (rel, kind, size) in files {
            stmt.execute(params![game_id, rel, kind, size])?;
        }
    }
    tx.commit()
}

pub fn list_files(conn: &Connection, game_id: i64) -> Result<Vec<FileInfo>> {
    let mut stmt = conn.prepare(
        "SELECT id, rel_path, kind, size FROM game_files WHERE game_id = ?1 ORDER BY rel_path",
    )?;
    let rows = stmt.query_map(params![game_id], |r| {
        Ok(FileInfo {
            id: r.get(0)?,
            rel_path: r.get(1)?,
            kind: r.get(2)?,
            size: r.get(3)?,
        })
    })?;
    rows.collect()
}

/// 判断某目录是否已被收录。
pub fn is_imported(conn: &Connection, dir: &str) -> bool {
    conn.query_row(
        "SELECT count(*) FROM games WHERE source_dir = ?1",
        params![dir],
        |r| r.get::<_, i64>(0),
    )
    .map(|c| c > 0)
    .unwrap_or(false)
}

pub fn get_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row("SELECT value FROM settings WHERE key = ?1", params![key], |r| r.get(0))
        .ok()
}

pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// 玩家统计：把一次会话计入游戏总时长与次数（由后台监视线程调用）。
/// 在线程内独立打开连接，避开持有 AppState 锁。
pub fn finish_session(db_path: &Path, session_id: i64, game_id: i64, started_at: i64) {
    if let Ok(conn) = Connection::open(db_path) {
        let now = crate::util::now_secs();
        let _ = conn.execute(
            "UPDATE play_sessions SET ended_at = ?2 WHERE id = ?1",
            params![session_id, now],
        );
        let duration = now.saturating_sub(started_at) as i64;
        let _ = conn.execute(
            "UPDATE games SET total_seconds = total_seconds + ?1, play_count = play_count + 1, last_played = ?2 WHERE id = ?3",
            params![duration, now, game_id],
        );
    }
}

// ---------------- 补丁仓储 ----------------

fn row_to_patch(r: &rusqlite::Row) -> Result<Patch> {
    Ok(Patch {
        id: r.get(0)?,
        game_id: r.get(1)?,
        name: r.get(2)?,
        kind: r.get(3)?,
        source_path: r.get(4)?,
        install_method: r.get(5)?,
        installed: r.get::<_, i64>(6)? != 0,
        installed_at: r.get(7)?,
        backup_dir: r.get(8)?,
        note: r.get(9)?,
    })
}

const PATCH_COLS: &str = "id,game_id,name,kind,source_path,install_method,installed,installed_at,backup_dir,note";

pub fn insert_patch(
    conn: &Connection,
    game_id: i64,
    name: &str,
    kind: &str,
    source_path: &str,
    install_method: &str,
    backup_dir: Option<&str>,
    note: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO patches (game_id,name,kind,source_path,install_method,backup_dir,note)
         VALUES (?1,?2,?3,?4,?5,?6,?7)",
        params![game_id, name, kind, source_path, install_method, backup_dir, note],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_patch(conn: &Connection, id: i64) -> Result<Patch> {
    conn.query_row(&format!("SELECT {PATCH_COLS} FROM patches WHERE id = ?1"), params![id], row_to_patch)
}

pub fn list_patches(conn: &Connection, game_id: i64) -> Result<Vec<Patch>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {PATCH_COLS} FROM patches WHERE game_id = ?1 ORDER BY installed DESC, id DESC"
    ))?;
    let rows = stmt.query_map(params![game_id], |r| row_to_patch(r))?;
    rows.collect()
}

pub fn set_patch_backup_dir(conn: &Connection, id: i64, dir: &str) -> Result<()> {
    conn.execute("UPDATE patches SET backup_dir = ?2 WHERE id = ?1", params![id, dir])?;
    Ok(())
}

pub fn set_patch_installed(conn: &Connection, id: i64, installed: bool) -> Result<Patch> {
    conn.execute(
        "UPDATE patches SET installed = ?2, installed_at = CASE WHEN ?2 = 1 THEN ?3 ELSE NULL END WHERE id = ?1",
        params![id, installed as i64, crate::util::now_secs()],
    )?;
    get_patch(conn, id)
}

pub fn insert_backup(conn: &Connection, patch_id: i64, target_rel: &str, backup_path: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO patch_backups (patch_id, target_rel, backup_path) VALUES (?1,?2,?3)",
        params![patch_id, target_rel, backup_path],
    )?;
    Ok(())
}

/// 某个补丁已备份过的（相对目标，去重用）。
#[allow(dead_code)]
pub fn list_backups(conn: &Connection, patch_id: i64) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT target_rel, backup_path FROM patch_backups WHERE patch_id = ?1",
    )?;
    let rows = stmt.query_map(params![patch_id], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    rows.collect()
}

pub fn clear_patch_backups(conn: &Connection, patch_id: i64) -> Result<()> {
    conn.execute("DELETE FROM patch_backups WHERE patch_id = ?1", params![patch_id])?;
    Ok(())
}

pub fn delete_patch(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM patches WHERE id = ?1", params![id])?;
    Ok(())
}
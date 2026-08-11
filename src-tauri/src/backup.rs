//! 整库备份 / 恢复。
//!
//! 备份 zip 内容：`gal_launcher.db`（VACUUM INTO 一致快照）+ `covers/` + `backups/`（补丁备份）。
//! 排除 `assets/`（可重新解包再生的缓存，体积可能巨大）。
//!
//! 恢复采用「先解压到 staging → 校验 → 热切换连接 → 替换文件 → 重开连接」，
//! 避免 Windows 下替换被打开文件时的共享冲突；校验失败则不动任何现有数据。

use std::path::{Path, PathBuf};

use crate::db;
use crate::util;
use rusqlite::Connection;
use serde::Serialize;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

/// 备份产物信息（返回给前端）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupOut {
    pub path: String,
    pub file_count: usize,
}

/// 恢复结果信息（返回给前端）。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOut {
    pub games: usize,
    pub covers: usize,
    pub backups: usize,
}

fn data_dir(db_path: &Path) -> PathBuf {
    db_path.parent().unwrap_or(Path::new(".")).to_path_buf()
}

/// 文件名用的时间戳：YYYYMMDD-HHMMSS（本地时间近似，够用于排序/辨识）。
fn ts_stamp() -> String {
    let s = util::now_secs();
    let days = s.div_euclid(86_400);
    let secs = s.rem_euclid(86_400);
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let sec = secs % 60;
    // civil-from-days（Howard Hinnant），UTC 基准
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if mo <= 2 { y + 1 } else { y };
    format!("{y:04}{mo:02}{d:02}-{h:02}{m:02}{sec:02}")
}

/// 导出备份到 `dest_dir`，返回 zip 路径与文件数。
pub fn export(db_path: &Path, dest_dir: &Path) -> Result<BackupOut, String> {
    let dir = data_dir(db_path);
    let tmp = dir.join(".backup_tmp.db");
    let _ = std::fs::remove_file(&tmp);
    {
        let conn = Connection::open(db_path).map_err(|e| format!("打开数据库失败: {e}"))?;
        let sql = format!(
            "VACUUM INTO '{}'",
            tmp.display().to_string().replace('\'', "''")
        );
        conn.execute_batch(&sql)
            .map_err(|e| format!("生成数据库快照失败: {e}"))?;
    }
    std::fs::create_dir_all(dest_dir).map_err(|e| format!("创建导出目录失败: {e}"))?;
    let zip_path = dest_dir.join(format!("GAL启动器备份-{}.zip", ts_stamp()));

    let file = std::fs::File::create(&zip_path).map_err(|e| format!("创建备份文件失败: {e}"))?;
    let mut zw = ZipWriter::new(file);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o600);

    let mut count = 0usize;
    // 数据库快照
    zw.start_file("gal_launcher.db", opts)
        .map_err(|e| e.to_string())?;
    let mut f = std::fs::File::open(&tmp).map_err(|e| e.to_string())?;
    std::io::copy(&mut f, &mut zw).map_err(|e| format!("写入数据库失败: {e}"))?;
    count += 1;

    // covers / backups（排除可再生的 assets/）
    for sub in ["covers", "backups"] {
        let src = dir.join(sub);
        if !src.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(&src).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = entry
                .path()
                .strip_prefix(&dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default()
                .replace('\\', "/");
            zw.start_file(rel, opts).map_err(|e| e.to_string())?;
            let mut f = std::fs::File::open(entry.path()).map_err(|e| e.to_string())?;
            std::io::copy(&mut f, &mut zw).map_err(|e| format!("写入文件失败: {e}"))?;
            count += 1;
        }
    }

    zw.finish().map_err(|e| format!("写入备份文件失败: {e}"))?;
    let _ = std::fs::remove_file(&tmp);
    Ok(BackupOut {
        path: zip_path.to_string_lossy().into_owned(),
        file_count: count,
    })
}

/// 从备份 zip 恢复：校验通过后热切换数据库连接并替换文件。
/// `conn` 是 AppState 持有的连接，会被就地替换为恢复后的新连接。
pub fn restore(
    db_path: &Path,
    zip_path: &Path,
    conn: &mut Connection,
) -> Result<RestoreOut, String> {
    let dir = data_dir(db_path);
    let staging = dir.join(".restore_staging");
    let _ = std::fs::remove_dir_all(&staging);
    std::fs::create_dir_all(&staging).map_err(|e| format!("创建临时目录失败: {e}"))?;

    // 1. 解压到 staging（用 enclosed_name 防路径穿越）
    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开备份文件失败: {e}"))?;
    let mut za = zip::ZipArchive::new(file).map_err(|e| format!("不是有效的 zip 备份: {e}"))?;
    let mut games = 0usize;
    let mut covers = 0usize;
    let mut backups = 0usize;
    for i in 0..za.len() {
        let mut entry = za.by_index(i).map_err(|e| e.to_string())?;
        let safe = entry.enclosed_name().ok_or_else(|| "备份内含非法路径".to_string())?;
        if safe.components().count() == 0 {
            continue;
        }
        let out = staging.join(&safe);
        if let Some(parent) = out.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("解压失败: {e}"))?;
        }
        let mut f = std::fs::File::create(&out).map_err(|e| format!("解压失败: {e}"))?;
        std::io::copy(&mut entry, &mut f).map_err(|e| format!("解压失败: {e}"))?;
        let name = entry.name();
        if name == "gal_launcher.db" {
            games = 1;
        } else if name.starts_with("covers/") {
            covers += 1;
        } else if name.starts_with("backups/") {
            backups += 1;
        }
    }
    drop(za);

    if games == 0 {
        let _ = std::fs::remove_dir_all(&staging);
        return Err("备份里没有 gal_launcher.db，不是有效的整库备份".into());
    }

    // 2. 先校验 staging 里的库（别拿损坏文件覆盖现有数据）
    {
        let v = Connection::open(staging.join("gal_launcher.db"))
            .map_err(|e| format!("备份数据库无法打开: {e}"))?;
        v.query_row("SELECT COUNT(*) FROM games", [], |r| r.get::<_, i64>(0))
            .map_err(|_| "备份数据库缺少 games 表，已中止恢复".to_string())?;
    }

    // 3. 释放旧连接文件句柄（换成一个内存连接），再替换文件
    let _ = std::mem::replace(
        conn,
        Connection::open_in_memory().map_err(|e| format!("内部错误: {e}"))?,
    );
    for suffix in ["-wal", "-shm"] {
        let _ = std::fs::remove_file(dir.join(format!("gal_launcher.db{suffix}")));
    }
    std::fs::copy(staging.join("gal_launcher.db"), db_path)
        .map_err(|e| format!("替换数据库失败: {e}"))?;
    for sub in ["covers", "backups"] {
        let src = staging.join(sub);
        if src.is_dir() {
            copy_dir_merge(&src, &dir.join(sub)).map_err(|e| format!("恢复 {sub}/ 失败: {e}"))?;
        }
    }

    // 4. 重开连接（重新走 WAL + 建表 + 老库补列）
    let reopened = db::init(db_path).map_err(|e| format!("重新打开恢复后的数据库失败: {e}"))?;
    let _ = std::mem::replace(conn, reopened);

    let _ = std::fs::remove_dir_all(&staging);
    Ok(RestoreOut {
        games,
        covers,
        backups,
    })
}

/// 把 `src` 目录下的文件递归复制合并进 `dst`（覆盖同名文件）。
fn copy_dir_merge(src: &Path, dst: &Path) -> std::io::Result<()> {
    for entry in walkdir::WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(src).unwrap_or(entry.path());
        let target = dst.join(rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(entry.path(), &target)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn scratch(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "galbackup-test-{tag}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn export_then_restore_roundtrip() {
        let root = scratch("rt");
        let data = root.join("data");
        std::fs::create_dir_all(&data).unwrap();
        let db_path = data.join("gal_launcher.db");
        {
            let conn = db::init(&db_path).unwrap();
            conn.execute_batch(
                "INSERT INTO games (title, source_dir, launch_path, launch_candidates, added_at) \
                 VALUES ('测试', 'C:/t', 'C:/t/a.exe', '[]', 1);",
            )
            .unwrap();
        }
        std::fs::create_dir_all(data.join("covers")).unwrap();
        std::fs::write(data.join("covers/v1.png"), b"cover").unwrap();

        // 导出
        let dest = root.join("out");
        let b = export(&db_path, &dest).unwrap();
        assert!(Path::new(&b.path).exists());
        assert!(b.file_count >= 2);

        // 破坏原库，再恢复
        std::fs::write(&db_path, b"corrupted").unwrap();
        let mut conn = Connection::open_in_memory().unwrap();
        let r = restore(&db_path, Path::new(&b.path), &mut conn).unwrap();
        assert_eq!(r.games, 1);
        assert_eq!(r.covers, 1);
        assert_eq!(r.backups, 0);

        // 恢复后的库能查到原游戏
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM games", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
        assert!(data.join("covers/v1.png").exists());

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn restore_rejects_non_backup() {
        let root = scratch("bad");
        let data = root.join("data");
        std::fs::create_dir_all(&data).unwrap();
        let db_path = data.join("gal_launcher.db");
        db::init(&db_path).unwrap();
        let bad = root.join("bad.zip");
        std::fs::write(&bad, b"not a zip").unwrap();
        let mut conn = Connection::open_in_memory().unwrap();
        assert!(restore(&db_path, &bad, &mut conn).is_err());
        let _ = std::fs::remove_dir_all(&root);
    }
}

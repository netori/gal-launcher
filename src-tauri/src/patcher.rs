//! 补丁的安装与回滚。
//!
//! 覆盖式补丁：把来源（文件夹或 .zip）里的文件写进游戏目录，
//! 覆盖任何已存在文件前先把它备份到 backup_dir，记录到 patch_backups，
//! 以便一键回滚。安全上会拒绝 zip 里的 `..` 路径穿越，避免写到目录外。

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db;

/// 校验相对路径安全：拒绝绝对路径、盘符、`..` 穿越；抹掉 `./` 前缀。
fn safe_rel(raw: &str) -> Option<String> {
    let norm = raw.replace('\\', "/");
    let norm = norm.trim_start_matches("./").to_string();
    if norm.starts_with('/') {
        return None;
    }
    if norm.as_bytes().get(1) == Some(&b':') {
        return None; // 盘符
    }
    if norm.split('/').any(|p| p == "..") {
        return None;
    }
    if norm.is_empty() {
        return None;
    }
    Some(norm)
}

/// 安装覆盖式补丁：返回实际写入/覆盖的文件数。
pub fn install_replace(
    game_dir: &Path,
    source: &Path,
    backup_dir: &Path,
    db: &Connection,
    patch_id: i64,
) -> Result<usize, String> {
    if !game_dir.is_dir() {
        return Err("游戏目录不存在".into());
    }
    if !source.exists() {
        return Err(format!("补丁来源不存在：{}", source.display()));
    }

    let count = if source.is_dir() {
        install_from_dir(game_dir, source, backup_dir, db, patch_id)?
    } else if source
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("zip"))
        .unwrap_or(false)
    {
        install_from_zip(game_dir, source, backup_dir, db, patch_id)?
    } else {
        return Err("补丁来源必须是文件夹或 .zip 压缩包".into());
    };

    if count == 0 {
        return Err("补丁中没有任何文件".into());
    }
    Ok(count)
}

fn write_file(
    game_dir: &Path,
    backup_dir: &Path,
    db: &Connection,
    patch_id: i64,
    rel: &str,
    content_from: &mut dyn Read,
) -> Result<(), String> {
    let rel = safe_rel(rel).ok_or_else(|| format!("补丁内含非法路径：{rel}"))?;
    let target = game_dir.join(&rel);

    // 覆盖前备份已有文件
    if target.is_file() {
        let backup_file = backup_dir.join(&rel);
        if let Some(parent) = backup_file.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建备份目录失败: {e}"))?;
        }
        fs::copy(&target, &backup_file).map_err(|e| format!("备份 {} 失败: {e}", target.display()))?;
        let _ = db::insert_backup(db, patch_id, &rel, &backup_file.to_string_lossy());
    }

    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
    }
    let mut out = fs::File::create(&target).map_err(|e| format!("写入 {} 失败: {e}", target.display()))?;
    std::io::copy(content_from, &mut out).map_err(|e| format!("写入 {} 失败: {e}", target.display()))?;
    Ok(())
}

fn install_from_dir(
    game_dir: &Path,
    source: &Path,
    backup_dir: &Path,
    db: &Connection,
    patch_id: i64,
) -> Result<usize, String> {
    let mut count = 0usize;
    for entry in walkdir::WalkDir::new(source).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(source)
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        let mut file = fs::File::open(entry.path()).map_err(|e| format!("读取 {} 失败: {e}", entry.path().display()))?;
        write_file(game_dir, backup_dir, db, patch_id, &rel, &mut file)?;
        count += 1;
    }
    Ok(count)
}

fn install_from_zip(
    game_dir: &Path,
    source: &Path,
    backup_dir: &Path,
    db: &Connection,
    patch_id: i64,
) -> Result<usize, String> {
    let file = fs::File::open(source).map_err(|e| format!("打开 zip 失败: {e}"))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解析 zip 失败: {e}"))?;

    // 不少补丁包会把所有文件包在一层文件夹里，安装时剥掉这层公共前缀。
    let strip = common_prefix(&mut archive);

    let mut count = 0usize;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let rel = match &strip {
            Some(p) => name.strip_prefix(p).unwrap_or(&name).to_string(),
            None => name,
        };
        if safe_rel(&rel).is_none() {
            continue;
        }
        write_file(game_dir, backup_dir, db, patch_id, &rel, &mut entry)?;
        count += 1;
    }
    Ok(count)
}

/// 若 zip 里所有文件都以同一第一级目录开头，返回该前缀（不含尾部斜杠）。
fn common_prefix(archive: &mut zip::ZipArchive<std::fs::File>) -> Option<String> {
    let mut names: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(e) = archive.by_index(i) {
            if !e.is_dir() {
                names.push(e.name().to_string());
            }
        }
    }
    if names.len() < 2 {
        return None;
    }
    names.sort();
    let first = names.first()?.clone();
    let last = names.last()?.clone();
    if !first.contains('/') && !last.contains('/') {
        return None;
    }
    // 公共前缀：逐层找共同目录
    let common = first
        .split('/')
        .zip(last.split('/'))
        .take_while(|(a, b)| a == b)
        .map(|(a, _)| a)
        .collect::<Vec<&str>>()
        .join("/");
    if common.is_empty() {
        return None;
    }
    let p = format!("{common}/");
    if names.iter().all(|n| n.starts_with(&p)) {
        Some(p)
    } else {
        None
    }
}

/// 回滚：把备份目录下记录的文件恢复回游戏目录。
pub fn rollback(game_dir: &Path, db: &Connection, patch_id: i64) -> Result<usize, String> {
    let backups = db::list_backups(db, patch_id).map_err(|e| e.to_string())?;
    let mut count = 0usize;
    for (rel, backup_path) in backups {
        let rel = safe_rel(&rel).ok_or_else(|| format!("非法备份路径：{rel}"))?;
        let bp = PathBuf::from(&backup_path);
        if !bp.is_file() {
            continue;
        }
        let target = game_dir.join(&rel);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).ok();
        }
        fs::copy(&bp, &target).map_err(|e| format!("恢复 {} 失败: {e}", target.display()))?;
        let _ = fs::remove_file(&bp);
        count += 1;
    }
    db::clear_patch_backups(db, patch_id).map_err(|e| e.to_string())?;
    Ok(count)
}

/// 启动安装器型补丁（直接运行安装程序）。仅桌面端（exe 在移动端无意义）。
#[cfg(target_os = "windows")]
pub fn run_installer(game_dir: &Path, source: &Path) -> Result<(), String> {
    if !source.is_file() {
        return Err(format!("安装程序不存在：{}", source.display()));
    }
    std::process::Command::new(source)
        .current_dir(game_dir)
        .spawn()
        .map(|_| ())
        .map_err(|e| format!("启动安装程序失败: {e}"))
}

/// 移动端不支持运行 .exe 安装器；统一报错提示。
#[cfg(not(target_os = "windows"))]
pub fn run_installer(_game_dir: &Path, _source: &Path) -> Result<(), String> {
    Err("安装器型补丁（.exe）仅支持桌面端".into())
}

/// 用补丁来源的扩展名粗判安装方式。
pub fn guess_method(source: &Path) -> &'static str {
    match source.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()) {
        Some(e) if e == "exe" => "installer",
        _ => "replace",
    }
}

/// 删除补丁时清理备份目录。
pub fn remove_backup_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn mem_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE patch_backups (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                patch_id INTEGER NOT NULL,
                target_rel TEXT NOT NULL,
                backup_path TEXT NOT NULL
            );",
        )
        .unwrap();
        conn
    }

    struct Scratch {
        root: PathBuf,
    }
    impl Scratch {
        fn new(tag: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "galpatcher-test-{tag}-{}",
                std::process::id()
            ));
            let _ = fs::remove_dir_all(&root);
            fs::create_dir_all(&root).unwrap();
            Scratch { root }
        }
        fn sub(&self, p: &str) -> PathBuf {
            let d = self.root.join(p);
            fs::create_dir_all(&d).unwrap();
            d
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    fn write(p: &Path, content: &str) {
        fs::write(p, content).unwrap();
    }

    #[test]
    fn install_then_rollback_roundtrip() {
        let game_scr = Scratch::new("game");
        let src_scr = Scratch::new("src");
        let bak_scr = Scratch::new("bak");
        let game_dir = game_scr.sub("game");
        let source = src_scr.sub("patch");
        let backup = bak_scr.sub("b");
        write(&game_dir.join("game.exe"), "old");
        write(&source.join("game.exe"), "new");
        write(&source.join("a.txt"), "hello");
        let db = mem_db();

        let n = install_replace(&game_dir, &source, &backup, &db, 42).unwrap();
        assert_eq!(n, 2);
        assert_eq!(fs::read_to_string(game_dir.join("game.exe")).unwrap(), "new");
        // 只有原本存在的 game.exe 才有备份
        let backups = db::list_backups(&db, 42).unwrap();
        assert_eq!(backups.len(), 1);

        let r = rollback(&game_dir, &db, 42).unwrap();
        assert_eq!(r, 1);
        assert_eq!(fs::read_to_string(game_dir.join("game.exe")).unwrap(), "old");
        // 新增文件 a.txt 保留
        assert_eq!(fs::read_to_string(game_dir.join("a.txt")).unwrap(), "hello");
    }

    #[test]
    fn zip_common_prefix_is_stripped() {
        // 构造一个 zip：所有文件都在 "外层/..." 下
        let root = Scratch::new("zip");
        let zipped = root.sub("make").join("p.zip");
        {
            let f = fs::File::create(&zipped).unwrap();
            let mut w = zip::ZipWriter::new(f);
            let mut missing = zip::write::SimpleFileOptions::default();
            missing = missing.compression_method(zip::CompressionMethod::Stored);
            for (p, c) in [
                ("外层/game.exe", "new-bin"),
                ("外层/data/a.txt", "hello"),
            ] {
                w.start_file(p, missing).unwrap();
                use std::io::Write;
                w.write_all(c.as_bytes()).unwrap();
            }
            w.finish().unwrap();
        }
        let game_dir = root.sub("game");
        let backup = root.sub("bak");
        write(&game_dir.join("game.exe"), "old");
        let db = mem_db();

        let n = install_replace(&game_dir, &zipped, &backup, &db, 7).unwrap();
        assert_eq!(n, 2);
        assert_eq!(fs::read_to_string(game_dir.join("game.exe")).unwrap(), "new-bin");
        assert_eq!(fs::read_to_string(game_dir.join("data/a.txt")).unwrap(), "hello");

        let r = rollback(&game_dir, &db, 7).unwrap();
        assert_eq!(r, 1);
        assert_eq!(fs::read_to_string(game_dir.join("game.exe")).unwrap(), "old");
    }
}
//! 游戏启动与游玩时长统计。

use std::path::Path;
use std::path::PathBuf;
use std::process::{Child, Command};

use crate::db;
use crate::models::Game;

/// 生成启动子进程。返回 Child 交给调用方做后台时长统计。
pub fn spawn_child(game: &Game, le_path: Option<&str>, use_locale: bool) -> Result<Child, String> {
    if !Path::new(&game.launch_path).exists() {
        return Err(format!("启动文件不存在：{}", game.launch_path));
    }
    let dir = Path::new(&game.source_dir);
    if !dir.is_dir() {
        return Err(format!("游戏目录不存在：{}", game.source_dir));
    }

    if use_locale {
        let le = le_path
            .filter(|p| Path::new(p).exists())
            .ok_or("未配置有效的 Locale Emulator 路径，请先在设置中指定")?;
        Command::new(le)
            .args(["-runas", "ja-JP", &game.launch_path])
            .current_dir(dir)
            .spawn()
            .map_err(|e| format!("通过 Locale Emulator 启动失败: {e}"))
    } else {
        Command::new(&game.launch_path)
            .current_dir(dir)
            .spawn()
            .map_err(|e| format!("启动失败: {e}"))
    }
}

/// 后台线程任务：等游戏进程退出后，结算本次会话时长并把结果写入数据库。
pub fn watch_session(db_path: PathBuf, session_id: i64, game_id: i64, started_at: i64, child: Child) {
    std::thread::spawn(move || {
        let mut child = child;
        let _ = child.wait();
        db::finish_session(&db_path, session_id, game_id, started_at);
    });
}
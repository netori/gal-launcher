//! 游戏扫描与识别引擎。
//!
//! 职责：在用户指定的根目录下递归找出「疑似游戏目录」，
//! 为每个目录判定启动文件、识别游戏引擎、输出目录文件画像、探测本地封面。

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::models::Candidate;
use crate::util;

/// 扫描深度上限（层数）。
const MAX_DEPTH: usize = 10;

/// 扫描根目录，返回所有疑似游戏目录候选。
/// `imported` 是已收录目录集合（用于标注 already_imported），
/// 由调用方在短时间内查库取得，避免在漫长的磁盘扫描期间长时间占用数据库锁。
pub fn scan_directory(root: &Path, imported: &std::collections::HashSet<String>) -> Vec<Candidate> {
    if !root.is_dir() {
        return Vec::new();
    }

    // 第一遍：记录所有目录，以及每个目录直接子文件中的 exe 名。
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut exe_map: HashMap<PathBuf, Vec<String>> = HashMap::new();

    for entry in WalkDir::new(root)
        .min_depth(1)
        .max_depth(MAX_DEPTH)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_skip_dir(e))
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if entry.depth() > MAX_DEPTH {
            continue;
        }
        if entry.file_type().is_dir() {
            dirs.push(entry.path().to_path_buf());
            continue;
        }
        let fname = match entry.file_name().to_str() {
            Some(s) => s,
            None => continue,
        };
        if is_exe(fname) {
            if let Some(parent) = entry.path().parent() {
                exe_map.entry(parent.to_path_buf()).or_default().push(fname.to_string());
            }
        }
    }

    // 第二遍：候选 = 直接包含合格 exe 的目录；再为每个候选选出主启动文件、识别引擎、探测封面。
    let mut raw: Vec<(PathBuf, String, Vec<String>, String, Option<String>)> = Vec::new();
    // 结构：dir, 主启动名, 全部候选名, engine, cover
    for dir in dirs.iter() {
        let exes = match exe_map.get(dir) {
            Some(v) => v.clone(),
            None => continue,
        };
        let eligible: Vec<&String> = exes.iter().filter(|e| is_launch_eligible(e)).collect();
        if eligible.is_empty() {
            continue;
        }
        let candidates: Vec<String> = eligible.iter().map(|e| e.to_string()).collect();
        let chosen = pick_main_exe(dir, &eligible);
        let engine = detect_engine(dir, &chosen);
        let cover = find_local_cover(dir);
        raw.push((dir.clone(), chosen, candidates, engine, cover));
    }

    // 去重：若某目录的祖先目录也是候选，只保留更靠近根目录的那个（避免把父目录无谓列出来）。
    raw.sort_by(|a, b| a.0.components().count().cmp(&b.0.components().count()));
    let mut final_cands: Vec<(PathBuf, String, Vec<String>, String, Option<String>)> = Vec::new();
    for c in raw {
        let ancestor_is_candidate = final_cands
            .iter()
            .any(|(k, _, _, _, _)| c.0 != *k && c.0.starts_with(k));
        if !ancestor_is_candidate {
            final_cands.push(c);
        }
    }

    // 第三步：组装成对外数据结构。
    let mut out = Vec::new();
    for (dir, exe, candidates, engine, cover) in final_cands {
        let title = dir.file_name().and_then(|s| s.to_str()).unwrap_or("未命名游戏").to_string();
        let source_dir = dir.to_string_lossy().to_string();
        let launch_path = dir.join(&exe).to_string_lossy().to_string();
        let already = imported.contains(&util::norm_path(&source_dir));
        out.push(Candidate {
            title,
            source_dir,
            launch_path,
            launch_candidates: candidates,
            engine: engine.clone(),
            cover_path: cover,
            file_count: count_files(&dir),
            already_imported: already,
            note: describe_engine(&engine),
        });
    }
    out
}

fn is_skip_dir(e: &walkdir::DirEntry) -> bool {
    if !e.file_type().is_dir() || e.depth() == 0 {
        return false;
    }
    let name = e.file_name().to_string_lossy();
    name.starts_with('.') || name.eq_ignore_ascii_case("system volume information")
}

fn is_exe(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".exe")
}

/// 哪些名字铁定不是主启动程序的 exe。
fn is_launch_eligible(name: &str) -> bool {
    let low = name.to_ascii_lowercase();
    const BAD: &[&str] = &[
        "install",
        "setup",
        "update",
        "updater",
        "unins",
        "uninstall",
        "dxsetup",
        "redist",
        "vcredist",
        "dotnet",
        "commonredist",
        "fomod",
        "manual",
        "说明",
        "説明书",
        "アップデート",
        "修正パッチ",
        "启动器",
    ];
    !BAD.iter().any(|b| low.contains(b))
}

/// 从同一目录的多个合格 exe 中挑出主启动文件。
///
/// 原则：体积大的更像游戏本体；但 config/configure/startup/data 这类工具
/// 即便尺寸略大也是误选，要重罚；与目录名共享词根的更可能是本体。
fn pick_main_exe(dir: &Path, exes: &[&String]) -> String {
    let folder = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    // 目录名里的“词根”（字母数字段，长度≥3）
    let folder_tokens: Vec<String> = folder
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| t.len() >= 3)
        .map(|t| t.to_string())
        .collect();

    let mut best: Option<(String, i64, i64)> = None; // (name, score, mtime)
    for exe in exes {
        let path = dir.join(exe);
        let size = std::fs::metadata(&path).map(|m| m.len() as i64).unwrap_or(0);
        let mtime = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        let low = exe.to_ascii_lowercase();
        let mut score = size.min(4 * 1024 * 1024);

        // 工具类：重罚到基本不可能赢（无其它真 exe 时仍能兜底）
        for kw in ["configure", "config", "configur", "startup", "option", "setting", "boot", "data", "debug", "test", "demo", "update", "updater", "manual", "patch_tool", "工具", "设置", "设定"] {
            if low.contains(kw) {
                score -= 4 * 1024 * 1024;
                break;
            }
        }
        // 与目录名共享词根 → 更像本体
        let stem = Path::new(exe)
            .file_stem()
            .map(|s| s.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        for t in &folder_tokens {
            if stem.contains(t.as_str()) || low.contains(t.as_str()) {
                score += 1024 * 1024;
                break;
            }
        }

        let cur = best.as_ref().map(|b| (b.1, b.2)).unwrap_or((i64::MIN, i64::MIN));
        if (score, mtime) > cur {
            best = Some((exe.to_string(), score, mtime));
        }
    }
    best.map(|b| b.0).unwrap_or_else(|| {
        // 不可达兜底：eligible 非空则必然有 best
        let mut v: Vec<String> = exes.iter().map(|s| s.to_string()).collect();
        v.sort();
        v.swap_remove(0)
    })
}

/// 引擎签名检测：读 exe 前 ~256KB 找特征串，并配合目录内已知文件判断。
fn detect_engine(dir: &Path, exe_name: &str) -> String {
    let bytes = read_head(&dir.join(exe_name), 256 * 1024);
    let hay = String::from_utf8_lossy(&bytes).to_lowercase();
    let has = |needle: &str| hay.contains(needle);
    let names = dir_file_names(dir);
    let any_name = |needle: &str| names.iter().any(|n| n.to_ascii_lowercase().contains(needle));

    if has("artemis") || any_name(".pfs") || any_name("artemis") {
        return "Artemis".into();
    }
    if has("kiri") || has("tvp") || any_name(".xp3") || any_name(".xfp3") {
        return "吉里吉里 (KiriKiri/KrkrZ)".into();
    }
    if has("unity") || any_name("unityplayer.dll") {
        return "Unity".into();
    }
    if has("renpy") || any_name("renpy") {
        return "Ren'Py".into();
    }
    if has("rgss") || has("rpg_rt") || any_name("rgss") {
        return "RPG Maker".into();
    }
    if has("nscr") || any_name("nscript.") {
        return "NScripter".into();
    }
    if has("wolf") || any_name("wolf.exe") {
        return "WOLF RPG".into();
    }
    "常规自研引擎".into()
}

/// 引擎说明文案。
fn describe_engine(engine: &str) -> String {
    format!("引擎：{engine}")
}

/// 目录直接子文件名（小写）。
fn dir_file_names(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_lowercase()).collect())
        .unwrap_or_default()
}

/// 读取文件前 N 字节。
fn read_head(path: &Path, max: usize) -> Vec<u8> {
    std::fs::File::open(path).ok().and_then(|f| {
        use std::io::Read;
        let mut buf = Vec::with_capacity(max);
        let mut handle = f.take(max as u64);
        handle.read_to_end(&mut buf).ok().map(|_| buf)
    }).unwrap_or_default()
}

/// 在候选目录内（深度 ≤2）找一张像封面的本地图片（名字含 cover/封面/jacket 等）。
fn find_local_cover(dir: &Path) -> Option<String> {
    let mut found: Option<String> = None;
    for entry in WalkDir::new(dir).min_depth(1).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        let is_img = name.ends_with(".jpg")
            || name.ends_with(".jpeg")
            || name.ends_with(".png")
            || name.ends_with(".webp");
        if !is_img {
            continue;
        }
        if name.contains("cover") || name.contains("封面") || name.contains("jacket") || name.contains("thumb") {
            let p = entry.path().to_string_lossy().to_string();
            if entry.depth() == 1 {
                return Some(p);
            }
            if found.is_none() {
                found = Some(p);
            }
        }
    }
    found
}

/// 统计目录内文件数。
pub fn count_files(dir: &Path) -> usize {
    WalkDir::new(dir)
        .into_iter()
        .filter(|e| e.is_ok() && e.as_ref().unwrap().file_type().is_file())
        .count()
}

/// 把一个游戏目录完整列成文件画像，供入库时写入 game_files。
pub fn collect_game_files(dir: &Path) -> Vec<(String, String, i64)> {
    let mut out = Vec::new();
    for entry in WalkDir::new(dir).max_depth(MAX_DEPTH).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry.path().strip_prefix(dir).map(|p| p.to_string_lossy().into_owned()).unwrap_or_default();
        let size = entry.metadata().map(|m| m.len() as i64).unwrap_or(0);
        let kind = classify(&rel);
        if kind != "skip" {
            out.push((rel, kind.to_string(), size));
        }
    }
    out
}

/// 文件分类：launch / engine / asset / image / media / save / patch / archive / doc / other / skip
pub fn classify(rel: &str) -> &'static str {
    let low = rel.to_ascii_lowercase();

    let path_like = rel.replace('\\', "/");
    // 存档目录
    if path_like.contains("savedata")
        || path_like.contains("save_data")
        || path_like.contains("/save/")
        || path_like.starts_with("save/")
        || path_like.contains("/save")
    {
        return "save";
    }
    // 补丁类（名称关键词）
    if low.contains("patch")
        || low.contains("汉化")
        || low.contains("补丁")
        || low.contains("パッチ")
        || low.contains("r18")
        || low.contains("18禁")
    {
        return "patch";
    }

    let ext = low.rsplit('.').next().unwrap_or("");
    match ext {
        "exe" => "launch",
        "dll" | "ini" | "cfg" | "config" | "dat" => "engine",
        "xp3" | "xfp3" | "pfs" | "pfs0" | "pak" | "arc" | "bin" | "glb" | "gsc" | "uask"
        | "spb" | "asb" | "ks" | "tjs" | "js" | "nac" => "asset",
        "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp" | "svg" => "image",
        "ogg" | "wav" | "mp3" | "m4a" | "opus" | "wma" | "flac" | "mp4" | "avi" | "mkv"
        | "wmv" | "mpg" | "mpeg" | "amv" | "webm" => "media",
        "zip" | "rar" | "7z" | "lzh" | "tar" | "gz" | "iso" => "archive",
        "txt" | "html" | "htm" | "pdf" | "chm" | "url" | "lnk" | "doc" | "docx" | "xml" => "doc",
        "" => "skip",
        _ => "other",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp_tree(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gal_launcher_test_{name}_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn classify_map() {
        assert_eq!(classify("Gs.exe"), "launch");
        assert_eq!(classify("data/data.xp3"), "asset");
        assert_eq!(classify("汉化补丁/readme.txt"), "patch");
        assert_eq!(classify("savedata/sys.dat"), "save");
        assert_eq!(classify("bgm/01.ogg"), "media");
        assert_eq!(classify("壁纸/01.png"), "image");
        assert_eq!(classify("manual.pdf"), "doc");
    }

    #[test]
    fn eligibility_filter() {
        assert!(is_launch_eligible("GS.exe"));
        assert!(is_launch_eligible("ハルカノン.exe"));
        assert!(!is_launch_eligible("setup.exe"));
        assert!(!is_launch_eligible("unins000.exe"));
        assert!(!is_launch_eligible("vcredist_x86.exe"));
    }

    #[test]
    fn engine_detection() {
        let dir = tmp_tree("unity");
        fs::File::create(dir.join("Game.exe")).unwrap();
        fs::File::create(dir.join("UnityPlayer.dll")).unwrap();
        assert!(detect_engine(&dir, "Game.exe").contains("Unity"));
        let _ = fs::remove_dir_all(&dir);

        let dir2 = tmp_tree("krkr");
        fs::write(dir2.join("Gs.exe"), b"some bytes KiriKiri engine").unwrap();
        assert!(detect_engine(&dir2, "Gs.exe").contains("吉里"));
        let _ = fs::remove_dir_all(&dir2);

        let dir3 = tmp_tree("artemis");
        fs::write(dir3.join("Game.exe"), b"Artemis pfs container").unwrap();
        fs::write(dir3.join("data.pfs"), b"pf8").unwrap();
        assert_eq!(detect_engine(&dir3, "Game.exe"), "Artemis".to_string());
        let _ = fs::remove_dir_all(&dir3);
    }

    #[test]
    fn scan_finds_game_dir_skips_installer() {
        let root = tmp_tree("scan");
        let g1 = root.join("アプリゲーム1");
        fs::create_dir_all(g1.join("data")).unwrap();
        fs::write(g1.join("GS.exe"), b"fake exe KiriKiri xp3").unwrap();
        fs::write(g1.join("data/arc.xp3"), b"xp3").unwrap();
        // 安装工具目录的 setup.exe 不应产出候选
        fs::create_dir_all(root.join("安装工具")).unwrap();
        fs::write(root.join("安装工具/setup.exe"), b"installer").unwrap();

        let imported = std::collections::HashSet::new();
        let cands = scan_directory(&root, &imported);

        assert_eq!(cands.len(), 1, "应有 1 个候选，实际 {:?}", cands);
        let c = &cands[0];
        assert_eq!(c.title, "アプリゲーム1");
        assert_eq!(c.launch_path, g1.join("GS.exe").to_string_lossy().to_string());
        assert!(c.engine.contains("吉里"), "引擎应为吉里，实际 {}", c.engine);
        assert_eq!(c.file_count, 2);
        assert!(!c.already_imported);
        let _ = fs::remove_dir_all(&root);
    }

    /// config/configure/startup 这类工具即使体积略大，也不能被选为主启动。
    #[test]
    fn pick_prefers_real_exe_over_config_tool() {
        let root = tmp_tree("pick");
        fs::write(root.join("configure.exe"), vec![0u8; 1_900_000]).unwrap(); // 更大，但应被重罚
        fs::write(root.join("haison_fd.exe"), vec![0u8; 1_700_000]).unwrap();

        let names: Vec<String> = vec!["configure.exe".into(), "haison_fd.exe".into()];
        let refs: Vec<&String> = names.iter().collect();
        assert_eq!(pick_main_exe(&root, &refs), "haison_fd.exe");
        let _ = fs::remove_dir_all(&root);
    }

    /// 真实目录复现（--ignored）：看用户库里的识别结果。
    #[test]
    #[ignore]
    fn scan_real_library_repro() {
        let imported = std::collections::HashSet::new();
        for root in [r"F:\game\gal"] {
            println!("===== scanning {root} =====");
            let cands = scan_directory(Path::new(root), &imported);
            for c in &cands {
                println!(
                    "CAND | imp={} | {} | engine={}",
                    c.already_imported, c.title, c.engine
                );
                println!("     launch={}", c.launch_path);
            }
            println!("total={}", cands.len());
        }
    }
}
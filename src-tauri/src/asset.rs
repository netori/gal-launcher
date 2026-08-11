//! M3：资源解包与浏览。
//!
//! 支持格式：
//! - `.xp3`（吉里吉里 / KrKr）→ 使用 Apache-2.0 的 `xp3` crate 还原索引与解压
//! - `.pfs`（Artemis 引擎）→ 内置解析器：SHA-1(index) 派生 XOR 流密钥解密文件数据
//!
//! 解包产物写入应用数据目录 `assets/<game_id>/<archive_stem>/` 缓存，供前端画廊浏览。
//! 解包完成后自动按分类（立绘/CG/背景/语音/BGM/视频/脚本/文档/数据…）物理整理到子文件夹。
//! 边界：仅针对用户本地合法自持的游戏；受保护条目跳过并提示，不提供任何爆破。

use std::io::Read;
use std::path::Path;

use serde::Serialize;

/// 一个资源包（.xp3 / .pfs）的描述。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveInfo {
    /// 相对游戏目录的路径
    pub rel_path: String,
    pub abs_path: String,
    /// Xp3 / Pfs / Unknown
    pub format: String,
    pub size_bytes: i64,
    /// 已被解包的条目数（缓存存在时为 >0）
    pub extracted_count: usize,
}

/// 一个解包条目。
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetEntry {
    pub rel: String,
    /// 磁盘上绝对路径（供前端读图预览）；extract() 返回时为空
    #[serde(default)]
    pub abs_path: String,
    pub size: i64,
    /// 分类：立绘 / CG / 背景 / 其他图片 / 语音 / BGM / 音效 / 视频 / 脚本 / 其他
    pub category: String,
}

/// 探测资源包格式。
pub fn detect_format(path: &Path) -> Option<&'static str> {
    let mut buf = [0u8; 8];
    let mut f = std::fs::File::open(path).ok()?;
    if f.read_exact(&mut buf).is_err() {
        return None;
    }
    if buf.starts_with(b"XP3") {
        Some("Xp3")
    } else if buf.starts_with(b"pf8") || buf.starts_with(b"pf6") {
        Some("Pfs")
    } else if buf.starts_with(b"ESC-ARC1") || buf.starts_with(b"ESC-ARC2") {
        Some("EscArc")
    } else if buf.starts_with(b"PAC") && buf.get(3) != Some(&b'K') {
        Some("Pac")
    } else if crate::nsa::is_nsa(path) {
        Some("Nsa")
    } else {
        None
    }
}

/// 列出游戏目录树里的资源包。
pub fn list_archives(game_dir: &Path, cache_root: &Path) -> Vec<ArchiveInfo> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(game_dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        // 常见资源包扩展名（含魔数可能藏在其中的 bin/pac）
        let is_arch = matches!(
            entry.path().extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase()).as_deref(),
            Some("xp3") | Some("xfp3") | Some("pfs") | Some("pac") | Some("pac1") | Some("bin")
                | Some("dat") | Some("nsa") | Some("ns2") | Some("pkg") | Some("arc") | Some("pak")
        );
        if !is_arch {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(game_dir)
            .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
            .unwrap_or_else(|_| entry.file_name().to_string_lossy().into_owned());
        out.push(ArchiveInfo {
            rel_path: rel.clone(),
            abs_path: entry.path().to_string_lossy().to_string(),
            format: detect_format(entry.path()).unwrap_or("Unknown").to_string(),
            size_bytes: entry.metadata().map(|m| m.len() as i64).unwrap_or(0),
            extracted_count: extracted_entries_count(cache_root, &rel),
        });
    }
    // 大的放前面
    out.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    out
}

/// 解包一个资源包到 its-dir，返回条目列表。
/// 解包完成后会自动把文件物理整理到「<分类>」子文件夹（见 sort_extracted）。
pub fn extract(archive: &Path, out_dir: &Path) -> Result<Vec<AssetEntry>, String> {
    let fmt = detect_format(archive).ok_or_else(|| {
        "无法识别资源包格式（支持 .xp3 与 Artemis .pfs）；可用外部工具解包后手动浏览。".to_string()
    })?;
    std::fs::create_dir_all(out_dir).map_err(|e| format!("创建解包目录失败: {e}"))?;
    match fmt {
        "Xp3" => xp3_extract(archive, out_dir),
        "Pfs" => pfs_extract(archive, out_dir),
        "EscArc" => esc_extract(archive, out_dir),
        "Nsa" => nsa_extract(archive, out_dir),
        "Pac" => pac_extract(archive, out_dir),
        _ => Err("不支持的格式".into()),
    }?;
    // 自动归类：把解出的文件物理整理到分类子文件夹，再按磁盘实际布局返回
    sort_extracted(out_dir)?;
    Ok(list_extracted(out_dir))
}

/// NeXAS PAC 归档解包。
fn pac_extract(archive: &Path, out_dir: &Path) -> Result<Vec<AssetEntry>, String> {
    let mut out = Vec::new();
    for (name, bytes) in crate::pac::extract(archive)? {
        let rel = sanitize_rel(&name.replace('\\', "/"))
            .ok_or_else(|| format!("非法条目名：{name}"))?;
        let target = out_dir.join(&rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&target, &bytes).map_err(|e| format!("写入失败: {e}"))?;
                let category = classify_asset(&rel).to_string();
        out.push(AssetEntry {
            rel,
            abs_path: String::new(),
            size: bytes.len() as i64,
            category,
        });
    }
    if out.is_empty() {
        return Err("没有解出任何文件".into());
    }
    Ok(out)
}

/// NScripter NSA 归档解包。
fn nsa_extract(archive: &Path, out_dir: &Path) -> Result<Vec<AssetEntry>, String> {
    let mut out = Vec::new();
    for (name, bytes) in crate::nsa::extract(archive)? {
        let rel = sanitize_rel(&name.replace('\\', "/"))
            .ok_or_else(|| format!("非法条目名：{name}"))?;
        let target = out_dir.join(&rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&target, &bytes).map_err(|e| format!("写入失败: {e}"))?;
                let category = classify_asset(&rel).to_string();
        out.push(AssetEntry {
            rel,
            abs_path: String::new(),
            size: bytes.len() as i64,
            category,
        });
    }
    if out.is_empty() {
        return Err("没有解出任何文件".into());
    }
    Ok(out)
}

/// ESCude ESC-ARC1/2 (.bin) 解包。
fn esc_extract(archive: &Path, out_dir: &Path) -> Result<Vec<AssetEntry>, String> {
    let mut out = Vec::new();
    for (name, bytes) in crate::esc::extract(archive)? {
        let rel = sanitize_rel(&name.replace('\\', "/"))
            .ok_or_else(|| format!("非法条目名：{name}"))?;
        let target = out_dir.join(&rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&target, &bytes).map_err(|e| format!("写入失败: {e}"))?;
                let category = classify_asset(&rel).to_string();
        out.push(AssetEntry {
            rel,
            abs_path: String::new(),
            size: bytes.len() as i64,
            category,
        });
    }
    if out.is_empty() {
        return Err("没有解出任何文件".into());
    }
    Ok(out)
}

/// 用外部工具解包（约定命令行：`<工具> <压缩包> <输出目录>`）。
/// 兼容 GARbro 控制台版 / GalArc / arc_unpacker 这类「传文件+输出目录」的工具。
pub fn extract_external(archive: &Path, out_dir: &Path, tool: &str) -> Result<Vec<AssetEntry>, String> {
    std::fs::create_dir_all(out_dir).map_err(|e| format!("创建输出目录失败: {e}"))?;
    let status = std::process::Command::new(tool)
        .arg(archive)
        .arg(out_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("运行外部解包工具失败: {e}"))?;
    if !status.success() {
        return Err(format!(
            "外部解包工具退出码 {}；请确认该工具支持此格式、命令行与此约定一致。",
            status.code().unwrap_or(-1)
        ));
    }
    sort_extracted(out_dir)?;
    let list = list_extracted(out_dir);
    if list.is_empty() {
        return Err("外部工具没有产生输出文件，请检查工具路径与兼容性".into());
    }
    Ok(list)
}

// ---------------- XP3 ----------------

fn xp3_extract(archive: &Path, out_dir: &Path) -> Result<Vec<AssetEntry>, String> {
    xp3_runtime().block_on(async {
        use tokio::io::AsyncReadExt;

        let file = tokio::fs::File::open(archive)
            .await
            .map_err(|e| format!("打开资源包失败: {e}"))?;
        let mut ar = xp3::read::XP3Archive::open(tokio::io::BufReader::new(file))
            .await
            .map_err(|e| format!("解析 XP3 失败: {e}"))?;

        let metas: Vec<(String, u64, bool)> = ar
            .entries()
            .iter()
            .map(|e| (e.name.clone(), e.size, e.protected))
            .collect();

        let mut out = Vec::new();
        let mut wrote = 0usize;
        for (idx, (name, size, protected)) in metas.iter().enumerate() {
            if name.ends_with('/') || name.is_empty() {
                continue;
            }
            if *protected {
                // 受保护条目跳过提示（不提供绕过）
                out.push(AssetEntry {
                    rel: name.clone(),
                    abs_path: String::new(),
                    size: *size as i64,
                    category: "其他".into(),
                });
                continue;
            }
            let rel = match sanitize_rel(name) {
                Some(r) => r,
                None => continue, // 路径穿越等非法路径
            };
            let target = out_dir.join(&rel);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
            }
            let mut f = ar
                .by_index(idx)
                .await
                .ok_or("索引越界")?
                .map_err(|e| format!("读取条目失败: {e}"))?;
            let mut buf = Vec::with_capacity((*size).min(64 << 20) as usize);
            f.read_to_end(&mut buf).await.map_err(|e| format!("解压失败: {e}"))?;
            std::fs::write(&target, &buf).map_err(|e| format!("写入失败: {e}"))?;
            wrote += 1;
            let category = classify_asset(&rel).to_string();
            out.push(AssetEntry {
                rel,
                abs_path: String::new(),
                size: buf.len() as i64,
                category,
            });
        }
        if wrote == 0 {
            return Err("没有解出任何文件（包可能受保护或为空）".into());
        }
        Ok(out)
    })
}

fn xp3_runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("创建 tokio 运行时失败")
    })
}

// ---------------- PFS (Artemis) ----------------

fn pfs_extract(archive: &Path, out_dir: &Path) -> Result<Vec<AssetEntry>, String> {
    let buf = std::fs::read(archive).map_err(|e| format!("读取 .pfs 失败: {e}"))?;
    let n = buf.len();
    if n < 12 {
        return Err("pfs 文件过小".into());
    }
    // 头部：magic(3) + index_size(4)；index 区域从 7 开始
    let index_size = u32le(&buf, 3) as usize;
    if 7 + index_size > n {
        return Err("pfs 索引越界".into());
    }
    let index_region = &buf[7..7 + index_size];
    use sha1::{Digest, Sha1};
    let key = Sha1::digest(index_region);

    let file_count = u32le(&buf, 7) as usize;
    let mut entry = 11usize;
    let mut list = Vec::new();
    let mut wrote = 0usize;
    for _ in 0..file_count {
        if entry + 4 > 7 + index_size {
            return Err("pfs 条目越界".into());
        }
        let name_len = u32le(&buf, entry) as usize;
        let name_start = entry + 4;
        if name_start + name_len > 7 + index_size {
            return Err("pfs 文件名越界".into());
        }
        let name = String::from_utf8_lossy(&buf[name_start..name_start + name_len]).into_owned();
        let data_off_off = name_start + name_len + 4; // +separator(4)
        let size_off = data_off_off + 4;
        let data_offset = u32le(&buf, data_off_off) as usize;
        let file_size = u32le(&buf, size_off) as usize;
        entry = size_off + 4;

        if data_offset + file_size > n {
            continue; // 防御性跳过越界数据
        }
        let rel = match sanitize_rel(&name) {
            Some(r) => r,
            None => continue,
        };
        // XOR 解密：每个文件独立从字节 0 起，密钥周期重复
        let mut data = buf[data_offset..data_offset + file_size].to_vec();
        for (i, b) in data.iter_mut().enumerate() {
            *b ^= key[i % key.len()];
        }
        let target = out_dir.join(&rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::write(&target, &data).map_err(|e| format!("写入失败: {e}"))?;
        wrote += 1;
        let category = classify_asset(&rel).to_string();
        list.push(AssetEntry {
            rel,
            abs_path: String::new(),
            size: file_size as i64,
            category,
        });
    }
    if wrote == 0 {
        return Err("没有解出任何文件（可能是受保护/未知加密变体）".into());
    }
    Ok(list)
}

fn u32le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

// ---------------- 通用 ----------------

/// 拒绝路径穿越/绝对路径，返回安全相对路径。
pub fn sanitize_rel(raw: &str) -> Option<String> {
    let norm = raw.replace('\\', "/");
    let norm = norm.trim_start_matches("./").to_string();
    if norm.starts_with('/') {
        return None;
    }
    if norm.as_bytes().get(1) == Some(&b':') {
        return None;
    }
    if norm.split('/').any(|p| p == "..") {
        return None;
    }
    if norm.is_empty() {
        return None;
    }
    Some(norm)
}

/// 解包文件的分类名（也是物理整理时的子文件夹名）。
/// 注意：`classify_asset` 必须能识别这些名字本身，保证「整理后再扫描」分类不漂移。
pub const CATEGORIES: &[&str] = &[
    "立绘", "表情", "CG", "背景", "界面", "其他图片",
    "语音", "BGM", "音效", "其他音频",
    "视频", "脚本", "文档", "数据", "其他",
];

/// 读取某文件的分类。若相对路径首段已是已知分类名（物理整理后的布局），直接用；
/// 否则回退到按文件名/路径启发式归类。
pub fn category_from_rel(rel: &str) -> &'static str {
    let first = rel.split('/').next().unwrap_or("");
    CATEGORIES
        .iter()
        .find(|c| **c == first)
        .copied()
        .unwrap_or_else(|| classify_asset(rel))
}

/// 给解包出的文件归类，供画廊按类浏览，也用于解包后物理整理到分类子文件夹。
/// 覆盖常见引擎惯例：krkr（st/ev/bg/voice/bgm…）、NScripter（nscript）、
/// NeXAS（PGD/PAC 图片、v 开头语音）、通用日英文目录名等。
pub fn classify_asset(rel: &str) -> &'static str {
    let low = rel.to_ascii_lowercase();
    let ext = low.rsplit('.').next().unwrap_or("");
    // 前导斜杠让“/ev/”这类目录段匹配能命中路径首段
    let p = format!("/{low}");
    // 文件名主干（去扩展名），用于 v001 这类语音编号判断
    let stem = low.rsplit('/').next().unwrap_or("");
    let stem_noext = stem.split('.').next().unwrap_or("");
    let v_voice = stem_noext.len() > 1
        && stem_noext.starts_with('v')
        && stem_noext[1..].chars().all(|c| c.is_ascii_digit());

    let is_img = matches!(
        ext,
        "png" | "jpg" | "jpeg" | "bmp" | "webp" | "gif" | "tlg" | "tga" | "psd" | "pgd" | "pdt" | "qnt" | "spb"
    );
    let is_aud = matches!(ext, "ogg" | "wav" | "mp3" | "m4a" | "aac" | "mp2" | "opus" | "wma" | "flac" | "ac3");
    let is_vid = matches!(ext, "avi" | "mp4" | "mkv" | "wmv" | "amv" | "webm" | "mpg" | "mpeg" | "mov" | "ogv");
    let is_script_ext = matches!(ext, "ks" | "tjs" | "js" | "nsc" | "nscript" | "scr" | "asc" | "amts" | "asm");
    let is_doc_ext = matches!(ext, "txt" | "html" | "htm" | "pdf" | "md" | "chm" | "doc" | "docx" | "xls" | "xlsx");
    let is_data_ext = matches!(
        ext,
        "bin" | "dat" | "ini" | "cfg" | "conf" | "json" | "xml" | "sav" | "edb" | "db" | "exe" | "dll" | "sys"
    );
    let script_like = p.contains("nscript") || p.contains("script") || p.contains("剧本") || p.contains("シナリオ") || p.contains("scenario");

    if is_img {
        if p.contains("/st/")
            || p.contains("/char")
            || p.contains("立ち絵")
            || p.contains("立绘")
            || p.contains("sprite")
            || p.contains("figure")
            || p.contains("standing")
            || p.contains("_stand")
            || p.contains("全身")
        {
            "立绘"
        } else if p.contains("/face/") || p.contains("_face") || p.contains("表情") || p.contains("顔") {
            "表情"
        } else if p.contains("/ev/")
            || p.contains("/cg/")
            || p.contains("_ev")
            || p.contains("event")
            || p.contains("事件")
            || p.contains("特典")
            || p.contains("_eve")
            || p.contains("hscene")
        {
            "CG"
        } else if p.contains("/bg/")
            || p.contains("/back/")
            || p.contains("背景")
            || p.contains("background")
            || p.contains("_bg")
            || p.contains("haikei")
        {
            "背景"
        } else if p.contains("/system")
            || p.contains("/sys/")
            || p.contains("/icon/")
            || p.contains("/ui/")
            || p.contains("_ui")
            || p.contains("_icon")
            || p.contains("图标")
            || p.contains("アイコン")
            || p.contains("interface")
            || p.contains("menu")
            || p.contains("メニュー")
            || p.contains("界面")
        {
            "界面"
        } else {
            "其他图片"
        }
    } else if is_vid {
        "视频"
    } else if is_aud {
        if p.contains("/voice")
            || p.contains("/vo/")
            || p.contains("_vo")
            || p.contains("voice")
            || p.contains("语音")
            || p.contains("ヴォイス")
            || p.contains("音声")
            || v_voice
        {
            "语音"
        } else if p.contains("/bgm") || p.contains("音楽") || p.contains("音乐") || p.contains("music")
            || p.contains("_bgm") || p.contains("theme") || p.contains("ost")
        {
            "BGM"
        } else if p.contains("/se/") || p.contains("_se") || p.contains("sound") || p.contains("sfx")
            || p.contains("/snd/") || p.contains("效果音") || p.contains("効果音") || p.contains("音效")
        {
            "音效"
        } else {
            "其他音频"
        }
    } else if is_script_ext || (script_like && matches!(ext, "txt" | "dat")) {
        "脚本"
    } else if is_doc_ext {
        "文档"
    } else if is_data_ext {
        "数据"
    } else {
        "其他"
    }
}

/// 将解包目录下的文件物理整理到「<分类>」子文件夹（保持各自的相对路径），返回移动数。
/// 顶层已是分类名（已整理过）的文件跳过；整理后清掉留下的空目录。
pub fn sort_extracted(out_dir: &Path) -> Result<usize, String> {
    let mut files: Vec<std::path::PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(out_dir).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(out_dir)
            .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
            .unwrap_or_default();
        let first = rel.split('/').next().unwrap_or("");
        if CATEGORIES.contains(&first) {
            continue; // 已在分类子文件夹里
        }
        files.push(entry.path().to_path_buf());
    }
    let mut moved = 0usize;
    for src in &files {
        let rel = src
            .strip_prefix(out_dir)
            .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
            .unwrap_or_default();
        let cat = classify_asset(&rel);
        let dest = out_dir.join(cat).join(&rel);
        if dest == *src {
            continue;
        }
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建分类目录失败: {e}"))?;
        }
        std::fs::rename(src, &dest).map_err(|e| format!("归类移动失败 {rel}: {e}"))?;
        moved += 1;
    }
    prune_empty_dirs(out_dir);
    Ok(moved)
}

/// 删除整理后留下的空目录（分类目录本身保留）。
fn prune_empty_dirs(root: &Path) {
    let mut dirs: Vec<std::path::PathBuf> = walkdir::WalkDir::new(root)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_dir())
        .map(|e| e.path().to_path_buf())
        .collect();
    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for d in dirs {
        let rel = d
            .strip_prefix(root)
            .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
            .unwrap_or_default();
        let first = rel.split('/').next().unwrap_or("");
        if CATEGORIES.contains(&first) {
            continue;
        }
        let empty = std::fs::read_dir(&d).map(|mut r| r.next().is_none()).unwrap_or(false);
        if empty {
            let _ = std::fs::remove_dir(&d);
        }
    }
}

/// 某资源包已解包的条目数（看缓存目录下的 <stem>/）。
pub fn extracted_entries_count(cache_root: &Path, archive_rel: &str) -> usize {
    let stem = archive_stem(archive_rel);
    count_files(&cache_root.join(stem))
}

fn archive_stem(rel: &str) -> String {
    Path::new(rel)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| rel.to_string())
}

pub fn count_files(dir: &Path) -> usize {
    if !dir.is_dir() {
        return 0;
    }
    walkdir::WalkDir::new(dir)
        .into_iter()
        .filter(|e| e.is_ok() && e.as_ref().unwrap().file_type().is_file())
        .count()
}

/// 遍历缓存目录，返回全部已解包条目。
pub fn list_extracted(cache_root: &Path) -> Vec<AssetEntry> {
    let mut out = Vec::new();
    if !cache_root.is_dir() {
        return out;
    }
    for entry in walkdir::WalkDir::new(cache_root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(cache_root)
            .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
            .unwrap_or_default();
        let size = entry.metadata().map(|m| m.len() as i64).unwrap_or(0);
        let category = category_from_rel(&rel).to_string();
        out.push(AssetEntry {
            rel,
            abs_path: entry.path().to_string_lossy().into_owned(),
            size,
            category,
        });
    }
    out
}

/// 批量导出某分类（None=全部）的文件到 dst，保持相对结构。
pub fn export_matching(cache_root: &Path, dst: &Path, category: Option<&str>) -> Result<usize, String> {
    let mut count = 0usize;
    for entry in walkdir::WalkDir::new(cache_root).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = entry
            .path()
            .strip_prefix(cache_root)
            .map(|p| p.to_string_lossy().into_owned().replace('\\', "/"))
            .unwrap_or_default();
        if let Some(cat) = category {
            if category_from_rel(&rel) != cat {
                continue;
            }
        }
        let dest = dst.join(&rel);
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        }
        std::fs::copy(entry.path(), &dest).map_err(|e| format!("复制失败: {e}"))?;
        count += 1;
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造一个精简但符合规范的 pf8 包（数据用同一密钥加密）。
    fn build_pfs(files: &[(&str, &[u8])]) -> Vec<u8> {
        use sha1::{Digest, Sha1};

        let mut idx: Vec<u8> = Vec::new();
        idx.extend_from_slice(&(files.len() as u32).to_le_bytes());

        // 先写条目（偏移留 0 占位），记下每个 offset 字段在 idx 里的位置
        let mut off_fields: Vec<(usize, usize)> = Vec::new(); // (idx_pos, data_len)
        let mut entry_pos = 4usize;
        for (name, data) in files {
            let nb = name.as_bytes();
            idx.extend_from_slice(&(nb.len() as u32).to_le_bytes());
            idx.extend_from_slice(nb);
            idx.extend_from_slice(&[0u8; 4]); // separator
            let off_pos = entry_pos + 4 + nb.len() + 4;
            off_fields.push((off_pos, data.len()));
            idx.extend_from_slice(&[0u8; 4]); // 占位 data_offset
            idx.extend_from_slice(&(data.len() as u32).to_le_bytes());
            // 条目步长 = name_len(4) + name + sep(4) + off(4) + size(4) = N + 16
            entry_pos += nb.len() + 16;
        }
        // fs_count + size offsets + padding + index_end（解析器用不到，但让 sha1 覆盖更真实）
        let fs_count = files.len() + 1;
        idx.extend_from_slice(&(fs_count as u32).to_le_bytes());
        for _ in 0..fs_count {
            idx.extend_from_slice(&[0u8; 8]);
        }
        idx.extend_from_slice(&[0u8; 8]);
        idx.extend_from_slice(&[0u8; 4]);

        let index_size = idx.len();

        // 回填数据偏移（数据区紧接着索引）——必须在算密钥之前，真实格式哈希的是最终索引
        let mut cursor = 7 + index_size;
        for (idx_pos, dlen) in off_fields {
            idx[idx_pos..idx_pos + 4].copy_from_slice(&(cursor as u32).to_le_bytes());
            cursor += dlen;
        }

        // 密钥 = SHA-1(最终索引区域)
        let key = Sha1::digest(&idx);

        let mut whole = Vec::new();
        whole.extend_from_slice(b"pf8");
        whole.extend_from_slice(&(index_size as u32).to_le_bytes());
        whole.extend_from_slice(&idx);
        for (_, data) in files {
            let mut enc = data.to_vec();
            for (i, b) in enc.iter_mut().enumerate() {
                *b ^= key[i % key.len()];
            }
            whole.extend_from_slice(&enc);
        }
        whole
    }

    #[test]
    fn pfs_roundtrip() {
        let files: Vec<(&str, &[u8])> = vec![
            ("ev/cg01.png", b"\x89PNG-DATACG001"),
            ("voice/x01.ogg", b"OggS-VOICE-DATA1234"),
            ("立ち絵/h1.png", b"PNG-SPRITENAME"),
        ];
        let blob = build_pfs(&files);

        let tmp = std::env::temp_dir();
        let path = tmp.join(format!("galasset-test-{}.pfs", std::process::id()));
        let out = tmp.join(format!("galasset-out-{}", std::process::id()));
        std::fs::write(&path, &blob).unwrap();
        let _ = std::fs::remove_dir_all(&out);

        let list = extract(&path, &out).unwrap();
        assert_eq!(list.len(), 3);

        // 解密内容一致（且已物理整理到分类子文件夹）
        let cg = std::fs::read(out.join("CG/ev/cg01.png")).unwrap();
        assert_eq!(cg, b"\x89PNG-DATACG001");
        let voice = std::fs::read(out.join("语音/voice/x01.ogg")).unwrap();
        assert_eq!(voice, b"OggS-VOICE-DATA1234");
        let sprite = std::fs::read(out.join("立绘/立ち絵/h1.png")).unwrap();
        assert_eq!(sprite, b"PNG-SPRITENAME");
        // 空目录已清理
        assert!(!out.join("ev").exists());

        // 分类正确（返回列表按磁盘实际布局给出，顺序不保证）
        let mut cat: Vec<&str> = list.iter().map(|a| category_from_rel(&a.rel)).collect();
        cat.sort();
        assert_eq!(cat, vec!["CG", "立绘", "语音"]);

        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn classify_rules() {
        // 图片：立绘 / 表情 / CG / 背景 / 界面 / 其他图片
        assert_eq!(classify_asset("st/hero.png"), "立绘");
        assert_eq!(classify_asset("立ち絵/h1.png"), "立绘");
        assert_eq!(classify_asset("face/f01.png"), "表情");
        assert_eq!(classify_asset("ev/EV001A.png"), "CG");
        assert_eq!(classify_asset("bg/BG001.jpg"), "背景");
        assert_eq!(classify_asset("ui/icon_ok.png"), "界面");
        assert_eq!(classify_asset("common_art.tlg"), "其他图片");
        // 音频：语音（含 v 开头编号）/ BGM / 音效 / 其他音频
        assert_eq!(classify_asset("voice/v001.wav"), "语音");
        assert_eq!(classify_asset("v002.wav"), "语音");
        assert_eq!(classify_asset("bgm02.ogg"), "BGM");
        assert_eq!(classify_asset("se/click.wav"), "音效");
        assert_eq!(classify_asset("common.wav"), "其他音频");
        // 视频 / 脚本 / 文档 / 数据 / 其他
        assert_eq!(classify_asset("movie/op.mov"), "视频");
        assert_eq!(classify_asset("nscript.dat"), "脚本");
        assert_eq!(classify_asset("main.ks"), "脚本");
        assert_eq!(classify_asset("readme.txt"), "文档");
        assert_eq!(classify_asset("save/sav01.sav"), "数据");
        assert_eq!(classify_asset("misc.zzz"), "其他");
    }

    #[test]
    fn sort_extracted_is_idempotent() {
        let tmp = std::env::temp_dir();
        let out = tmp.join(format!("galasset-sort-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&out);
        for p in ["ev/a.png", "voice/b.ogg", "readme.txt", "misc.zzz"] {
            let f = out.join(p);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(&f, b"x").unwrap();
        }

        let moved = sort_extracted(&out).unwrap();
        assert_eq!(moved, 4);
        assert!(out.join("CG/ev/a.png").exists());
        assert!(out.join("语音/voice/b.ogg").exists());
        assert!(out.join("文档/readme.txt").exists());
        assert!(out.join("其他/misc.zzz").exists());
        // 空目录被清理
        assert!(!out.join("ev").exists());
        assert!(!out.join("voice").exists());
        // 二次整理幂等
        assert_eq!(sort_extracted(&out).unwrap(), 0);

        let _ = std::fs::remove_dir_all(&out);
    }

    #[test]
    fn path_traversal_rejected() {
        assert_eq!(sanitize_rel("../../evil.exe"), None);
        assert_eq!(sanitize_rel("C:/evil.exe"), None);
        assert_eq!(sanitize_rel("/abs.png"), None);
        assert_eq!(sanitize_rel("a/b.png"), Some("a/b.png".into()));
    }
}
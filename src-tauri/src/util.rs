//! 通用工具函数。

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::{DirListing, FsEntry};

/// 当前 Unix 时间戳（秒）。
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 归一化路径，用于去重比较：优先 canonicalize（解析 junction/符号链接、
/// 还原磁盘真实大小写），失败则退化为字符串级归一化（统一分隔符、去尾部斜杠）。
/// Windows 上折叠大小写（NTFS 大小写不敏感，避免同一目录因大小写不同被重复导入）。
pub fn norm_path(path: &str) -> String {
    let t = path.trim();
    if t.is_empty() {
        return String::new();
    }
    let resolved = std::fs::canonicalize(t)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| t.to_string());
    #[cfg(windows)]
    {
        let mut s = resolved.replace('/', "\\");
        while s.len() > 3 && s.ends_with('\\') {
            s.pop();
        }
        s.to_lowercase()
    }
    #[cfg(not(windows))]
    {
        resolved
    }
}

/// child 是否等于 parent，或是 parent 的子目录（两者都应先 norm_path）。
/// 按路径组件边界比较，避免 "Game" 误判为 "Game2" 的祖先。
pub fn is_same_or_descendant(child: &str, parent: &str) -> bool {
    if child == parent {
        return true;
    }
    #[cfg(windows)]
    let sep = "\\";
    #[cfg(not(windows))]
    let sep = "/";
    let p = parent.trim_end_matches(['/', '\\']);
    child.starts_with(&format!("{p}{sep}"))
}

/// 把秒格式化为 "xh ym" 形式。
#[allow(dead_code)]
pub fn fmt_duration(secs: i64) -> String {
    if secs < 60 {
        return format!("{secs}s");
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}

/// 把本地图片读成 data URI，供前端 <img> 直接展示。
/// 仅用于封面等小图片，规避本地资产协议的作用域配置问题。
pub fn read_image_data_uri(path: &str) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| format!("读取图片失败: {e}"))?;
    if data.len() > 8 * 1024 * 1024 {
        return Err("图片过大（>8MB）".into());
    }
    let mime = match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        _ => "image/png",
    };
    Ok(format!("data:{mime};base64,{}", encode_base64(&data)))
}

/// 生成封面缩略图并返回 data URI（带磁盘缓存）。
///
/// 封面墙用，避免把整张原图 base64 进内存。缩略图统一编码成 JPEG，
/// 最长边 `max_side`；缓存到 `thumbs_dir`，key = sha1(path|mtime|size|max_side)，
/// 源文件一旦变化（mtime/size 不同）自然生成新 key，旧缓存成为孤儿文件。
pub fn read_cover_thumb(path: &str, thumbs_dir: &Path, max_side: u32) -> Result<String, String> {
    let src = Path::new(path);
    let meta = std::fs::metadata(src).map_err(|e| format!("读取封面失败: {e}"))?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let key = format!("{path}|{mtime}|{}|{max_side}", meta.len());
    let cache = thumbs_dir.join(format!("{}.jpg", sha1_hex(&key)));

    if cache.is_file() {
        if let Ok(data) = std::fs::read(&cache) {
            return Ok(format!("data:image/jpeg;base64,{}", encode_base64(&data)));
        }
    }

    let img = image::open(src).map_err(|e| format!("解码封面失败: {e}"))?;
    let thumb = img.thumbnail(max_side, max_side);
    let mut buf = std::io::Cursor::new(Vec::new());
    thumb
        .write_to(&mut buf, image::ImageFormat::Jpeg)
        .map_err(|e| format!("生成缩略图失败: {e}"))?;
    let bytes = buf.into_inner();

    if let Some(parent) = cache.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(&cache, &bytes);
    Ok(format!("data:image/jpeg;base64,{}", encode_base64(&bytes)))
}

fn sha1_hex(s: &str) -> String {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn encode_base64(bytes: &[u8]) -> String {
    const TBL: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let a = chunk[0] as u32;
        let b = chunk.get(1).copied().unwrap_or(0) as u32;
        let c = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (a << 16) | (b << 8) | c;
        out.push(TBL[((n >> 18) & 63) as usize] as char);
        out.push(TBL[((n >> 12) & 63) as usize] as char);
        if chunk.len() > 1 {
            out.push(TBL[((n >> 6) & 63) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TBL[(n & 63) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// 大小写不敏感的子串判断。
#[allow(dead_code)]
pub fn contains_ci(hay: &str, needle: &str) -> bool {
    hay.to_ascii_lowercase().contains(&needle.to_ascii_lowercase())
}

/// 把某个目录/文件在 Windows 上设置（或取消）隐藏和系统属性。
#[cfg(windows)]
pub fn set_hidden_attr(path: &str, hidden: bool) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        FILE_ATTRIBUTE_HIDDEN, FILE_ATTRIBUTE_NORMAL, FILE_ATTRIBUTE_SYSTEM, GetFileAttributesW,
        INVALID_FILE_ATTRIBUTES, SetFileAttributesW,
    };

    let wide: Vec<u16> =
        std::ffi::OsStr::new(path).encode_wide().chain(std::iter::once(0)).collect();
    let mut attrs = unsafe { GetFileAttributesW(wide.as_ptr()) };
    if attrs == INVALID_FILE_ATTRIBUTES || attrs == 0 {
        return Err(format!("无法读取目标属性（路径不存在？）: {path}"));
    }
    let hidden_bits = FILE_ATTRIBUTE_HIDDEN | FILE_ATTRIBUTE_SYSTEM;
    if hidden {
        attrs |= hidden_bits;
    } else {
        attrs &= !hidden_bits;
    }
    if attrs & FILE_ATTRIBUTE_NORMAL != 0 {
        attrs &= !FILE_ATTRIBUTE_NORMAL;
    }
    let ok = unsafe { SetFileAttributesW(wide.as_ptr(), attrs) };
    if ok == 0 {
        return Err(format!("设置隐藏属性失败: {path}"));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn set_hidden_attr(_path: &str, _hidden: bool) -> Result<(), String> {
    Err("当前平台不支持系统级隐藏".into())
}

/// 列出一个目录的内容（子目录 + 文件），供内置文件选择器用。
/// `exts` 指定时只返回这些扩展名的文件（如 ["exe"]，不设文件数上限）；
/// 不指定时返回所有文件但最多 `FILE_CAP` 个，超出部分通过 `truncated` 告知前端。
pub fn list_dir(path: &str, exts: Option<&[String]>) -> Result<DirListing, String> {
    const FILE_CAP: usize = 500;
    let rd = std::fs::read_dir(path).map_err(|e| format!("无法读取目录: {e}"))?;
    let wanted: Option<Vec<String>> = exts.map(|v| {
        v.iter()
            .map(|e| e.trim().trim_start_matches('.').to_ascii_lowercase())
            .filter(|e| !e.is_empty())
            .collect()
    });

    let mut entries = Vec::new();
    let mut files_seen = 0usize;
    let mut truncated = false;
    for e in rd.filter_map(|e| e.ok()) {
        let is_dir = e.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let name = e.file_name().to_string_lossy().into_owned();

        if !is_dir {
            if let Some(w) = &wanted {
                let ext = name.rsplit('.').next().map(|s| s.to_ascii_lowercase()).unwrap_or_default();
                if !w.iter().any(|x| *x == ext) {
                    continue;
                }
            } else {
                if files_seen >= FILE_CAP {
                    truncated = true;
                    continue;
                }
                files_seen += 1;
            }
        }

        let meta = e.metadata().ok();
        let size = if is_dir {
            0
        } else {
            meta.as_ref().map(|m| m.len() as i64).unwrap_or(0)
        };
        let modified = meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        entries.push(FsEntry {
            name,
            is_dir,
            size,
            modified,
        });
    }
    Ok(DirListing { entries, truncated })
}

/// 列出可用的磁盘根目录（如 "C:\\"）。仅 Windows 有意义。
#[cfg(windows)]
pub fn list_drives() -> Vec<String> {
    // 跳过软驱 A:/B:，避免空软驱上 metadata 卡顿；从 C 开始枚举已存在的盘符。
    let mut out = Vec::new();
    for c in 'C'..='Z' {
        let root = format!("{c}:\\");
        if Path::new(&root).exists() {
            out.push(root);
        }
    }
    out
}

#[cfg(not(windows))]
pub fn list_drives() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_standard() {
        assert_eq!(encode_base64(b"hello world"), "aGVsbG8gd29ybGQ=");
        // PNG 魔数
        assert_eq!(encode_base64(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]), "iVBORw0KGgo=");
        // 空
        assert_eq!(encode_base64(b""), "");
        // 长度 1 / 2 / 3 的边界
        assert_eq!(encode_base64(b"a"), "YQ==");
        assert_eq!(encode_base64(b"ab"), "YWI=");
        assert_eq!(encode_base64(b"abc"), "YWJj");
    }

    #[test]
    fn data_uri_prefix_and_roundtrip() {
        let path = std::env::temp_dir().join("galtest-util.bin");
        std::fs::write(&path, b"\x89PNG-fake-data").unwrap();
        let uri = read_image_data_uri(&path.to_string_lossy()).unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
        let _ = std::fs::remove_file(&path);
    }
}
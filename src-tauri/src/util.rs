//! 通用工具函数。

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// 当前 Unix 时间戳（秒）。
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
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
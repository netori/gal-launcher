//! NeXAS PAC（带 "PAC " 头的变体）归档解包（.pac）。
//!
//! 结构与规范见 GARbro NeXAS，但本变体（神明的选择 / Dimension Totsu Lovers 使用）的
//! 索引是：每 40 字节一个条目 = 文件名(固定宽，前 12 字节) + 20 字节填0 + u32 大小 + u32 偏移。
//! 数据区紧接名称表，pack 类型为 0（无压缩）。
//! 自校验：`offset[0] == 名称表起点 + count*40`，用它兜底定位索引起点（不同游戏首部结构略有差异）。

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const SLOT: usize = 40;

struct Entry {
    name: String,
    size: u64,
    offset: u64,
}

/// 在 [12..limit) 扫描可以当索引起点的位置：offset[0] == p + count*SLOT 即为自洽起点。
fn locate_index(data: &[u8], filesize: u64, count: usize) -> Option<usize> {
    let limit = data.len().min(4 << 20);
    let last = limit.saturating_sub(SLOT + 4);
    for p in (12..last).step_by(1) {
        // 名称表结束时 = 第一个数据文件的绝对偏移
        let off0 = u32le(data, p + 36) as u64;
        let end = p as u64 + (count as u64 * SLOT as u64);
        if off0 != end {
            continue;
        }
        if off0 >= filesize || off0 < 12 {
            continue;
        }
        // 名称首字节应是可打印 ASCII
        let c = data[p];
        if !(0x20..=0x7e).contains(&c) {
            continue;
        }
        return Some(p);
    }
    None
}

fn valid_slot(data: &[u8], p: usize, filesize: u64) -> bool {
    for j in 0..SLOT {
        if p + j >= data.len() {
            return false;
        }
    }
    let c = data[p];
    if !(0x20..=0x7e).contains(&c) {
        return false;
    }
    let off = u32le(data, p + 36) as u64;
    let size = u32le(data, p + 32) as u64;
    off != 0 && size > 0 && off + size <= filesize
}

/// 解包整个 PAC，返回 (相对路径, 内容字节) 列表。
pub fn extract(path: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("打开 PAC 失败: {e}"))?;
    let filesize = f.metadata().map(|m| m.len()).unwrap_or(0);

    let mut head = [0u8; 16];
    f.read_exact(&mut head).map_err(|e| format!("读取头部失败: {e}"))?;
    if &head[0..3] != b"PAC" {
        return Err("不是 PAC 归档".into());
    }
    let count = u32le(&head, 8) as usize;
    if count == 0 || count > 1_000_000 {
        return Err("PAC 头非法".into());
    }
    // 包类型：为 0 时数据原样；非 0 需要解压（暂不支持）
    let pack_type = u32le(&head, 4);

    // 读前 4MB 用于定位名称表
    let probe_len = (filesize.min(4 << 20)) as usize;
    let mut buf = vec![0u8; probe_len];
    f.seek(SeekFrom::Start(0)).ok();
    f.read_exact(&mut buf).map_err(|e| format!("读取探针失败: {e}"))?;

    let start = locate_index(&buf, filesize, count)
        .ok_or_else(|| "无法定位 PAC 名称表".to_string())?;

    // 校验并解析 count 个条目
    let mut entries = Vec::with_capacity(count);
    for i in 0..count {
        let p = start + i * SLOT;
        if !valid_slot(&buf, p, filesize) {
            return Err(format!("PAC 第 {i} 个条目非法"));
        }
        let name_field = &buf[p..p + 32];
        let name_end = name_field.iter().position(|&b| b == 0).unwrap_or(32);
        let name = String::from_utf8_lossy(&name_field[..name_end]).into_owned();
        entries.push(Entry {
            name,
            size: u32le(&buf, p + 32) as u64,
            offset: u32le(&buf, p + 36) as u64,
        });
    }

    if pack_type != 0 {
        return Err(format!("此 PAC 使用压缩类型 {pack_type}，内置暂不支持（可用外部工具）"));
    }

    let mut out = Vec::new();
    for e in &entries {
        let mut data = vec![0u8; e.size as usize];
        f.seek(SeekFrom::Start(e.offset)).map_err(|e2| e2.to_string())?;
        f.read_exact(&mut data).map_err(|e2| format!("读取 {} 失败: {e2}", e.name))?;
        out.push((e.name.clone(), data));
    }
    Ok(out)
}

fn u32le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真实文件验证（--ignored）。
    #[test]
    #[ignore]
    fn real_pac() {
        for p in [
            r"F:\game\gal\kami\神明的选择\ev.pac",
            r"F:\game\gal\dimension\Dimension Totsu Lovers!!\st.pac",
        ] {
            println!("===== {p} =====");
            match extract(Path::new(p)) {
                Ok(list) => {
                    println!("解出 = {}", list.len());
                    let mut ok_ge = 0u32;
                    for (n, d) in list.iter().take(6) {
                        let head = if d.len() >= 4 {
                            format!("{:02x} {:02x} {:02x} {:02x}", d[0], d[1], d[2], d[3])
                        } else {
                            "<空>".into()
                        };
                        if d.len() >= 2 && &d[0..2] == b"GE" {
                            ok_ge += 1;
                        }
                        println!("  {n}  ({}B)  head={head}", d.len());
                    }
                    println!(" 前 6 条含 GE 图头: {ok_ge}");
                }
                Err(e) => println!("  ERR {e}"),
            }
        }
    }
}
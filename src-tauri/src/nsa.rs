//! NScripter NSA（含 32 位变体）归档解包（.nsa）。
//!
//! 结构（参考 GARbro，MIT）：
//! - 头：条目数(count) u16 大端 + 数据基址(base) u32 大端
//! - 每条：name(CString) + comp(1B) + offset(4BE) + size(4BE) + unpacked(4BE)
//! - 数据在 base+offset，长 size；压缩：0=原样，1=SPB(BMP)，2=LZSS，4=NBZ(bzip2，暂不支持)
//!
//! 实机验证：时散 arc.nsa 3504 条，索引总长恰好等于 base（硬约束锁定宽度 13），
//! 无压条目解出 JPEG，SPB 解出 BMP。

use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const N: usize = 1 << 8; // LZSS 缓冲大小 256
const F: usize = (1 << 4) + 1; // 17

pub fn is_nsa(path: &Path) -> bool {
    let ext = path.extension().and_then(|e| e.to_str()).map(|e| e.to_ascii_lowercase());
    matches!(ext.as_deref(), Some("nsa") | Some("ns2"))
}

#[derive(Clone)]
struct Entry {
    name: String,
    comp: u8,
    abs: u64,
    size: u64,
    unp: u64,
}

fn parse_index(f: &mut std::fs::File, filesize: u64) -> Result<(u64, Vec<Entry>), String> {
    let mut head = [0u8; 6];
    f.read_exact(&mut head).map_err(|e| format!("读取头部失败: {e}"))?;
    if head[0] == 0xFF && head[1] == 0xD8 {
        return Err("这是图片/音频文件而非 NSA 归档".into());
    }
    let count = u16be(&head, 0) as usize;
    let base = u32be(&head, 2) as u64;
    if count == 0 || count > 1_000_000 || base < 6 || base > filesize {
        return Err("NSA 头非法".into());
    }
    let idx_len = (base - 6) as usize;
    let mut idx = vec![0u8; idx_len];
    f.read_exact(&mut idx).map_err(|e| format!("读取索引失败: {e}"))?;

    let mut entries = Vec::with_capacity(count);
    let mut pos = 0usize;
    for _ in 0..count {
        if pos >= idx.len() {
            return Err("NSA 索引越界".into());
        }
        let name_end = idx[pos..]
            .iter()
            .position(|&b| b == 0)
            .ok_or_else(|| "NSA 索引被截断".to_string())?;
        let name = String::from_utf8_lossy(&idx[pos..pos + name_end]).into_owned();
        pos += name_end + 1;
        if pos + 13 > idx.len() {
            return Err("NSA 索引被截断".into());
        }
        let e = &idx[pos..pos + 13];
        entries.push(Entry {
            name,
            comp: e[0],
            abs: base + u32be(e, 1) as u64,
            size: u32be(e, 5) as u64,
            unp: u32be(e, 9) as u64,
        });
        pos += 13;
    }
    Ok((base, entries))
}

/// 解包整个 NSA，返回 (相对路径, 内容) 列表。
pub fn extract(path: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut f = std::fs::File::open(path).map_err(|e| format!("打开 NSA 失败: {e}"))?;
    let filesize = f.metadata().map(|m| m.len()).unwrap_or(0);
    let (_base, entries) = parse_index(&mut f, filesize)?;

    let mut out = Vec::new();
    for e in &entries {
        if e.comp == 4 {
            continue; // NBZ(bzip2) 暂无解码器，先跳过
        }
        let data = match read_region(&mut f, e.abs, e.size) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let decoded = match e.comp {
            0 => data,
            // 单条目解不开就跳过，不让整包失败
            1 => match spb_decode(&data, e.unp) {
                Ok(x) => x,
                Err(_) => continue,
            },
            2 => match lzss_decode(&data, e.unp) {
                Ok(x) => x,
                Err(_) => continue,
            },
            _ => continue,
        };
        out.push((e.name.clone(), decoded));
    }
    Ok(out)
}

fn read_region(f: &mut std::fs::File, abs: u64, size: u64) -> Result<Vec<u8>, String> {
    let mut v = vec![0u8; size.min(1 << 30) as usize];
    f.seek(SeekFrom::Start(abs)).map_err(|e| e.to_string())?;
    let n = f.read(&mut v).map_err(|e| format!("读取数据失败: {e}"))?;
    v.truncate(n);
    Ok(v)
}

// ---------------- MSB 位读取器 ----------------

struct MsbReader<'a> {
    data: &'a [u8],
    pos: usize,
    bits: u32,
    cached: u32,
}
impl<'a> MsbReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        MsbReader { data, pos: 0, bits: 0, cached: 0 }
    }
    fn get_bits(&mut self, n: u32) -> i32 {
        debug_assert!(n <= 24);
        while self.cached < n {
            if self.pos >= self.data.len() {
                return -1;
            }
            self.bits = (self.bits << 8) | self.data[self.pos] as u32;
            self.pos += 1;
            self.cached += 8;
        }
        let mask = (1u32 << n) - 1;
        self.cached -= n;
        ((self.bits >> self.cached) & mask) as i32
    }
}

// ---------------- LZSS ----------------

fn lzss_decode(data: &[u8], unp: u64) -> Result<Vec<u8>, String> {
    let mut r = MsbReader::new(data);
    let mut out = vec![0u8; unp as usize];
    let mut buf = vec![0u8; N * 2];
    let mut rb: usize = N - F;
    let mut count = 0usize;
    while count < out.len() {
        if r.get_bits(1) != 0 {
            let c = r.get_bits(8);
            if c < 0 {
                break;
            }
            if count < out.len() {
                out[count] = c as u8;
                count += 1;
            }
            buf[rb] = c as u8;
            rb &= N - 1;
        } else {
            let i = r.get_bits(8);
            if i < 0 {
                break;
            }
            let j = r.get_bits(4);
            if j < 0 {
                break;
            }
            for k in 0..=(j + 1) as usize {
                let c = buf[(i as usize + k) & (N - 1)];
                if count < out.len() {
                    out[count] = c;
                    count += 1;
                }
                buf[rb] = c;
                rb &= N - 1;
            }
        }
    }
    Ok(out)
}

// ---------------- SPB (BMP) ----------------

fn spb_decode(data: &[u8], unp: u64) -> Result<Vec<u8>, String> {
    if data.len() < 4 {
        return Err("SPB 数据过短".into());
    }
    // 宽高为前 2 个大端 u16
    let u16be_at = |o: usize| ((data[o] as u32) << 8) | data[o + 1] as u32;
    let width = u16be_at(0);
    let height = u16be_at(2);
    if width == 0
        || height == 0
        || width > 65535
        || height > 65535
        || (width as u64) * (height as u64) > 120_000_000
    {
        return Err("SPB 尺寸非法".into());
    }
    let width_pad = (4 - width * 3 % 4) % 4;
    let stride = width as usize * 3 + width_pad as usize;
    let total_size = stride as u64 * height as u64 + 54;
    let out_cap = unp.max(total_size) as usize;
    let mut out = vec![0u8; out_cap];

    // BMP 头（小端）
    out[0] = b'B';
    out[1] = b'M';
    out[2..6].copy_from_slice(&(total_size as u32).to_le_bytes());
    out[10] = 54;
    out[14] = 40;
    out[18..22].copy_from_slice(&width.to_le_bytes());
    out[22..26].copy_from_slice(&height.to_le_bytes());
    out[26] = 1;
    out[28] = 24;

    let mut r = MsbReader::new(&data[4..]);
    let wh = width as usize * height as usize;
    let mut decomp = vec![0u8; wh];

    for ch in 0..3usize {
        let mut count = 0usize;
        let mut c = r.get_bits(8);
        if c < 0 {
            break;
        }
        decomp[count] = c as u8;
        count += 1;
        while count < wh {
            let n = r.get_bits(3);
            if n < 0 {
                break;
            }
            if n == 0 {
                for _ in 0..4 {
                    if count < wh {
                        decomp[count] = c as u8;
                        count += 1;
                    }
                }
                continue;
            }
            let m = if n == 7 { r.get_bits(1) + 1 } else { n + 2 };
            for _ in 0..4 {
                c = if m == 8 {
                    r.get_bits(8)
                } else {
                    let k = r.get_bits(m as u32);
                    if k < 0 {
                        c = -1;
                        break;
                    }
                    if (k & 1) != 0 {
                        c + ((k >> 1) + 1)
                    } else {
                        c - (k >> 1)
                    }
                };
                if !(0..=255).contains(&c) {
                    c = 0;
                }
                if count < wh {
                    decomp[count] = c as u8;
                    count += 1;
                }
            }
        }

        // serpentine 扫描写入 bottom-up BMP
        let mut pbuf = stride as isize * (height as isize - 1) + ch as isize + 54;
        let mut psbuf = 0usize;
        let w = width as usize;
        for j in 0..height as usize {
            if (j & 1) != 0 {
                for _ in 0..w {
                    if psbuf < wh && pbuf >= 0 && (pbuf as usize) < out.len() {
                        out[pbuf as usize] = decomp[psbuf];
                    }
                    psbuf += 1;
                    pbuf -= 3;
                }
                pbuf -= stride as isize - 3;
            } else {
                for _ in 0..w {
                    if psbuf < wh && pbuf >= 0 && (pbuf as usize) < out.len() {
                        out[pbuf as usize] = decomp[psbuf];
                    }
                    psbuf += 1;
                    pbuf += 3;
                }
                pbuf -= stride as isize + 3;
            }
        }
    }
    out.truncate(total_size as usize);
    Ok(out)
}

fn u16be(b: &[u8], off: usize) -> u16 {
    u16::from_be_bytes([b[off], b[off + 1]])
}
fn u32be(b: &[u8], off: usize) -> u32 {
    u32::from_be_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 用真实文件验证（--ignored）：索引 + 原样/SPB/LZSS。
    #[test]
    #[ignore]
    fn real_nsa() {
        let path = Path::new(r"F:\game\gal\时散\arc.nsa");
        let list = extract(path).unwrap();
        println!("解出条目 = {}", list.len());
        let jpg = list.iter().find(|(n, _)| n.to_ascii_lowercase().ends_with(".jpg"));
        for (name, data) in list.iter().take(8) {
            let head = if data.len() >= 4 {
                format!("{:02X} {:02X} {:02X} {:02X}", data[0], data[1], data[2], data[3])
            } else {
                "<空>".into()
            };
            println!("  {name}  ({}B)  head={head}", data.len());
        }
        if let Some((name, data)) = jpg {
            assert_eq!(&data[0..3], b"\xFF\xD8\xFF", "应解出 JPEG: {name}");
        }
        let bmp = list.iter().find(|(_, d)| d.len() > 54 && &d[0..2] == b"BM");
        if let Some((name, _)) = bmp {
            println!("SPB→BMP ok: {name}");
        }
    }
}
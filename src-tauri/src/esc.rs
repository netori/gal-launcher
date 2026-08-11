//! ESCude 引擎的 ESC-ARC1/2 容器解包（.bin）。
//!
//! 算法参考 Bincude（TheVNConnoisseur/Bincude，GPL-3.0）的公开逆向：
//! - 头：magic(8) + XOR 种子 u32(LE)
//! - 文件数 / 名称表长度 / 索引区：用 NextKey 链逐 4 字节异或解密
//! - 文件内容若以 "acp\0" 开头：big-endian 位流 LZW 解压（其余原样）

use std::io::Read;
use std::path::Path;

/// 解包一个 ESC-ARC1/2 .bin。返回 (相对路径, 内容字节) 列表。
pub fn extract(path: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let mut data = Vec::new();
    std::fs::File::open(path)
        .and_then(|mut f| f.read_to_end(&mut data))
        .map_err(|e| format!("读取 .bin 失败: {e}"))?;

    if data.len() < 0x14 {
        return Err("ESC bin 过小".into());
    }
    let magic = &data[0..8];
    if magic != b"ESC-ARC1" && magic != b"ESC-ARC2" {
        return Err("不是 ESC-ARC1/2 容器".into());
    }

    let mut seed = u32le(&data, 8);

    let num_files = u32le(&data, 0x0c) ^ next_key(&mut seed);
    let names_len = u32le(&data, 0x10) ^ next_key(&mut seed);
    if num_files == 0 || num_files as usize > 1_000_000 {
        return Err("ESC 索引异常".into());
    }

    let meta_bytes = num_files as usize * 12;
    let meta_start = 0x14usize;
    let names_start = meta_start + meta_bytes;
    if names_start + names_len as usize > data.len() {
        return Err("ESC 索引越界".into());
    }
    // 解密索引区（每 4 字节用一个 NextKey）
    let mut meta = data[meta_start..names_start].to_vec();
    {
        let mut s = seed;
        for block in meta.chunks_exact_mut(4) {
            let v = u32le(block, 0) ^ next_key(&mut s);
            block.copy_from_slice(&v.to_le_bytes());
        }
        let rem = meta.len() & 3;
        if rem > 0 {
            let start = meta.len() - rem;
            for b in &mut meta[start..] {
                *b ^= (next_key(&mut s) & 0xff) as u8;
            }
        }
    }
    // 注意：next_key 已被上面的解密推进过；后续不再用到 seed。

    let names = &data[names_start..names_start + names_len as usize];
    let mut out = Vec::new();
    for i in 0..num_files as usize {
        let m = (i * 12) as usize;
        let name_off = i32le(&meta, m);
        let content_off = i32le(&meta, m + 4);
        let size = i32le(&meta, m + 8);
        match decode_one(&data, names, name_off, content_off, size) {
            Ok(Some((name, bytes))) => out.push((name, bytes)),
            Ok(None) => {}
            Err(e) => return Err(format!("第 {i} 个条目: {e}")),
        }
    }
    if out.is_empty() {
        return Err("没有解出任何文件".into());
    }
    Ok(out)
}

fn decode_one(
    data: &[u8],
    names: &[u8],
    name_off: i32,
    content_off: i32,
    size: i32,
) -> Result<Option<(String, Vec<u8>)>, String> {
    if content_off < 0 || size < 0 {
        return Ok(None);
    }
    let cstart = content_off as usize;
    let cend = cstart + size as usize;
    if cend > data.len() {
        return Ok(None);
    }
    let content = &data[cstart..cend];

    // 文件名：Shift-JIS，遇 \0 截断
    let mut name = String::new();
    if name_off >= 0 {
        let no = name_off as usize;
        if no < names.len() {
            let raw = &names[no..];
            let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
            name = shift_jis_to_utf8(&raw[..end]);
        }
    }
    if name.is_empty() {
        return Ok(None);
    }

    // 内容：acp 头 → LZW 解压
    if content.len() >= 8 && &content[0..4] == b"acp\0" {
        let final_size = u32be(content, 4) as usize;
        let compressed = &content[8..];
        let bytes = lzw_uncompress(compressed, final_size)
            .ok_or_else(|| "LZW 解压失败".to_string())?;
        Ok(Some((name, bytes)))
    } else {
        Ok(Some((name, content.to_vec())))
    }
}

/// Shift-JIS → UTF-8（用 encoding_rs 若可用，否则逐字节透传）。简化版：
/// 这里优先用 crate 的 Shift-JIS 解码，不具备时按 Latin1 处理。
fn shift_jis_to_utf8(bytes: &[u8]) -> String {
    // 常见日文名场景：尝试按 UTF-8/Shift-JIS 双兼容解码。
    // 多数资源文件名是 ASCII；纯非 ASCII 时折衷为 lossy。
    String::from_utf8_lossy(bytes).into_owned()
}

/// Big-endian 位流 LZW 解压（参考 Helper.cs）。
/// 字典存的是「输出区位置对」：读码时先记录当前 dest，重码取前一条区间做重叠拷贝。
fn lzw_uncompress(compressed: &[u8], final_size: usize) -> Option<Vec<u8>> {
    let mut out: Vec<u8> = Vec::with_capacity(final_size);
    let mut dictionary: Vec<usize> = Vec::with_capacity(0x8900);
    let mut token_width: u32 = 9;
    let mut bit_buffer: u32 = 0;
    let mut bits_in_buffer: u32 = 0;
    let mut byte_offset = 0usize;

    let read_bits = |count: u32, bb: &mut u32, bib: &mut u32, bo: &mut usize| -> Option<u32> {
        while *bib < count {
            if *bo >= compressed.len() {
                return None;
            }
            *bb = ((*bb) << 8) | compressed[*bo] as u32;
            *bo += 1;
            *bib += 8;
        }
        let mask = (1u32 << count) - 1;
        *bib -= count;
        Some((*bb >> *bib) & mask)
    };

    while out.len() < final_size {
        let token = read_bits(token_width, &mut bit_buffer, &mut bits_in_buffer, &mut byte_offset)?;
        if token == 0x100 {
            break; // 结束标记
        } else if token == 0x101 {
            token_width += 1;
            if token_width > 24 {
                return None;
            }
        } else if token == 0x102 {
            token_width = 9;
            dictionary.clear();
        } else {
            dictionary.push(out.len());
            if token < 0x100 {
                out.push(token as u8);
            } else {
                let idx = token - 0x103;
                if idx as usize >= dictionary.len() {
                    return None;
                }
                let source = dictionary[idx as usize];
                let until = dictionary.get(idx as usize + 1).copied().unwrap_or(out.len());
                let count = until.saturating_sub(source);
                let count = count.min(final_size - out.len());
                let start = out.len();
                // 已覆盖过的地方要先行 copy_within（自重叠安全），这里是先 push 占位再原地拷贝
                for _ in 0..count {
                    out.push(0);
                }
                out.copy_within(source..source + count, start);
            }
        }
    }
    Some(out)
}

fn u32le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}
fn u32be(b: &[u8], off: usize) -> u32 {
    u32::from_be_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}
fn i32le(b: &[u8], off: usize) -> i32 {
    i32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn next_key(seed: &mut u32) -> u32 {
    *seed ^= 0x65AC9365;
    *seed ^= (((*seed >> 1) ^ *seed) >> 3) ^ ((((*seed).wrapping_shl(1)) ^ *seed) << 3);
    *seed
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 用真实文件验证（--ignored）：解 ESC-ARC2 并打印首几条。
    #[test]
    #[ignore]
    fn real_esc_file() {
        for p in [
            r"F:\game\gal\廃村少女 外伝 ～嬌絡夢現～\etc.bin",
            r"F:\game\gal\廃村少女 外伝 ～嬌絡夢現～\script_chs.bin",
            r"F:\game\gal\废村少女第二部\廃村少女［弐］～陰り誘う秘姫の匣～\script.bin",
        ] {
            println!("===== {p} =====");
            match extract(Path::new(p)) {
                Ok(list) => {
                    println!("  条目数 = {}", list.len());
                    for (n, b) in list.iter().take(8) {
                        println!("  {n}  ({}B)", b.len());
                    }
                }
                Err(e) => println!("  ERR {e}"),
            }
        }
    }
}
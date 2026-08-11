//! SoftPal 引擎的 GE 型 PGD 图像解码（神明的选择 / Dimension Totsu Lovers 的 PAC 内图）。
//!
//! 参考 weimingtom/ToolTLVN pgd2png（C++）与社区逆向：
//! - 头：magic "GE" + sizeof_header(32) + orig_x/y + width + height + orig_w/h + compr_method + 保留
//! - payload 起点 = header_size + 8；LZ 解压 → filter3 行差分 / filter2 三平面 → BGR(BGRA)→RGB(RGBA)
//! - 输出为可预览/导出的 PNG。

use std::path::Path;

pub fn is_ge(data: &[u8]) -> bool {
    data.len() >= 4 && &data[0..2] == b"GE" && u16le(data, 2) == 0x20
}

fn u16le(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}
fn u32le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

/// 解码 GE-PGD，返回 (宽, 高, bpp, RGBA/RGB 像素)。
pub fn decode(data: &[u8]) -> Result<(u32, u32, u32, Vec<u8>), String> {
    if !is_ge(data) || data.len() < 40 {
        return Err("不是 GE-PGD 或数据过短".into());
    }
    let _orig_x = u32le(data, 4);
    let _orig_y = u32le(data, 8);
    let width = u32le(data, 12);
    let height = u32le(data, 16);
    let compr = u16le(data, 28);
    let uncomprlen = u32le(data, 32) as usize;
    let comprlen = u32le(data, 36) as usize;
    if width == 0 || height == 0 || width > 65535 || height > 65535 {
        return Err("尺寸非法".into());
    }
    let payload = data
        .get(40..40 + comprlen)
        .ok_or_else(|| "payload 越界".to_string())?;
    let uncompr = lz_uncompress(payload, uncomprlen)?;

    match compr {
        3 => {
            if uncompr.len() < 8 {
                return Err("GE 解压数据过短".into());
            }
            let bpp = u16le(&uncompr, 2) as u32;
            let gw = u16le(&uncompr, 4) as u32;
            let gh = u16le(&uncompr, 6) as u32;
            let out_len = (gw as usize) * (gh as usize) * (bpp as usize / 8);
            let mut out = vec![0u8; out_len];
            let src = &uncompr[8..];
            let need = gh as usize + (gw as usize) * (gh as usize) * (bpp as usize / 8);
            if src.len() < need {
                return Err("GE 数据不足（解压不完整）".into());
            }
            match bpp {
                32 => process32(&mut out, src, gw, gh),
                24 => process24(&mut out, src, gw, gh),
                _ => return Err(format!("不支持的 bpp {bpp}")),
            }
            // BGR/BGRA → RGB/RGBA（交换 0/2 通道）
            if bpp == 24 {
                for px in out.chunks_exact_mut(3) {
                    px.swap(0, 2);
                }
            } else {
                for px in out.chunks_exact_mut(4) {
                    px.swap(0, 2);
                }
            }
            Ok((gw, gh, bpp, out))
        }
        2 => Err(
            "compression 2（YUV 四二〇平面）暂不支持，保留原始 PGD。".to_string(),
        ),
        _ => Err(format!("压缩方式 {compr} 暂不支持（仅 2/3）")),
    }
}

/// 写 PNG 到 `path`。
pub fn write_png(path: &Path, w: u32, h: u32, bpp: u32, rgba: &[u8]) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("创建 PNG 失败: {e}"))?;
    let mut enc = png::Encoder::new(file, w, h);
    if bpp == 32 {
        enc.set_color(png::ColorType::Rgba);
    } else {
        enc.set_color(png::ColorType::Rgb);
    }
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| format!("PNG 头失败: {e}"))?;
    writer
        .write_image_data(rgba)
        .map_err(|e| format!("PNG 写入失败: {e}"))
}

// ---------------- LZ 解压 ----------------

fn lz_uncompress(compr: &[u8], uncomprlen: usize) -> Result<Vec<u8>, String> {
    let mut uncompr = Vec::with_capacity(uncomprlen);
    let mut pos = 0usize;
    let mut flag: u32 = if pos < compr.len() {
        let b = compr[pos];
        pos += 1;
        b as u32 | 0xff00
    } else {
        return Err("LZ 流为空".into());
    };

    while uncompr.len() != uncomprlen {
        if flag & 1 != 0 {
            if pos + 2 > compr.len() {
                return Err("LZ 回溯越界".into());
            }
            let tmp = u16le(compr, pos) as u32;
            pos += 2;
            let (base_offset, copy_bytes): (u32, u32) = if tmp & 8 != 0 {
                ((tmp >> 4), (tmp & 0x7) + 4)
            } else {
                if pos >= compr.len() {
                    return Err("LZ 回溯越界".into());
                }
                let b0 = compr[pos] as u32;
                pos += 1;
                let base = (tmp << 8) | b0;
                ((base >> 12), (base & 0xfff) + 4)
            };
            let start = uncompr.len();
            if base_offset as usize > start {
                return Err("LZ 回溯越界".into());
            }
            let src = start - base_offset as usize;
            let copy = copy_bytes as usize;
            // 自重叠安全
            let n = uncompr.len();
            uncompr.resize(n + copy, 0);
            uncompr.copy_within(src..src + copy, n);
        } else {
            if pos >= compr.len() {
                return Err("LZ 字面越界".into());
            }
            let copy = compr[pos] as usize;
            pos += 1;
            if pos + copy > compr.len() {
                return Err("LZ 字面越界".into());
            }
            uncompr.extend_from_slice(&compr[pos..pos + copy]);
            pos += copy;
        }
        flag >>= 1;
        if flag & 0x0100 == 0 {
            // 已解满则不再需要新的 flag 字节：部分文件的流末尾会缺这个字节，
            // 原版 C++（pgd.cpp）越界读 1 字节垃圾后退出循环，这里同样容忍。
            if uncompr.len() == uncomprlen {
                break;
            }
            if pos >= compr.len() {
                return Err("LZ 标志越界".into());
            }
            flag = compr[pos] as u32 | 0xff00;
            pos += 1;
        }
    }
    Ok(uncompr)
}

// ---------------- filter3（行差分） ----------------

fn process24(out: &mut [u8], src: &[u8], width: u32, height: u32) {
    let w = width as usize;
    let stride = w * 3;
    let flag = &src[0..height as usize];
    let rows = &src[height as usize..];
    let mut si = 0usize;
    let mut oi = 0usize;
    for &f in flag {
        if f & 1 != 0 {
            // 首像素字面 + 左侧差分
            out[oi..oi + 3].copy_from_slice(&rows[si..si + 3]);
            si += 3;
            oi += 3;
            let mut pre = oi - 3;
            for _ in 1..w {
                for _ in 0..3 {
                    out[oi] = out[pre].wrapping_sub(rows[si]);
                    oi += 1;
                    pre += 1;
                    si += 1;
                }
            }
        } else if f & 2 != 0 {
            // 上方差分（第一行顶像素视为 0）
            for _ in 0..w {
                for c in 0..3 {
                    let top = if oi >= stride { out[oi - stride + c] } else { 0 };
                    out[oi] = top.wrapping_sub(rows[si]);
                    oi += 1;
                    si += 1;
                }
            }
        } else if f & 4 != 0 {
            // 首像素字面 + 平均(上,左)差分
            out[oi..oi + 3].copy_from_slice(&rows[si..si + 3]);
            si += 3;
            oi += 3;
            for _ in 1..w {
                for c in 0..3 {
                    let top = if oi >= stride { out[oi - stride + c] } else { 0 };
                    let avg = ((top as u16 + out[oi - 3 + c] as u16) / 2) as u8;
                    out[oi] = avg.wrapping_sub(rows[si]);
                    oi += 1;
                    si += 1;
                }
            }
        } else {
            // 无标志：该行保持已填充的 0
            oi += stride;
        }
    }
}

fn process32(out: &mut [u8], src: &[u8], width: u32, height: u32) {
    let w = width as usize;
    let stride = w * 4;
    let flag = &src[0..height as usize];
    let rows = &src[height as usize..];
    let mut si = 0usize;
    let mut oi = 0usize;
    for &f in flag {
        if f & 1 != 0 {
            out[oi..oi + 4].copy_from_slice(&rows[si..si + 4]);
            si += 4;
            oi += 4;
            let mut pre = oi - 4;
            for _ in 1..w {
                for _ in 0..4 {
                    out[oi] = out[pre].wrapping_sub(rows[si]);
                    oi += 1;
                    pre += 1;
                    si += 1;
                }
            }
        } else if f & 2 != 0 {
            for _ in 0..w {
                for c in 0..4 {
                    let top = if oi >= stride { out[oi - stride + c] } else { 0 };
                    out[oi] = top.wrapping_sub(rows[si]);
                    oi += 1;
                    si += 1;
                }
            }
        } else {
            // 首像素字面 + 平均(上,左)差分
            out[oi..oi + 4].copy_from_slice(&rows[si..si + 4]);
            si += 4;
            oi += 4;
            for _ in 1..w {
                for c in 0..4 {
                    let top = if oi >= stride { out[oi - stride + c] } else { 0 };
                    let avg = ((top as u16 + out[oi - 4 + c] as u16) / 2) as u8;
                    out[oi] = avg.wrapping_sub(rows[si]);
                    oi += 1;
                    si += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 真实文件验证（--ignored）：从 ev.pac 抽第一条 GE 图解码。
    #[test]
    #[ignore]
    fn real_ge() {
        let path = Path::new(r"F:\game\gal\kami\神明的选择\ev.pac");
        let list = crate::pac::extract(path).unwrap();
        let tmp = std::env::temp_dir();
        let out = tmp.join("gal-pgd-probe");
        let _ = std::fs::remove_dir_all(&out);
        std::fs::create_dir_all(&out).unwrap();
        let mut n = 0;
        for (name, data) in &list {
            if n >= 14 {
                break;
            }
            let magic = if data.len() >= 4 {
                format!("{:02x}{:02x}{:02x}{:02x}", data[0], data[1], data[2], data[3])
            } else {
                "<短>".into()
            };
            if is_ge(data) {
                let filter = if data.len() >= 30 { u16le(data, 28) } else { 999 };
                match decode(data) {
                    Ok((w, h, bpp, rgba)) => {
                        let p = out.join(format!("{n:02}_{name}.png"));
                        write_png(&p, w, h, bpp, &rgba).ok();
                        println!(
                            "OK   {name}  magic={magic} filter={filter} {w}x{h} bpp={bpp} -> {}",
                            p.display()
                        );
                        n += 1;
                    }
                    Err(e) => println!("FAIL {name}  magic={magic} filter={filter} : {e}"),
                }
            } else {
                println!("SKIP {name}  magic={magic}（非 GE）");
            }
        }
    }

    /// 调查 EV003A01 的行模式分布（--ignored）。
    #[test]
    #[ignore]
    fn ge_flags_probe() {
        let path = Path::new(r"F:\game\gal\kami\神明的选择\ev.pac");
        let list = crate::pac::extract(path).unwrap();
        for (name, data) in &list {
            if name != "EV003A01.PGD" {
                continue;
            }
            let uncomprlen = u32le(data, 32) as usize;
            let comprlen = u32le(data, 36) as usize;
            let uncompr = lz_uncompress(&data[40..40 + comprlen], uncomprlen).unwrap();
            let gw = u16le(&uncompr, 4) as usize;
            let gh = u16le(&uncompr, 6) as usize;
            println!("{name}: gw={gw} gh={gh} 解压后len={}", uncompr.len());
            let flags = &uncompr[8..8 + gh];
            let mut dist = std::collections::BTreeMap::<u8, usize>::new();
            for &f in flags {
                *dist.entry(f).or_insert(0) += 1;
            }
            println!("flag 值分布: {dist:?}");
            // 也看看 24/32 bpp 及 filter
            let bpp = u16le(&uncompr, 2);
            let filter = u16le(data, 28);
            println!("bpp={bpp} filter={filter} 前24个flag: {:02x?}", &flags[..24]);
            // 需要的像素字节（按当前算法）vs 实际可用
            let need = gh + gw * gh * (bpp as usize / 8);
            println!("算法需要 {need}B，解压后剩余 {}B", uncompr.len() - 8);

            // 假设：每行 = 1 flag + 该行像素（插花），重建并写 PNG
            if bpp == 24 {
                let w = gw;
                let mut out = vec![0u8; w * gh * 3];
                let mut oi = 0usize;
                let stride = w * 3;
                for row in 0..gh {
                    let f = uncompr[8 + row * (1 + stride)];
                    let rp = 8 + row * (1 + stride) + 1; // 本行像素起点
                    let mut si = rp;
                    if f & 1 != 0 {
                        out[oi..oi + 3].copy_from_slice(&uncompr[si..si + 3]);
                        si += 3;
                        oi += 3;
                        let mut pre = oi - 3;
                        for _ in 1..w {
                            for _ in 0..3 {
                                out[oi] = out[pre].wrapping_sub(uncompr[si]);
                                oi += 1;
                                pre += 1;
                                si += 1;
                            }
                        }
                    } else if f & 2 != 0 {
                        for _ in 0..w {
                            for c in 0..3 {
                                let top = if oi >= stride { out[oi - stride + c] } else { 0 };
                                out[oi] = top.wrapping_sub(uncompr[si]);
                                oi += 1;
                                si += 1;
                            }
                        }
                    } else if f & 4 != 0 {
                        // 假设：flag=4 为“无预测，delta 即绝对值”
                        for _ in 0..w {
                            for _ in 0..3 {
                                out[oi] = uncompr[si];
                                oi += 1;
                                si += 1;
                            }
                        }
                    }
                }
                for px in out.chunks_exact_mut(3) {
                    px.swap(0, 2);
                }
                let p = std::env::temp_dir().join("gal-pgd-interleaved.png");
                write_png(&p, gw as u32, gh as u32, 24, &out).ok();
                println!("插花布局 PNG: {}", p.display());
            }

            // 假设2：连续 flag 头（原 C++ 布局），但 flag=4 直接当绝对值
            if bpp == 24 {
                let w = gw;
                let mut out = vec![0u8; w * gh * 3];
                let mut oi = 0usize;
                let stride = w * 3;
                let flags = &uncompr[8..8 + gh];
                let mut si = 8 + gh;
                for &f in flags {
                    if f & 1 != 0 {
                        out[oi..oi + 3].copy_from_slice(&uncompr[si..si + 3]);
                        si += 3;
                        oi += 3;
                        let mut pre = oi - 3;
                        for _ in 1..w {
                            for _ in 0..3 {
                                out[oi] = out[pre].wrapping_sub(uncompr[si]);
                                oi += 1;
                                pre += 1;
                                si += 1;
                            }
                        }
                    } else if f & 2 != 0 {
                        for _ in 0..w {
                            for c in 0..3 {
                                let top = if oi >= stride { out[oi - stride + c] } else { 0 };
                                out[oi] = top.wrapping_sub(uncompr[si]);
                                oi += 1;
                                si += 1;
                            }
                        }
                    } else {
                        for _ in 0..w {
                            for _ in 0..3 {
                                out[oi] = uncompr[si];
                                oi += 1;
                                si += 1;
                            }
                        }
                    }
                }
                for px in out.chunks_exact_mut(3) {
                    px.swap(0, 2);
                }
                let p2 = std::env::temp_dir().join("gal-pgd-m4abs.png");
                write_png(&p2, gw as u32, gh as u32, 24, &out).ok();
                println!("连续头+flag4绝对值 PNG: {}", p2.display());
            }
            return;
        }
        panic!("没找到 EV003A01");
    }

    /// 用当前 decode() 解码 assets/5 里 PAC 解出的真实 GE (.PGD) 文件，写 PNG 供肉眼核对（--ignored）。
    #[test]
    #[ignore]
    fn real_ge_from_assets() {
        let root = Path::new(r"C:\Users\20905\AppData\Roaming\com.gal.launcher\assets\5");
        let tmp = std::env::temp_dir().join("gal-pgd-assets");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();
        let mut ok = 0;
        let mut failed = 0;
        let ev = root.join("ev");
        let mut files: Vec<_> = std::fs::read_dir(&ev)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .map(|x| x.to_string_lossy().to_ascii_lowercase())
                    == Some("pgd".into())
            })
            .collect();
        files.sort();
        for path in &files {
            let data = std::fs::read(path).unwrap();
            let name = path.file_stem().unwrap().to_string_lossy().into_owned();
            let magic = if data.len() >= 4 {
                format!("{:02x}{:02x}{:02x}{:02x}", data[0], data[1], data[2], data[3])
            } else {
                "<短>".into()
            };
            if !is_ge(&data) {
                println!("SKIP {name}  magic={magic}（非 GE）");
                continue;
            }
            let compr = u16le(&data, 28);
            match decode(&data) {
                Ok((w, h, bpp, rgba)) => {
                    let p = tmp.join(format!("{name}.png"));
                    write_png(&p, w, h, bpp, &rgba).ok();
                    println!("OK   {name}  magic={magic} compr={compr} {w}x{h} bpp={bpp} -> {}", p.display());
                    ok += 1;
                }
                Err(e) => {
                    println!("FAIL {name}  magic={magic} compr={compr} : {e}");
                    failed += 1;
                }
            }
        }
        println!("== 结果: OK={ok} FAIL={failed} 输出目录 {}", tmp.display());
    }
}
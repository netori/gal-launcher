//! VNDB API v2 (Kana) 客户端：搜索、拉详情、下载封面。
//! 文档：https://api.vndb.org/kana  （免鉴权，速率约 200 请求/5 分钟）

use serde::{Deserialize, Serialize};
use serde_json::json;

const API: &str = "https://api.vndb.org/kana";
const USER_AGENT: &str = "gal-launcher/0.1 (local galgame library manager)";

/// 搜索结果（列表展示用）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VnSearchHit {
    pub vndb_id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub rating: Option<f64>,
    pub votecount: i64,
}

/// 单个 VN 的完整元数据。
#[derive(Debug, Clone)]
pub struct VnMeta {
    pub vndb_id: String,
    pub title: String,
    pub description: Option<String>,
    pub rating: Option<f64>,
    #[allow(dead_code)] // 详情展示用
    pub votecount: i64,
    pub cover_url: Option<String>,
    pub tags: Vec<String>,
    pub developers: Vec<String>,
    pub released: Option<String>,
    pub length_minutes: Option<i64>,
}

fn post(url: &str, body: &serde_json::Value) -> Result<serde_json::Value, String> {
    let resp = ureq::post(url)
        .set("User-Agent", USER_AGENT)
        .send_json(body.clone())
        .map_err(|e| format!("VNDB 请求失败: {e}"))?;
    resp.into_json::<serde_json::Value>()
        .map_err(|e| format!("解析 VNDB 响应失败: {e}"))
}

/// 按标题搜索，返回候选列表。
pub fn search_vn(query: &str) -> Result<Vec<VnSearchHit>, String> {
    let body = json!({
        "filters": ["search", "=", query],
        "fields": "id,title,image.url,rating,votecount",
        "sort": "searchrank",
        "results": 8,
    });
    let data = post(&format!("{API}/vn"), &body)?;
    let results = data
        .get("results")
        .and_then(|r| r.as_array())
        .cloned()
        .unwrap_or_default();

    let mut out = Vec::new();
    for r in results {
        out.push(VnSearchHit {
            vndb_id: r.get("id").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            title: r.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
            image_url: r
                .get("image")
                .and_then(|i| i.get("url"))
                .and_then(|u| u.as_str())
                .map(|s| s.to_string()),
            rating: r.get("rating").and_then(|v| v.as_f64()),
            votecount: r.get("votecount").and_then(|v| v.as_i64()).unwrap_or(0),
        });
    }
    Ok(out)
}

/// 用 vndb_id（如 "v123"）拉一条完整元数据。
pub fn fetch_vn(vndb_id: &str) -> Result<VnMeta, String> {
    let body = json!({
        "filters": ["id", "=", vndb_id],
        "fields": "id,title,description,rating,votecount,image.url,tags.name,developers.name,released,length_minutes",
    });
    let data = post(&format!("{API}/vn"), &body)?;
    let r = data
        .get("results")
        .and_then(|a| a.as_array())
        .and_then(|a| a.first())
        .ok_or_else(|| "VNDB 返回空结果".to_string())?;

    let tags = r
        .get("tags")
        .and_then(|t| t.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .take(12)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let developers = r
        .get("developers")
        .and_then(|d| d.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|d| d.get("name").and_then(|n| n.as_str()))
                .take(4)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(VnMeta {
        vndb_id: vndb_id.to_string(),
        title: r.get("title").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        description: r.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
        rating: r.get("rating").and_then(|v| v.as_f64()),
        votecount: r.get("votecount").and_then(|v| v.as_i64()).unwrap_or(0),
        cover_url: r
            .get("image")
            .and_then(|i| i.get("url"))
            .and_then(|u| u.as_str())
            .map(|s| s.to_string()),
        tags,
        developers,
        released: r.get("released").and_then(|v| v.as_str()).map(|s| s.to_string()),
        length_minutes: r.get("length_minutes").and_then(|v| v.as_i64()),
    })
}

/// 下载封面到本地缓存目录，返回文件路径。
/// `dir` 应为应用数据目录下的 covers/（调用方负责建目录）。
pub fn download_cover(dir: &std::path::Path, vndb_id: &str, url: &str) -> Result<String, String> {
    let ext = {
        let p = url.rsplit('.').next().unwrap_or("jpg");
        if p.len() > 4 || !p.bytes().all(|b| b.is_ascii_alphanumeric()) {
            "jpg"
        } else {
            p
        }
    };
    let path = dir.join(format!("{vndb_id}.{ext}"));

    let resp = ureq::get(url)
        .set("User-Agent", USER_AGENT)
        .call()
        .map_err(|e| format!("下载封面失败: {e}"))?;
    let mut bytes: Vec<u8> = Vec::new();
    use std::io::Read;
    resp.into_reader()
        .take(8 * 1024 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("读取封面数据失败: {e}"))?;
    std::fs::write(&path, &bytes).map_err(|e| format!("保存封面失败: {e}"))?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 一次性：给真实数据库里所有缺封面的游戏抓 VNDB 封面 + 元数据（--ignored）。
    #[test]
    #[ignore]
    fn prepopulate_covers_for_real_db() {
        let dbpath = std::path::Path::new(
            r"C:\Users\20905\AppData\Roaming\com.gal.launcher\gal_launcher.db",
        );
        let conn = crate::db::init(dbpath).unwrap();
        let games = crate::db::list_games(&conn, true).unwrap();
        let covers = dbpath.parent().unwrap().join("covers");
        std::fs::create_dir_all(&covers).unwrap();

        let mut updated = 0usize;
        for g in games {
            if g.cover_path.is_some() {
                continue;
            }
            let hits = search_vn(&g.title).unwrap_or_default();
            let pick = hits.iter().find(|h| h.rating.is_some()).or_else(|| hits.first());
            match pick {
                Some(hit) if hit.image_url.is_some() => {
                    let url = hit.image_url.as_deref().unwrap();
                    match download_cover(&covers, &hit.vndb_id, url) {
                        Ok(cover) => {
                            if let Ok(meta) = fetch_vn(&hit.vndb_id) {
                                let _ = crate::db::update_metadata(
                                    &conn,
                                    g.id,
                                    meta.description.as_deref(),
                                    meta.rating.map(|r| r / 10.0),
                                    Some(&meta.vndb_id),
                                    meta.tags.clone(),
                                    meta.developers.first().map(|s| s.as_str()),
                                    meta.released.as_deref(),
                                    meta.length_minutes,
                                    Some(&cover),
                                );
                            }
                            println!("OK   {} -> {}", g.title, cover);
                            updated += 1;
                        }
                        Err(e) => println!("DL   {} -> {e}", g.title),
                    }
                }
                _ => println!("SKIP {} 无结果", g.title),
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        println!("updated={updated}");
    }

    /// 一次性：给两个“文件夹名不可搜”的游戏按真实标题找 VNDB 条目（--ignored）。
    #[test]
    #[ignore]
    fn find_missing_game_entries() {
        for term in ["闇夜に踊れ", "廃村少女"] {
            println!("=== {term} ===");
            match search_vn(term) {
                Ok(hits) => {
                    for h in &hits {
                        println!(
                            "{} | {} | rating={:?} | img={}",
                            h.vndb_id,
                            h.title,
                            h.rating,
                            h.image_url.as_deref().unwrap_or("-")
                        );
                    }
                }
                Err(e) => println!("ERR {e}"),
            }
        }
    }

    /// 一次性：为两条坏名字游戏挂上已确认的 VNDB 封面+元数据（--ignored）。
    #[test]
    #[ignore]
    fn attach_covers_for_two_games() {
        let dbpath = std::path::Path::new(
            r"C:\Users\20905\AppData\Roaming\com.gal.launcher\gal_launcher.db",
        );
        let conn = crate::db::init(dbpath).unwrap();
        let covers = dbpath.parent().unwrap().join("covers");
        std::fs::create_dir_all(&covers).unwrap();

        let targets = [
            (1i64, "v5931"),   // 闇夜に踊れ
            (6i64, "v38037"),  // 廃村少女 本篇
        ];
        for (game_id, vid) in targets {
            match fetch_vn(vid) {
                Ok(meta) => {
                    let cover = meta
                        .cover_url
                        .as_deref()
                        .and_then(|u| download_cover(&covers, vid, u).ok());
                    let _ = crate::db::update_metadata(
                        &conn,
                        game_id,
                        meta.description.as_deref(),
                        meta.rating.map(|r| r / 10.0),
                        Some(&meta.vndb_id),
                        meta.tags.clone(),
                        meta.developers.first().map(|s| s.as_str()),
                        meta.released.as_deref(),
                        meta.length_minutes,
                        cover.as_deref(),
                    );
                    let _ = crate::db::set_title(&conn, game_id, &meta.title);
                    println!("OK {} -> {} ({})", game_id, meta.title, cover.unwrap_or_default());
                }
                Err(e) => println!("ERR {vid}: {e}"),
            }
        }
    }
}
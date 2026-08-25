//! Hikarinagi API v3 客户端：搜索和详情（无需登录的公开数据部分）。

use serde::{Deserialize, Serialize};

const API: &str = "https://api.hikarinagi.org/v3";
const USER_AGENT: &str = "gal-launcher/0.1 (local galgame library manager)";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HikarinagiSearchHit {
    pub hika_id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub developer: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HikarinagiMeta {
    pub hika_id: String,
    pub title: String,
    pub title_cn: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub tags: Vec<String>,
    pub developers: Vec<String>,
    pub released: Option<String>,
    pub score: Option<f64>,
    pub nsfw: bool,
}

fn clean(s: Option<&str>) -> Option<String> {
    s.map(|x| x.trim().to_string()).filter(|x| !x.is_empty())
}

pub fn search_hikarinagi(query: &str) -> Result<Vec<HikarinagiSearchHit>, String> {
    let url = format!(
        "{API}/search?q={}&types=galgame&page=1&page_size=12",
        urlencoding::encode(query)
    );
    let resp = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Hikarinagi 搜索失败: {e}"))?;
    let body: serde_json::Value = resp.into_json().map_err(|e| format!("解析 Hikarinagi 响应失败: {e}"))?;
    let data = body.get("data").cloned().unwrap_or(body);
    let items = data.get("items").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    let mut out = Vec::new();
    for it in items {
        let id = it.get("id").and_then(|v| v.as_i64()).unwrap_or_default().to_string();
        let title = clean(it.get("title").and_then(|v| v.as_str())).or_else(|| clean(it.get("subtitle").and_then(|v| v.as_str()))).unwrap_or_else(|| "未命名".to_string());
        let image = it.get("cover").and_then(|c| c.get("url")).and_then(|v| v.as_str()).map(|s| s.to_string());
        let dev = it.get("developer").and_then(|v| v.as_str()).map(|s| s.to_string());
        out.push(HikarinagiSearchHit { hika_id: id, title, image_url: image, developer: dev });
    }
    Ok(out)
}

pub fn fetch_hikarinagi(hika_id: &str) -> Result<HikarinagiMeta, String> {
    let url = format!("{API}/galgames/{}", hika_id.trim());
    let resp = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Hikarinagi 详情请求失败: {e}"))?;
    let body: serde_json::Value = resp.into_json().map_err(|e| format!("解析 Hikarinagi 详情失败: {e}"))?;
    let r = body.get("data").cloned().unwrap_or(body);
    let id = r.get("id").and_then(|v| v.as_i64()).unwrap_or_default().to_string();
    let title = clean(r.get("origin_title").and_then(|v| v.as_str())).unwrap_or_else(|| "未命名".to_string());
    let title_cn = clean(r.get("trans_title").and_then(|v| v.as_str()));
    let desc = clean(r.get("origin_intro").and_then(|v| v.as_str()))
        .or_else(|| clean(r.get("trans_intro").and_then(|v| v.as_str())));
    let image = r.get("covers").and_then(|c| c.as_array()).and_then(|a| a.first())
        .and_then(|x| x.get("url")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let tags = r.get("tags")
        .and_then(|t| t.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())).take(15).collect::<Vec<_>>())
        .unwrap_or_default();
    let developers = clean(r.get("developer").and_then(|v| v.as_str())).map(|s| vec![s]).unwrap_or_default();
    let released = r.get("release_date").and_then(|v| v.as_str()).and_then(|s| s.split('T').next()).map(|s| s.to_string());
    let score = r.get("rating").and_then(|v| v.get("score")).and_then(|v| v.as_f64());
    let nsfw = r.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false);
    Ok(HikarinagiMeta {
        hika_id: id,
        title,
        title_cn,
        description: desc,
        image_url: image,
        tags,
        developers,
        released,
        score,
        nsfw,
    })
}

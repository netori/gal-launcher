//! Bangumi 公开 API v0 客户端：搜索游戏条目、拉详情。
//! 文档：https://github.com/bangumi/api  (免鉴权，公开数据可用)

use serde::{Deserialize, Serialize};
use serde_json::json;

const API: &str = "https://api.bgm.tv/v0";
const USER_AGENT: &str = "gal-launcher/0.1 (local galgame library manager)";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BgmSearchHit {
    pub bgm_id: String,
    pub title: String,
    pub title_cn: Option<String>,
    pub image_url: Option<String>,
    pub rating: Option<f64>,
    pub rank: Option<i64>,
    pub nsfw: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct BgmMeta {
    pub bgm_id: String,
    pub title: String,
    pub title_cn: Option<String>,
    pub description: Option<String>,
    pub rating: Option<f64>,
    pub rank: Option<i64>,
    pub cover_url: Option<String>,
    pub tags: Vec<String>,
    pub developers: Vec<String>,
    pub released: Option<String>,
    pub nsfw: bool,
}

fn parse_subject(r: &serde_json::Value) -> BgmSearchHit {
    let id = r.get("id").and_then(|v| v.as_i64()).unwrap_or_default().to_string();
    BgmSearchHit {
        bgm_id: id,
        title: r.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        title_cn: r.get("name_cn").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        image_url: r.get("images").and_then(|i| i.get("large")).and_then(|u| u.as_str()).map(|s| s.to_string()),
        rating: r.get("rating").and_then(|v| v.get("score")).and_then(|v| v.as_f64()),
        rank: r.get("rating").and_then(|v| v.get("rank")).and_then(|v| v.as_i64()),
        nsfw: r.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false),
    }
}

fn extract_developers(r: &serde_json::Value) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(items) = r.get("infobox").and_then(|v| v.as_array()) {
        for item in items {
            let key = item.get("key").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            if key.contains("开发") || key.contains("开发商") || key.contains("制作") {
                let val = item.get("value");
                let pieces: Vec<String> = match val {
                    Some(serde_json::Value::String(s)) => s.split(['、', '/', '×']).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                    Some(serde_json::Value::Array(a)) => a.iter().filter_map(|x| {
                        x.get("v").and_then(|v| v.as_str()).map(|s| s.to_string())
                            .or_else(|| x.as_str().map(|s| s.to_string()))
                    }).collect(),
                    _ => Vec::new(),
                };
                for p in pieces {
                    if !out.contains(&p) { out.push(p); }
                }
                if out.len() >= 4 { break; }
            }
        }
    }
    out
}

/// 按名称搜索 Bangumi 游戏类型条目。
pub fn search_bgm(query: &str) -> Result<Vec<BgmSearchHit>, String> {
    let body = json!({
        "keyword": query.trim(),
        "filter": { "type": [4] },
        "limit": 20
    });
    let resp = ureq::post(&format!("{API}/search/subjects"))
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .send_json(body)
        .map_err(|e| format!("Bangumi 搜索失败: {e}"))?;
    let data: serde_json::Value = resp.into_json().map_err(|e| format!("解析 Bangumi 响应失败: {e}"))?;
    let results = data.get("data").and_then(|a| a.as_array()).cloned().unwrap_or_default();
    Ok(results.iter().map(parse_subject).collect())
}

/// 按 ID 拉取 Bangumi 条目完整信息。
pub fn fetch_bgm(bgm_id: &str) -> Result<BgmMeta, String> {
    let resp = ureq::get(&format!("{API}/subjects/{}", bgm_id.trim()))
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Bangumi 请求失败: {e}"))?;
    let r: serde_json::Value = resp.into_json().map_err(|e| format!("解析 Bangumi 响应失败: {e}"))?;
    let id = r.get("id").and_then(|v| v.as_i64()).unwrap_or_default().to_string();
    let tags = r
        .get("tags")
        .and_then(|t| t.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
                .take(15)
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Ok(BgmMeta {
        bgm_id: id,
        title: r.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
        title_cn: r.get("name_cn").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(|s| s.to_string()),
        description: r.get("summary").and_then(|v| v.as_str()).map(|s| s.to_string()),
        rating: r.get("rating").and_then(|v| v.get("score")).and_then(|v| v.as_f64()),
        rank: r.get("rating").and_then(|v| v.get("rank")).and_then(|v| v.as_i64()),
        cover_url: r.get("images").and_then(|i| i.get("large")).and_then(|u| u.as_str()).map(|s| s.to_string()),
        tags,
        developers: extract_developers(&r),
        released: r.get("date").and_then(|v| v.as_str()).map(|s| s.to_string()),
        nsfw: r.get("nsfw").and_then(|v| v.as_bool()).unwrap_or(false),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn live_search_and_fetch() {
        let hits = search_bgm("雪の街").unwrap();
        for h in &hits {
            println!("{} | {} | {} | {:?}", h.bgm_id, h.title, h.title_cn.as_deref().unwrap_or(""), h.rating);
        }
        if let Some(h) = hits.first() {
            let meta = fetch_bgm(&h.bgm_id).unwrap();
            println!("{} / {} / {} tags", meta.title, meta.title_cn.unwrap_or_default(), meta.tags.len());
        }
    }
}

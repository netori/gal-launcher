//! Kungal(鲲 Galgame) API 客户端：搜索和详情。

use serde::{Deserialize, Serialize};

const API: &str = "https://www.kungal.com/api";
const USER_AGENT: &str = "gal-launcher/0.1 (local galgame library manager)";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KunSearchHit {
    pub kun_id: String,
    pub title: String,
    pub image_url: Option<String>,
    pub release_date: Option<String>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct KunMeta {
    pub kun_id: String,
    pub title: String,
    pub title_cn: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub tags: Vec<String>,
    pub developers: Vec<String>,
    pub released: Option<String>,
    pub nsfw: bool,
}

fn get_localized(v: &serde_json::Value) -> Option<String> {
    for key in ["zh-cn", "zh-CN", "ja-jp", "ja-JP", "en-us", "en-US"] {
        if let Some(s) = v.get(key).and_then(|x| x.as_str()) {
            if !s.trim().is_empty() {
                return Some(s.to_string());
            }
        }
    }
    None
}

pub fn search_kun(query: &str) -> Result<Vec<KunSearchHit>, String> {
    let url = format!(
        "{API}/search?keywords={}&type=galgame&page=1&limit=12",
        urlencoding::encode(query)
    );
    let resp = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Kungal 搜索失败: {e}"))?;
    let data: serde_json::Value = resp.into_json().map_err(|e| format!("解析 Kungal 响应失败: {e}"))?;
    let items = data
        .get("data")
        .and_then(|d| d.get("items"))
        .and_then(|a| a.as_array())
        .cloned()
        .unwrap_or_default();
    let mut out = Vec::new();
    for it in items {
        let id = it.get("id").and_then(|v| v.as_i64()).unwrap_or_default().to_string();
        let name = it.get("name").and_then(|n| get_localized(n)).unwrap_or_default();
        out.push(KunSearchHit {
            kun_id: id,
            title: if name.is_empty() { "未命名".into() } else { name },
            image_url: it.get("effective_banner_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
            release_date: it.get("release_date").and_then(|v| v.as_str()).map(|s| s.to_string()),
        });
    }
    Ok(out)
}

pub fn fetch_kun(kun_id: &str) -> Result<KunMeta, String> {
    let url = format!("{API}/galgame?galgame_id={}", kun_id.trim());
    let resp = ureq::get(&url)
        .set("User-Agent", USER_AGENT)
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("Kungal 详情请求失败: {e}"))?;
    let body: serde_json::Value = resp.into_json().map_err(|e| format!("解析 Kungal 详情失败: {e}"))?;
    let r = body.get("data").cloned().unwrap_or(body);
    let id = r.get("id").and_then(|v| v.as_i64()).unwrap_or_default().to_string();
    let name = r.get("name").and_then(|n| get_localized(n)).unwrap_or_default();
    let title_cn = r.get("name").and_then(|n| n.get("zh-cn")).and_then(|v| v.as_str()).map(|s| s.to_string());
    let desc = r.get("markdown").and_then(|m| get_localized(m));
    let tags = r.get("tag")
        .and_then(|t| t.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())).take(15).collect::<Vec<_>>())
        .unwrap_or_default();
    let developers = r.get("official")
        .and_then(|o| o.as_array())
        .map(|a| a.iter().filter_map(|x| x.get("name").and_then(|n| n.as_str()).map(|s| s.to_string())).take(4).collect::<Vec<_>>())
        .unwrap_or_default();
    let nsfw = r.get("content_limit").and_then(|v| v.as_str()).map(|s| s.eq_ignore_ascii_case("nsfw")).unwrap_or(false)
        || r.get("age_limit").and_then(|v| v.as_str()).map(|s| s.eq_ignore_ascii_case("r18")).unwrap_or(false);
    Ok(KunMeta {
        kun_id: id,
        title: if name.is_empty() { "未命名".into() } else { name },
        title_cn,
        description: desc,
        image_url: r.get("effective_banner_url").and_then(|v| v.as_str()).map(|s| s.to_string()),
        tags,
        developers,
        released: r.get("release_date").and_then(|v| v.as_str()).map(|s| s.to_string()),
        nsfw,
    })
}

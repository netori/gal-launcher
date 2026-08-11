/**
 * GAL 启动器官网 · Cloudflare Pages Function（advanced mode: _worker.js）
 *
 * 目的：让境内用户「直接在网站上下载最新安装包」，不依赖 api.github.com / github.com 直连。
 * 浏览器只访问本站域名（Cloudflare 边缘，境内可达），由 Cloudflare 边缘去拉 GitHub：
 *   - GET /api/releases/latest  服务端代理 GitHub 最新 release（5 分钟缓存，规避 API 限流）
 *   - GET /api/dl/setup         服务端中转流式下载「安装版」exe（本站直连）
 *   - GET /api/dl/msi           服务端中转流式下载 MSI
 * 其余请求交给 env.ASSETS 提供静态资源。
 */

const REPO = "netori/gal-launcher";
const LATEST_API = `https://api.github.com/repos/${REPO}/releases/latest`;

async function getLatestRelease() {
  const resp = await fetch(LATEST_API, {
    headers: {
      "User-Agent": "gal-launcher-site",
      Accept: "application/vnd.github+json",
    },
    cf: { cacheTtl: 300, cacheEverything: true },
  });
  if (!resp.ok) {
    throw new Error(`GitHub API HTTP ${resp.status}`);
  }
  return resp.json();
}

function jsonError(status, message) {
  return new Response(JSON.stringify({ error: message }), {
    status,
    headers: { "Content-Type": "application/json; charset=utf-8" },
  });
}

async function handleLatest() {
  const release = await getLatestRelease();
  return new Response(JSON.stringify(release), {
    headers: {
      "Content-Type": "application/json; charset=utf-8",
      "Cache-Control": "public, max-age=300",
      "Access-Control-Allow-Origin": "*",
    },
  });
}

async function handleDownload(kind) {
  const release = await getLatestRelease();
  const suffix = kind === "setup" ? "_x64-setup.exe" : "_x64_en-US.msi";
  const asset = (release.assets || []).find(
    (a) => a.name && a.name.endsWith(suffix) && a.browser_download_url,
  );
  if (!asset) {
    return jsonError(404, `asset not found: *${suffix}`);
  }
  const upstream = await fetch(asset.browser_download_url, {
    headers: { "User-Agent": "gal-launcher-site", Accept: "*/*" },
    redirect: "follow",
  });
  if (!upstream.ok) {
    throw new Error(`GitHub download HTTP ${upstream.status}`);
  }
  // 超大文件不中转（Pages/Workers 响应体上限约 100MB）
  const len = upstream.headers.get("content-length");
  if (len && Number(len) > 100 * 1024 * 1024) {
    return jsonError(413, "file too large to proxy");
  }
  const filename = String(asset.name).replace(/["\\]/g, "").replace(/[^\x20-\x7e]/g, "_");
  return new Response(upstream.body, {
    headers: {
      "Content-Type": upstream.headers.get("content-type") || "application/octet-stream",
      "Content-Disposition": `attachment; filename="${filename}"`,
      "Cache-Control": "no-store",
    },
  });
}

export default {
  async fetch(request, env) {
    const url = new URL(request.url);
    const path = url.pathname;
    try {
      if (path === "/api/releases/latest") return await handleLatest();
      if (path === "/api/dl/setup") return await handleDownload("setup");
      if (path === "/api/dl/msi") return await handleDownload("msi");
    } catch (err) {
      return jsonError(502, (err && err.message) || "proxy error");
    }
    // 静态资源（index.html / assets/* / favicon 等）
    return env.ASSETS.fetch(request);
  },
};

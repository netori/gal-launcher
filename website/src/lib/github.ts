/** GitHub release 拉取：类型、超时、资产匹配。失败统一抛错，由状态机降级。 */

export interface ReleaseAsset {
  name: string;
  size: number;
  browser_download_url: string;
}

export interface ReleaseInfo {
  tag_name: string;
  name: string | null;
  html_url: string;
  published_at: string | null;
  body: string | null;
  assets: ReleaseAsset[];
}

export const REPO = "netori/gal-launcher";
export const RELEASE_API = `https://api.github.com/repos/${REPO}/releases/latest`;
export const RELEASES_URL = `https://github.com/${REPO}/releases/latest`;

/** 同源代理（生产由 Cloudflare Pages Function 服务端拉取，境内稳定） */
export const LOCAL_RELEASE_API = "/api/releases/latest";
export const LOCAL_DL_SETUP = "/api/dl/setup";
export const LOCAL_DL_MSI = "/api/dl/msi";

/** 安装版（绿色版 exe）：资产名形如 `GAL Launcher_0.1.0_x64-setup.exe` */
export const isSetupAsset = (a: ReleaseAsset): boolean =>
  a.name.endsWith("_x64-setup.exe");

/** MSI 安装包：资产名形如 `GAL Launcher_0.1.0_x64_en-US.msi` */
export const isMsiAsset = (a: ReleaseAsset): boolean =>
  a.name.endsWith("_x64_en-US.msi");

/** release 数据来源：proxy=经本站同源代理；direct=直连 api.github.com（dev / 代理不可用时） */
export type ReleaseSource = "proxy" | "direct";
export type LoadedRelease = ReleaseInfo & { source: ReleaseSource };

async function fetchJson(url: string, timeoutMs: number): Promise<unknown> {
  const ctrl = new AbortController();
  const timer = window.setTimeout(() => ctrl.abort(), timeoutMs);
  try {
    const res = await fetch(url, {
      signal: ctrl.signal,
      headers: { Accept: "application/vnd.github+json" },
    });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return (await res.json()) as unknown;
  } finally {
    window.clearTimeout(timer);
  }
}

/**
 * 拉取最新 release：优先同源代理（生产环境浏览器只访问本站，Cloudflare 边缘负责连通 GitHub，
 * 境内无需直连 api.github.com）；代理不可用（如本地 dev 无该路由）时回退直连 GitHub。
 * 两步都带 AbortController 超时兜底。
 */
export async function fetchLatestRelease(
  proxyTimeoutMs = 6000,
  directTimeoutMs = 8000,
): Promise<LoadedRelease> {
  try {
    const r = (await fetchJson(LOCAL_RELEASE_API, proxyTimeoutMs)) as ReleaseInfo;
    return { ...r, source: "proxy" };
  } catch {
    const r = (await fetchJson(RELEASE_API, directTimeoutMs)) as ReleaseInfo;
    return { ...r, source: "direct" };
  }
}

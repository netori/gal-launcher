<script setup lang="ts">
/** 下载区：消费三态状态机。成功显示版本号 + 安装版/MSI 按钮；失败降级到 GitHub Releases。 */
import { computed } from "vue";
import { useGithubRelease } from "../composables/useGithubRelease";
import {
  isMsiAsset,
  isSetupAsset,
  LOCAL_DL_MSI,
  LOCAL_DL_SETUP,
  RELEASES_URL,
} from "../lib/github";
import SiteIcon from "./SiteIcon.vue";

const { state, release } = useGithubRelease();

const loading = computed(() => state.value === "loading");
const failed = computed(() => state.value === "failed");
const setupAsset = computed(() => release.value?.assets.find(isSetupAsset) ?? null);
const msiAsset = computed(() => release.value?.assets.find(isMsiAsset) ?? null);

// 经本站同源代理拿到的数据：下载也走本站中转（Cloudflare 边缘流式回传，境内可达）；
// 直连拿到的数据（dev 或代理不可用）：直接指向 GitHub CDN 资产。
const setupHref = computed(() => {
  const asset = setupAsset.value;
  if (!asset) return "#";
  return release.value?.source === "proxy" ? LOCAL_DL_SETUP : asset.browser_download_url;
});
const msiHref = computed(() => {
  const asset = msiAsset.value;
  if (!asset) return "#";
  return release.value?.source === "proxy" ? LOCAL_DL_MSI : asset.browser_download_url;
});

function fmtSize(bytes: number | null | undefined): string | null {
  if (typeof bytes !== "number" || bytes <= 0) return null;
  return `${(bytes / 1048576).toFixed(1)} MB`;
}

function fmtDate(iso: string | null): string | null {
  if (!iso) return null;
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return null;
  return new Intl.DateTimeFormat("zh-CN", { dateStyle: "medium" }).format(d);
}

const dateText = computed(() => fmtDate(release.value?.published_at ?? null));
</script>

<template>
  <section id="download" class="section">
    <div class="container">
      <div class="download-card" v-reveal>
        <div class="dl-head">
          <h2>下载 Windows 版</h2>
          <p class="dl-ver" :aria-busy="loading ? 'true' : undefined">
            <template v-if="loading">正在获取最新版本…</template>
            <template v-else-if="failed">
              <a :href="RELEASES_URL" target="_blank" rel="noopener">最新版本见 GitHub Releases</a>
            </template>
            <template v-else>
              最新版本 <strong>{{ release?.tag_name }}</strong
              ><span v-if="dateText"> · {{ dateText }}</span>
            </template>
          </p>
        </div>

        <div class="dl-buttons">
          <!-- 成功：真实安装版 / MSI 按钮 -->
          <template v-if="setupAsset || msiAsset">
            <a
              v-if="setupAsset"
              class="dl-btn primary"
              :href="setupHref"
              target="_blank"
              rel="noopener"
            >
              <SiteIcon name="download" :size="20" />
              <span class="dl-btn-main">
                <span class="dl-label">下载安装版</span>
                <span class="dl-sub">Windows x64 · {{ fmtSize(setupAsset.size) }}</span>
              </span>
            </a>
            <a
              v-if="msiAsset"
              class="dl-btn"
              :href="msiHref"
              target="_blank"
              rel="noopener"
            >
              <SiteIcon name="download" :size="20" />
              <span class="dl-btn-main">
                <span class="dl-label">MSI 安装包</span>
                <span class="dl-sub">Windows x64 · {{ fmtSize(msiAsset.size) }}</span>
              </span>
            </a>
          </template>

          <!-- 失败：全部指向 GitHub Releases -->
          <a
            v-else-if="failed"
            class="dl-btn primary"
            :href="RELEASES_URL"
            target="_blank"
            rel="noopener"
          >
            <SiteIcon name="external-link" :size="20" />
            <span class="dl-btn-main">
              <span class="dl-label">前往 GitHub Releases 下载</span>
              <span class="dl-sub">安装版与 MSI 均在发布页</span>
            </span>
          </a>

          <!-- 加载中：占位但禁用，防重复点击 -->
          <a v-else class="dl-btn" aria-disabled="true" tabindex="-1">
            <SiteIcon name="download" :size="20" />
            <span class="dl-btn-main">
              <span class="dl-label">正在获取下载链接…</span>
              <span class="dl-sub">请稍候</span>
            </span>
          </a>
        </div>

        <p v-if="release?.source === 'proxy'" class="dl-note">
          由本站中转下载，无需访问 GitHub
        </p>

        <p class="dl-ghost">
          <a :href="RELEASES_URL" target="_blank" rel="noopener">
            其他平台 / 历史版本
            <SiteIcon name="external-link" :size="14" />
          </a>
        </p>
      </div>
    </div>
  </section>
</template>

<style scoped>
.download-card {
  padding: clamp(28px, 5vw, 48px);
  border-radius: 18px;
  background:
    radial-gradient(80% 120% at 20% 0%, rgba(217, 126, 61, 0.1), transparent 65%),
    var(--surface);
  border: 1px solid var(--border-strong);
  box-shadow:
    inset 0 1px 0 rgba(255, 250, 244, 0.06),
    0 30px 60px rgba(12, 8, 5, 0.4);
  text-align: center;
}

.dl-head h2 {
  font-size: clamp(1.6rem, 3.6vw, 2.2rem);
  font-weight: 600;
}
.dl-ver {
  margin-top: 10px;
  color: var(--text-dim);
  font-size: 0.98rem;
}
.dl-ver strong {
  font-family: var(--font-display);
  font-weight: 700;
  color: var(--accent);
}
.dl-ver a {
  color: var(--accent);
  border-bottom: 1px dashed rgba(217, 126, 61, 0.5);
}
.dl-ver a:hover {
  color: #ec9a5c;
}

.dl-buttons {
  margin-top: 30px;
  display: flex;
  justify-content: center;
  gap: 16px;
  flex-wrap: wrap;
}

/* 下载卡片按钮：两行（名称 + 体积） */
.dl-btn {
  display: inline-flex;
  align-items: center;
  gap: 14px;
  min-width: 236px;
  padding: 16px 22px;
  border-radius: 14px;
  text-align: left;
  text-decoration: none;
  background: var(--bg-soft);
  border: 1px solid var(--border-strong);
  box-shadow: inset 0 1px 0 rgba(255, 250, 244, 0.05);
  color: var(--text);
  transition:
    transform var(--d-fast) var(--ease-out),
    border-color var(--d-fast) var(--ease-out),
    background-color var(--d-fast) var(--ease-out);
}
.dl-btn.primary {
  background: linear-gradient(180deg, #e68a4a, var(--accent));
  border-color: transparent;
  color: var(--accent-ink);
  box-shadow:
    inset 0 1px 0 rgba(255, 235, 210, 0.4),
    0 12px 28px rgba(217, 126, 61, 0.28);
}
.dl-btn:hover {
  transform: translateY(-2px);
}
.dl-btn.primary:hover {
  filter: brightness(1.04);
}
.dl-btn[aria-disabled="true"] {
  opacity: 0.6;
  cursor: default;
  transform: none;
}
.dl-btn-main {
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.dl-label {
  font-size: 1rem;
  font-weight: 700;
}
.dl-sub {
  font-size: 0.76rem;
  opacity: 0.72;
  font-variant-numeric: tabular-nums;
}

.dl-note {
  margin-top: 16px;
  font-size: 0.8rem;
  color: var(--text-faint);
  letter-spacing: 0.02em;
}

.dl-ghost {
  margin-top: 22px;
}
.dl-ghost a {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-dim);
  font-size: 0.9rem;
}
.dl-ghost a:hover {
  color: var(--accent);
}

@media (max-width: 520px) {
  .dl-buttons {
    flex-direction: column;
    align-items: stretch;
  }
  .dl-btn {
    width: 100%;
    min-width: 0;
  }
}
</style>

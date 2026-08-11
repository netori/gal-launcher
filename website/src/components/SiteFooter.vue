<script setup lang="ts">
/** 页脚：GitHub + 开源免费 + 免责声明 */
import { REPO_URL } from "../data/site";
import BrandLogo from "./BrandLogo.vue";
import SiteIcon from "./SiteIcon.vue";
</script>

<template>
  <footer class="footer">
    <div class="container">
      <div class="footer-top">
        <div class="footer-brand">
          <BrandLogo :size="30" />
          <div class="fb-text">
            <span class="fb-name">GAL 启动器</span>
            <span class="fb-tag"><i></i>开源 · 免费</span>
          </div>
        </div>
        <nav class="footer-links">
          <a :href="REPO_URL" target="_blank" rel="noopener">
            GitHub 仓库
            <SiteIcon name="external-link" :size="14" />
          </a>
          <a :href="REPO_URL + '/issues'" target="_blank" rel="noopener">
            反馈 Bug
            <SiteIcon name="external-link" :size="14" />
          </a>
        </nav>
      </div>

      <div class="footer-legal">
        <p>
          本软件为本地工具与导航，站内提及的资源版权归原作者所有，请支持正版。
        </p>
        <p>© 2026 GAL 启动器</p>
      </div>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  margin-top: clamp(3rem, 6vw, 5rem);
  border-top: 1px solid var(--border);
  padding: 40px 0 44px;
  background: linear-gradient(180deg, transparent, rgba(20, 12, 6, 0.4));
}
.footer-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
  flex-wrap: wrap;
}
.footer-brand {
  display: flex;
  align-items: center;
  gap: 14px;
}
.fb-text {
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.fb-name {
  font-family: var(--font-display);
  font-size: 1.05rem;
  font-weight: 700;
  color: var(--text);
}
.fb-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 0.76rem;
  color: var(--text-dim);
}
.fb-tag i {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--accent);
  box-shadow: 0 0 8px rgba(217, 126, 61, 0.6);
}
.footer-links {
  display: flex;
  gap: 26px;
}
.footer-links a {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-dim);
  font-size: 0.92rem;
}
.footer-links a:hover {
  color: var(--accent);
}
.footer-legal {
  margin-top: 30px;
  padding-top: 20px;
  border-top: 1px solid var(--border);
  display: flex;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  font-size: 0.8rem;
  color: var(--text-faint);
}
.footer-legal p {
  max-width: 52ch;
}
</style>

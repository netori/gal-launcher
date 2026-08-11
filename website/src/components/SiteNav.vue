<script setup lang="ts">
/** 固定顶栏：漂浮药丸形态，半透明暖底 + 顶内高光发丝线 */
import { NAV_LINKS, REPO_URL } from "../data/site";
import BrandLogo from "./BrandLogo.vue";
</script>

<template>
  <header class="site-nav">
    <div class="nav-pill">
      <a class="nav-brand" href="#top">
        <BrandLogo :size="26" />
        <span class="nav-name">GAL 启动器</span>
      </a>
      <nav class="nav-links" aria-label="站内导航">
        <a v-for="l in NAV_LINKS" :key="l.href" :href="l.href">{{ l.label }}</a>
      </nav>
      <a
        class="nav-github"
        :href="REPO_URL"
        target="_blank"
        rel="noopener"
      >GitHub</a>
    </div>
  </header>
</template>

<style scoped>
.site-nav {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 40;
  display: flex;
  justify-content: center;
  padding-top: 14px;
  pointer-events: none;
}
.site-nav a {
  pointer-events: auto;
}
.nav-pill {
  display: flex;
  align-items: center;
  gap: 20px;
  width: min(1120px, 100% - 48px);
  height: 52px;
  padding: 0 10px 0 16px;
  border-radius: 999px;
  background: rgba(22, 18, 15, 0.72);
  border: 1px solid var(--border-strong);
  box-shadow:
    inset 0 1px 0 rgba(255, 250, 244, 0.07),
    0 12px 32px rgba(16, 10, 6, 0.5);
  backdrop-filter: blur(14px);
  -webkit-backdrop-filter: blur(14px);
}
.nav-brand {
  display: inline-flex;
  align-items: center;
  gap: 10px;
  color: var(--text);
  font-family: var(--font-display);
  font-weight: 700;
  font-size: 1.05rem;
  letter-spacing: 0.01em;
}
.nav-brand:hover {
  color: var(--text);
}
.nav-links {
  display: flex;
  gap: 2px;
  margin-left: auto;
}
.nav-links a {
  color: var(--text-dim);
  font-size: 0.92rem;
  padding: 6px 12px;
  border-radius: 999px;
  transition:
    color var(--d-fast) var(--ease-out),
    background-color var(--d-fast) var(--ease-out);
}
.nav-links a:hover {
  color: var(--text);
  background: rgba(255, 250, 244, 0.05);
}
.nav-github {
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--text);
  border: 1px solid var(--border-strong);
  padding: 7px 16px;
  border-radius: 999px;
  transition:
    border-color var(--d-fast) var(--ease-out),
    color var(--d-fast) var(--ease-out),
    background-color var(--d-fast) var(--ease-out);
}
.nav-github:hover {
  color: var(--accent);
  border-color: rgba(217, 126, 61, 0.45);
  background: rgba(217, 126, 61, 0.06);
}

@media (max-width: 960px) {
  .nav-links {
    display: none;
  }
}
</style>

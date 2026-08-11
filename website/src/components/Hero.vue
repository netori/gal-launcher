<script setup lang="ts">
/** Hero：居中编辑式。手柄 logo（紫粉原色）置于暖琥珀光晕圆盘，大标题 + 双按钮 + 键盘意象条 */
import { REPO_URL } from "../data/site";
import BrandLogo from "./BrandLogo.vue";
import SiteIcon from "./SiteIcon.vue";
</script>

<template>
  <section id="top" class="hero">
    <div class="hero-inner container">
      <!-- 记忆点：logo 浮在暖琥珀光晕上 -->
      <div class="hero-emblem">
        <div class="hero-glow" aria-hidden="true"></div>
        <BrandLogo :size="112" />
      </div>

      <h1 class="hero-title">
        把本地 galgame 变成一座<em>图书馆</em>
      </h1>

      <p class="hero-tag">扫描识别 · 封面整理 · 转区启动 · 时长统计 · 资源解包</p>

      <div class="hero-cta">
        <a class="btn btn-primary btn-lg" href="#download">
          下载 Windows 版
          <SiteIcon name="download" :size="18" />
        </a>
        <a
          class="btn btn-secondary btn-lg"
          :href="REPO_URL"
          target="_blank"
          rel="noopener"
        >
          GitHub
          <SiteIcon name="external-link" :size="16" />
        </a>
      </div>

      <!-- 键盘意象条：呼应「启动器」与游戏键位 -->
      <div class="hero-keys" aria-hidden="true">
        <span class="key">ESC</span>
        <span class="key">空格</span>
        <span class="key">ENTER</span>
        <span class="key">←</span>
        <span class="key">→</span>
      </div>
    </div>
  </section>
</template>

<style scoped>
.hero {
  position: relative;
  display: flex;
  min-height: 100dvh;
  padding: 110px 0 48px;
}
.hero-inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  justify-content: center;
}

/* logo 光晕圆盘 */
.hero-emblem {
  position: relative;
  margin-bottom: clamp(1.6rem, 4vw, 2.6rem);
}
.hero-glow {
  position: absolute;
  inset: -52%;
  border-radius: 50%;
  background: radial-gradient(
    circle at 50% 45%,
    rgba(217, 126, 61, 0.32),
    rgba(217, 126, 61, 0.12) 45%,
    transparent 70%
  );
  filter: blur(2px);
  animation: glow-breathe 7s var(--ease-in-out) infinite;
}
@keyframes glow-breathe {
  0%,
  100% {
    opacity: 0.85;
    transform: scale(1);
  }
  50% {
    opacity: 1;
    transform: scale(1.05);
  }
}
@media (prefers-reduced-motion: reduce) {
  .hero-glow {
    animation: none;
  }
}
.hero-emblem :deep(.brand-logo) {
  position: relative;
  border-radius: 22px;
}

/* 大标题 */
.hero-title {
  font-size: clamp(2.3rem, 6.4vw, 4.4rem);
  font-weight: 700;
  line-height: 1.08;
  letter-spacing: -0.015em;
  color: var(--text);
  text-wrap: balance;
}
.hero-title em {
  font-style: italic;
  font-weight: 600;
  color: var(--accent);
  font-family: var(--font-display);
}

.hero-tag {
  margin-top: clamp(1.1rem, 2.6vw, 1.6rem);
  color: var(--text-dim);
  font-size: clamp(0.95rem, 1.6vw, 1.08rem);
  letter-spacing: 0.04em;
}

.hero-cta {
  margin-top: clamp(1.7rem, 3.6vw, 2.3rem);
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 0.8rem;
}

/* 键盘意象条 */
.hero-keys {
  margin-top: clamp(3rem, 7vh, 5rem);
  display: flex;
  gap: 8px;
  opacity: 0.75;
}
.key {
  font-family: var(--font-sans);
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  color: var(--text-dim);
  min-width: 40px;
  padding: 7px 10px;
  text-align: center;
  border-radius: 7px;
  background: linear-gradient(180deg, var(--surface-2), var(--surface));
  border: 1px solid var(--border-strong);
  box-shadow:
    inset 0 1px 0 rgba(255, 250, 244, 0.08),
    0 3px 0 rgba(0, 0, 0, 0.35);
  user-select: none;
}

@media (max-width: 640px) {
  .key {
    min-width: 34px;
    padding: 6px 6px;
    font-size: 0.66rem;
  }
}
</style>

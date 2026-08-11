<script setup lang="ts">
/** 特性区：编辑式索引行（非「居中小图标 + 圆角卡片」套路） */
import { FEATURES } from "../data/site";
import FeatureIcon from "./FeatureIcon.vue";
import SiteIcon from "./SiteIcon.vue";
</script>

<template>
  <section id="features" class="section">
    <div class="container">
      <div class="section-head" v-reveal>
        <h2>该做的，它都帮你做了</h2>
        <p>从扫描入库到日常管理，覆盖本地 galgame 收藏的完整链路，全部在本地完成。</p>
      </div>

      <div class="features-grid">
        <article
          v-for="(f, i) in FEATURES"
          :key="f.title"
          class="feat"
          v-reveal="Math.floor(i / 2)"
        >
          <div class="feat-side">
            <FeatureIcon :name="f.icon" :size="30" />
            <span class="feat-num">{{ String(i + 1).padStart(2, "0") }}</span>
          </div>
          <div class="feat-body">
            <h3>{{ f.title }}</h3>
            <p>{{ f.desc }}</p>
          </div>
          <span class="feat-arrow">
            <SiteIcon name="arrow-right" :size="18" />
          </span>
        </article>
      </div>
    </div>
  </section>
</template>

<style scoped>
.features-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  column-gap: clamp(32px, 6vw, 64px);
}

.feat {
  position: relative;
  display: grid;
  grid-template-columns: 48px 1fr auto;
  gap: 16px;
  align-items: start;
  padding: 30px 8px 30px 4px;
  border-top: 1px solid var(--border);
  transition: background-color var(--d-base) var(--ease-out);
}
.features-grid .feat:first-child,
.features-grid .feat:nth-child(2) {
  border-top-color: var(--border-strong);
}
.feat::before {
  content: "";
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  border-radius: 2px;
  background: var(--accent);
  opacity: 0;
  transform: scaleY(0.4);
  transition:
    opacity var(--d-base) var(--ease-out),
    transform var(--d-base) var(--ease-out);
}
.feat:hover::before {
  opacity: 1;
  transform: scaleY(1);
}
.feat:hover {
  background: linear-gradient(90deg, rgba(217, 126, 61, 0.055), transparent 70%);
}

.feat-side {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  color: var(--text-faint);
  transition: color var(--d-base) var(--ease-out);
}
.feat:hover .feat-side {
  color: var(--accent);
}
.feat-num {
  font-family: var(--font-sans);
  font-size: 0.7rem;
  font-weight: 500;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.08em;
}

.feat-body h3 {
  font-size: 1.16rem;
  font-weight: 600;
  color: var(--text);
  transition: color var(--d-base) var(--ease-out);
}
.feat:hover .feat-body h3 {
  color: #ffd9b8;
}
.feat-body p {
  margin-top: 5px;
  color: var(--text-dim);
  font-size: 0.94rem;
  line-height: 1.65;
  max-width: 34ch;
}

.feat-arrow {
  margin-top: 6px;
  color: var(--accent);
  opacity: 0;
  transform: translateX(-6px);
  transition:
    opacity var(--d-base) var(--ease-out),
    transform var(--d-base) var(--ease-out);
}
.feat:hover .feat-arrow {
  opacity: 1;
  transform: none;
}

@media (max-width: 760px) {
  .features-grid {
    grid-template-columns: 1fr;
  }
  .features-grid .feat:nth-child(2) {
    border-top-color: var(--border);
  }
  .feat {
    grid-template-columns: 44px 1fr;
    padding-block: 24px;
  }
  .feat-arrow {
    display: none;
  }
}
</style>

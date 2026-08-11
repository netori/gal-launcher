<script setup lang="ts">
/** 资源站：常用社区/补丁站快捷链接 */
import { RESOURCES } from "../data/site";
import SiteIcon from "./SiteIcon.vue";
</script>

<template>
  <section id="resources" class="section">
    <div class="container">
      <div class="section-head" v-reveal>
        <h2>常用的资源站，替你放在这里</h2>
        <p>从资讯、补丁到数据库，收藏 galgame 玩家常用到的公开站点。</p>
      </div>

      <ul class="res-list" v-reveal>
        <li v-for="r in RESOURCES" :key="r.url">
          <a
            class="res-row"
            :href="r.url"
            target="_blank"
            rel="noopener"
          >
            <span class="res-name">{{ r.name }}</span>
            <span class="res-domain">{{ r.domain }}</span>
            <span class="res-desc">{{ r.desc }}</span>
            <span class="res-arrow">
              <SiteIcon name="external-link" :size="15" />
            </span>
          </a>
        </li>
      </ul>

      <p class="res-note muted">以上为社区公开站点，仅供个人收藏参考。</p>
    </div>
  </section>
</template>

<style scoped>
.res-list {
  list-style: none;
}
.res-row {
  display: grid;
  grid-template-columns: 150px 150px 1fr auto;
  align-items: center;
  gap: 20px;
  padding: 18px 12px 18px 4px;
  border-top: 1px solid var(--border);
  transition: background-color var(--d-base) var(--ease-out);
}
.res-list li:first-child .res-row {
  border-top-color: var(--border-strong);
}
.res-row:hover {
  background: linear-gradient(90deg, rgba(217, 126, 61, 0.05), transparent 70%);
}
.res-name {
  font-family: var(--font-display);
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--text);
  transition: color var(--d-fast) var(--ease-out);
}
.res-row:hover .res-name {
  color: var(--accent);
}
.res-domain {
  font-size: 0.84rem;
  color: var(--text-faint);
  font-variant-numeric: tabular-nums;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.res-desc {
  font-size: 0.92rem;
  color: var(--text-dim);
}
.res-arrow {
  color: var(--accent);
  opacity: 0;
  transform: translate(-5px, 5px);
  transition:
    opacity var(--d-base) var(--ease-out),
    transform var(--d-base) var(--ease-out);
}
.res-row:hover .res-arrow {
  opacity: 1;
  transform: none;
}

.res-note {
  margin-top: 18px;
  font-size: 0.84rem;
}

@media (max-width: 760px) {
  .res-row {
    grid-template-columns: auto 1fr auto;
    gap: 6px 14px;
    padding-block: 16px;
  }
  .res-desc {
    grid-column: 1 / -1;
    grid-row: 2;
    font-size: 0.86rem;
    padding-left: 2px;
  }
  .res-arrow {
    opacity: 1;
    transform: none;
  }
  .res-domain {
    justify-self: start;
  }
}
</style>

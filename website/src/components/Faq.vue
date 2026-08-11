<script setup lang="ts">
/** FAQ：原生 details/summary 折叠，零 JS */
import { FAQS } from "../data/site";
import SiteIcon from "./SiteIcon.vue";
</script>

<template>
  <section id="faq" class="section">
    <div class="container">
      <div class="section-head" v-reveal>
        <h2>常见问题</h2>
        <p>转区、解包、备份与反馈，先看这里。</p>
      </div>

      <div class="faq-list" v-reveal>
        <details v-for="f in FAQS" :key="f.q">
          <summary>
            <span>{{ f.q }}</span>
            <SiteIcon name="chevron-down" :size="18" />
          </summary>
          <p>{{ f.a }}</p>
        </details>
      </div>
    </div>
  </section>
</template>

<style scoped>
.faq-list {
  max-width: 720px;
}
details {
  border-top: 1px solid var(--border);
}
details:first-child {
  border-top-color: var(--border-strong);
}
details:last-child {
  border-bottom: 1px solid var(--border);
}
summary {
  list-style: none;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 20px 6px;
  cursor: pointer;
  font-family: var(--font-display);
  font-size: 1.08rem;
  font-weight: 600;
  color: var(--text);
  transition: color var(--d-fast) var(--ease-out);
  user-select: none;
}
summary::-webkit-details-marker {
  display: none;
}
summary:hover {
  color: var(--accent);
}
summary svg {
  color: var(--text-faint);
  transition: transform var(--d-base) var(--ease-out), color var(--d-fast) var(--ease-out);
  flex-shrink: 0;
}
details[open] summary svg {
  transform: rotate(180deg);
  color: var(--accent);
}
details p {
  padding: 0 6px 22px;
  color: var(--text-dim);
  font-size: 0.95rem;
  line-height: 1.75;
  max-width: 60ch;
}
</style>

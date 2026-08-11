<script setup lang="ts">
/** 更新日志：渲染同一 release 的 body；失败或空 body 时整个区块隐藏（v-if），不留空壳。 */
import { computed } from "vue";
import { useGithubRelease } from "../composables/useGithubRelease";
import { renderChangelog } from "../lib/renderMarkdown";

const { state, release } = useGithubRelease();

const body = computed(
  () => (state.value === "ok" && release.value?.body ? release.value.body : null),
);
const html = computed(() => (body.value ? renderChangelog(body.value) : ""));
</script>

<template>
  <section v-if="body" id="changelog" class="section">
    <div class="container">
      <div class="section-head" v-reveal>
        <h2>更新日志</h2>
        <p>最近一次发布（{{ release?.tag_name }}）都改了什么。</p>
      </div>
      <div class="changelog" v-reveal v-html="html"></div>
    </div>
  </section>
</template>

<style scoped>
.changelog {
  max-width: 720px;
}
.changelog :deep(h3) {
  font-family: var(--font-display);
  font-size: 1.05rem;
  font-weight: 600;
  color: var(--accent);
  margin: 30px 0 12px;
}
.changelog :deep(h3:first-child) {
  margin-top: 0;
}
.changelog :deep(p) {
  color: var(--text-dim);
  font-size: 0.94rem;
  margin: 8px 0;
}
.changelog :deep(ul) {
  list-style: none;
  margin: 4px 0 12px;
}
.changelog :deep(li) {
  position: relative;
  padding: 7px 0 7px 22px;
  color: var(--text-dim);
  font-size: 0.94rem;
  border-top: 1px solid var(--border);
}
.changelog :deep(li:first-child) {
  border-top: none;
}
.changelog :deep(li)::before {
  content: "";
  position: absolute;
  left: 4px;
  top: 16px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--accent);
  opacity: 0.85;
}
.changelog :deep(code) {
  font-family: ui-monospace, "Cascadia Mono", Consolas, monospace;
  font-size: 0.86em;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 5px;
  padding: 1px 6px;
  color: #ffd9b8;
}
.changelog :deep(a) {
  color: var(--accent);
  text-decoration: underline;
  text-underline-offset: 3px;
}
.changelog :deep(strong) {
  color: var(--text);
  font-weight: 600;
}
</style>

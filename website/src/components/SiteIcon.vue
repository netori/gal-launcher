<script setup lang="ts">
/** 全站小图标：24×24 描边 SVG，继承 currentColor（风格同产品 Icon.vue） */
import { computed } from "vue";

const props = withDefaults(defineProps<{ name: string; size?: number }>(), {
  size: 16,
});

const PATHS: Record<string, string> = {
  "external-link": "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14 21 3",
  "chevron-down": "m6 9 6 6 6-6",
  "arrow-right": "M5 12h14m0 0-6-6m6 6-6 6",
  download: "M12 4v12m0 0 4-4m-4 4-4-4M4 20h16",
  "arrow-up-right": "M7 17 17 7m0 0H8m9 0v9",
};

const d = computed(() => PATHS[props.name] ?? PATHS["external-link"]);
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    focusable="false"
    style="display: inline-block; vertical-align: -0.15em; flex-shrink: 0"
  >
    <path :d="d" />
  </svg>
</template>

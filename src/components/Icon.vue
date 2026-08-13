<script setup lang="ts">
/**
 * 全站统一图标：24×24 描边 SVG，继承父级颜色（currentColor）。
 * 名称查 PATHS；星形（star）默认实心，可传 filled=false 变描边。
 */
import { computed } from "vue";

const props = withDefaults(defineProps<{ name: string; size?: number; filled?: boolean }>(), {
  size: 16,
});

const PATHS: Record<string, string> = {
  search: "M11 17a6 6 0 1 0 0-12 6 6 0 0 0 0 12ZM21 21l-4.35-4.35",
  plus: "M12 5v14M5 12h14",
  sliders: "M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6",
  image: "M3 5h18v14H3ZM8.5 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3ZM21 15l-5-5L6 19",
  folder: "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z",
  file: "M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z M14 2v6h6",
  "hard-drive": "M22 12H2M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11ZM6 16h.01M10 16h.01",
  play: "M7 5l12 7-12 7Z",
  star: "M12 3l2.8 5.9 6.5.65-4.85 4.35 1.3 6.4L12 17.1 6.25 20.3l1.3-6.4L2.7 9.55l6.5-.65Z",
  eye: "M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7ZM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6Z",
  "eye-off": "M2 12s3-6.5 10-6.5 10 6.5 10 6.5-3 6.5-10 6.5-10-6.5-10-6.5ZM9.9 9.9a3 3 0 0 0 4.24 4.24M3 3l18 18",
  trash: "M3 6h18M8 6V4a1 1 0 0 1 1-1h6a1 1 0 0 1 1 1v2M19 6l-1.1 14a2 2 0 0 1-2 1.9H8.1a2 2 0 0 1-2-1.9L5 6M10 11v6M14 11v6",
  close: "M18 6 6 18M6 6l12 12",
  globe: "M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18ZM3 12h18M12 3a15.3 15.3 0 0 1 4 9 15.3 15.3 0 0 1-4 9 15.3 15.3 0 0 1-4-9 15.3 15.3 0 0 1 4-9Z",
  box: "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16ZM3.27 6.96 12 12.01l8.73-5.05M12 22.08V12",
  cloud: "M17.5 19a4.5 4.5 0 1 0-.83-8.94A6 6 0 1 0 6.5 19h11Z",
  "chevron-down": "m6 9 6 6 6-6",
  check: "m5 12 5 5 9-10",
  upload: "M12 16V4m0 0 4 4m-4-4L8 8M4 20h16",
  "arrow-up": "M12 19V5m-7 7 7-7 7 7",
  download: "M12 4v12m0 0 4-4m-4 4-4-4M4 20h16",
  "external-link": "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14 21 3",
};

const isFilled = computed(() => props.filled || props.name === "star");
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    :fill="isFilled ? 'currentColor' : 'none'"
    :stroke="isFilled ? 'none' : 'currentColor'"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    focusable="false"
    style="display: inline-block; vertical-align: -0.15em; flex-shrink: 0"
  >
    <path :d="PATHS[name] ?? PATHS['plus']" />
  </svg>
</template>
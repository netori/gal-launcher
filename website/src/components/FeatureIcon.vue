<script setup lang="ts">
/**
 * 特性区图标：24×24 描边 SVG，继承 currentColor。
 * 视觉语言与产品 Icon.vue 一致（stroke 1.8 / round）。
 * play / box / download 复用产品已验证 path，其余为同风格手绘。
 */
withDefaults(defineProps<{ name: string; size?: number }>(), { size: 32 });
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
    style="display: block; flex-shrink: 0"
  >
    <!-- 扫描与引擎识别：同心圆靶心 -->
    <template v-if="name === 'scan'">
      <circle cx="12" cy="12" r="9" />
      <circle cx="12" cy="12" r="4.5" />
      <circle cx="12" cy="12" r="0.75" fill="currentColor" />
    </template>

    <!-- 封面墙：4 宫格 -->
    <template v-else-if="name === 'grid'">
      <rect x="3.2" y="3.2" width="7.6" height="7.6" rx="1.6" />
      <rect x="13.2" y="3.2" width="7.6" height="7.6" rx="1.6" />
      <rect x="3.2" y="13.2" width="7.6" height="7.6" rx="1.6" />
      <rect x="13.2" y="13.2" width="7.6" height="7.6" rx="1.6" />
    </template>

    <!-- 一键启动：播放键（产品同款） -->
    <template v-else-if="name === 'play'">
      <path d="M7 5l12 7-12 7Z" />
    </template>

    <!-- 游玩统计：柱状图 -->
    <template v-else-if="name === 'chart'">
      <path d="M4 4v16h16" />
      <path d="M9 16v-4M14 16V8M18.5 16v-2" />
    </template>

    <!-- VNDB 元数据：翻开的书 -->
    <template v-else-if="name === 'book'">
      <path d="M4 5a2 2 0 0 1 2-2h4v14H6a2 2 0 0 0-2 2Z" />
      <path d="M20 5a2 2 0 0 0-2-2h-4v14h4a2 2 0 0 1 2 2Z" />
    </template>

    <!-- 补丁管理：折角标签 -->
    <template v-else-if="name === 'tag'">
      <path
        d="M20.6 13.6 12 22.2a1.4 1.4 0 0 1-2 0L3 15.2a1.4 1.4 0 0 1 0-2l8.6-8.6a1.4 1.4 0 0 1 1-.4H19a2 2 0 0 1 2 2v5a1.4 1.4 0 0 1-.4 1Z"
      />
      <circle cx="15.8" cy="8.2" r="0.5" fill="currentColor" />
    </template>

    <!-- 资源解包：箱子（产品同款） -->
    <template v-else-if="name === 'box'">
      <path
        d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16ZM3.27 6.96 12 12.01l8.73-5.05M12 22.08V12"
      />
    </template>

    <!-- 备份恢复：下载（产品同款） -->
    <template v-else-if="name === 'download'">
      <path d="M12 4v12m0 0 4-4m-4 4-4-4M4 20h16" />
    </template>

    <!-- 兜底 -->
    <template v-else>
      <circle cx="12" cy="12" r="9" />
      <path d="M12 8v8M8 12h8" />
    </template>
  </svg>
</template>

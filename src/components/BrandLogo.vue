<template>
  <!-- 品牌图标：应用真实 logo（游戏手柄款，保持原色）。圆角裁切 + 细描边，融入暖暗色界面 -->
  <span class="brand-logo" :style="{ width: size + 'px', height: size + 'px' }" aria-hidden="true">
    <img src="../assets/brand-logo.png" alt="GAL 启动器" draggable="false" />
  </span>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ size?: number }>(), { size: 30 });
</script>

<style scoped>
.brand-logo {
  display: inline-block;
  border-radius: 9px;
  overflow: hidden;
  flex-shrink: 0;
  border: 1px solid rgba(255, 250, 244, 0.12);
  box-shadow:
    0 4px 14px rgba(20, 12, 6, 0.55),
    inset 0 0 0 1px rgba(255, 255, 255, 0.03);
}
.brand-logo img {
  display: block;
  width: 100%;
  height: 100%;
  object-fit: cover;
  user-select: none;
  -webkit-user-drag: none;
}
</style>

<script setup lang="ts">
import FileBrowser from "./FileBrowser.vue";

defineProps<{ root: string }>();
const emit = defineEmits<{ picked: [path: string]; close: [] }>();
</script>

<template>
  <FileBrowser
    :root="root"
    title="选择启动程序（exe）"
    mode="file"
    :exts="['exe']"
    @picked="(p) => emit('picked', p)"
    @close="emit('close')"
  />
</template>

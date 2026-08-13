<script setup lang="ts">
import FileBrowser from "./FileBrowser.vue";

defineProps<{ root: string; title?: string }>();
const emit = defineEmits<{ picked: [path: string]; close: [] }>();
</script>

<template>
  <FileBrowser
    :root="root"
    :title="title ?? '选择目录'"
    mode="dir"
    @picked="(p) => emit('picked', p)"
    @close="emit('close')"
  />
</template>

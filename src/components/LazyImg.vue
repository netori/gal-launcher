<script setup lang="ts">
import { onMounted, ref } from "vue";
import type { AssetEntry } from "../api";

const props = defineProps<{
  entry: AssetEntry;
  srcFn: (e: AssetEntry) => Promise<string | null>;
}>();

const src = ref("");

onMounted(async () => {
  const s = await props.srcFn(props.entry);
  if (s) src.value = s;
});
</script>

<template>
  <img v-if="src" class="thumb-img" :src="src" loading="lazy" :alt="entry.rel" />
  <div v-else class="thumb-img"></div>
</template>
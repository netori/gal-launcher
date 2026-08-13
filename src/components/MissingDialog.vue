<script setup lang="ts">
import { onMounted, ref } from "vue";
import { api, type Game } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const emit = defineEmits<{ close: []; removed: [] }>();
useCloseOnEscape(() => emit("close"));

const loading = ref(true);
const missing = ref<Game[]>([]);
const err = ref("");
const busy = ref(false);

onMounted(async () => {
  try {
    missing.value = await api.checkMissing();
  } catch (e) {
    err.value = String(e);
  } finally {
    loading.value = false;
  }
});

async function removeOne(g: Game) {
  busy.value = true;
  err.value = "";
  try {
    await api.removeFromLibrary(g.id);
    missing.value = missing.value.filter((m) => m.id !== g.id);
    emit("removed");
  } catch (e) {
    err.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function removeAll() {
  busy.value = true;
  err.value = "";
  try {
    for (const g of [...missing.value]) {
      await api.removeFromLibrary(g.id);
    }
    missing.value = [];
    emit("removed");
  } catch (e) {
    err.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(560px, 92vw)">
      <div class="head">
        <h2>失效游戏</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <p class="hint">
          以下游戏的目录已不存在（被移动或删除）。启动会失败，建议从库里移除记录；若只是移动了位置，请重新扫描导入。
        </p>

        <div v-if="loading" class="center-h" style="padding: 24px"><div class="spinner"></div></div>

        <div v-else-if="!missing.length" class="empty" style="height: auto; padding: 24px 0">
          <span class="glyph-icon"><Icon name="check" :size="28" /></span>
          <p style="color: var(--text-dim)">没有失效游戏，库内目录都完好。</p>
        </div>

        <div v-else class="missing-list">
          <div v-for="g in missing" :key="g.id" class="mrow">
            <div style="flex: 1; min-width: 0">
              <div class="nm">{{ g.title }}</div>
              <div class="dir">{{ g.sourceDir }}</div>
            </div>
            <button class="btn small danger" :disabled="busy" @click="removeOne(g)">移除记录</button>
          </div>
        </div>

        <div v-if="err" class="toast err" style="margin-top: 10px">{{ err }}</div>
      </div>

      <div class="foot" v-if="missing.length">
        <button class="btn danger" :disabled="busy" @click="removeAll">移除全部失效记录</button>
        <div class="spacer"></div>
        <button class="btn" @click="emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.missing-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.mrow {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.nm {
  font-size: 13.5px;
  font-weight: 600;
}
.dir {
  font-size: 11px;
  color: var(--text-faint);
  word-break: break-all;
  margin-top: 2px;
}
</style>

<script setup lang="ts">
/**
 * 应用内轻量目录选择器：替代原生「选择文件夹」对话框。
 * 原生目录对话框在定位到巨型目录时会卡死整个窗口；本组件只列当前目录的子目录，
 * 进入/上级/粘贴路径/确认，全程轻量流畅。
 */
import { computed, onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ root: string; title?: string }>();
const emit = defineEmits<{ picked: [path: string]; close: [] }>();
useCloseOnEscape(() => emit("close"));

const cur = ref("");
const dirs = ref<string[]>([]);
const pathInput = ref("");
const err = ref("");
const loading = ref(false);

const sep = () => (props.root.includes("\\") ? "\\" : "/");
const isDriveRoot = computed(() => /^[A-Za-z]:[\\/]?$/.test(cur.value));

onMounted(() => enter(props.root));

async function enter(dir: string) {
  cur.value = dir;
  pathInput.value = dir;
  loading.value = true;
  err.value = "";
  dirs.value = [];
  try {
    dirs.value = await invoke<string[]>("list_directory", { path: dir });
  } catch (e) {
    err.value = String(e);
  } finally {
    loading.value = false;
  }
}

function goUp() {
  if (isDriveRoot.value) return;
  const s = sep();
  const idx = cur.value.lastIndexOf(s);
  if (idx <= 0) return;
  let parent = cur.value.slice(0, idx);
  if (/^[A-Za-z]:$/.test(parent)) parent += s; // "C:" → "C:\"
  enter(parent);
}

function joinPath(a: string, b: string) {
  return `${a}${sep()}${b}`;
}

function onPathEnter() {
  const p = pathInput.value.trim();
  if (p) enter(p);
}

function confirmDir() {
  emit("picked", cur.value);
}

function fmtTrailing(p: string): string {
  const b = p.replace(/\\+$/, "").replace(/\/+$/, "");
  return b.split(/[\\/]/).pop() ?? p;
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(640px, 92vw)">
      <div class="head">
        <h2>{{ props.title ?? "选择目录" }}</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <div class="crumb" :title="cur">
          <button class="btn small ghost" @click="goUp" :disabled="isDriveRoot">↑ 上级</button>
          <span class="crumb-path">{{ cur }}</span>
        </div>
        <input
          v-model="pathInput"
          type="text"
          placeholder="也可以直接粘贴完整路径后回车"
          @keyup.enter="onPathEnter"
        />

        <div v-if="err" class="toast err">{{ err }}</div>
        <div v-if="loading" class="muted" style="padding: 12px">加载中…</div>

        <div v-if="!loading" class="browser">
          <button class="row dir" v-for="d in dirs" :key="d" @click="enter(joinPath(cur, d))">
            <Icon name="folder" :size="14" /><span class="nm">{{ d }}</span>
          </button>
          <div v-if="!dirs.length && !err" class="muted" style="padding: 12px">
            此目录没有子目录，可直接「选择此目录」。
          </div>
        </div>
      </div>

      <div class="foot">
        <span class="muted" style="margin-right: auto">当前：{{ fmtTrailing(cur) }}</span>
        <button class="btn ghost" @click="emit('close')">取消</button>
        <button class="btn primary" @click="confirmDir">选择此目录</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.crumb {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}
.crumb-path {
  font-size: 12px;
  color: var(--text-dim);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
}
.browser {
  margin-top: 10px;
  max-height: 46vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 8px;
  cursor: pointer;
  text-align: left;
  font-size: 13px;
}
.row:hover {
  border-color: var(--accent);
}
.nm {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
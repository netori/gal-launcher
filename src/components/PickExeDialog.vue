<script setup lang="ts">
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

/**
 * 应用内轻量文件选择器：替代原生对话框选启动 exe。
 * 原生对话框在定位到巨型游戏目录时会卡死整个窗口，这个组件只列当前目录的子目录和 exe，流畅得多。
 */
const props = defineProps<{ root: string }>();
const emit = defineEmits<{ picked: [path: string]; close: [] }>();
useCloseOnEscape(() => emit("close"));

const cur = ref("");
const dirs = ref<string[]>([]);
const exes = ref<string[]>([]);
const pathInput = ref("");
const err = ref("");
const loading = ref(false);

const sep = () => (props.root.includes("\\") ? "\\" : "/");

onMounted(() => enter(props.root));

async function enter(dir: string) {
  cur.value = dir;
  pathInput.value = dir;
  loading.value = true;
  err.value = "";
  dirs.value = [];
  exes.value = [];
  try {
    dirs.value = await invoke<string[]>("list_directory", { path: dir });
    exes.value = await invoke<string[]>("list_exe_files", { path: dir });
  } catch (e) {
    err.value = String(e);
  } finally {
    loading.value = false;
  }
}

function goUp() {
  const s = sep();
  const idx = cur.value.lastIndexOf(s);
  if (idx > 0) enter(cur.value.slice(0, idx));
}

function joinPath(a: string, b: string) {
  return `${a}${sep()}${b}`;
}

function pickedExe(name: string) {
  emit("picked", joinPath(cur.value, name));
}

function onPathEnter() {
  enter(pathInput.value.trim());
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
        <h2>选择启动程序（exe）</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <div class="crumb" :title="cur">
          <button class="btn small ghost" @click="goUp" :disabled="cur === props.root">↑ 上级</button>
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
          <button
            class="row exe"
            v-for="e in exes"
            :key="e"
            @click="pickedExe(e)"
            :title="joinPath(cur, e)"
          >
            <Icon name="play" :size="13" /><span class="nm">{{ e }}</span>
          </button>
          <div v-if="!dirs.length && !exes.length" class="muted" style="padding: 12px">
            此目录没有子目录或 exe。
          </div>
        </div>
      </div>

      <div class="foot">
        <span class="muted" style="margin-right: auto">当前：{{ fmtTrailing(cur) }}</span>
        <button class="btn ghost" @click="emit('close')">取消</button>
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
.row.exe {
  color: #d9a25e;
}
.nm {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
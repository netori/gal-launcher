<script setup lang="ts">
import { ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, type Candidate } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [v: boolean];
  imported: [count: number];
}>();
useCloseOnEscape(() => emit("update:modelValue", false));

const root = ref("");
const scanning = ref(false);
const candidates = ref<Candidate[]>([]);
const selected = ref<Set<number>>(new Set());
const importing = ref(false);
const err = ref("");

watch(
  () => props.modelValue,
  (v) => {
    if (v) reset();
  }
);

function reset() {
  root.value = "";
  candidates.value = [];
  selected.value = new Set();
  scanning.value = false;
  importing.value = false;
  err.value = "";
}

async function pickFolder() {
  const dir = await open({ directory: true, multiple: false, title: "选择游戏根目录" });
  if (!dir) return;
  root.value = dir;
  err.value = "";
  scanning.value = true;
  try {
    const list = await api.scanDirectory(dir);
    candidates.value = list;
    selected.value = new Set(list.map((_, i) => i).filter((i) => !list[i].alreadyImported));
  } catch (e) {
    err.value = String(e);
  } finally {
    scanning.value = false;
  }
}

function toggle(i: number) {
  const s = new Set(selected.value);
  if (s.has(i)) s.delete(i);
  else s.add(i);
  selected.value = s;
}

function selAll() {
  selected.value = new Set(candidates.value.map((_, i) => i));
}

const GRADS = [
  "linear-gradient(150deg,#b96a2f,#e08a5e)",
  "linear-gradient(150deg,#8a4b26,#c98a4a)",
  "linear-gradient(150deg,#a3562f,#d9776a)",
  "linear-gradient(150deg,#6f7a3c,#a3c585)",
  "linear-gradient(150deg,#7a4a3a,#b97a5e)",
  "linear-gradient(150deg,#4c4a33,#9a8a5c)",
];
function grad(i: number) {
  return GRADS[i % GRADS.length];
}

async function confirmImport() {
  const chosen = [...selected.value]
    .sort((a, b) => a - b)
    .map((i) => candidates.value[i]);
  if (!chosen.length) return;
  importing.value = true;
  try {
    const count = await api.importGames(chosen);
    emit("imported", count);
    emit("update:modelValue", false);
  } catch (e) {
    err.value = String(e);
  } finally {
    importing.value = false;
  }
}
</script>

<template>
  <div v-if="props.modelValue" class="overlay" @click.self="emit('update:modelValue', false)">
    <div class="modal">
      <div class="head">
        <h2>扫描 &amp; 导入游戏</h2>
        <button class="btn icon-btn ghost" @click="emit('update:modelValue', false)"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <div class="field">
          <label>扫描根目录</label>
          <div class="row">
            <input type="text" :value="root" placeholder="点击浏览选择存放 galgame 的文件夹" disabled />
            <button class="btn" @click="pickFolder" :disabled="scanning || importing">浏览…</button>
          </div>
        </div>

        <div v-if="scanning" class="center-h" style="padding: 30px">
          <div class="spinner"></div>
        </div>

        <div v-else-if="err" class="toast err">{{ err }}</div>

        <template v-else-if="candidates.length">
          <div class="row" style="justify-content: space-between; margin: 4px 0 10px">
            <span class="muted">发现 {{ candidates.length }} 个疑似游戏目录</span>
            <button class="btn small ghost" @click="selAll">全选</button>
          </div>
          <div class="cand-list">
            <div v-for="(c, i) in candidates" :key="c.sourceDir" class="cand">
              <input
                type="checkbox"
                class="ck"
                :checked="selected.has(i)"
                :disabled="c.alreadyImported"
                @change="toggle(i)"
              />
              <div class="cv" :style="{ background: grad(i) }">{{ c.title.charAt(0) }}</div>
              <div style="flex: 1; min-width: 0">
                <div class="nm">{{ c.title }}</div>
                <div class="sub">
                  <span>{{ c.engine }}</span>
                  <span>{{ c.fileCount }} 个文件</span>
                  <span class="already" v-if="c.alreadyImported">已在库中</span>
                </div>
                <div class="dir" :title="c.sourceDir">{{ c.sourceDir }}</div>
              </div>
              <button
                class="btn small"
                :class="{ primary: selected.has(i) && !c.alreadyImported }"
                @click="toggle(i)"
              >
                {{ selected.has(i) ? "入选" : "跳过" }}
              </button>
            </div>
          </div>
        </template>

        <div v-else-if="!scanning" class="muted" style="padding: 10px 2px">
          尚未扫描。点击上方「浏览…」选择一个游戏根目录。
        </div>
      </div>

      <div class="foot">
        <span class="muted" style="margin-right: auto">选中 {{ selected.size }} 项</span>
        <button class="btn" @click="emit('update:modelValue', false)" :disabled="importing">取消</button>
        <button
          class="btn primary"
          :disabled="!selected.size || importing"
          @click="confirmImport"
        >
          {{ importing ? "导入中…" : `导入 ${selected.size} 个游戏` }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.cv {
  display: grid;
  place-items: center;
  color: rgba(255, 255, 255, 0.9);
  font-size: 17px;
  font-weight: 700;
  width: 40px;
  height: 40px;
  border-radius: 10px;
  flex-shrink: 0;
}
</style>
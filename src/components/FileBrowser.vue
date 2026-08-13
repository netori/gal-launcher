<script setup lang="ts">
/**
 * 应用内文件管理器（替代原生目录/文件对话框）。
 * 原生对话框在定位到巨型目录时会卡死整个窗口，故自研一个轻量浏览器：
 * 磁盘切换 + 面包屑导航 + 粘贴路径 + 目录/文件列表（排序）+ 新建文件夹。
 *
 * mode="dir"：选目录（底部「选择此目录」确认当前路径）；
 * mode="file"：选文件（点击文件即选中；`exts` 指定可见扩展名）。
 */
import { computed, onMounted, ref } from "vue";
import { api, type FsEntry } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = withDefaults(
  defineProps<{
    root: string;
    title?: string;
    mode?: "dir" | "file";
    exts?: string[];
  }>(),
  { title: "选择目录", mode: "dir", exts: () => [] }
);
const emit = defineEmits<{ picked: [path: string]; close: [] }>();
useCloseOnEscape(() => emit("close"));

const cur = ref("");
const entries = ref<FsEntry[]>([]);
const truncated = ref(false);
const drives = ref<string[]>([]);
const pathInput = ref("");
const err = ref("");
const loading = ref(false);
const sortKey = ref<"name" | "size" | "modified">("name");
const mkdirOpen = ref(false);
const mkdirName = ref("");

const sep = () => (props.root.includes("\\") ? "\\" : "/");
const isDriveRoot = computed(() => /^[A-Za-z]:[\\/]?$/.test(cur.value));

onMounted(async () => {
  try {
    drives.value = await api.listDrives();
  } catch {
    drives.value = [];
  }
  enter(props.root);
});

async function enter(dir: string) {
  const t = dir.trim();
  if (!t) return;
  cur.value = t;
  pathInput.value = t;
  loading.value = true;
  err.value = "";
  entries.value = [];
  try {
    const l = await api.listDir(t, props.exts.length ? props.exts : undefined);
    entries.value = l.entries;
    truncated.value = l.truncated;
  } catch (e) {
    err.value = String(e);
  } finally {
    loading.value = false;
  }
}

function join(a: string, b: string) {
  const s = sep();
  return `${a.replace(/[\\/]+$/, "")}${s}${b}`;
}

function parentOf(p: string): string {
  const s = sep();
  const b = p.replace(/[\\/]+$/, "");
  const idx = Math.max(b.lastIndexOf("\\"), b.lastIndexOf("/"));
  if (idx <= 0) return b;
  const head = b.slice(0, idx);
  if (/^[A-Za-z]:$/.test(head)) return head + s; // "C:" → "C:\"
  return head;
}

function goUp() {
  if (isDriveRoot.value) return;
  enter(parentOf(cur.value));
}

/** 把当前路径拆成可点击的面包屑段。 */
const crumbs = computed(() => {
  const p = cur.value;
  const s = sep();
  const parts = p.split(/[\\/]/).filter(Boolean);
  const out: { label: string; path: string }[] = [];
  let acc = "";
  for (const part of parts) {
    if (/^[A-Za-z]:$/.test(part)) {
      acc = part + s;
      out.push({ label: acc, path: acc });
    } else if (acc.endsWith(s)) {
      acc += part;
      out.push({ label: part, path: acc });
    } else {
      acc += s + part;
      out.push({ label: part, path: acc });
    }
  }
  return out;
});

const sorted = computed(() => {
  const dirs = entries.value.filter((e) => e.isDir);
  const files = entries.value.filter((e) => !e.isDir);
  const byName = (a: FsEntry, b: FsEntry) => a.name.localeCompare(b.name, "zh-Hans");
  const byKey = (a: FsEntry, b: FsEntry) => {
    if (sortKey.value === "size") return b.size - a.size || byName(a, b);
    if (sortKey.value === "modified") return b.modified - a.modified || byName(a, b);
    return byName(a, b);
  };
  dirs.sort(byName);
  files.sort(byKey);
  return [...dirs, ...files];
});

function onRowClick(e: FsEntry) {
  if (e.isDir) enter(join(cur.value, e.name));
  else if (props.mode === "file") emit("picked", join(cur.value, e.name));
}

function onPathEnter() {
  enter(pathInput.value);
}

async function createFolder() {
  const name = mkdirName.value.trim();
  if (!name) return;
  const target = join(cur.value, name);
  try {
    await api.createDir(target);
    mkdirName.value = "";
    mkdirOpen.value = false;
    await enter(cur.value);
  } catch (e) {
    err.value = String(e);
  }
}

function fmtSize(b: number): string {
  if (b >= 1024 * 1024 * 1024) return `${(b / 1024 / 1024 / 1024).toFixed(1)} GB`;
  if (b >= 1024 * 1024) return `${(b / 1024 / 1024).toFixed(1)} MB`;
  if (b >= 1024) return `${Math.round(b / 1024)} KB`;
  return `${b} B`;
}

function fmtDate(t: number): string {
  if (!t) return "—";
  return new Date(t * 1000).toLocaleDateString("zh-CN");
}

function fmtTrailing(p: string): string {
  const b = p.replace(/\\+$/, "").replace(/\/+$/, "");
  return b.split(/[\\/]/).pop() ?? p;
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(680px, 92vw)">
      <div class="head">
        <h2>{{ props.title }}</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <!-- 磁盘切换 -->
        <div v-if="drives.length" class="drives">
          <span class="muted">磁盘：</span>
          <button
            v-for="d in drives"
            :key="d"
            class="drive"
            :class="{ on: cur.toUpperCase().startsWith(d.toUpperCase()) }"
            @click="enter(d)"
          >
            <Icon name="hard-drive" :size="13" /> {{ d.replace(/[\\/]$/, "") }}
          </button>
        </div>

        <!-- 面包屑 + 上级 -->
        <div class="crumb">
          <button class="btn small ghost" @click="goUp" :disabled="isDriveRoot">
            <Icon name="arrow-up" :size="13" /> 上级
          </button>
          <div class="crumbs">
            <template v-for="(c, i) in crumbs" :key="c.path">
              <span v-if="i > 0" class="crumb-sep">{{ sep() }}</span>
              <button class="crumb-seg" @click="enter(c.path)">{{ c.label }}</button>
            </template>
          </div>
        </div>

        <input
          v-model="pathInput"
          type="text"
          placeholder="粘贴完整路径后回车跳转"
          @keyup.enter="onPathEnter"
        />

        <!-- 工具栏 -->
        <div class="fb-toolbar">
          <button class="btn small" @click="mkdirOpen = !mkdirOpen">
            <Icon name="folder" :size="13" /> 新建文件夹
          </button>
          <div class="spacer"></div>
          <span class="muted" v-if="truncated">文件过多，仅显示前 500 个</span>
          <select v-model="sortKey" class="sort">
            <option value="name">按名称</option>
            <option value="size">按大小</option>
            <option value="modified">按修改时间</option>
          </select>
        </div>

        <!-- 新建文件夹内联输入 -->
        <div v-if="mkdirOpen" class="mkdir-row">
          <input
            v-model="mkdirName"
            type="text"
            placeholder="新文件夹名称"
            @keyup.enter="createFolder"
          />
          <button class="btn small primary" :disabled="!mkdirName.trim()" @click="createFolder">创建</button>
        </div>

        <div v-if="err" class="toast err">{{ err }}</div>
        <div v-if="loading" class="muted" style="padding: 12px">加载中…</div>

        <!-- 列表 -->
        <div v-if="!loading" class="fb-list">
          <div class="fb-row fb-head">
            <span class="nm">名称</span>
            <span class="dt">修改时间</span>
            <span class="sz">大小</span>
          </div>
          <button
            v-for="e in sorted"
            :key="e.name + e.isDir"
            class="fb-row"
            :class="{ file: !e.isDir, clickable: e.isDir || props.mode === 'file' }"
            :title="join(cur, e.name)"
            @click="onRowClick(e)"
          >
            <Icon :name="e.isDir ? 'folder' : 'file'" :size="15" />
            <span class="nm">{{ e.name }}</span>
            <span class="dt">{{ fmtDate(e.modified) }}</span>
            <span class="sz">{{ e.isDir ? "—" : fmtSize(e.size) }}</span>
          </button>
          <div v-if="!sorted.length && !err" class="muted" style="padding: 16px">
            {{ props.mode === "dir" ? "此目录为空，可直接「选择此目录」。" : "此目录没有可选择的文件。" }}
          </div>
        </div>
      </div>

      <div class="foot">
        <span class="muted" style="margin-right: auto">当前：{{ fmtTrailing(cur) || cur }}</span>
        <button class="btn ghost" @click="emit('close')">取消</button>
        <button v-if="props.mode === 'dir'" class="btn primary" @click="emit('picked', cur)">
          选择此目录
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.drives {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 8px;
}
.drive {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 9px;
  border-radius: 7px;
  border: 1px solid var(--border);
  background: var(--bg-soft);
  color: var(--text-dim);
  font-size: 12px;
  cursor: pointer;
}
.drive:hover {
  color: var(--text);
  border-color: var(--border-strong);
}
.drive.on {
  color: var(--accent);
  border-color: var(--accent);
}
.crumb {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}
.crumbs {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  overflow-x: auto;
  white-space: nowrap;
  font-size: 12.5px;
  color: var(--text-dim);
}
.crumb-sep {
  color: var(--text-faint);
  margin: 0 3px;
  flex-shrink: 0;
}
.crumb-seg {
  border: none;
  background: none;
  color: var(--text-dim);
  font-size: 12.5px;
  cursor: pointer;
  padding: 2px 4px;
  border-radius: 5px;
}
.crumb-seg:hover {
  color: var(--text);
  background: var(--surface-2);
}
.fb-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 8px 0 6px;
}
.fb-toolbar .sort {
  padding: 6px 26px 6px 8px;
  font-size: 12px;
}
.mkdir-row {
  display: flex;
  gap: 8px;
  margin: 4px 0 8px;
}
.fb-list {
  margin-top: 4px;
  max-height: 42vh;
  overflow-y: auto;
  border: 1px solid var(--border);
  border-radius: 10px;
}
.fb-row {
  display: flex;
  align-items: center;
  gap: 9px;
  width: 100%;
  padding: 7px 12px;
  background: transparent;
  border: none;
  color: var(--text);
  font-size: 13px;
  text-align: left;
  cursor: default;
}
.fb-row.clickable {
  cursor: pointer;
}
.fb-row.clickable:hover {
  background: var(--surface-2);
}
.fb-row.file {
  color: var(--text-dim);
}
.fb-row .nm {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.fb-row .dt {
  width: 92px;
  flex-shrink: 0;
  font-size: 11.5px;
  color: var(--text-faint);
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.fb-row .sz {
  width: 74px;
  flex-shrink: 0;
  font-size: 11.5px;
  color: var(--text-faint);
  text-align: right;
  font-variant-numeric: tabular-nums;
}
.fb-row.fb-head {
  position: sticky;
  top: 0;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
  color: var(--text-faint);
  font-size: 11.5px;
  cursor: default;
}
.fb-row.fb-head:hover {
  background: var(--surface);
}
</style>

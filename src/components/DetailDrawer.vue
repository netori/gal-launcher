<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { api, engineNeedsLocale, STATUS_META, type FileInfo, type Game, type Patch } from "../api";
import MetadataDialog from "./MetadataDialog.vue";
import PatchDialog from "./PatchDialog.vue";
import AssetDialog from "./AssetDialog.vue";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ game: Game | null }>();
const emit = defineEmits<{
  close: [];
  launch: [g: Game, le: boolean];
  updated: [g: Game];
  favorite: [g: Game];
  hide: [g: Game];
  del: [g: Game];
  remove: [g: Game];
  picklaunch: [g: Game];
  notice: [msg: string];
}>();
useCloseOnEscape(close);

const files = ref<FileInfo[]>([]);
const patches = ref<Patch[]>([]);
const hero = ref("");
const showMetadata = ref(false);
const showPatch = ref(false);
const showAssets = ref(false);
const err = ref("");
const opBusy = ref(false);
const closing = ref(false);

/** 关闭：先播放退出动效，再通知父级卸载。 */
function close() {
  if (closing.value) return;
  closing.value = true;
  window.setTimeout(() => emit("close"), 180);
}

function reloadPatches(g: Game) {
  api
    .getPatches(g.id)
    .then((list) => (patches.value = list))
    .catch(() => (patches.value = []));
}

watch(
  () => props.game,
  async (g) => {
    files.value = [];
    patches.value = [];
    hero.value = "";
    err.value = "";
    if (!g) return;
    closing.value = false;
    if (g.coverPath) {
      try {
        hero.value = await api.readImage(g.coverPath);
      } catch {
        /* ignore */
      }
    }
    try {
      files.value = await api.getGameFiles(g.id);
    } catch {
      /* ignore */
    }
    reloadPatches(g);
  },
  { immediate: true }
);

const KIND_LABEL: Record<string, string> = {
  launch: "启动",
  engine: "内核",
  asset: "资源包",
  image: "图片",
  media: "音视频",
  save: "存档",
  patch: "补丁",
  archive: "压缩档",
  doc: "文档",
  other_program: "其他程序",
  other: "其他",
};

const breakdown = computed(() => {
  const m = new Map<string, number>();
  for (const f of files.value) m.set(f.kind, (m.get(f.kind) ?? 0) + 1);
  return [...m.entries()].map(([k, n]) => ({ k, n, label: KIND_LABEL[k] ?? k }));
});

const totalSize = computed(() => files.value.reduce((s, f) => s + f.size, 0));

function fmtSize(b: number): string {
  const mb = b / (1024 * 1024);
  if (mb > 1024) return `${(mb / 1024).toFixed(1)} GB`;
  if (mb >= 1) return `${Math.round(mb)} MB`;
  return `${Math.round(b / 1024)} KB`;
}

function fmtLen(min: number | null): string {
  if (!min) return "";
  const h = Math.floor(min / 60);
  const m = min % 60;
  return h > 0 ? `${h}h${m}m` : `${m}m`;
}

const needsLocale = computed(() => (props.game ? engineNeedsLocale(props.game.engine) : false));
const patchKindColor = (k: string) =>
  k === "汉化" ? "#a3c585" : k === "R18" ? "#e08a5e" : k === "修正" ? "#86a8c0" : "#cfc6b9";

function srcLeaf(p: string): string {
  return p.replace(/\\/g, "/").split("/").pop() ?? p;
}

function pickLaunch() {
  if (props.game) emit("picklaunch", props.game);
}

async function onStatusChange(e: Event) {
  const g = props.game;
  if (!g) return;
  const status = (e.target as HTMLSelectElement).value;
  try {
    const updated = await api.setStatus(g.id, status);
    emit("updated", updated);
    emit("notice", "已更新游玩状态");
  } catch (err2) {
    emit("notice", String(err2));
  }
}

async function changeCover() {
  const g = props.game;
  if (!g) return;
  try {
    const p = await open({
      multiple: false,
      title: "选择封面图片",
      filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "webp", "bmp"] }],
    });
    if (!p) return;
    const updated = await api.setCover(g.id, p);
    emit("updated", updated);
    emit("notice", "已更换封面");
  } catch (e) {
    emit("notice", String(e));
  }
}

async function onMetadataApplied(g: Game) {
  emit("updated", g);
  emit("notice", "已应用 VNDB 元数据");
}

async function installPatch(p: Patch) {
  err.value = "";
  opBusy.value = true;
  try {
    await api.installPatch(p.id);
    reloadPatches(props.game!);
    emit("notice", `已安装补丁：${p.name}`);
  } catch (e) {
    err.value = String(e);
  } finally {
    opBusy.value = false;
  }
}

async function uninstallPatch(p: Patch) {
  const ok = await confirm(`回滚后，被「${p.name}」覆盖的文件会从备份恢复。`, {
    title: "回滚补丁",
    kind: "warning",
  });
  if (!ok) return;
  opBusy.value = true;
  err.value = "";
  try {
    await api.uninstallPatch(p.id);
    reloadPatches(props.game!);
  } catch (e) {
    err.value = String(e);
  } finally {
    opBusy.value = false;
  }
}

async function removePatchEntry(p: Patch) {
  const ok = await confirm(`删除补丁「${p.name}」的记录与备份（不会回滚已装的文件）。`, {
    title: "删除补丁",
    kind: "warning",
  });
  if (!ok) return;
  opBusy.value = true;
  err.value = "";
  try {
    await api.removePatch(p.id);
    reloadPatches(props.game!);
  } catch (e) {
    err.value = String(e);
  } finally {
    opBusy.value = false;
  }
}
</script>

<template>
  <template v-if="props.game">
    <div class="drawer-mask" :class="{ closing }" @click="close"></div>
    <aside class="drawer" :class="{ closing }">
      <div class="head">
        <div class="row">
          <h2 style="flex: 1">{{ props.game.title }}</h2>
          <button class="btn icon-btn ghost" @click="close()"><Icon name="close" :size="15" /></button>
        </div>
        <div class="subline">
          {{ props.game.engine }}<template v-if="props.game.developer"> · {{ props.game.developer }}</template>
        </div>
      </div>

      <div class="body">
        <img v-if="hero" class="hero" :src="hero" :alt="props.game.title" />

        <div class="row" style="margin: 10px 0 12px; gap: 8px">
          <select :value="props.game.status" @change="onStatusChange" style="flex: 1">
            <option v-for="s in STATUS_META" :key="s.key || '_'" :value="s.key">
              {{ s.label }}
            </option>
          </select>
          <button class="btn small" @click="changeCover">
            <Icon name="image" :size="13" /> 更换封面
          </button>
        </div>

        <div class="section-title" style="margin-top: 0">信息</div>
        <dl class="kv">
          <dt>路径</dt>
          <dd style="word-break: break-all">{{ props.game.launchPath }}</dd>
          <dt>目录</dt>
          <dd style="word-break: break-all">{{ props.game.sourceDir }}</dd>
          <template v-if="props.game.rating != null">
            <dt>评分</dt>
            <dd>★ {{ props.game.rating.toFixed(1) }}</dd>
          </template>
          <template v-if="props.game.released">
            <dt>发售日</dt>
            <dd>{{ props.game.released }}</dd>
          </template>
          <template v-if="props.game.lengthMinutes">
            <dt>时长</dt>
            <dd>{{ fmtLen(props.game.lengthMinutes) }}</dd>
          </template>
          <dt>游玩</dt>
          <dd>
            {{
              props.game.totalSeconds > 0
                ? `${Math.floor(props.game.totalSeconds / 3600)}h ${Math.round((props.game.totalSeconds % 3600) / 60)}m`
                : "暂无"
            }}
            · {{ props.game.playCount }} 次
          </dd>
        </dl>

        <div class="row" style="justify-content: flex-end; margin-bottom: 12px">
          <button class="btn small" @click="pickLaunch" style="margin-right: auto">
            <Icon name="sliders" :size="13" /> 启动文件…
          </button>
          <button class="btn small" @click="showAssets = true"><Icon name="box" :size="13" /> 资源解包</button>
          <button class="btn small primary" @click="showMetadata = true"><Icon name="cloud" :size="13" /> 从 VNDB 补全</button>
        </div>

        <template v-if="props.game.description">
          <div class="section-title">简介</div>
          <p class="desc">{{ props.game.description }}</p>
        </template>

        <template v-if="props.game.tags.length">
          <div class="section-title">标签</div>
          <div class="tags">
            <span v-for="t in props.game.tags" :key="t" class="tag">{{ t }}</span>
          </div>
        </template>

        <div class="section-title">文件画像</div>
        <div class="files">
          <span v-if="!files.length" class="muted">（暂无数据）</span>
          <span v-for="b in breakdown" :key="b.k" class="filetag">
            <span class="k">{{ b.label }}</span>
            <b>{{ b.n }}</b>
          </span>
        </div>
        <div class="muted" style="margin-top: 6px">合计 {{ files.length }} 个文件 · {{ fmtSize(totalSize) }}</div>

        <div class="section-title" style="display: flex; align-items: center; justify-content: space-between">
          补丁
          <button class="btn small" @click="showPatch = true"><Icon name="plus" :size="13" /> 添加</button>
        </div>
        <div class="patches">
          <div v-if="!patches.length" class="muted">（暂无补丁，汉化 / R18 补丁可在此登记）</div>
          <div v-for="p in patches" :key="p.id" class="patch">
            <div style="flex: 1; min-width: 0">
              <div class="nm">
                <span class="pkind" :style="{ color: patchKindColor(p.kind) }">{{ p.kind }}</span>
                {{ p.name }}
                <span class="state" :class="p.installed ? 'on' : ''">{{ p.installed ? "已装" : "未装" }}</span>
              </div>
              <div class="sub">{{ srcLeaf(p.sourcePath) }} · {{ p.installMethod === "installer" ? "安装器" : "覆盖式" }}</div>
              <div class="warn" v-if="p.kind === '汉化'">⚠ 汉化补丁可能已包含成人内容</div>
            </div>
            <div class="row" style="gap: 6px; flex-wrap: wrap; justify-content: flex-end">
              <button
                v-if="!p.installed && p.installMethod === 'replace'"
                class="btn small primary"
                :disabled="opBusy"
                @click="installPatch(p)"
              >
                安装
              </button>
              <button
                v-if="!p.installed && p.installMethod === 'installer'"
                class="btn small primary"
                :disabled="opBusy"
                @click="installPatch(p)"
              >
                运行安装器
              </button>
              <button v-if="p.installed" class="btn small" :disabled="opBusy" @click="uninstallPatch(p)">
                回滚
              </button>
              <button class="btn small danger" :disabled="opBusy" @click="removePatchEntry(p)">删除</button>
            </div>
          </div>
        </div>

        <div v-if="err" class="toast err" style="margin-top: 8px">{{ err }}</div>
      </div>

      <div class="foot">
        <button class="btn primary" @click="emit('launch', props.game, false)">
          <Icon name="play" :size="14" /> 启动
        </button>
        <button class="btn" v-if="needsLocale" @click="emit('launch', props.game, true)">
          <Icon name="globe" :size="14" /> 转区启动
        </button>
        <button class="btn" @click="emit('favorite', props.game)">
          <Icon name="star" :size="14" :filled="props.game.favorite" />
          {{ props.game.favorite ? "已收藏" : "收藏" }}
        </button>
        <button class="btn" @click="emit('hide', props.game)">
          <Icon :name="props.game.hidden ? 'eye' : 'eye-off'" :size="14" />
          {{ props.game.hidden ? "显示" : "隐藏" }}
        </button>
        <button class="btn danger" @click="emit('remove', props.game)">从库移除</button>
        <button class="btn danger" @click="emit('del', props.game)"><Icon name="trash" :size="14" /> 删除文件</button>
      </div>
    </aside>

    <Transition name="overlay">
      <MetadataDialog
        :game="props.game"
        @close="showMetadata = false"
        @applied="(g) => { onMetadataApplied(g); showMetadata = false; }"
        v-if="showMetadata"
      />
    </Transition>
    <Transition name="overlay">
      <PatchDialog
        :game="props.game"
        @close="showPatch = false"
        @added="(p) => { reloadPatches(props.game!); showPatch = false; emit('notice', `已登记补丁：${p.name}`); }"
        v-if="showPatch"
      />
    </Transition>
    <Transition name="overlay">
      <AssetDialog
        :game="props.game"
        @close="showAssets = false"
        @notice="(m) => emit('notice', String(m))"
        v-if="showAssets"
      />
    </Transition>
  </template>
</template>

<style scoped>
.subline {
  margin-top: 6px;
  font-size: 12.5px;
  color: var(--text-dim);
}
.desc {
  font-size: 12.5px;
  line-height: 1.7;
  color: var(--text-dim);
  max-height: 120px;
  overflow-y: auto;
  margin: 0 0 12px;
}
.tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 14px;
}
.tag {
  font-size: 11.5px;
  padding: 3px 8px;
  border-radius: 999px;
  background: rgba(217, 126, 61, 0.12);
  color: #d9a25e;
}
.patches {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
}
.patch {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 10px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.pkind {
  font-size: 11px;
  font-weight: 700;
  margin-right: 5px;
}
.state {
  font-size: 10.5px;
  padding: 1px 7px;
  border-radius: 999px;
  margin-left: 6px;
  background: rgba(255, 255, 255, 0.07);
  color: var(--text-dim);
}
.state.on {
  background: rgba(147, 180, 110, 0.16);
  color: #a3c585;
}
.warn {
  margin-top: 2px;
  font-size: 11.5px;
  color: #d9a25e;
}
</style>
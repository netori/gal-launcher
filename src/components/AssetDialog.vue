<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { api, type ArchiveInfo, type AssetEntry, type Game } from "../api";
import LazyImg from "./LazyImg.vue";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ game: Game | null }>();
const emit = defineEmits<{ close: []; notice: [msg: string] }>();
useCloseOnEscape(() => emit("close"));

const CATEGORIES = [
  "立绘", "表情", "CG", "背景", "界面", "其他图片",
  "语音", "BGM", "音效", "其他音频",
  "视频", "脚本", "文档", "数据", "其他",
];

const archives = ref<ArchiveInfo[]>([]);
const extracted = ref<AssetEntry[]>([]);
const category = ref<string>("all");
const scanning = ref(false);
const extracting = ref(""); // 正在解包的资源包 rel
const exporting = ref(false);
const err = ref("");
const previews = ref(60);
const lightbox = ref<AssetEntry | null>(null);

const uriCache = new Map<string, string>();

onMounted(load);

async function load() {
  if (!props.game) return;
  scanning.value = true;
  err.value = "";
  try {
    const [a, e] = await Promise.all([
      api.listAssetArchives(props.game.id),
      api.listExtractedAssets(props.game.id),
    ]);
    archives.value = a;
    extracted.value = e;
  } catch (e) {
    err.value = String(e);
  } finally {
    scanning.value = false;
  }
}

function isImage(e: AssetEntry): boolean {
  return /\.(png|jpe?g|bmp|webp|gif)$/i.test(e.rel);
}

const filteredByCategory = computed(() =>
  category.value === "all"
    ? extracted.value
    : extracted.value.filter((e) => e.category === category.value)
);
const imgEntries = computed(() => filteredByCategory.value.filter(isImage));
const otherEntries = computed(() => filteredByCategory.value.filter((e) => !isImage(e)));
const visibleImgs = computed(() => imgEntries.value.slice(0, previews.value));

async function thumbSrc(e: AssetEntry): Promise<string | null> {
  if (!e.absPath) return null;
  const hit = uriCache.get(e.absPath);
  if (hit) return hit;
  try {
    const uri = await api.readImage(e.absPath);
    uriCache.set(e.absPath, uri);
    return uri;
  } catch {
    return null;
  }
}

async function extractAll() {
  if (!props.game) return;
  err.value = "";
  for (const a of archives.value) {
    const ok = await extractOne(a);
    if (!ok) break;
  }
}

async function extractOne(a: ArchiveInfo): Promise<boolean> {
  if (!props.game) return false;
  extracting.value = a.relPath;
  err.value = "";
  try {
    await api.extractAssets(props.game.id, a.relPath);
    emit("notice", `「${a.relPath}」解包完成`);
    await load();
    return true;
  } catch (e) {
    err.value = `「${a.relPath}」解包失败：${e}`;
    return false;
  } finally {
    extracting.value = "";
  }
}

const lightboxSrc = computed(() =>
  lightbox.value ? (uriCache.get(lightbox.value.absPath) ?? "") : ""
);

function stepPreview(dir: number) {
  const list = imgEntries.value;
  if (!list.length) return;
  const i = list.findIndex((x) => x.rel === (lightbox.value?.rel ?? ""));
  const base = i < 0 ? 0 : i;
  lightbox.value = list[(base + dir + list.length) % list.length];
}

async function exportCat() {
  if (!props.game) return;
  const dest = await open({ directory: true, multiple: false, title: "选择导出目标文件夹" });
  if (!dest) return;
  exporting.value = true;
  err.value = "";
  try {
    const n = await api.exportAssets(
      props.game.id,
      dest,
      category.value === "all" ? undefined : category.value
    );
    emit("notice", `已导出 ${n} 个文件`);
  } catch (e) {
    err.value = String(e);
  } finally {
    exporting.value = false;
  }
}

async function clearCache() {
  if (!props.game || !extracted.value.length) return;
  const ok = await confirm(
    `确认清空「${props.game.title}」的解包缓存（${extracted.value.length} 个文件）？已导出的文件不受影响。`,
    { title: "清空解包缓存", kind: "warning" }
  );
  if (!ok) return;
  err.value = "";
  exporting.value = true;
  try {
    const n = await api.clearAssetCache(props.game.id);
    extracted.value = [];
    await load();
    emit("notice", `已清理解包缓存（${n} 个文件）`);
  } catch (e) {
    err.value = String(e);
  } finally {
    exporting.value = false;
  }
}

function fmtSize(b: number): string {
  const mb = b / (1024 * 1024);
  if (mb > 1024) return `${(mb / 1024).toFixed(1)} GB`;
  if (mb >= 1) return `${Math.round(mb)} MB`;
  return `${Math.round(b / 1024)} KB`;
}

function countByCat(): string {
  const m = new Map<string, number>();
  for (const e of extracted.value) m.set(e.category, (m.get(e.category) ?? 0) + 1);
  return [...m.entries()].map(([k, n]) => `${k}×${n}`).join(" · ");
}

// 切换分类时重置懒加载数量
watch(category, () => {
  previews.value = 60;
});
</script>

<template>
  <div v-if="props.game" class="overlay" style="z-index: 60" @click.self="emit('close')">
    <div class="modal assets">
      <div class="head">
        <h2 style="flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap">
          <Icon name="box" :size="15" /> 资源 · {{ props.game.title }}
        </h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <div v-if="err" class="toast err" style="margin-bottom: 8px">{{ err }}</div>

        <div class="section-title" style="margin-top: 0">资源包（.xp3 / .pfs）</div>
        <div class="archives">
          <div v-if="scanning" class="muted">扫描中…</div>
          <div v-else-if="!archives.length" class="muted">
            未在游戏目录里找到 .xp3 / .pfs 资源包。
          </div>
          <template v-else>
            <div class="arow">
              <span class="muted" style="margin-right: auto">共 {{ archives.length }} 个</span>
              <button class="btn small primary" :disabled="extracting.length > 0" @click="extractAll">
                {{ extracting ? "解包中…" : "全部解包" }}
              </button>
            </div>
            <div v-for="a in archives" :key="a.relPath" class="arch">
              <div style="flex: 1; min-width: 0">
                <div class="nm" style="word-break: break-all">{{ a.relPath }}</div>
                <div class="sub">
                  <span class="fmt">{{ a.format }}</span>
                  {{ fmtSize(a.sizeBytes) }}
                  <span v-if="a.format === 'Unknown'" style="color: #ff9f43">（内置不支持 → 用外部工具）</span>
                  <span v-if="a.extractedCount > 0" style="color: #a3c585"> · 已解 {{ a.extractedCount }} 文件</span>
                </div>
              </div>
              <button
                class="btn small"
                :class="a.extractedCount > 0 ? '' : 'primary'"
                :disabled="extracting.length > 0"
                @click="extractOne(a)"
              >
                {{ extracting === a.relPath ? "…" : a.extractedCount > 0 ? "重新解包" : a.format === "Unknown" ? "外部解包" : "解包" }}
              </button>
            </div>
          </template>
        </div>

        <div class="section-title">分类</div>
        <div class="tabs">
          <button class="tab" :class="{ on: category === 'all' }" @click="category = 'all'">
            全部 <b>{{ extracted.length }}</b>
          </button>
          <button
            v-for="c in CATEGORIES"
            :key="c"
            class="tab"
            :class="{ on: category === c }"
            @click="category = c"
          >
            {{ c }} <b>{{ extracted.filter((e) => e.category === c).length }}</b>
          </button>
        </div>

        <div class="muted" style="margin: 8px 0 10px; font-size: 12.5px">
          {{ imgEntries.length }} 张图片<template v-if="otherEntries.length"> · {{ otherEntries.length }} 个音频/文件</template>
          <template v-if="extracted.length"> · 归类：{{ countByCat() }}</template>
        </div>

        <div class="gallery" :class="{ empty: !filteredByCategory.length }">
          <div v-if="!filteredByCategory.length" class="muted" style="padding: 20px 0">
            还没有解包文件 —— 先在上方「解包」，或换个分类。
          </div>
          <template v-else>
            <figure v-for="e in visibleImgs" :key="e.rel" class="thumb" @click="lightbox = e">
              <LazyImg :entry="e" :src-fn="thumbSrc" />
              <figcaption>{{ e.rel.split("/").pop() }}</figcaption>
            </figure>

            <button
              v-if="previews < imgEntries.length"
              class="btn small ghost more"
              @click="previews += 60"
            >
              加载更多图片（余 {{ imgEntries.length - previews }}）
            </button>

            <div v-for="e in otherEntries" :key="'o' + e.rel" class="oth">
              <span class="cat">{{ e.category }}</span>
              <span style="word-break: break-all">{{ e.rel }}</span>
              <span class="muted" style="margin-left: auto">{{ fmtSize(e.size) }}</span>
            </div>
          </template>
        </div>
      </div>

      <div class="foot">
        <button class="btn" :disabled="exporting || !filteredByCategory.length" @click="exportCat">
          {{ exporting ? "导出中…" : `导出${category === "all" ? "全部" : category}到…` }}
        </button>
        <span class="muted" style="margin-right: auto">解包后自动按类别整理到子文件夹，缓存可随时导出精选</span>
        <button class="btn danger" :disabled="exporting || !extracted.length" @click="clearCache">
          <Icon name="trash" :size="13" /> 清空解包缓存
        </button>
        <button class="btn ghost" @click="emit('close')">关闭</button>
      </div>
    </div>

    <div v-if="lightbox" class="overlay lb" @click.self="lightbox = null">
      <div class="lb-box">
        <button class="btn icon-btn ghost lb-nav prev" @click="stepPreview(-1)">‹</button>
        <img :src="lightboxSrc" :alt="lightbox.rel" />
        <button class="btn icon-btn ghost lb-nav next" @click="stepPreview(1)">›</button>
        <div class="lb-cap">{{ lightbox.rel }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.assets {
  width: min(1100px, 96vw);
  height: 88vh;
  display: flex;
  flex-direction: column;
}
.assets .body {
  overflow-y: auto;
}
.archives {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 6px 0 16px;
}
.arow {
  display: flex;
  align-items: center;
  gap: 8px;
}
.arch {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.fmt {
  font-size: 11px;
  padding: 1px 7px;
  border-radius: 999px;
  background: rgba(217, 126, 61, 0.14);
  color: #d9a25e;
  margin-right: 6px;
}
.tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.tab {
  font-size: 12.5px;
  padding: 4px 10px;
  border-radius: 999px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text-dim);
  cursor: pointer;
}
.tab.on {
  background: rgba(217, 126, 61, 0.16);
  color: var(--text);
  border-color: var(--accent);
  box-shadow: inset 0 0 0 1px rgba(217, 126, 61, 0.14);
}
.tab b {
  font-weight: 700;
  margin-left: 3px;
}
.gallery {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
  gap: 10px;
}
.gallery.empty {
  display: block;
}
.thumb {
  margin: 0;
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
  background: var(--surface-2);
  cursor: zoom-in;
  transition: transform 0.12s ease;
}
.thumb:hover {
  transform: translateY(-2px);
}
.thumb :deep(.thumb-img) {
  width: 100%;
  height: 200px;
  object-fit: cover;
  display: block;
}
.thumb figcaption {
  font-size: 11.5px;
  padding: 5px 8px;
  color: var(--text-dim);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.more {
  grid-column: 1 / -1;
  justify-self: center;
  margin: 6px 0;
}
.oth {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
  padding: 6px 8px;
  border: 1px dashed var(--border);
  border-radius: 8px;
  color: var(--text-dim);
}
.cat {
  font-size: 11px;
  padding: 1px 7px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.07);
  flex-shrink: 0;
}
.lb .lb-box {
  position: relative;
  display: grid;
  place-items: center;
}
.lb-box img {
  max-width: 88vw;
  max-height: 84vh;
  border-radius: 8px;
  box-shadow: 0 10px 60px rgba(0, 0, 0, 0.7);
}
.lb-nav {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  font-size: 26px;
}
.lb-nav.prev {
  left: -14px;
}
.lb-nav.next {
  right: -14px;
}
.lb-cap {
  position: absolute;
  bottom: -34px;
  left: 0;
  right: 0;
  text-align: center;
  color: var(--text-dim);
  font-size: 12.5px;
}
</style>
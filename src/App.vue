<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useLibrary } from "./store";
import { api, type Game, type UpdateInfo, STATUS_META } from "./api";

import GameCard from "./components/GameCard.vue";
import ScanDialog from "./components/ScanDialog.vue";
import SettingsDialog from "./components/SettingsDialog.vue";
import ResourceDialog from "./components/ResourceDialog.vue";
import MissingDialog from "./components/MissingDialog.vue";
import DetailDrawer from "./components/DetailDrawer.vue";
import LaunchDialog from "./components/LaunchDialog.vue";
import Icon from "./components/Icon.vue";
import BrandLogo from "./components/BrandLogo.vue";
import brandLogo from "./assets/brand-logo.png";
import { useCloseOnEscape } from "./composables/useCloseOnEscape";

const lib = useLibrary();
const { state, visible, counts } = lib;

const selectedGame = ref<Game | null>(null);
const showScan = ref(false);
const showSettings = ref(false);
const showResources = ref(false);
const showMissing = ref(false);

// 多选模式
const selectMode = ref(false);
const selected = ref<Set<number>>(new Set());
const batchStatus = ref("");
function toggleSelectMode() {
  selectMode.value = !selectMode.value;
  selected.value = new Set();
  batchStatus.value = "";
}
function toggleSelect(game: Game) {
  const s = new Set(selected.value);
  if (s.has(game.id)) s.delete(game.id);
  else s.add(game.id);
  selected.value = s;
}
function clearSelection() {
  selected.value = new Set();
}

// 更新提示：非打扰式横幅，启动延迟检查；失败静默
const updateInfo = ref<UpdateInfo | null>(null);
const updateBannerOpen = ref(false);
async function checkForUpdate() {
  try {
    const u = await api.checkUpdate();
    if (u) {
      updateInfo.value = u;
      updateBannerOpen.value = true;
    }
  } catch {
    /* 网络/限流等原因查不到就不打扰 */
  }
}
function goUpdate() {
  const u = updateInfo.value;
  if (!u) return;
  updateBannerOpen.value = false;
  openUrl(u.downloadUrl ?? u.url).catch(() => {});
}
function dismissUpdateForever() {
  const v = updateInfo.value?.version;
  if (v) api.dismissUpdate(v).catch(() => {});
  updateBannerOpen.value = false;
}
const updateNote = () => (updateInfo.value?.note ?? "").replace(/\s+/g, " ").slice(0, 90);

// 启动文件选择：首次启动 / 手动更换
const launchPick = ref<{ game: Game; useLocale: boolean; pickOnly: boolean } | null>(null);

// 右键菜单
const ctx = reactive<{ x: number; y: number; game: Game | null }>({ x: 0, y: 0, game: null });
function openCtx(game: Game, e: MouseEvent) {
  ctx.x = Math.min(e.clientX, window.innerWidth - 230);
  ctx.y = Math.min(e.clientY, window.innerHeight - 320);
  ctx.game = game;
}
function closeCtx() {
  ctx.game = null;
}
/** 菜单项统一入口：关菜单后对目标游戏执行动作。 */
function ctxAction(fn: (g: Game) => void) {
  const g = ctx.game;
  closeCtx();
  if (g) fn(g);
}

// 顶层 Esc：关右键菜单 / 确认框（对话框与抽屉各自处理自己的 Esc，只在有东西可关时才拦截）
useCloseOnEscape(
  () => {
    if (ctx.game) closeCtx();
    else confirmBox.show = false;
  },
  () => !!(ctx.game || confirmBox.show)
);

/** 封面墙方向键：焦点在卡片上时 ←/→/↑/↓ 移动焦点（Enter 由卡片自身处理为打开）。 */
function onWallKey(e: KeyboardEvent) {
  if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)) return;
  const t = e.target as HTMLElement;
  if (t instanceof HTMLInputElement || t instanceof HTMLTextAreaElement || t instanceof HTMLSelectElement) return;
  const cards = Array.from(document.querySelectorAll<HTMLElement>(".wall .card"));
  if (!cards.length) return;
  const idx = cards.indexOf(document.activeElement as HTMLElement);
  if (idx < 0) return;
  e.preventDefault();
  const step = e.key === "ArrowRight" || e.key === "ArrowDown" ? 1 : -1;
  const next = Math.max(0, Math.min(cards.length - 1, idx + step));
  cards[next].focus();
}

// 确认弹窗
const confirmBox = reactive<{
  show: boolean;
  title: string;
  msg: string;
  ok: string;
  danger: boolean;
  run: (() => Promise<void>) | null;
}>({ show: false, title: "", msg: "", ok: "确认", danger: false, run: null });
function ask(o: { title: string; msg: string; ok?: string; danger?: boolean; run: () => Promise<void> }) {
  confirmBox.title = o.title;
  confirmBox.msg = o.msg;
  confirmBox.ok = o.ok ?? "确认";
  confirmBox.danger = o.danger ?? false;
  confirmBox.run = o.run;
  confirmBox.show = true;
}

// Toast
const toasts = reactive<{ id: number; msg: string; type: "ok" | "err" }[]>([]);
let tid = 0;
function toast(msg: string, type: "ok" | "err" = "ok") {
  const id = ++tid;
  toasts.push({ id, msg, type });
  setTimeout(() => {
    const i = toasts.findIndex((t) => t.id === id);
    if (i >= 0) toasts.splice(i, 1);
  }, 2600);
}

// ---- Android 首启：检测「所有文件访问」权限，未授权时展示引导遮罩 ----
const isAndroid = /android/i.test(navigator.userAgent);
const filesAccess = ref<boolean | null>(null);
const filesAccessHint = ref("");
async function checkFilesAccess() {
  try {
    filesAccess.value = await api.checkFilesAccess();
    if (filesAccess.value) filesAccessHint.value = "";
  } catch {
    filesAccess.value = null;
  }
}
async function requestFilesAccess() {
  try {
    await api.requestFilesAccess();
    filesAccessHint.value = "";
  } catch (e) {
    filesAccessHint.value = String(e);
  }
}

onMounted(() => {
  lib.refresh();
  // 启动后稍等片刻再查更新，不抢首屏
  window.setTimeout(() => checkForUpdate(), 1500);
  if (isAndroid) checkFilesAccess();
});

async function runAction(fn: () => Promise<void>, okMsg?: string) {
  try {
    await fn();
    if (okMsg) toast(okMsg);
  } catch (e) {
    toast(String(e), "err");
  }
}

function onSearch(e: Event) {
  lib.setSearch((e.target as HTMLInputElement).value);
}
function onSort(e: Event) {
  lib.setSort((e.target as HTMLSelectElement).value as never);
}

/** 设置页里从备份恢复完成后：刷新库并关闭设置。 */
function onSettingsRestored() {
  showSettings.value = false;
  lib.refresh();
  toast("已从备份恢复");
}

async function onImported(n: number) {
  await lib.refresh();
  toast(`已导入 ${n} 个游戏`);
  const missing = lib.visible.value.filter((g) => !g.coverPath).length;
  if (missing > 0) {
    toast(`正在自动补全 ${missing} 个封面…`);
    try {
      const r = await api.fetchMissingCovers();
      await lib.refresh();
      if (r.updated > 0) toast(`已自动补全 ${r.updated} 个封面`);
      if (r.failed.length) {
        toast(`有 ${r.failed.length} 个没匹配上，可在详情里手动搜 VNDB`, "err");
      }
    } catch (e) {
      toast(String(e), "err");
    }
  }
}

function handleClick(game: Game) {
  selectedGame.value = game;
}
/**
 * 启动：该游戏有多个启动文件且用户没选过 → 弹选择器；
 * 否则直接启动。启动成功后把返回的 Game 同步回内存。
 */
function handleLaunch(game: Game, le: boolean) {
  if (!game.launchSet && game.launchCandidates.length > 1) {
    launchPick.value = { game, useLocale: le, pickOnly: false };
    return;
  }
  runAction(async () => {
    const updated = await api.launchGame(game.id, le);
    lib.upsertGame(updated);
  }, le ? "已通过转区启动" : "已启动");
}
/** 手动更换该游戏的默认启动文件（只设默认，不启动）。 */
function pickLaunchFile(game: Game) {
  launchPick.value = { game, useLocale: false, pickOnly: true };
}
function onLaunchPickDone(updated: Game, msg: string) {
  lib.upsertGame(updated);
  toast(msg);
  launchPick.value = null;
}
/** 详情抽屉里元数据/标题变化后回写。 */
function onDrawerUpdated(updated: Game) {
  lib.upsertGame(updated);
  selectedGame.value = updated;
}
async function handleFavorite(game: Game) {
  try {
    const g = await api.toggleFavorite(game.id);
    lib.upsertGame(g);
  } catch (e) {
    toast(String(e), "err");
  }
}
async function handleHide(game: Game) {
  try {
    const g = await api.setHidden(game.id, !game.hidden);
    lib.upsertGame(g);
  } catch (e) {
    toast(String(e), "err");
  }
}
async function openDir(game: Game) {
  // 后台分离启动 explorer 并定位到启动文件，应用不等待，避免大目录下卡顿
  try {
    await api.reveal(game.launchPath || game.sourceDir);
  } catch (e) {
    toast(String(e), "err");
  }
}

// ---- 批量操作 ----
const batchBusy = ref(false);

async function batchSetHidden(hidden: boolean) {
  const ids = [...selected.value];
  if (!ids.length) return;
  batchBusy.value = true;
  try {
    for (const id of ids) {
      const g = await api.setHidden(id, hidden);
      lib.upsertGame(g);
    }
    toast(hidden ? `已隐藏 ${ids.length} 个游戏` : `已恢复 ${ids.length} 个游戏`);
    clearSelection();
  } catch (e) {
    toast(String(e), "err");
  } finally {
    batchBusy.value = false;
  }
}

async function batchSetStatus() {
  const ids = [...selected.value];
  if (!ids.length) return;
  batchBusy.value = true;
  try {
    for (const id of ids) {
      const g = await api.setStatus(id, batchStatus.value);
      lib.upsertGame(g);
    }
    toast(`已更新 ${ids.length} 个游戏的状态`);
    clearSelection();
    batchStatus.value = "";
  } catch (e) {
    toast(String(e), "err");
  } finally {
    batchBusy.value = false;
  }
}

function batchRemove() {
  const n = selected.value.size;
  if (!n) return;
  ask({
    title: "批量移除",
    msg: `将把 ${n} 个游戏从库里移除（磁盘文件不动）。`,
    ok: "移除",
    run: async () => {
      batchBusy.value = true;
      try {
        for (const id of [...selected.value]) await api.removeFromLibrary(id);
        await lib.refresh();
        toast(`已移除 ${n} 个游戏`);
        clearSelection();
      } catch (e) {
        toast(String(e), "err");
      } finally {
        batchBusy.value = false;
      }
    },
  });
}

function folderHidden(game: Game, hidden: boolean) {
  runAction(
    async () => await api.setHiddenAttr(game.sourceDir, hidden),
    hidden ? "已设置系统级隐藏属性" : "已取消隐藏属性"
  );
}

function removeFromLibrary(game: Game) {
  ask({
    title: "从库中移除",
    msg: `「${game.title}」将只从库里移除，磁盘上的游戏文件不会动。`,
    ok: "移除",
    run: async () => {
      await runAction(async () => await api.removeFromLibrary(game.id));
      await lib.refresh();
    },
  });
}

function trashGame(game: Game) {
  const permanent = isAndroid; // Android 无回收站，删除即永久
  ask({
    title: "删除游戏",
    msg: permanent
      ? `「${game.title}」整个目录将被永久删除（Android 无回收站），不可恢复。`
      : `「${game.title}」整个目录将送进回收站。之后可从回收站恢复，但需要重新导入。`,
    ok: permanent ? "永久删除" : "送回收站",
    danger: true,
    run: async () => {
      await runAction(async () => await api.deleteGame(game.id), permanent ? "已删除" : "已送回收站");
      await lib.refresh();
    },
  });
}
</script>

<template>
  <header class="toolbar">
    <div class="brand">
      <div class="logo"><BrandLogo /></div>
      GAL 启动器
    </div>

    <div class="search">
      <span class="icon"><Icon name="search" :size="14" /></span>
      <input :value="state.search" placeholder="搜索游戏标题 / 引擎…" @input="onSearch" />
    </div>

    <div class="chips">
      <button class="chip" :class="{ active: state.view === 'all' }" @click="lib.setView('all')">
        全部 <span class="dot">{{ counts.total.value }}</span>
      </button>
      <button
        class="chip"
        :class="{ active: state.view === 'favorites' }"
        @click="lib.setView('favorites')"
      >
        <Icon name="star" :size="12" filled style="margin-right: 5px" /> 收藏
        <span class="dot">{{ counts.favorites.value }}</span>
      </button>
      <button
        class="chip"
        :class="{ active: state.view === 'hidden' }"
        @click="lib.setView('hidden')"
      >
        <Icon name="eye-off" :size="13" style="margin-right: 5px" /> 隐藏
        <span class="dot">{{ counts.hidden.value }}</span>
      </button>
    </div>

    <select :value="state.sort" @change="onSort">
      <option value="recent">最近游玩</option>
      <option value="title">标题</option>
      <option value="rating">评分</option>
      <option value="favorite">收藏优先</option>
    </select>

    <select :value="state.status" @change="lib.setStatus(($event.target as HTMLSelectElement).value)">
      <option value="">全部状态</option>
      <option v-for="s in STATUS_META.filter((s) => s.key)" :key="s.key" :value="s.key">
        {{ s.label }}
      </option>
    </select>

    <div class="spacer"></div>
    <button class="btn" :class="{ primary: selectMode }" title="多选：批量隐藏 / 改状态 / 移除" @click="toggleSelectMode">
      <Icon name="check" :size="15" style="margin-right: 2px" /> 多选
    </button>
    <button class="btn" title="检测目录已失效的游戏" @click="showMissing = true">
      <Icon name="eye-off" :size="15" style="margin-right: 2px" /> 失效检测
    </button>
    <button class="btn icon-btn" title="设置" @click="showSettings = true">
      <Icon name="sliders" :size="16" />
    </button>
    <button class="btn" title="galgame 资源站导航（社区 / 补丁 / 资源站）" @click="showResources = true">
      <Icon name="external-link" :size="15" style="margin-right: 2px" /> 资源站
    </button>
    <button class="btn primary" @click="showScan = true">
      <Icon name="plus" :size="15" /> 扫描导入
    </button>
  </header>

  <main class="wall" @keydown="onWallKey">
    <div v-if="state.loading && !visible.length" class="grid" aria-hidden="true">
      <div v-for="n in 12" :key="n" class="card ph"></div>
    </div>

    <div v-else-if="!counts.total.value" class="empty">
      <img class="glyph-logo" :src="brandLogo" alt="GAL 启动器" draggable="false" />
      <h2>还没有任何游戏</h2>
      <p>
        点击右上角「扫描导入」，选择一个存放 galgame 的根目录。我们会自动找到每个游戏的启动文件、
        识别引擎，并尽量为它配上封面。
      </p>
      <button class="btn primary" @click="showScan = true">
        <Icon name="plus" :size="15" /> 第一次扫描
      </button>
    </div>

    <div v-else-if="!visible.length" class="empty">
      <span class="glyph-icon"><Icon name="search" :size="34" /></span>
      <h2>没有匹配结果</h2>
      <p>换个关键词，或者切到「全部」看看。</p>
    </div>

    <div v-else class="grid">
      <GameCard
        v-for="(g, i) in visible"
        :key="g.id"
        :game="g"
        :select-mode="selectMode"
        :selected="selected.has(g.id)"
        :style="{ '--i': i }"
        @click="handleClick"
        @select="toggleSelect"
        @launch="handleLaunch"
        @favorite="handleFavorite"
        @hide="handleHide"
        @del="trashGame"
        @context="openCtx"
      />
    </div>
  </main>

  <!-- 批量操作条 -->
  <Transition name="update">
    <div v-if="selectMode && selected.size" class="batch-bar">
      <span class="bb-count">已选 {{ selected.size }} 个</span>
      <select v-model="batchStatus" style="margin-left: 8px">
        <option value="" disabled>设为状态…</option>
        <option v-for="s in STATUS_META" :key="s.key || '_'" :value="s.key">{{ s.label }}</option>
      </select>
      <button class="btn small" :disabled="!batchStatus || batchBusy" @click="batchSetStatus">应用状态</button>
      <div class="sep-v"></div>
      <button class="btn small" :disabled="batchBusy" @click="batchSetHidden(true)">隐藏</button>
      <button class="btn small" :disabled="batchBusy" @click="batchSetHidden(false)">恢复显示</button>
      <button class="btn small danger" :disabled="batchBusy" @click="batchRemove">移除记录</button>
      <div class="spacer"></div>
      <button class="btn icon-btn ghost" title="退出多选" @click="toggleSelectMode"><Icon name="close" :size="14" /></button>
    </div>
  </Transition>

  <!-- 右键菜单 -->
  <template v-if="ctx.game">
    <div class="ctx" :style="{ left: ctx.x + 'px', top: ctx.y + 'px' }">
      <button class="item" @click="ctxAction(openDir)"><Icon name="folder" :size="15" /> 打开游戏目录</button>
      <button class="item" @click="ctxAction((g) => handleLaunch(g, false))"><Icon name="play" :size="15" /> 启动</button>
      <button class="item" @click="ctxAction((g) => handleLaunch(g, true))"><Icon name="globe" :size="15" /> 转区启动</button>
      <button class="item" @click="ctxAction(pickLaunchFile)"><Icon name="sliders" :size="15" /> 启动文件…</button>
      <div class="sep"></div>
      <button class="item" @click="ctxAction(handleFavorite)">
        <Icon name="star" :size="14" :filled="ctx.game.favorite" /> {{ ctx.game.favorite ? "取消收藏" : "收藏" }}
      </button>
      <button class="item" @click="ctxAction(handleHide)">
        <Icon :name="ctx.game.hidden ? 'eye' : 'eye-off'" :size="15" />
        {{ ctx.game.hidden ? "恢复显示" : "隐藏" }}
      </button>
      <div class="sep"></div>
      <button class="item" @click="ctxAction((g) => folderHidden(g, true))"><Icon name="eye-off" :size="15" /> 系统级隐藏目录</button>
      <button class="item" @click="ctxAction((g) => folderHidden(g, false))"><Icon name="eye" :size="15" /> 取消目录隐藏属性</button>
      <div class="sep"></div>
      <button class="item" @click="ctxAction(removeFromLibrary)">从库中移除</button>
      <button class="item danger" @click="ctxAction(trashGame)"><Icon name="trash" :size="15" /> 删除（回收站）</button>
    </div>
    <div
      class="overlay"
      style="background: transparent; backdrop-filter: none"
      @click="closeCtx()"
    ></div>
  </template>

  <!-- 确认框 -->
  <Transition name="overlay">
    <div v-if="confirmBox.show" class="overlay" @click.self="confirmBox.show = false">
      <div class="modal" style="width: min(420px, 90vw)">
        <div class="head">
          <h2>{{ confirmBox.title }}</h2>
        </div>
        <div class="body">
          <p class="hint">{{ confirmBox.msg }}</p>
        </div>
        <div class="foot">
          <button class="btn" @click="confirmBox.show = false">取消</button>
          <button
            class="btn"
            :class="confirmBox.danger ? 'danger' : 'primary'"
            @click="
              confirmBox.show = false;
              confirmBox.run?.();
            "
          >
            {{ confirmBox.ok }}
          </button>
        </div>
      </div>
    </div>
  </Transition>

  <!-- 抽屉 -->
  <DetailDrawer
    :game="selectedGame"
    @close="selectedGame = null"
    @launch="handleLaunch"
    @picklaunch="pickLaunchFile"
    @updated="onDrawerUpdated"
    @notice="(m) => toast(String(m))"
    @favorite="handleFavorite"
    @hide="handleHide"
    @remove="removeFromLibrary"
    @del="trashGame"
  />

  <!-- 对话框 -->
  <Transition name="overlay">
    <ScanDialog v-if="showScan" v-model="showScan" @imported="onImported" />
  </Transition>
  <Transition name="overlay">
    <SettingsDialog v-if="showSettings" v-model="showSettings" @restored="onSettingsRestored" />
  </Transition>
  <Transition name="overlay">
    <ResourceDialog v-if="showResources" @close="showResources = false" />
  </Transition>
  <Transition name="overlay">
    <MissingDialog
      v-if="showMissing"
      @close="showMissing = false"
      @removed="lib.refresh()"
    />
  </Transition>
  <Transition name="overlay">
    <LaunchDialog
      v-if="launchPick"
      :game="launchPick.game"
      :use-locale="launchPick.useLocale"
      :pick-only="launchPick.pickOnly"
      @close="launchPick = null"
      @done="onLaunchPickDone"
    />
  </Transition>

  <!-- Toast -->
  <TransitionGroup name="toast" tag="div" class="toast-wrap">
    <div v-for="t in toasts" :key="t.id" class="toast" :class="t.type">{{ t.msg }}</div>
  </TransitionGroup>

  <!-- 更新提示横幅（左下角，非打扰；失败静默） -->
  <Transition name="update">
    <div v-if="updateBannerOpen && updateInfo" class="update-banner">
      <div class="ub-ic"><Icon name="external-link" :size="16" /></div>
      <div class="ub-body">
        <div class="ub-title">发现新版本 <b>{{ updateInfo.version }}</b></div>
        <div class="ub-note" v-if="updateNote()">{{ updateNote() }}</div>
      </div>
      <div class="ub-actions">
        <button class="btn small primary" @click="goUpdate"><Icon name="download" :size="13" /> 下载更新</button>
        <button class="btn small ghost" @click="dismissUpdateForever">不再提示</button>
        <button class="btn icon-btn ghost" title="暂时关闭（下次启动再看）" @click="updateBannerOpen = false">
          <Icon name="close" :size="14" />
        </button>
      </div>
    </div>
  </Transition>

  <!-- Android 首启权限引导遮罩 -->
  <Transition name="overlay">
    <div v-if="isAndroid && filesAccess === false" class="perm-gate">
      <div class="perm-card">
        <img class="glyph-logo" :src="brandLogo" alt="GAL 启动器" draggable="false" />
        <h2>需要「所有文件访问」权限</h2>
        <p class="perm-desc">
          GAL 启动器需要读取手机上的游戏目录，才能扫描识别引擎、管理补丁与解包资源。
          请在本应用的「权限 → 所有文件访问」中开启。
        </p>
        <button class="btn primary" @click="requestFilesAccess">
          <Icon name="sliders" :size="15" /> 去开启权限
        </button>
        <p class="hint" v-if="filesAccessHint">{{ filesAccessHint }}</p>
        <button class="btn ghost" @click="checkFilesAccess">我已开启，重新检测</button>
      </div>
    </div>
  </Transition>
</template>
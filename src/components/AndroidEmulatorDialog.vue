<script setup lang="ts">
import { computed, ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";
import type { Game } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ game: Game }>();
const emit = defineEmits<{ close: [] }>();
useCloseOnEscape(() => emit("close"));

interface EmulatorEntry {
  id: string;
  name: string;
  packageName: string;
}


const emulators = ref<EmulatorEntry[]>([]);
const result = ref("");
const launching = ref(false);

const QUICK_EMULATORS = [
  { id: "kirikiroid2", name: "Kirikiroid2", desc: "吉里吉里 / KiriKiri 引擎" },
  { id: "onscripter", name: "ONScripter", desc: "NScripter / ONScripter 引擎" },
  { id: "tyranor", name: "Tyranor", desc: "Tyranor 脚本引擎" },
];

const recommended = computed(() => {
  const e = (props.game.engine || "").toLowerCase();
  if (e.includes("吉里") || e.includes("kiri") || e.includes("krkr")) return "kirikiroid2";
  if (e.includes("nscr") || e.includes("onscrip")) return "onscripter";
  if (e.includes("tyranor")) return "tyranor";
  return "";
});

const DOWNLOADS: { name: string; desc: string; url: string }[] = [
  { name: "JoiPlay", desc: "RPG Maker / Ren'Py / Unity 运行时", url: "https://joiplay.com" },
  { name: "ExaGear", desc: "Windows 游戏模拟器", url: "https://play.google.com/store/search?q=ExaGear" },
  { name: "Kirikiroid2", desc: "吉里吉里引擎运行时", url: "https://github.com/Kirikiroid2/Kirikiroid2" },
  { name: "ONScripter", desc: "NScripter / ONScripter 模拟器", url: "https://play.google.com/store/search?q=ONScripter" },
  { name: "Tyranor", desc: "Tyranor 脚本引擎", url: "https://tyranor.jp/" },
  { name: "Ren'Py", desc: "Ren'Py 官方站", url: "https://www.renpy.org/" },
  { name: "ScummVM", desc: "文字冒险 / 解谜通用模拟器", url: "https://www.scummvm.org/" },
  { name: "RetroArch", desc: "跨平台全能模拟器", url: "https://www.retroarch.com/" },
];

function openDownload(u: string) {
  openUrl(u).catch(() => {});
}

const bridge = () =>
  (
    window as unknown as {
      GalLauncherAndroid?: {
        getKnownEmulators?: () => string;
        launchFileWithChooser?: (path: string) => void;
        launchWithPackage?: (path: string, pkg: string) => boolean;
        launchKnownEmulator?: (path: string, id: string) => void;
      };
    }
  ).GalLauncherAndroid;

function loadEmulators() {
  const b = bridge();
  if (!b?.getKnownEmulators) return;
  try {
    emulators.value = JSON.parse(b.getKnownEmulators()) as EmulatorEntry[];
  } catch {
    emulators.value = [];
  }
}

function launchSystemChooser() {
  const b = bridge();
  if (!b?.launchFileWithChooser) return;
  b.launchFileWithChooser(props.game.launchPath || props.game.sourceDir);
  emit("close");
}

function launch(pkg: string) {
  const b = bridge();
  if (!b?.launchWithPackage) {
    result.value = "当前版本不支持直接唤起模拟器，请改用系统选择器。";
    return;
  }
  launching.value = true;
  result.value = "";
  // 原生桥是异步拉起，这里不再依赖返回值判断，直接提示已尝试交给对应应用
  b.launchWithPackage(props.game.launchPath || props.game.sourceDir, pkg);
  launching.value = false;
  result.value = "已尝试交给模拟器，请留意是否弹出模拟器窗口；如果没有反应，可用「系统打开方式」重试。";
  window.setTimeout(() => emit("close"), 1600);
}

function launchKnown(id: string) {
  const b = bridge();
  if (!b?.launchKnownEmulator) {
    result.value = "当前版本不支持直接唤起该模拟器，请先安装最新 APK。";
    return;
  }
  launching.value = true;
  result.value = "正在尝试启动「" + (QUICK_EMULATORS.find((q) => q.id === id)?.name ?? id) + "」…";
  b.launchKnownEmulator(props.game.launchPath || props.game.sourceDir, id);
  window.setTimeout(() => {
    launching.value = false;
    result.value = "已尝试启动模拟器；如果没有自动加载游戏，请在模拟器内手动打开游戏目录。";
  }, 1200);
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(460px, 92vw)">
      <div class="head">
        <h2>选择手机模拟器</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>
      <div class="body">
        <p class="hint">
          「{{ props.game.title }}」在 Android 上不能直接运行 PC 启动文件。
          请选择已安装的 galgame 模拟器 / 运行时来打开。
        </p>

        <button class="emu-row primary" @click="launchSystemChooser">
          <Icon name="external-link" :size="15" />
          <span>
            <b>系统打开方式</b>
            <small>让 Android 自己列出能处理该文件的模拟器</small>
          </span>
        </button>

        <div v-if="emulators.length" class="row" style="justify-content: space-between; margin: 4px 0 4px">
          <div class="section-title" style="margin: 0">已检测到的模拟器</div>
          <button class="btn small ghost" @click="loadEmulators">
            <Icon name="search" :size="12" /> 重新检测
          </button>
        </div>
        <div v-else class="row" style="justify-content: space-between; margin: 4px 0 10px">
          <span class="muted">未检测到常见模拟器，可点击「重新检测」或使用系统打开方式。</span>
          <button class="btn small ghost" @click="loadEmulators">
            <Icon name="search" :size="12" /> 检测
          </button>
        </div>

        <button
          v-for="e in emulators"
          :key="e.id"
          class="emu-row"
          :disabled="launching"
          @click="launch(e.packageName)"
        >
          <Icon name="play" :size="15" />
          <span>
            <b>{{ e.name }}</b>
            <small>{{ e.packageName }}</small>
          </span>
        </button>

        <div class="section-title" style="margin-top: 16px">已集成模拟器</div>
        <button
          v-for="q in QUICK_EMULATORS"
          :key="q.id"
          class="emu-row"
          :class="{ recommended: recommended === q.id }"
          :disabled="launching"
          @click="launchKnown(q.id)"
        >
          <Icon name="play" :size="15" />
          <span>
            <b>{{ q.name }} <em v-if="recommended === q.id" class="rec">推荐</em></b>
            <small>{{ q.desc }}</small>
          </span>
        </button>

        <div class="section-title" style="margin-top: 18px">模拟器下载</div>
        <div class="downloads">
          <div v-for="d in DOWNLOADS" :key="d.name" class="download-row">
            <div style="flex: 1; min-width: 0">
              <div class="nm">{{ d.name }}</div>
              <div class="muted">{{ d.desc }}</div>
            </div>
            <a class="btn small" href="javascript:void(0)" @click="openDownload(d.url)">
              <Icon name="external-link" :size="13" /> 下载
            </a>
          </div>
        </div>

        <p v-if="result" class="hint" style="color: var(--accent); margin: 8px 0 0">{{ result }}</p>
      </div>
      <div class="foot">
        <span class="muted" style="margin-right: auto">Windows 版转区启动在手机端不适用</span>
        <button class="btn" @click="emit('close')">取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.emu-row {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 12px 14px;
  margin-bottom: 8px;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--bg-soft);
  color: var(--text);
  text-align: left;
  transition:
    background-color var(--d-fast) var(--ease-out),
    border-color var(--d-fast) var(--ease-out);
}
.emu-row:hover {
  background: var(--surface-2);
  border-color: var(--border-strong);
}
.emu-row.primary {
  border-color: rgba(217, 126, 61, 0.4);
  background: rgba(217, 126, 61, 0.12);
}
.emu-row span {
  display: flex;
  flex-direction: column;
  min-width: 0;
  flex: 1;
}
.app-list {
  max-height: 260px;
  overflow-y: auto;
  margin-bottom: 10px;
}

.downloads {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}
.download-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  border-radius: 10px;
  background: var(--surface-2);
  border: 1px solid var(--border);
}
.download-row .nm {
  font-size: 13px;
  font-weight: 600;
}
.download-row .muted {
  font-size: 11.5px;
  margin-top: 2px;
}

.emu-row.recommended {
  border-color: rgba(217, 126, 61, 0.5);
  background: rgba(217, 126, 61, 0.1);
}
.emu-row b {
  font-size: 13.5px;
  font-weight: 600;
}
.rec {
  font-style: normal;
  font-size: 10px;
  color: var(--accent-ink);
  background: var(--accent);
  border-radius: 999px;
  padding: 1px 6px;
  margin-left: 6px;
  vertical-align: middle;
}
.emu-row small {
  font-size: 11.5px;
  color: var(--text-faint);
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

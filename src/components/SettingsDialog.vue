<script setup lang="ts">
import { ref, watch } from "vue";
import { confirm, open } from "@tauri-apps/plugin-dialog";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api } from "../api";
import Icon from "./Icon.vue";
import brandLogo from "../assets/brand-logo.png";
import FolderPickerDialog from "./FolderPickerDialog.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [v: boolean]; restored: [] }>();
useCloseOnEscape(() => emit("update:modelValue", false));

const lePath = ref("");
const gameRoot = ref("");
const unpackTool = ref("");
const saved = ref(false);
const err = ref("");
const bakBusy = ref(false);
const bakMsg = ref("");
const coverBusy = ref(false);
const coverMsg = ref("");

watch(
  () => props.modelValue,
  async (v) => {
    if (!v) return;
    saved.value = false;
    err.value = "";
    bakBusy.value = false;
    bakMsg.value = "";
    try {
      const s = await api.getSettings();
      lePath.value = s.localeEmulatorPath ?? "";
      gameRoot.value = s.gameRoot ?? "";
      unpackTool.value = s.unpackTool ?? "";
    } catch (e) {
      err.value = String(e);
    }
  }
);

async function browseLE() {
  const p = await open({
    multiple: false,
    title: "选择 Locale Emulator 的 LEProc.exe",
    filters: [{ name: "可执行文件", extensions: ["exe"] }],
  });
  if (p) lePath.value = p;
}

const folderStart = ref("C:\\");
const showFolder = ref(false);
/** 内置轻量目录选择器（原生对话框在巨型目录下会卡死窗口，故不用）。 */
function openFolderPicker() {
  folderStart.value = gameRoot.value || "C:\\";
  showFolder.value = true;
}
function onFolderPicked(dir: string) {
  showFolder.value = false;
  gameRoot.value = dir;
}

async function browseUnpack() {
  const p = await open({
    multiple: false,
    title: "选择外部解包工具主程序（如 GarBro / GalArc / arc_unpacker）",
    filters: [{ name: "可执行文件", extensions: ["exe"] }],
  });
  if (p) unpackTool.value = p;
}

/** 预设的常用外部解包工具（exe 文件名用于自动检测；url 为官方下载入口，空表示暂无稳定来源）。 */
const UNPACK_PRESETS = [
  {
    name: "GARbro",
    exe: "GARbro.exe",
    gui: true,
    url: "https://github.com/morkt/GARbro/releases",
  },
  {
    name: "GalArc",
    exe: "GalArc.exe",
    gui: false,
    url: "https://github.com/detached64/GalArc/releases/latest",
  },
  {
    name: "arc_unpacker",
    exe: "arc_unpacker.exe",
    gui: false,
    url: "https://github.com/vn-tools/arc_unpacker/releases",
  },
  { name: "Dumper2", exe: "dumper2.exe", gui: true, url: "" },
] as const;
const presetIdx = ref(-1); // -1 = 自定义
const foundTools = ref<Record<string, string>>({});
const detectMsg = ref("");
const detecting = ref(false);

function applyPreset() {
  if (presetIdx.value < 0) return;
  const p = UNPACK_PRESETS[presetIdx.value];
  const hit = foundTools.value[p.exe.toLowerCase()];
  if (hit) unpackTool.value = hit;
}

/** 在默认浏览器里打开所选预设工具的官方下载页。 */
function openPresetUrl() {
  const u = UNPACK_PRESETS[presetIdx.value]?.url;
  if (u) openUrl(u).catch(() => {});
}

async function detectTools() {
  detecting.value = true;
  detectMsg.value = "";
  try {
    foundTools.value = await api.searchUnpackTools(UNPACK_PRESETS.map((p) => p.exe));
    const entries = Object.entries(foundTools.value);
    if (entries.length) {
      detectMsg.value = `常见位置找到 ${entries.length} 个：` + entries.map(([e, p]) => `${e} → ${p}`).join("；");
      applyPreset();
    } else {
      detectMsg.value = "常见位置（下载/桌面/文档/工具目录、Program Files 及各盘根目录）没找到，可用「浏览」手动选择。";
    }
  } catch (e) {
    detectMsg.value = String(e);
  } finally {
    detecting.value = false;
  }
}

/** 导出整库备份（db + 封面 + 补丁备份，不含解包缓存）。 */
async function doBackup() {
  const dest = await open({ directory: true, multiple: false, title: "选择备份存放目录" });
  if (!dest) return;
  bakBusy.value = true;
  bakMsg.value = "";
  try {
    const r = await api.exportBackup(String(dest));
    bakMsg.value = `已导出 ${r.fileCount} 个文件 → ${r.path}`;
  } catch (e) {
    bakMsg.value = String(e);
  } finally {
    bakBusy.value = false;
  }
}

/** 从备份 zip 恢复（会替换现有库 / 封面 / 补丁备份）。 */
async function doRestore() {  const file = await open({
    multiple: false,
    title: "选择备份文件",
    filters: [{ name: "备份文件", extensions: ["zip"] }],
  });
  if (!file) return;
  const ok = await confirm(
    "恢复会替换当前游戏库、封面与补丁备份（解包缓存不受影响）。建议先导出一份当前备份。确定继续？",
    { title: "从备份恢复", kind: "warning" }
  );
  if (!ok) return;
  bakBusy.value = true;
  bakMsg.value = "";
  try {
    const r = await api.importBackup(String(file));
    emit("restored");
    bakMsg.value = `已恢复：${r.games} 个游戏记录、${r.covers} 个封面、${r.backups} 个补丁备份`;
  } catch (e) {
    bakMsg.value = String(e);
  } finally {
    bakBusy.value = false;
  }
}

/** 为所有还没有封面的游戏从 VNDB 批量补封面+元数据（移到设置页，避免顶栏常驻低频操作）。 */
async function doFetchCovers() {
  if (coverBusy.value) return;
  coverBusy.value = true;
  coverMsg.value = "";
  try {
    const r = await api.fetchMissingCovers();
    if (r.updated > 0) coverMsg.value = `已补全 ${r.updated} 个封面`;
    else coverMsg.value = "没有需要补封面的游戏了";
    if (r.failed.length) coverMsg.value += `；${r.failed.length} 个未匹配上（可在详情里手动搜 VNDB）`;
  } catch (e) {
    coverMsg.value = String(e);
  } finally {
    coverBusy.value = false;
  }
}

async function save() {
  err.value = "";
  try {
    await api.saveSetting("locale_emulator_path", lePath.value.trim());
    await api.saveSetting("game_root", gameRoot.value.trim());
    await api.saveSetting("unpack_tool", unpackTool.value.trim());
    saved.value = true;
    setTimeout(() => emit("update:modelValue", false), 500);
  } catch (e) {
    err.value = String(e);
  }
}
</script>

<template>
  <div v-if="props.modelValue" class="overlay" @click.self="emit('update:modelValue', false)">
    <div class="modal" style="width: min(520px, 92vw)">
      <div class="head">
        <h2>设置</h2>
        <button class="btn icon-btn ghost" @click="emit('update:modelValue', false)"><Icon name="close" :size="15" /></button>
      </div>
      <div class="body">
        <div class="field">
          <label>Locale Emulator 路径（用于日文原版转区启动）</label>
          <div class="row">
            <input type="text" v-model="lePath" placeholder="如 D:\Locale Emulator\LEProc.exe" />
            <button class="btn small" @click="browseLE">浏览</button>
          </div>
          <p class="muted">未配置时，转区启动会直接提示错误，普通启动不受影响。</p>
        </div>
        <div class="field">
          <label>默认游戏根目录（可选，仅记录用于快捷扫描）</label>
          <div class="row">
            <input type="text" v-model="gameRoot" placeholder="如 E:\Galgame" />
            <button class="btn small" @click="openFolderPicker">浏览</button>
          </div>
        </div>
        <div class="field">
          <label>外部解包工具（可选；PAC / NSA / PKG 等内置不支持的格式会用它）</label>
          <div class="row">
            <select v-model.number="presetIdx" @change="applyPreset">
              <option :value="-1">自定义路径…</option>
              <option v-for="(p, i) in UNPACK_PRESETS" :key="p.exe" :value="i">{{ p.name }}</option>
            </select>
            <button class="btn small" :disabled="detecting" @click="detectTools">
              <Icon name="search" :size="13" /> {{ detecting ? "检测中…" : "检测" }}
            </button>
            <button
              v-if="presetIdx >= 0 && UNPACK_PRESETS[presetIdx].url"
              class="btn small ghost"
              :title="UNPACK_PRESETS[presetIdx].url"
              @click="openPresetUrl"
            >
              <Icon name="download" :size="13" /> 下载
            </button>
          </div>
          <div class="row">
            <input type="text" v-model="unpackTool" placeholder="如 F:\tools\GARbro.exe" />
            <button class="btn small" @click="browseUnpack">浏览</button>
          </div>
          <p class="muted" v-if="presetIdx >= 0">
            {{
              UNPACK_PRESETS[presetIdx].gui
                ? "提示：这是 GUI 工具，命令行桥接可能不适用 —— 填入路径后，可在该工具界面里手动解包。"
                : "约定命令行：<工具> <压缩包> <输出目录>；若你的版本参数不同，请切回「自定义路径…」。"
            }}
          </p>
          <p class="muted" v-if="presetIdx >= 0 && !UNPACK_PRESETS[presetIdx].url">
            {{ UNPACK_PRESETS[presetIdx].name }} 暂无稳定的官方下载地址，可用「浏览」手动指定已有工具。
          </p>
          <p class="muted" v-if="detectMsg">{{ detectMsg }}</p>
        </div>

        <div class="field">
          <label>整库备份 / 恢复</label>
          <div class="row">
            <button class="btn small" :disabled="bakBusy" @click="doBackup">
              <Icon name="upload" :size="13" /> 导出备份…
            </button>
            <button class="btn small" :disabled="bakBusy" @click="doRestore">
              <Icon name="download" :size="13" /> 从备份恢复…
            </button>
          </div>
          <p class="muted">备份 = 游戏库 + 封面 + 补丁备份；不含可重新解包的资源缓存。</p>
          <p class="muted" v-if="bakMsg" style="word-break: break-all">{{ bakMsg }}</p>
        </div>

        <div class="field">
          <label>元数据补全</label>
          <div class="row">
            <button class="btn small" :disabled="coverBusy" @click="doFetchCovers">
              <Icon name="image" :size="13" /> 批量补全缺失封面
            </button>
          </div>
          <p class="muted">为没有封面的游戏从 VNDB 拉取封面与元数据（评分 / 简介 / 标签 / 厂商 / 时长）。</p>
          <p class="muted" v-if="coverMsg" style="word-break: break-all">{{ coverMsg }}</p>
        </div>

        <div v-if="err" class="toast err">{{ err }}</div>
        <div v-if="saved" class="toast ok">已保存</div>
      </div>
      <div class="foot">
        <div class="about">
          <img :src="brandLogo" alt="GAL 启动器" draggable="false" />
          <span><b>GAL 启动器</b><em>v0.1.0</em></span>
        </div>
        <div class="spacer"></div>
        <button class="btn" @click="emit('update:modelValue', false)">取消</button>
        <button class="btn primary" @click="save">保存</button>
      </div>
    </div>

    <FolderPickerDialog
      v-if="showFolder"
      :root="folderStart"
      title="选择默认游戏根目录"
      @picked="onFolderPicked"
      @close="showFolder = false"
    />
  </div>
</template>

<style scoped>
.about {
  display: flex;
  align-items: center;
  gap: 9px;
  margin-right: auto;
  opacity: 0.82;
}
.about img {
  width: 26px;
  height: 26px;
  border-radius: 7px;
  object-fit: cover;
  border: 1px solid rgba(255, 250, 244, 0.12);
  box-shadow: 0 2px 8px rgba(20, 12, 6, 0.45);
  user-select: none;
  -webkit-user-drag: none;
}
.about span {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}
.about b {
  font-family: var(--font-display);
  font-weight: 650;
  font-size: 12.5px;
  color: var(--text);
  letter-spacing: 0.01em;
}
.about em {
  font-style: normal;
  font-size: 10.5px;
  color: var(--text-faint);
  font-variant-numeric: tabular-nums;
}
</style>
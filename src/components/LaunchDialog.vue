<script setup lang="ts">
import { computed, ref } from "vue";
import { api, type Game } from "../api";
import PickExeDialog from "./PickExeDialog.vue";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

/**
 * 启动文件选择器。
 * - autoLaunch（默认）：用户点某个候选 → 立刻用它启动，并持久化为该游戏默认启动文件。
 * - pickOnly：只把选中的设为默认启动文件，不启动（用于「更换启动文件」）。
 */
const props = defineProps<{
  game: Game | null;
  useLocale?: boolean;
  pickOnly?: boolean;
}>();
const emit = defineEmits<{
  close: [];
  done: [g: Game, msg: string];
}>();
useCloseOnEscape(() => emit("close"));

const busy = ref("");
const err = ref("");
const showPicker = ref(false);

const sep = computed(() => (props.game?.sourceDir.includes("\\") ? "\\" : "/"));

function resolve(rel: string): string {
  const g = props.game;
  if (!g) return "";
  return `${g.sourceDir}${sep.value}${rel}`;
}

/** rel 路径的展示：文件名 + 所在子目录。 */
function splitRel(rel: string): { name: string; dir: string } {
  const idx = Math.max(rel.lastIndexOf("\\"), rel.lastIndexOf("/"));
  if (idx < 0) return { name: rel, dir: "" };
  return { name: rel.slice(idx + 1), dir: rel.slice(0, idx) };
}

function isCurrent(rel: string): boolean {
  return resolve(rel).toLowerCase() === props.game?.launchPath.toLowerCase();
}

async function choose(fullPath: string) {
  const g = props.game;
  if (!g) return;
  busy.value = fullPath;
  err.value = "";
  try {
    if (props.pickOnly) {
      const updated = await api.setLaunchFile(g.id, fullPath);
      emit("done", updated, "已设为默认启动文件");
    } else {
      const updated = await api.launchGame(g.id, props.useLocale ?? false, fullPath);
      emit("done", updated, props.useLocale ? "已通过转区启动" : "已启动");
    }
  } catch (e) {
    err.value = String(e);
  } finally {
    busy.value = "";
  }
}

function custom() {
  // 用内置文件选择器替代原生对话框（原生对话框在巨目录下会卡死整个窗口）
  showPicker.value = true;
}

const empty = computed(() => !props.game?.launchCandidates.length);
</script>

<template>
  <div v-if="props.game" class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(560px, 92vw)">
      <div class="head">
        <h2>{{ props.pickOnly ? "更换启动文件" : "选择启动文件" }}</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <p class="hint" v-if="props.pickOnly">「{{ props.game.title }}」当前使用：</p>
        <p class="hint" v-else>
          首次启动「{{ props.game.title }}」，目录里识别到 {{ props.game.launchCandidates.length }} 个启动文件。
          选一个吧（之后会记住，也可随时更换）：
        </p>

        <div class="entry" v-if="empty">（没有自动识别到的启动文件，请用下方按钮手动指定）</div>

        <div class="cand-list launch">
          <div
            v-for="rel in props.game.launchCandidates"
            :key="rel"
            class="cand launch-cand"
            :class="{ cur: isCurrent(rel) }"
          >
            <div style="flex: 1; min-width: 0">
              <div class="nm">{{ splitRel(rel).name }}</div>
              <div class="sub">
                <template v-if="splitRel(rel).dir">位于 {{ splitRel(rel).dir }}</template>
                <template v-else>游戏根目录</template>
                <span v-if="isCurrent(rel)" class="already">当前默认</span>
              </div>
            </div>
            <button class="btn primary small" :disabled="busy.length > 0" @click="choose(resolve(rel))">
              <template v-if="busy !== resolve(rel) && !props.pickOnly"><Icon name="play" :size="13" /></template>
              {{ busy === resolve(rel) ? "启动中…" : props.pickOnly ? "设为默认" : (busy ? "…" : "启动") }}
            </button>
          </div>
        </div>

        <div v-if="err" class="toast err">{{ err }}</div>
      </div>

      <div class="foot">
        <button class="btn" @click="custom"><Icon name="folder" :size="14" /> 自定义启动文件…</button>
        <button class="btn ghost" @click="emit('close')">取消</button>
      </div>
    </div>

    <PickExeDialog
      v-if="showPicker"
      :root="props.game.sourceDir"
      @picked="(p) => { showPicker = false; choose(String(p)); }"
      @close="showPicker = false"
    />
  </div>
</template>

<style scoped>
.launch-cand.cur {
  border-color: var(--accent);
}
</style>
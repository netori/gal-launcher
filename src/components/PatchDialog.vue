<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { api, type Game, type Patch } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ game: Game | null }>();
const emit = defineEmits<{
  close: [];
  added: [p: Patch];
}>();
useCloseOnEscape(() => emit("close"));

const name = ref("");
const kind = ref("汉化");
const sourcePath = ref("");
const installMethod = ref("replace");
const busy = ref(false);
const err = ref("");

watch(
  () => props.game,
  () => {
    name.value = "";
    kind.value = "汉化";
    sourcePath.value = "";
    installMethod.value = "replace";
    err.value = "";
  }
);

function guessFromPath(p: string) {
  const base = p.replace(/\\/g, "/");
  const leaf = base.split("/").pop() ?? p;
  name.value = leaf.replace(/\.(zip|7z|rar|exe)$/i, "");
  installMethod.value = /\.exe$/i.test(leaf) ? "installer" : "replace";
}

async function pickFile() {
  const p = await open({
    multiple: false,
    title: "选择补丁压缩包或安装程序（zip / exe）",
    filters: [{ name: "补丁文件", extensions: ["zip", "exe"] }],
    // 注意：不给 defaultPath 锚到游戏目录，避免原生对话框在巨型目录下卡死
  });
  if (p && typeof p === "string") {
    sourcePath.value = p;
    guessFromPath(p);
  }
}

async function pickFolder() {
  const p = await open({
    directory: true,
    multiple: false,
    title: "选择补丁文件夹（覆盖式）",
  });
  if (p) {
    sourcePath.value = p;
    const leaf = (p.replace(/\\/g, "/").split("/").pop() ?? "补丁").trim();
    name.value = leaf || "补丁";
    installMethod.value = "replace";
  }
}

const canSave = computed(() => name.value.trim() && sourcePath.value.trim());

async function save() {
  if (!props.game || !canSave.value) return;
  busy.value = true;
  err.value = "";
  try {
    const patch = await api.addPatch({
      gameId: props.game.id,
      name: name.value.trim(),
      kind: kind.value,
      sourcePath: sourcePath.value.trim(),
      installMethod: installMethod.value,
    });
    emit("added", patch);
    emit("close");
  } catch (e) {
    err.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div v-if="props.game" class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(520px, 92vw)">
      <div class="head">
        <h2>添加补丁 · {{ props.game.title }}</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <p class="hint">
          汉化 / R18 补丁登记后即可一键安装与回滚。覆盖式补丁安装前会自动备份被覆盖的文件。
        </p>

        <div class="field">
          <label>补丁来源</label>
          <div class="row">
            <input v-model="sourcePath" type="text" placeholder="选择 zip / 文件夹 / 安装器 exe" />
            <button class="btn small" @click="pickFile" :disabled="busy">文件…</button>
            <button class="btn small" @click="pickFolder" :disabled="busy">文件夹…</button>
          </div>
        </div>

        <div class="field">
          <label>名称</label>
          <input v-model="name" type="text" placeholder="如：官方汉化补丁 / R18 追加补丁" />
        </div>

        <div class="field">
          <label>类型</label>
          <select v-model="kind">
            <option>汉化</option>
            <option>R18</option>
            <option>修正</option>
            <option>其他</option>
          </select>
        </div>

        <div class="field">
          <label>安装方式</label>
          <div class="row" style="gap: 10px; font-size: 13px">
            <label style="display: flex; align-items: center; gap: 5px; cursor: pointer">
              <input type="radio" value="replace" v-model="installMethod" /> 覆盖式（自动备份）
            </label>
            <label style="display: flex; align-items: center; gap: 5px; cursor: pointer">
              <input type="radio" value="installer" v-model="installMethod" /> 安装器（运行 exe）
            </label>
          </div>
        </div>

        <div v-if="err" class="toast err" style="margin-top: 6px">{{ err }}</div>
      </div>

      <div class="foot">
        <button class="btn" @click="emit('close')" :disabled="busy">取消</button>
        <button class="btn primary" :disabled="!canSave || busy" @click="save">
          {{ busy ? "登记中…" : "登记补丁" }}
        </button>
      </div>
    </div>
  </div>
</template>
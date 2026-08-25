<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api, type CollectionSummary, type Game } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const emit = defineEmits<{ close: [] }>();
useCloseOnEscape(() => emit("close"));

const collections = ref<CollectionSummary[]>([]);
const games = ref<Game[]>([]);
const selected = ref<CollectionSummary | null>(null);
const selectedGames = ref<Game[]>([]);
const name = ref("");
const addGameId = ref<number | "">("");
const err = ref("");
const busy = ref(false);

const availableGames = computed(() => {
  const inSet = new Set(selectedGames.value.map((g) => g.id));
  return games.value.filter((g) => !inSet.has(g.id));
});

async function refreshCollections() {
  try {
    collections.value = await api.listCollections();
  } catch (e) {
    err.value = String(e);
  }
}

async function openCollection(c: CollectionSummary) {
  selected.value = c;
  try {
    selectedGames.value = await api.listCollectionGames(c.id);
  } catch (e) {
    err.value = String(e);
  }
}

async function create() {
  if (!name.value.trim() || busy.value) return;
  busy.value = true;
  err.value = "";
  try {
    await api.createCollection(name.value.trim());
    name.value = "";
    await refreshCollections();
  } catch (e) {
    err.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function removeCollection(c: CollectionSummary) {
  if (!confirm(`删除分组「${c.name}」？不会删除游戏本体。`)) return;
  try {
    await api.deleteCollection(c.id);
    if (selected.value?.id === c.id) selected.value = null;
    await refreshCollections();
  } catch (e) {
    err.value = String(e);
  }
}

async function addGame() {
  if (!selected.value || !addGameId.value) return;
  try {
    await api.addToCollection(selected.value.id, Number(addGameId.value));
    addGameId.value = "";
    await openCollection(selected.value);
  } catch (e) {
    err.value = String(e);
  }
}

async function removeGame(g: Game) {
  if (!selected.value) return;
  try {
    await api.removeFromCollection(selected.value.id, g.id);
    selectedGames.value = selectedGames.value.filter((x) => x.id !== g.id);
  } catch (e) {
    err.value = String(e);
  }
}

onMounted(async () => {
  try {
    games.value = await api.listGames(true);
  } catch (e) {
    err.value = String(e);
  }
  await refreshCollections();
});
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(680px, 94vw)">
      <div class="head">
        <h2>收藏分组</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <div class="row" style="margin-bottom: 12px">
          <input v-model="name" type="text" placeholder="新建分组名称，如：高分 / 想玩 / 已通关" @keyup.enter="create" />
          <button class="btn primary" :disabled="busy || !name.trim()" @click="create">
            <Icon name="plus" :size="14" /> 新建分组
          </button>
        </div>

        <div v-if="!selected" class="group-grid">
          <button v-for="c in collections" :key="c.id" class="group-card" @click="openCollection(c)">
            <span class="group-name">{{ c.name }}</span>
            <span class="group-count">{{ c.count }} 款</span>
            <span class="group-del" @click.stop="removeCollection(c)"><Icon name="close" :size="13" /></span>
          </button>
          <div v-if="!collections.length" class="muted">还没有分组，先创建一个吧。</div>
        </div>

        <template v-else>
          <div class="row" style="justify-content: space-between">
            <div class="section-title" style="margin: 0">{{ selected.name }} <b>{{ selectedGames.length }}</b></div>
            <button class="btn small ghost" @click="selected = null"><Icon name="arrow-up" :size="13" /> 返回分组列表</button>
          </div>

          <div class="row" style="margin: 8px 0 12px">
            <select v-model="addGameId" style="flex: 1">
              <option value="" disabled>选择要添加的游戏…</option>
              <option v-for="g in availableGames" :key="g.id" :value="g.id">{{ g.title }}</option>
            </select>
            <button class="btn small" :disabled="!addGameId" @click="addGame">添加</button>
          </div>

          <div class="group-game-list">
            <div v-for="g in selectedGames" :key="g.id" class="group-game">
              <div class="g-title">{{ g.title }}</div>
              <button class="btn small ghost" @click="removeGame(g)">移除</button>
            </div>
            <div v-if="!selectedGames.length" class="muted">这个分组还没有游戏。</div>
          </div>
        </template>

        <div v-if="err" class="toast err" style="margin-top: 10px">{{ err }}</div>
      </div>

      <div class="foot">
        <button class="btn ghost" @click="emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.group-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 10px;
}
.group-card {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 6px;
  padding: 14px;
  border-radius: 12px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  color: var(--text);
  text-align: left;
  transition: border-color var(--d-fast), background-color var(--d-fast);
}
.group-card:hover {
  background: var(--surface);
  border-color: var(--border-strong);
}
.group-name {
  font-size: 14px;
  font-weight: 650;
}
.group-count {
  font-size: 12px;
  color: var(--text-dim);
}
.group-del {
  position: absolute;
  top: 8px;
  right: 8px;
  color: var(--text-faint);
  padding: 4px;
  border-radius: 6px;
}
.group-del:hover {
  color: var(--danger);
  background: rgba(226, 100, 92, 0.12);
}
.group-game-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.group-game {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.g-title {
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

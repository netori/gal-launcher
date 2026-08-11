<script setup lang="ts">
import { ref } from "vue";
import { api, type Game, type VnSearchHit } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ game: Game | null }>();
const emit = defineEmits<{
  close: [];
  applied: [g: Game];
}>();
useCloseOnEscape(() => emit("close"));

const q = ref("");
const hits = ref<VnSearchHit[]>([]);
const searching = ref(false);
const applying = ref("");
const err = ref("");
const useVndbTitle = ref(false);

async function search() {
  if (!q.value.trim()) return;
  searching.value = true;
  err.value = "";
  hits.value = [];
  try {
    hits.value = await api.searchVndb(q.value.trim());
  } catch (e) {
    err.value = String(e);
  } finally {
    searching.value = false;
  }
}

async function applyHit(h: VnSearchHit) {
  if (!props.game) return;
  applying.value = h.vndbId;
  err.value = "";
  try {
    const updated = await api.applyVndbMetadata(props.game.id, h.vndbId, useVndbTitle.value);
    emit("applied", updated);
    emit("close");
  } catch (e) {
    err.value = String(e);
  } finally {
    applying.value = "";
  }
}

function rating(h: VnSearchHit): string {
  return h.rating != null ? (h.rating / 10).toFixed(2) : "—";
}
</script>

<template>
  <div v-if="props.game" class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(680px, 94vw)">
      <div class="head">
        <h2>从 VNDB 补全元数据</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <p class="hint">
          「{{ props.game.title }}」 → 搜索 VNDB，选一条应用即可拉取封面 / 简介 / 评分 / 标签等。
        </p>

        <div class="row" style="gap: 8px">
          <input
            v-model="q"
            type="text"
            placeholder="输入要搜索的标题（梵文或中英文都行）"
            @keyup.enter="search"
            @focus="q = q || (props.game?.title ?? '')"
          />
          <button class="btn primary" :disabled="searching || !q.trim()" @click="search">
            {{ searching ? "搜索中…" : "搜索" }}
          </button>
        </div>

        <label class="toggle" style="margin-top: 8px">
          <input type="checkbox" v-model="useVndbTitle" />
          同时把游戏标题改为 VNDB 主标题
        </label>

        <div v-if="err" class="toast err" style="margin-top: 6px">{{ err }}</div>

        <div v-if="hits.length" class="vn-list">
          <div v-for="h in hits" :key="h.vndbId" class="vn">
            <img v-if="h.imageUrl" class="vh" :src="h.imageUrl" alt="" loading="lazy" />
            <div v-else class="vh no">{{ h.title.charAt(0) }}</div>
            <div style="flex: 1; min-width: 0">
              <div class="nm">{{ h.title }}</div>
              <div class="sub">vndb:{{ h.vndbId }} · ★ {{ rating(h) }}（{{ h.votecount }} 票）</div>
            </div>
            <button
              class="btn primary small"
              :disabled="applying.length > 0"
              @click="applyHit(h)"
            >
              {{ applying === h.vndbId ? "应用中…" : "应用" }}
            </button>
          </div>
        </div>

        <div v-else-if="!searching && q" class="muted" style="padding: 12px 2px">
          没有结果（VNDB 按标题检索，试试搜罗马音/日文原名）。
        </div>
      </div>

      <div class="foot">
        <button class="btn ghost" @click="emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.vn-list {
  margin-top: 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.vn {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.vh {
  width: 56px;
  height: 72px;
  object-fit: cover;
  border-radius: 6px;
  flex-shrink: 0;
  background: var(--surface-1);
}
.vh.no {
  display: grid;
  place-items: center;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.35);
}
.toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12.5px;
  color: var(--text-dim);
  cursor: pointer;
}
</style>
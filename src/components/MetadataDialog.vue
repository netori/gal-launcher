<script setup lang="ts">
import { ref } from "vue";
import { api, type BgmSearchHit, type Game, type VnSearchHit } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const props = defineProps<{ game: Game | null }>();
const emit = defineEmits<{
  close: [];
  applied: [g: Game];
}>();
useCloseOnEscape(() => emit("close"));

const q = ref("");
const source = ref<"vndb" | "bgm">("vndb");
const vndbHits = ref<VnSearchHit[]>([]);
const bgmHits = ref<BgmSearchHit[]>([]);
const searching = ref(false);
const applying = ref("");
const err = ref("");
const useVndbTitle = ref(false);
const useBgmTitle = ref(false);

async function search() {
  if (!q.value.trim()) return;
  searching.value = true;
  err.value = "";
  vndbHits.value = [];
  bgmHits.value = [];
  try {
    if (source.value === "vndb") {
      vndbHits.value = await api.searchVndb(q.value.trim());
    } else {
      bgmHits.value = await api.searchBgm(q.value.trim());
    }
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

async function applyBgmHit(h: BgmSearchHit) {
  if (!props.game) return;
  applying.value = "bgm:" + h.bgmId;
  err.value = "";
  try {
    const updated = await api.applyBgmMetadata(props.game.id, h.bgmId, useBgmTitle.value);
    emit("applied", updated);
    emit("close");
  } catch (e) {
    err.value = String(e);
  } finally {
    applying.value = "";
  }
}

function vndbRating(h: VnSearchHit): string {
  return h.rating != null ? (h.rating / 10).toFixed(2) : "—";
}

function bgmRating(h: BgmSearchHit): string {
  return h.rating != null ? h.rating.toFixed(2) : "—";
}
</script>

<template>
  <div v-if="props.game" class="overlay" @click.self="emit('close')">
    <div class="modal" style="width: min(680px, 94vw)">
      <div class="head">
        <h2>从多源补全元数据</h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <p class="hint">
          「{{ props.game.title }}」 → 选择 VNDB 或 Bangumi 搜索，应用后即可拉取封面 / 简介 / 评分 / 标签等。
        </p>

        <div class="source-tabs">
          <button class="source-tab" :class="{ on: source === 'vndb' }" @click="source = 'vndb'">VNDB</button>
          <button class="source-tab" :class="{ on: source === 'bgm' }" @click="source = 'bgm'">Bangumi</button>
        </div>

        <div class="row" style="gap: 8px">
          <input
            v-model="q"
            type="text"
            placeholder="输入要搜索的标题（日文原名 / 罗马音 / 中文译名均可）"
            @keyup.enter="search"
            @focus="q = q || (props.game?.title ?? '')"
          />
          <button class="btn primary" :disabled="searching || !q.trim()" @click="search">
            {{ searching ? "搜索中…" : "搜索" }}
          </button>
        </div>

        <label v-if="source === 'vndb'" class="toggle" style="margin-top: 8px">
          <input type="checkbox" v-model="useVndbTitle" />
          同时把游戏标题改为 VNDB 主标题
        </label>
        <label v-else class="toggle" style="margin-top: 8px">
          <input type="checkbox" v-model="useBgmTitle" />
          同时把游戏标题改为 Bangumi 中文名（无中文则用原名）
        </label>

        <div v-if="err" class="toast err" style="margin-top: 6px">{{ err }}</div>

        <!-- VNDB results -->
        <div v-if="source === 'vndb' && vndbHits.length" class="vn-list">
          <div v-for="h in vndbHits" :key="'vndb-' + h.vndbId" class="vn">
            <img v-if="h.imageUrl" class="vh" :src="h.imageUrl" alt="" loading="lazy" />
            <div v-else class="vh no">{{ h.title.charAt(0) }}</div>
            <div style="flex: 1; min-width: 0">
              <div class="nm">{{ h.title }}</div>
              <div class="sub">vndb:{{ h.vndbId }} · ★ {{ vndbRating(h) }}（{{ h.votecount }} 票）</div>
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

        <!-- Bangumi results -->
        <div v-if="source === 'bgm' && bgmHits.length" class="vn-list">
          <div v-for="h in bgmHits" :key="'bgm-' + h.bgmId" class="vn">
            <img v-if="h.imageUrl" class="vh" :src="h.imageUrl" alt="" loading="lazy" />
            <div v-else class="vh no">{{ (h.titleCn || h.title).charAt(0) }}</div>
            <div style="flex: 1; min-width: 0">
              <div class="nm">{{ h.titleCn || h.title }}</div>
              <div class="sub">
                bgm:{{ h.bgmId }} · ★ {{ bgmRating(h) }}
                <template v-if="h.rank != null"> · #{{ h.rank }}</template>
                <template v-if="h.nsfw"> · NSFW</template>
              </div>
            </div>
            <button
              class="btn primary small"
              :disabled="applying.length > 0"
              @click="applyBgmHit(h)"
            >
              {{ applying === 'bgm:' + h.bgmId ? "应用中…" : "应用" }}
            </button>
          </div>
        </div>

        <div
          v-else-if="!searching && q"
          class="muted"
          style="padding: 12px 2px"
        >
          {{ source === 'vndb' ? 'VNDB 没有结果' : 'Bangumi 没有结果' }}（试试日文原名 / 罗马音）。
        </div>
      </div>

      <div class="foot">
        <button class="btn ghost" @click="emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.source-tabs {
  display: flex;
  gap: 6px;
  margin-bottom: 10px;
}
.source-tab {
  padding: 6px 14px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-dim);
  font-size: 12.5px;
  font-weight: 600;
}
.source-tab.on {
  background: rgba(217, 126, 61, 0.14);
  border-color: var(--accent);
  color: var(--text);
}

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
<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { api, STATUS_META, type Game } from "../api";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const emit = defineEmits<{ close: [] }>();
useCloseOnEscape(() => emit("close"));

const games = ref<Game[]>([]);
const loading = ref(true);
const err = ref("");

const shownGames = computed(() => games.value.filter((g) => !g.hidden));

function fmtDur(secs: number): string {
  if (secs <= 0) return "0h";
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  if (h >= 1000) return `${(h / 1000).toFixed(1)}kh`;
  return h > 0 ? `${h}h${m}m` : `${m}m`;
}

function fmtDate(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleString("zh-CN", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

const totalGames = computed(() => shownGames.value.length);
const totalSeconds = computed(() =>
  shownGames.value.reduce((s, g) => s + g.totalSeconds, 0)
);
const totalPlayCount = computed(() =>
  shownGames.value.reduce((s, g) => s + g.playCount, 0)
);
const completedCount = computed(
  () => shownGames.value.filter((g) => g.status === "finished").length
);
const avgSeconds = computed(() =>
  totalGames.value ? Math.round(totalSeconds.value / totalGames.value) : 0
);

const topByTime = computed(() =>
  [...shownGames.value]
    .sort((a, b) => b.totalSeconds - a.totalSeconds)
    .filter((g) => g.totalSeconds > 0)
    .slice(0, 10)
);

const maxTopSeconds = computed(() =>
  Math.max(1, ...topByTime.value.map((g) => g.totalSeconds))
);

const recent = computed(() =>
  [...shownGames.value]
    .filter((g) => g.lastPlayed)
    .sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0))
    .slice(0, 8)
);

const statusCounts = computed(() =>
  STATUS_META.filter((s) => s.key).map((s) => ({
    ...s,
    count: shownGames.value.filter((g) => g.status === s.key).length,
  }))
);
const maxStatusCount = computed(() =>
  Math.max(1, ...statusCounts.value.map((s) => s.count))
);

onMounted(async () => {
  try {
    games.value = await api.listGames(true);
  } catch (e) {
    err.value = String(e);
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal stats">
      <div class="head">
        <h2 style="display: flex; align-items: center; gap: 8px">
          <Icon name="play" :size="15" /> 数据统计
        </h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <div v-if="loading" class="center-h" style="padding: 30px"><div class="spinner"></div></div>
        <div v-else-if="err" class="toast err">{{ err }}</div>

        <template v-else>
          <div class="stat-cards">
            <div class="stat-card"><b>{{ totalGames }}</b><span>游戏总数</span></div>
            <div class="stat-card"><b>{{ fmtDur(totalSeconds) }}</b><span>累计时长</span></div>
            <div class="stat-card"><b>{{ totalPlayCount }}</b><span>启动次数</span></div>
            <div class="stat-card"><b>{{ completedCount }}</b><span>已通关</span></div>
            <div class="stat-card"><b>{{ fmtDur(avgSeconds) }}</b><span>平均时长</span></div>
          </div>

          <div class="section-title">时长排行 Top 10</div>
          <div class="rank-list">
            <div v-for="(g, i) in topByTime" :key="g.id" class="rank-row">
              <span class="rank">{{ i + 1 }}</span>
              <div class="rank-info">
                <div class="rank-title">{{ g.title }}</div>
                <div class="bar"><i :style="{ width: (g.totalSeconds / maxTopSeconds) * 100 + '%' }"></i></div>
              </div>
              <span class="rank-time">{{ fmtDur(g.totalSeconds) }}</span>
            </div>
            <div v-if="!topByTime.length" class="muted" style="padding: 8px 2px">还没有游玩记录</div>
          </div>

          <div class="section-title">状态分布</div>
          <div class="status-list">
            <div v-for="s in statusCounts" :key="s.key" class="status-row">
              <span class="dot" :style="{ background: s.color }"></span>
              <span class="status-name">{{ s.label }}</span>
              <div class="bar small"><i :style="{ width: (s.count / maxStatusCount) * 100 + '%', background: s.color }"></i></div>
              <span class="status-count">{{ s.count }}</span>
            </div>
          </div>

          <div class="section-title">最近游玩</div>
          <div class="recent-list">
            <div v-for="g in recent" :key="g.id" class="recent-row">
              <span class="recent-title">{{ g.title }}</span>
              <span class="muted">{{ fmtDate(g.lastPlayed) }}</span>
            </div>
            <div v-if="!recent.length" class="muted" style="padding: 8px 2px">暂无最近游玩</div>
          </div>
        </template>
      </div>

      <div class="foot">
        <button class="btn ghost" @click="emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.stats {
  width: min(760px, 94vw);
}
.stat-cards {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 10px;
  margin-bottom: 18px;
}
.stat-card {
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 14px 12px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}
.stat-card b {
  font-family: var(--font-display);
  font-size: 22px;
  color: var(--accent);
  font-variant-numeric: tabular-nums;
}
.stat-card span {
  font-size: 12px;
  color: var(--text-dim);
}

.rank-list,
.status-list,
.recent-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}
.rank-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.rank {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  display: grid;
  place-items: center;
  background: var(--surface-2);
  color: var(--text-dim);
  font-size: 12px;
  font-weight: 700;
  flex-shrink: 0;
}
.rank-info {
  flex: 1;
  min-width: 0;
}
.rank-title {
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.rank-time {
  font-size: 12px;
  color: var(--text-dim);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}
.bar {
  height: 5px;
  border-radius: 999px;
  background: rgba(255, 250, 244, 0.08);
  overflow: hidden;
  margin-top: 5px;
}
.bar i {
  display: block;
  height: 100%;
  border-radius: 999px;
  background: linear-gradient(90deg, var(--accent), #f0a25e);
}
.bar.small {
  width: 100px;
  flex-shrink: 0;
  margin-top: 0;
}
.status-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12.5px;
}
.status-row .dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.status-name {
  width: 56px;
  color: var(--text-dim);
}
.status-count {
  font-variant-numeric: tabular-nums;
  color: var(--text);
}
.recent-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  background: var(--bg-soft);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.recent-title {
  font-size: 13px;
  font-weight: 600;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>

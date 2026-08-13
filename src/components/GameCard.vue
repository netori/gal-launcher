<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { api, engineNeedsLocale, statusColor, statusLabel, type Game } from "../api";
import Icon from "./Icon.vue";

const props = defineProps<{ game: Game; selectMode?: boolean; selected?: boolean }>();
const emit = defineEmits<{
  click: [game: Game];
  select: [game: Game];
  launch: [game: Game, locale: boolean];
  favorite: [game: Game];
  hide: [game: Game];
  del: [game: Game];
  context: [game: Game, e: MouseEvent];
}>();

// 模块级图片缓存：同一张封面只读盘一次（缩略图，体积小）。
const coverCache = new Map<string, string>();
const src = ref("");
let cancelled = false;
let io: IntersectionObserver | null = null;
const rootEl = ref<HTMLElement | null>(null);

onBeforeUnmount(() => {
  cancelled = true;
  io?.disconnect();
});

onMounted(() => {
  const p = props.game.coverPath;
  if (!p) return;
  if (coverCache.has(p)) {
    src.value = coverCache.get(p)!;
    return;
  }
  if (typeof IntersectionObserver === "undefined") {
    loadCover();
    return;
  }
  io = new IntersectionObserver(
    (entries) => {
      for (const en of entries) {
        if (en.isIntersecting) {
          loadCover();
          io?.disconnect();
          io = null;
          break;
        }
      }
    },
    { rootMargin: "300px" }
  );
  if (rootEl.value) io.observe(rootEl.value);
  else loadCover();
});

async function loadCover() {
  const p = props.game.coverPath;
  if (!p || src.value) return;
  try {
    const uri = await api.readCover(p, 400);
    if (cancelled) return;
    coverCache.set(p, uri);
    src.value = uri;
  } catch {
    /* 失败就显示占位图 */
  }
}

// 详情抽屉里「更换封面」后 coverPath 变化，key 不变的卡片实例不会重新挂载，这里手动重载。
watch(
  () => props.game.coverPath,
  (p) => {
    src.value = "";
    if (!p) return;
    if (coverCache.has(p)) {
      src.value = coverCache.get(p)!;
      return;
    }
    loadCover();
  }
);

function fmt(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.round((secs % 3600) / 60);
  if (h > 0) return `${h}h${m}m`;
  return `${Math.floor(secs / 60)}m`;
}

/** 记录鼠标在卡片内的位置，供 ::before 柔光跟随（对应 style.css 里 .card::before）。 */
function onMove(e: MouseEvent) {
  const el = e.currentTarget as HTMLElement;
  const r = el.getBoundingClientRect();
  el.style.setProperty("--mx", `${((e.clientX - r.left) / r.width) * 100}%`);
  el.style.setProperty("--my", `${((e.clientY - r.top) / r.height) * 100}%`);
}

function onCardClick() {
  if (props.selectMode) emit("select", props.game);
  else emit("click", props.game);
}
</script>

<style scoped>
.placeholder {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(120% 90% at 50% 0%, rgba(217, 126, 61, 0.18), transparent 60%),
    linear-gradient(165deg, rgba(217, 126, 61, 0.3), rgba(150, 92, 48, 0.2) 55%, rgba(96, 62, 34, 0.3)),
    var(--surface-2);
  display: grid;
  place-items: center;
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.05);
}
.ph-letter {
  font-family: var(--font-display);
  font-size: 44px;
  font-weight: 700;
  line-height: 1;
  color: rgba(255, 245, 235, 0.3);
  text-shadow: 0 2px 12px rgba(20, 12, 6, 0.4);
}
.ph-engine {
  position: absolute;
  left: 10px;
  right: 10px;
  bottom: 8px;
  font-size: 10.5px;
  color: rgba(255, 245, 235, 0.4);
  text-align: center;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.star.on {
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.6);
  filter: drop-shadow(0 0 6px rgba(240, 180, 41, 0.75));
}
.topline .star {
  display: inline-flex;
  align-items: center;
  cursor: pointer;
  transition: filter var(--d-base) var(--ease-out);
}
.topline .star svg {
  transition: transform var(--d-press) var(--ease-out);
}
.topline .star:active svg {
  transform: scale(1.25);
}
.del {
  margin-left: 6px;
  background: none;
  border: none;
  cursor: pointer;
  font-size: 15px;
  opacity: 0;
  transition: opacity 0.15s ease;
}
.card:hover .del {
  opacity: 0.9;
}
.del:hover {
  filter: brightness(1.35);
}
.engine-chip {
  display: inline-flex;
  align-items: center;
  max-width: 100%;
  padding: 2px 7px;
  border-radius: 999px;
  background: rgba(22, 16, 11, 0.62);
  backdrop-filter: blur(4px);
  color: rgba(255, 250, 244, 0.82);
  font-size: 10.5px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.status-badge {
  position: absolute;
  top: 8px;
  right: 8px;
  padding: 2px 7px;
  border-radius: 999px;
  font-size: 10px;
  font-weight: 700;
  color: #fff;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.45);
}
.playtime {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 7px;
  border-radius: 999px;
  background: rgba(22, 16, 11, 0.6);
  backdrop-filter: blur(4px);
  color: rgba(255, 250, 244, 0.82);
  font-size: 10.5px;
  font-variant-numeric: tabular-nums;
}
.sel-check {
  position: absolute;
  top: 10px;
  left: 10px;
  z-index: 5;
  width: 22px;
  height: 22px;
  border-radius: 7px;
  border: 2px solid rgba(255, 250, 244, 0.7);
  background: rgba(22, 16, 11, 0.55);
  display: grid;
  place-items: center;
  color: var(--accent-ink);
  transition: background-color var(--d-fast) var(--ease-out), border-color var(--d-fast) var(--ease-out);
}
.sel-check.on {
  background: var(--accent);
  border-color: var(--accent);
}
.card.is-selected {
  border-color: var(--accent);
  box-shadow:
    inset 0 1px 0 rgba(255, 250, 244, 0.06),
    0 0 0 2px rgba(217, 126, 61, 0.45),
    0 10px 24px rgba(20, 12, 6, 0.38);
}
</style>

<template>
  <article
    ref="rootEl"
    class="card"
    :class="{ 'fav-on': game.favorite, 'is-selected': selected }"
    tabindex="0"
    @click="onCardClick"
    @keydown.enter="onCardClick"
    @contextmenu.prevent="emit('context', game, $event)"
    @mousemove="onMove"
  >
    <span v-if="selectMode" class="sel-check" :class="{ on: selected }">
      <Icon v-if="selected" name="check" :size="14" />
    </span>
    <img v-if="src" class="cover" :src="src" :alt="game.title" />
    <div v-else class="placeholder">
      <span class="ph-letter">{{ game.title.charAt(0) }}</span>
      <span class="ph-engine" v-if="game.engine">{{ game.engine }}</span>
    </div>
    <div class="scrim"></div>

    <div class="hidden-tag" v-if="game.hidden">已隐藏</div>
    <span
      v-if="game.status && !game.hidden"
      class="status-badge"
      :style="{ background: statusColor(game.status) }"
      >{{ statusLabel(game.status) }}</span
    >

    <div class="topline">
      <span class="rating-badge" v-if="game.rating != null">★ {{ game.rating.toFixed(1) }}</span>
      <span
        class="star"
        :class="{ on: game.favorite }"
        :title="game.favorite ? '取消收藏' : '收藏'"
        @click.stop="emit('favorite', game)"
      >
        <Icon name="star" :size="15" :filled="game.favorite" />
      </span>
      <button class="del" title="删除（送入回收站）" @click.stop="emit('del', game)">
        <Icon name="trash" :size="14" />
      </button>
    </div>

    <div class="toolrow">
      <button class="act play" title="启动" @click.stop="emit('launch', game, false)">
        <Icon name="play" :size="16" />
      </button>
      <button
        v-if="engineNeedsLocale(game.engine)"
        class="act"
        title="Locale Emulator 转区启动"
        @click.stop="emit('launch', game, true)"
      >
        <Icon name="globe" :size="16" />
      </button>
      <button
        class="act"
        :title="game.hidden ? '恢复显示' : '隐藏（应用内）'"
        @click.stop="emit('hide', game)"
      >
        <Icon :name="game.hidden ? 'eye' : 'eye-off'" :size="16" />
      </button>
    </div>

    <div class="info">
      <h3>{{ game.title }}</h3>
      <div class="row" style="gap: 6px; margin-top: 5px; justify-content: space-between">
        <span class="engine-chip" v-if="game.engine">{{ game.engine }}</span>
        <span class="playtime" v-if="game.totalSeconds > 0">
          <Icon name="play" :size="10" /> {{ fmt(game.totalSeconds) }}
        </span>
      </div>
      <div class="meta">
        <template v-if="game.lastPlayed">
          上次 {{ new Date(game.lastPlayed * 1000).toLocaleDateString("zh-CN") }}
        </template>
        <template v-else>未游玩</template>
        <template v-if="game.playCount > 0"> · {{ game.playCount }} 次</template>
      </div>
    </div>
  </article>
</template>

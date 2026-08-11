<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from "vue";
import { api, engineNeedsLocale, type Game } from "../api";
import Icon from "./Icon.vue";

const props = defineProps<{ game: Game }>();
const emit = defineEmits<{
  click: [game: Game];
  launch: [game: Game, locale: boolean];
  favorite: [game: Game];
  hide: [game: Game];
  del: [game: Game];
  context: [game: Game, e: MouseEvent];
}>();

// 模块级图片缓存：同一张封面只读盘一次。
const coverCache = new Map<string, string>();
const src = ref("");
let cancelled = false;

onBeforeUnmount(() => (cancelled = true));

onMounted(loadCover);

async function loadCover() {
  const p = props.game.coverPath;
  if (!p) return;
  if (coverCache.has(p)) {
    src.value = coverCache.get(p)!;
    return;
  }
  src.value = "";
  try {
    const uri = await api.readImage(p);
    if (cancelled) return;
    coverCache.set(p, uri);
    src.value = uri;
  } catch {
    /* 失败就显示占位图 */
  }
}

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
</style>

<template>
  <article
    class="card"
    :class="{ 'fav-on': game.favorite }"
    tabindex="0"
    @click="emit('click', game)"
    @keydown.enter="emit('click', game)"
    @contextmenu.prevent="emit('context', game, $event)"
    @mousemove="onMove"
  >
    <img v-if="game.coverPath && src" class="cover" :src="src" :alt="game.title" loading="lazy" />
    <div v-else class="placeholder">
      <span class="ph-letter">{{ game.title.charAt(0) }}</span>
      <span class="ph-engine" v-if="game.engine">{{ game.engine }}</span>
    </div>
    <div class="scrim"></div>

    <div class="hidden-tag" v-if="game.hidden">已隐藏</div>

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
      <div class="engine" v-if="game.engine">{{ game.engine }}</div>
      <div class="meta">
        <template v-if="game.lastPlayed">
          上次 {{ new Date(game.lastPlayed * 1000).toLocaleDateString("zh-CN") }}
        </template>
        <template v-else>未游玩</template>
        <template v-if="game.totalSeconds > 0"> · {{ fmt(game.totalSeconds) }} · {{ game.playCount }} 次</template>
      </div>
    </div>
  </article>
</template>
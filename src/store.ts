import { computed, reactive, ref } from "vue";
import { api, type Game } from "./api";

export type SortKey = "recent" | "title" | "rating" | "favorite";
export type ViewKey = "all" | "favorites" | "hidden";

const raw = ref<Game[]>([]);

const state = reactive({
  loading: false,
  search: "",
  sort: "recent" as SortKey,
  view: "all" as ViewKey,
  error: null as string | null,
});

/** 对某条记录打分排序用：最近玩 / 标题 / 评分 / 收藏。 */
function applySort(arr: Game[]): Game[] {
  const out = [...arr];
  switch (state.sort) {
    case "title":
      return out.sort((a, b) => a.title.localeCompare(b.title, "zh-Hans"));
    case "rating":
      return out.sort((a, b) => (b.rating ?? -1) - (a.rating ?? -1));
    case "favorite":
      return out.sort(
        (a, b) => Number(b.favorite) - Number(a.favorite) || b.totalSeconds - a.totalSeconds
      );
    case "recent":
    default:
      return out.sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0) || b.addedAt - a.addedAt);
  }
}

function applySearch(arr: Game[]): Game[] {
  const q = state.search.trim().toLowerCase();
  if (!q) return arr;
  return arr.filter(
    (g) => g.title.toLowerCase().includes(q) || g.engine.toLowerCase().includes(q)
  );
}

function applyView(arr: Game[]): Game[] {
  switch (state.view) {
    case "favorites":
      return arr.filter((g) => g.favorite && !g.hidden);
    case "hidden":
      return arr.filter((g) => g.hidden);
    default:
      return arr;
  }
}

/** 当前要展示在封面墙上的列表（已应用搜索 / 视图 / 排序）。 */
export const visible = computed(() => applySort(applyView(applySearch(raw.value))));

const rawCount = computed(
  () => raw.value.length
);
const favoriteCount = computed(() => raw.value.filter((g) => g.favorite && !g.hidden).length);
const hiddenCount = computed(() => raw.value.filter((g) => g.hidden).length);

async function refresh() {
  state.loading = true;
  state.error = null;
  try {
    raw.value = await api.listGames(true);
  } catch (e) {
    state.error = String(e);
  } finally {
    state.loading = false;
  }
}

function upsertGame(updated: Game) {
  const i = raw.value.findIndex((g) => g.id === updated.id);
  if (i >= 0) raw.value.splice(i, 1, updated);
  else raw.value.push(updated);
  raw.value.sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0));
}

export function useLibrary() {
  return {
    state,
    visible,
    counts: { total: rawCount, favorites: favoriteCount, hidden: hiddenCount },
    refresh,
    upsertGame,
    setView: (v: ViewKey) => (state.view = v),
    setSort: (s: SortKey) => (state.sort = s),
    setSearch: (q: string) => (state.search = q),
  };
}
import { reactive, watch } from "vue";

export interface UiSettings {
  /** 过滤 NSFW / R18 游戏 */
  nsfwFilter: boolean;
}

const STORAGE_KEY = "gal-launcher.ui-settings";

const defaults: UiSettings = {
  nsfwFilter: false,
};

function load(): UiSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...defaults };
    const data = JSON.parse(raw) as Partial<UiSettings>;
    return {
      nsfwFilter: typeof data.nsfwFilter === "boolean" ? data.nsfwFilter : defaults.nsfwFilter,
    };
  } catch {
    return { ...defaults };
  }
}

export const uiSettings = reactive<UiSettings>(load());

watch(
  uiSettings,
  (v) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(v));
    } catch {
      /* ignore */
    }
  },
  { deep: true }
);

export function useUiSettings() {
  return {
    settings: uiSettings,
    defaults,
  };
}

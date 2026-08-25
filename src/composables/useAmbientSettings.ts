import { reactive, watch } from "vue";

export interface AmbientSettings {
  /** 是否启用背景插画轮换（关闭后显示纯暗色底） */
  enabled: boolean;
  /** 轮换间隔（秒） */
  interval: number;
  /** 额外压暗层强度（百分比，0 ~ 60） */
  dim: number;
}

const STORAGE_KEY = "gal-launcher.ambient-settings";

const defaults: AmbientSettings = {
  enabled: true,
  interval: 9,
  dim: 24,
};

function load(): AmbientSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return { ...defaults };
    const data = JSON.parse(raw) as Partial<AmbientSettings>;
    return {
      enabled: typeof data.enabled === "boolean" ? data.enabled : defaults.enabled,
      interval: typeof data.interval === "number" ? data.interval : defaults.interval,
      dim:
        typeof data.dim === "number"
          ? data.dim <= 1
            ? Math.round(data.dim * 100)
            : data.dim
          : defaults.dim,
    };
  } catch {
    return { ...defaults };
  }
}

export const ambientSettings = reactive<AmbientSettings>(load());

watch(
  ambientSettings,
  (v) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(v));
    } catch {
      /* localStorage 不可用或已满时静默忽略 */
    }
  },
  { deep: true }
);

export function useAmbientSettings() {
  return {
    settings: ambientSettings,
    defaults,
  };
}

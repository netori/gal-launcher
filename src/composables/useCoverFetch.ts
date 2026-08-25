import { ref } from "vue";
import { listen } from "@tauri-apps/api/event";
import { api } from "../api";

/** 批量补封面进度（后端每隔一个游戏发一次 covers-progress 事件）。 */
export interface CoverProgress {
  processed: number;
  total: number;
  current?: string;
  done?: boolean;
}

export interface CoverFetchResult {
  updated: number;
  failed: string[];
  cancelled: boolean;
}

/**
 * 共享的批量补封面任务状态：App.vue 的进度横幅与设置页按钮共用同一实例，
 * 任何入口启动后全局都能看到进度，避免两处各自发起任务互相打架。
 */
function createCoverFetch() {
  const running = ref(false);
  const progress = ref<CoverProgress>({ processed: 0, total: 0 });
  let unlisten: (() => void) | null = null;

  async function start(): Promise<CoverFetchResult> {
    if (running.value) throw new Error("已有补全任务正在进行");
    running.value = true;
    progress.value = { processed: 0, total: 0 };
    try {
      if (!unlisten) {
        unlisten = await listen<CoverProgress>("covers-progress", (e) => {
          progress.value = e.payload;
        });
      }
      return await api.fetchMissingCovers();
    } finally {
      unlisten?.();
      unlisten = null;
      running.value = false;
    }
  }

  function cancel() {
    api.cancelFetchCovers().catch(() => {});
  }

  return { running, progress, start, cancel };
}

/** 全应用单例（App.vue 与 SettingsDialog 共用）。 */
export const coverFetch = createCoverFetch();
import { readonly, ref } from "vue";
import { fetchLatestRelease, type LoadedRelease } from "../lib/github";

export type LoadState = "loading" | "ok" | "failed";

/** 模块级单例：Download 与 Changelog 共享一次请求，`started` 防 HMR/重挂载重复拉取。 */
const state = ref<LoadState>("loading");
const release = ref<LoadedRelease | null>(null);
let started = false;

export function useGithubRelease() {
  if (!started) {
    started = true;
    fetchLatestRelease()
      .then((r) => {
        release.value = r;
        state.value = "ok";
      })
      .catch(() => {
        state.value = "failed";
      });
  }
  return { state: readonly(state), release: readonly(release) };
}

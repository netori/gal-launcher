import { onBeforeUnmount, onMounted } from "vue";

/**
 * 组件挂载期间按 Esc 触发关闭（所有模态框 / 抽屉 / 右键菜单通用）。
 * `active` 为 false 时不拦截（也不阻止冒泡），让外层/后注册的处理器接管——
 * 例如 App 层的全局 Esc 只在右键菜单/确认框打开时才生效，平时把 Esc 让给对话框。
 * 用 capture + stopImmediatePropagation：子对话框先注册，能挡住父抽屉的 Esc，
 * 避免「按一次 Esc 连关两层」。
 */
export function useCloseOnEscape(
  close: () => void,
  active: () => boolean = () => true
) {
  function onKey(e: KeyboardEvent) {
    if (e.key !== "Escape") return;
    if (!active()) return;
    e.stopImmediatePropagation();
    close();
  }
  onMounted(() => window.addEventListener("keydown", onKey, true));
  onBeforeUnmount(() => window.removeEventListener("keydown", onKey, true));
}

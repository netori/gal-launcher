import type { Directive, DirectiveBinding } from "vue";

interface RevealElement extends HTMLElement {
  __revealObserver?: IntersectionObserver | null;
}

function prefersReducedMotion(): boolean {
  return (
    typeof window !== "undefined" &&
    window.matchMedia("(prefers-reduced-motion: reduce)").matches
  );
}

/**
 * v-reveal：进入视口时淡入上移。
 * 用法：`v-reveal` 或 `v-reveal="index"`（index 触发 70ms 递增 stagger）。
 * 尊重 prefers-reduced-motion：命中时完全不隐藏。
 */
export const reveal: Directive<RevealElement> = {
  mounted(el: RevealElement, binding: DirectiveBinding<number | undefined>) {
    if (prefersReducedMotion()) return;
    el.classList.add("reveal");
    if (typeof binding.value === "number") {
      el.style.setProperty("--reveal-delay", `${binding.value * 70}ms`);
    }
    const observer = new IntersectionObserver(
      (entries) => {
        for (const entry of entries) {
          if (entry.isIntersecting) {
            el.classList.add("reveal-visible");
            observer.disconnect();
          }
        }
      },
      { threshold: 0.12, rootMargin: "0px 0px -8% 0px" },
    );
    el.__revealObserver = observer;
    observer.observe(el);
  },
  unmounted(el: RevealElement) {
    el.__revealObserver?.disconnect();
  },
};

export default reveal;

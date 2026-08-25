<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useAmbientSettings } from "../composables/useAmbientSettings";

/**
 * 全屏环境背景，分桌面与移动两套：
 * - 桌面：本地生成的暖琥珀夜景 + 透明 SVG 流动层（星星/云/花瓣）
 * - 移动/竖屏：用户已生成的 1080×1920 竖版 galgame 插画，4 张轮换 + 轻微 Ken Burns，
 *   并叠一层便于阅读的暖色渐变，适合 Android 手机启动器。
 * prefers-reduced-motion 下会关闭所有持续动画，只保留静态背景图。
 */
const mobileBgs = [
  new URL("../assets/backgrounds/mobile/mobile-1.webp", import.meta.url).href,
  new URL("../assets/backgrounds/mobile/mobile-2.webp", import.meta.url).href,
  new URL("../assets/backgrounds/mobile/mobile-3.webp", import.meta.url).href,
  new URL("../assets/backgrounds/mobile/mobile-4.webp", import.meta.url).href,
];

const desktopBgs = [
  new URL("../assets/backgrounds/desktop/desktop-1.webp", import.meta.url).href,
  new URL("../assets/backgrounds/desktop/desktop-2.webp", import.meta.url).href,
  new URL("../assets/backgrounds/desktop/desktop-3.webp", import.meta.url).href,
  new URL("../assets/backgrounds/desktop/desktop-4.webp", import.meta.url).href,
];

const petals = [
  { id: 1, x: "96px", y: "-40px", rot: "18deg", dx: "30px", dur: "16s", delay: "-2s" },
  { id: 2, x: "272px", y: "-110px", rot: "-22deg", dx: "-18px", dur: "19s", delay: "-8s" },
  { id: 3, x: "438px", y: "-60px", rot: "40deg", dx: "42px", dur: "17s", delay: "-12s" },
  { id: 4, x: "614px", y: "-140px", rot: "-36deg", dx: "-24px", dur: "23s", delay: "-4s" },
  { id: 5, x: "790px", y: "-80px", rot: "72deg", dx: "36px", dur: "18s", delay: "-15s" },
  { id: 6, x: "962px", y: "-30px", rot: "-12deg", dx: "-30px", dur: "21s", delay: "-9s" },
  { id: 7, x: "1134px", y: "-100px", rot: "56deg", dx: "48px", dur: "19s", delay: "-6s" },
  { id: 8, x: "1306px", y: "-50px", rot: "-44deg", dx: "-16px", dur: "22s", delay: "-14s" },
  { id: 9, x: "1480px", y: "-120px", rot: "32deg", dx: "24px", dur: "17s", delay: "-18s" },
  { id: 10, x: "360px", y: "-180px", rot: "-58deg", dx: "22px", dur: "24s", delay: "-11s" },
  { id: 11, x: "904px", y: "-200px", rot: "64deg", dx: "-26px", dur: "19s", delay: "-1s" },
  { id: 12, x: "1224px", y: "-170px", rot: "-18deg", dx: "34px", dur: "21s", delay: "-16s" },
];

const { settings: ambientSettings } = useAmbientSettings();

const isMobile = ref(false);
const slide = ref(0);
let timer: ReturnType<typeof setInterval> | undefined;

function startTimer() {
  if (timer) clearInterval(timer);
  timer = setInterval(() => {
    slide.value = (slide.value + 1) % mobileBgs.length;
  }, ambientSettings.interval * 1000);
}

function updateMobile() {
  isMobile.value =
    window.innerWidth < 760 || /android/i.test(navigator.userAgent);
}

onMounted(() => {
  updateMobile();
  window.addEventListener("resize", updateMobile);
  startTimer();
});

watch(
  () => ambientSettings.interval,
  () => startTimer()
);

onBeforeUnmount(() => {
  window.removeEventListener("resize", updateMobile);
  if (timer) clearInterval(timer);
});
</script>

<template>
  <div
    v-if="ambientSettings.enabled"
    class="ambient"
    :style="{ '--ambient-dim': ambientSettings.dim / 100 }"
    aria-hidden="true"
  >
    <!-- 移动 / 竖屏：四张竖版插画轮换 -->
    <template v-if="isMobile">
      <div class="mobile-slides">
        <img
          v-for="(bg, i) in mobileBgs"
          :key="bg"
          :src="bg"
          class="mobile-slide"
          :class="{ active: i === slide }"
          alt=""
          draggable="false"
        />
      </div>
      <div class="mobile-scrim"></div>
    </template>

    <!-- 桌面：用户横版插画 + 流动 SVG -->
    <template v-else>
      <div class="desktop-stage">
        <div class="desktop-slides">
          <img
            v-for="(bg, i) in desktopBgs"
            :key="bg"
            :src="bg"
            class="desktop-slide"
            :class="{ active: i === slide }"
            alt=""
            draggable="false"
          />
        </div>
        <div class="desktop-scrim"></div>
        <svg
          class="ambient-overlay"
          viewBox="0 0 1600 900"
          preserveAspectRatio="xMidYMid slice"
          role="presentation"
        >
          <g class="amb-stars" fill="#f7e6cf">
            <circle cx="120" cy="118" r="1.6" />
            <circle cx="246" cy="72" r="1.2" />
            <circle cx="388" cy="190" r="1.1" />
            <circle cx="506" cy="58" r="1.8" />
            <circle cx="638" cy="142" r="1.2" />
            <circle cx="766" cy="72" r="1.4" />
            <circle cx="830" cy="220" r="1.1" />
            <circle cx="982" cy="72" r="1.7" />
            <circle cx="1088" cy="168" r="1.2" />
            <circle cx="1268" cy="86" r="1.5" />
            <circle cx="1410" cy="146" r="1.1" />
            <circle cx="1518" cy="70" r="1.8" />
            <circle cx="94" cy="266" r="1.1" />
            <circle cx="352" cy="320" r="1" />
            <circle cx="556" cy="250" r="1.2" />
            <circle cx="940" cy="270" r="1.1" />
            <circle cx="1240" cy="250" r="1.2" />
            <circle cx="1452" cy="286" r="1" />
          </g>

          <g class="amb-clouds" fill="#e8c3a0">
            <ellipse class="amb-cloud c1" cx="250" cy="246" rx="240" ry="26" opacity="0.08" />
            <ellipse class="amb-cloud c2" cx="720" cy="176" rx="320" ry="32" opacity="0.07" />
            <ellipse class="amb-cloud c3" cx="1340" cy="312" rx="280" ry="28" opacity="0.06" />
            <ellipse class="amb-cloud c4" cx="80" cy="450" rx="190" ry="22" opacity="0.05" />
          </g>

          <g class="amb-petals" fill="#e8a06a">
            <path
              v-for="p in petals"
              :key="p.id"
              :d="'M0 0 C5 -6 12 -5 14 0 C12 6 5 6 0 0 Z'"
              class="amb-petal"
              :style="{
                '--x': p.x,
                '--y': p.y,
                '--rot': p.rot,
                '--dx': p.dx,
                '--dur': p.dur,
                '--delay': p.delay,
              }"
            />
          </g>
        </svg>
      </div>
    </template>

    <div class="ambient-dim"></div>
  </div>
</template>

<style scoped>
.ambient {
  position: fixed;
  inset: 0;
  z-index: 0;
  overflow: hidden;
  pointer-events: none;
  user-select: none;
  -webkit-user-select: none;
}

.ambient-dim {
  position: absolute;
  inset: 0;
  background: #16120f;
  opacity: var(--ambient-dim, 0);
  pointer-events: none;
}

/* ---------- 桌面 ---------- */
.desktop-stage {
  position: absolute;
  inset: 0;
  transform-origin: 52% 48%;
  animation: ambient-pan 74s var(--ease-in-out) infinite alternate;
  will-change: transform;
}

.desktop-slides {
  position: absolute;
  inset: 0;
}

.desktop-slide {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transition: opacity 2.8s var(--ease-out);
}

.desktop-slide.active {
  opacity: 1;
}

.desktop-scrim {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(115% 70% at 50% 0%, rgba(22, 18, 15, 0.22), transparent 56%),
    linear-gradient(
      180deg,
      rgba(22, 18, 15, 0.3) 0%,
      rgba(22, 18, 15, 0.08) 32%,
      rgba(22, 18, 15, 0.3) 68%,
      rgba(22, 18, 15, 0.66) 100%
    );
}

.ambient-overlay {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  display: block;
  opacity: 0.88;
}

.amb-stars {
  animation: stars-twinkle 7s ease-in-out infinite alternate;
}

.amb-cloud {
  animation: clouds-drift 52s linear infinite;
}
.amb-cloud.c1 {
  animation-delay: -4s;
}
.amb-cloud.c2 {
  animation-delay: -19s;
}
.amb-cloud.c3 {
  animation-delay: -31s;
}
.amb-cloud.c4 {
  animation-delay: -47s;
}

.amb-petals {
  opacity: 0.9;
}
.amb-petal {
  transform: translate(var(--x), var(--y)) rotate(var(--rot));
  opacity: 0;
  animation: petal-fall var(--dur) linear var(--delay) infinite;
  transform-box: fill-box;
  transform-origin: center;
}

/* ---------- 移动端竖版 ---------- */
.mobile-slides {
  position: absolute;
  inset: 0;
}
.mobile-slide {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  opacity: 0;
  transform: scale(1.06);
  transition:
    opacity 2.4s var(--ease-out),
    transform 8s var(--ease-out);
  will-change: opacity, transform;
}
.mobile-slide.active {
  opacity: 1;
  transform: scale(1.0);
}
.mobile-scrim {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(110% 68% at 50% 6%, rgba(22, 18, 15, 0.2), transparent 52%),
    linear-gradient(
      180deg,
      rgba(22, 18, 15, 0.38) 0%,
      rgba(22, 18, 15, 0.12) 34%,
      rgba(22, 18, 15, 0.32) 68%,
      rgba(22, 18, 15, 0.72) 100%
    );
}

/* ---------- 动画 ---------- */
@keyframes ambient-pan {
  from {
    transform: scale(1.02) translate3d(0, 0, 0);
  }
  to {
    transform: scale(1.07) translate3d(-1.2%, -1.5%, 0);
  }
}

@keyframes stars-twinkle {
  from {
    opacity: 0.55;
  }
  to {
    opacity: 1;
  }
}

@keyframes clouds-drift {
  from {
    transform: translateX(-90px);
  }
  to {
    transform: translateX(90px);
  }
}

@keyframes petal-fall {
  0% {
    transform: translate(var(--x), var(--y)) rotate(var(--rot));
    opacity: 0;
  }
  8% {
    opacity: 0.72;
  }
  82% {
    opacity: 0.42;
  }
  100% {
    transform: translate(calc(var(--x) + var(--dx)), calc(var(--y) + 1120px)) rotate(calc(var(--rot) + 260deg));
    opacity: 0;
  }
}

@media (prefers-reduced-motion: reduce) {
  .desktop-stage,
  .amb-stars,
  .amb-cloud,
  .amb-petal,
  .mobile-slide,
  .desktop-slide {
    animation: none;
  }
  .desktop-stage {
    transform: none;
  }
  .amb-petal {
    opacity: 0.28;
    transform: translate(var(--x), var(--y)) rotate(var(--rot));
  }
  .mobile-slide,
  .desktop-slide {
    transition: opacity 1.2s ease;
    transform: none;
  }
}
</style>

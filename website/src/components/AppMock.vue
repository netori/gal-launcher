<script setup lang="ts">
/**
 * 预览区：HTML 假窗口，贴近产品真实 UI（暖琥珀暗色主题）。
 * 展示封面墙、游戏详情（启动/转区、时长统计）、资源解包分类。
 * 无真实截图，纯 HTML/CSS 绘制，不留占位符。
 * 移动端用容器查询折叠为单列，封面墙变横向滑动 strip。
 */
const games = [
  { name: "雪の街", init: "雪", tone: "c1" },
  { name: "星屑の夜", init: "星", tone: "c2" },
  { name: "あの日の約束", init: "約", tone: "c3" },
  { name: "夏の灯り", init: "夏", tone: "c4" },
  { name: "ひぐらし", init: "ひ", tone: "c5" },
  { name: "空の欠片", init: "空", tone: "c6" },
];
</script>

<template>
  <section id="preview" class="section">
    <div class="container">
      <div class="section-head" v-reveal>
        <h2>一块封面墙，装下整个本地库</h2>
        <p>搜索、排序、收藏，选中一款就能看详情、转区启动、查游玩时长。</p>
      </div>

      <div class="mock-stage" v-reveal>
        <div class="mock-window" role="img" aria-label="GAL 启动器界面示意：封面墙与游戏详情面板">
          <!-- 窗口 chrome -->
          <div class="mock-titlebar" aria-hidden="true">
            <div class="dots">
              <i></i><i class="on"></i><i></i>
            </div>
            <span class="win-title">GAL 启动器 · 本地库 v0.1.0</span>
            <span class="win-controls"><i>─</i><i>▢</i><i>✕</i></span>
          </div>

          <!-- 工具栏 -->
          <div class="mock-toolbar" aria-hidden="true">
            <div class="brand"><img src="../assets/brand-logo.png" alt="" />GAL 启动器</div>
            <div class="search">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"><path d="M11 17a6 6 0 1 0 0-12 6 6 0 0 0 0 12ZM21 21l-4.35-4.35" /></svg>
              <span>搜索封面、标题、引擎…</span>
            </div>
            <div class="chips"><span>资源站</span><span>设置</span></div>
          </div>

          <!-- 主体：左封面墙 + 右详情 -->
          <div class="mock-main">
            <div class="mock-wall" aria-hidden="true">
              <div v-for="g in games" :key="g.name" class="wall-card">
                <div class="cover" :class="g.tone">
                  <span class="cinit">{{ g.init }}</span>
                  <span class="cname">{{ g.name }}</span>
                </div>
              </div>
            </div>

            <div class="mock-detail" aria-hidden="true">
              <div class="d-head">
                <span class="d-title">雪の街</span>
                <span class="vndb-badge">VNDB</span>
              </div>
              <div class="d-meta">VNDB · 2024 · 吉里吉里 · 汉化</div>

              <div class="d-rating">
                <svg viewBox="0 0 24 24" fill="currentColor" stroke="none" width="15" height="15"><path d="M12 3l2.8 5.9 6.5.65-4.85 4.35 1.3 6.4L12 17.1 6.25 20.3l1.3-6.4L2.7 9.55l6.5-.65Z" /></svg>
                <span>8.4 · 12 人评分</span>
              </div>

              <div class="d-actions">
                <span class="d-btn primary">启动</span>
                <span class="d-btn secondary">转区启动 LE</span>
              </div>

              <div class="d-stat">
                <div class="stat-row">
                  <span>已游玩</span>
                  <strong>14 小时 32 分</strong>
                </div>
                <div class="bar"><i></i></div>
              </div>

              <div class="d-unpack">
                <div class="unpack-head">XP3 · 已解 1,204 项</div>
                <div class="unpack-chips">
                  <span>立绘 312</span><span>表情 96</span><span>语音 620</span><span>背景 34</span><span>界面 42</span>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部状态条 -->
          <div class="mock-status" aria-hidden="true">
            <span>共 128 款游戏 · 3 个目录 · 本地库</span>
            <span class="ok"><i></i>数据库就绪</span>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
/* 舞台：暖琥珀氛围光 + 溢出兜底 */
.mock-stage {
  position: relative;
  overflow: hidden;
  border-radius: 20px;
  background:
    radial-gradient(60% 80% at 50% 0%, rgba(217, 126, 61, 0.1), transparent 70%),
    var(--bg-soft);
  border: 1px solid var(--border-strong);
  box-shadow:
    inset 0 1px 0 rgba(255, 250, 244, 0.06),
    0 30px 60px rgba(12, 8, 5, 0.55);
  padding: clamp(14px, 2.4vw, 22px);
}

/* 假窗口（容器查询主体） */
.mock-window {
  container-type: inline-size;
  border-radius: 14px;
  overflow: hidden;
  border: 1px solid var(--border-strong);
  background: var(--bg);
  box-shadow: 0 18px 40px rgba(8, 5, 3, 0.45);
}

/* 标题栏 */
.mock-titlebar {
  display: flex;
  align-items: center;
  gap: 14px;
  height: 34px;
  padding: 0 14px;
  background: var(--surface);
  border-bottom: 1px solid var(--border);
}
.dots {
  display: flex;
  gap: 6px;
}
.dots i {
  width: 9px;
  height: 9px;
  border-radius: 50%;
  background: #3a322a;
  border: 1px solid rgba(0, 0, 0, 0.3);
}
.dots i.on {
  background: var(--accent);
}
.win-title {
  font-family: var(--font-display);
  font-size: 0.74rem;
  color: var(--text-faint);
  letter-spacing: 0.02em;
}
.win-controls {
  margin-left: auto;
  display: flex;
  gap: 10px;
  color: var(--text-faint);
  font-size: 0.68rem;
}
.win-controls i {
  font-style: normal;
}

/* 工具栏 */
.mock-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  height: 46px;
  padding: 0 14px;
  background: var(--bg-soft);
  border-bottom: 1px solid var(--border);
}
.brand {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-display);
  font-size: 0.82rem;
  font-weight: 600;
  color: var(--text);
  white-space: nowrap;
}
.brand img {
  width: 22px;
  height: 22px;
  border-radius: 6px;
  object-fit: cover;
  border: 1px solid var(--border-strong);
}
.search {
  flex: 1;
  max-width: 340px;
  display: flex;
  align-items: center;
  gap: 8px;
  height: 30px;
  padding: 0 12px;
  border-radius: 9px;
  background: var(--surface);
  border: 1px solid var(--border);
  color: var(--text-faint);
  font-size: 0.78rem;
}
.search svg {
  width: 15px;
  height: 15px;
  flex-shrink: 0;
}
.chips {
  margin-left: auto;
  display: flex;
  gap: 8px;
}
.chips span {
  font-size: 0.74rem;
  color: var(--text-dim);
  padding: 5px 12px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: var(--surface);
  white-space: nowrap;
}

/* 主体 */
.mock-main {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: 18px;
  padding: 18px;
}

/* 封面墙 */
.mock-wall {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 12px;
  align-content: start;
}
.wall-card {
  aspect-ratio: 3 / 4;
}
.cover {
  position: relative;
  height: 100%;
  border-radius: 10px;
  border: 1px solid rgba(0, 0, 0, 0.35);
  box-shadow:
    inset 0 1px 0 rgba(255, 250, 244, 0.09),
    0 8px 18px rgba(8, 5, 3, 0.35);
  overflow: hidden;
}
/* 单一暖色阶封面（无多色渐变） */
.c1 { background: linear-gradient(180deg, #3a2f22, #2b2419); }
.c2 { background: linear-gradient(180deg, #423428, #2e271c); }
.c3 { background: linear-gradient(180deg, #4a3a2a, #332a1d); }
.c4 { background: linear-gradient(180deg, #523d2a, #362a1d); }
.c5 { background: linear-gradient(180deg, #4a3a32, #31281f); }
.c6 { background: linear-gradient(180deg, #57432f, #3a2e20); }
.cinit {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--font-display);
  font-size: clamp(2.4rem, 6cqw, 4rem);
  font-weight: 600;
  color: rgba(255, 235, 210, 0.28);
  padding-top: 8px;
}
.cname {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  padding: 18px 12px 9px;
  font-size: 0.78rem;
  color: var(--text);
  background: linear-gradient(180deg, transparent, rgba(12, 8, 5, 0.82));
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 详情面板 */
.mock-detail {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  border-radius: 12px;
  background: var(--surface);
  border: 1px solid var(--border);
}
.d-head {
  display: flex;
  align-items: center;
  gap: 10px;
}
.d-title {
  font-family: var(--font-display);
  font-size: 1.15rem;
  font-weight: 600;
  color: var(--text);
}
.vndb-badge {
  font-size: 0.64rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  color: var(--accent);
  border: 1px solid rgba(217, 126, 61, 0.4);
  padding: 2px 7px;
  border-radius: 6px;
  background: rgba(217, 126, 61, 0.08);
}
.d-meta {
  font-size: 0.78rem;
  color: var(--text-dim);
}
.d-rating {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.84rem;
  color: var(--star);
}
.d-actions {
  display: flex;
  gap: 10px;
}
.d-btn {
  font-size: 0.82rem;
  font-weight: 600;
  padding: 8px 16px;
  border-radius: 999px;
  white-space: nowrap;
}
.d-btn.primary {
  background: var(--accent);
  color: var(--accent-ink);
  box-shadow: inset 0 1px 0 rgba(255, 235, 210, 0.35);
}
.d-btn.secondary {
  color: var(--text-dim);
  border: 1px solid var(--border-strong);
}
.d-stat {
  padding-top: 4px;
}
.stat-row {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  font-size: 0.78rem;
  color: var(--text-dim);
  margin-bottom: 8px;
}
.stat-row strong {
  color: var(--text);
  font-weight: 600;
}
.bar {
  height: 5px;
  border-radius: 999px;
  background: var(--surface-2);
  overflow: hidden;
}
.bar i {
  display: block;
  height: 100%;
  width: 62%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--accent), #e89a5c);
}
.d-unpack {
  padding-top: 6px;
  border-top: 1px solid var(--border);
}
.unpack-head {
  font-size: 0.78rem;
  color: var(--text-dim);
  margin-bottom: 9px;
}
.unpack-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.unpack-chips span {
  font-size: 0.7rem;
  color: var(--text-dim);
  background: var(--surface-2);
  border: 1px solid var(--border);
  padding: 4px 9px;
  border-radius: 7px;
}

/* 状态条 */
.mock-status {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 30px;
  padding: 0 14px;
  background: var(--surface);
  border-top: 1px solid var(--border);
  font-size: 0.72rem;
  color: var(--text-faint);
}
.mock-status .ok {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-dim);
}
.mock-status .ok i {
  width: 7px;
  height: 7px;
  border-radius: 50%;
  background: var(--ok);
  box-shadow: 0 0 8px rgba(147, 180, 110, 0.7);
}

/* ============ 容器查询：窄容器折叠（375px 移动端） ============ */
@container (max-width: 620px) {
  .mock-main {
    grid-template-columns: 1fr;
    gap: 14px;
  }
  .mock-wall {
    grid-auto-flow: column;
    grid-auto-columns: 104px;
    grid-template-columns: none;
    overflow-x: auto;
    padding-bottom: 4px;
    scrollbar-width: thin;
  }
  .mock-detail {
    gap: 11px;
  }
  .chips {
    display: none;
  }
  .search {
    max-width: none;
  }
}
</style>

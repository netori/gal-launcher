<script setup lang="ts">
import { openUrl } from "@tauri-apps/plugin-opener";
import Icon from "./Icon.vue";
import { useCloseOnEscape } from "../composables/useCloseOnEscape";

const emit = defineEmits<{ close: [] }>();
useCloseOnEscape(() => emit("close"));

interface Site {
  name: string;
  url: string;
  desc: string;
}
interface Group {
  title: string;
  sites: Site[];
}

// 均为当前较稳定、可公开访问的社区/补丁/资源站入口（链接定期核实过）。
// 如需增减，直接改这个数组即可。
const GROUPS: Group[] = [
  {
    title: "汉化 / 补丁",
    sites: [
      {
        name: "2DFan",
        url: "https://fan2d.top",
        desc: "汉化补丁 · 免CD · 全CG存档 · 攻略；2dfan.com 在大陆无法直连，用这个中转域名；域名常换，失效可在 GitHub 域名发布页查最新（github.com/2dfan/domains）",
      },
      {
        name: "鲲 Galgame 补丁",
        url: "https://www.moyu.moe",
        desc: "开源零门槛补丁站，部分内容需开启 NSFW 显示",
      },
      {
        name: "御爱同萌",
        url: "https://www.ai2.moe",
        desc: "Galgame 资源 + AI 翻译 / 汉化补丁论坛",
      },
      {
        name: "翼梦舞城",
        url: "http://www.otomedream.com",
        desc: "乙女向游戏与汉化资源论坛",
      },
    ],
  },
  {
    title: "资源 / 社区",
    sites: [
      {
        name: "鲲 Galgame",
        url: "https://www.kungal.com",
        desc: "开源 Galgame 论坛，附「资源网站」分级指南",
      },
      {
        name: "Nyaa",
        url: "https://nyaa.si",
        desc: "国际种子站，galgame 资源覆盖最广",
      },
      {
        name: "绯月 ScarletMoon",
        url: "https://bbs.kfpromax.com",
        desc: "中文 Galgame 交流核心论坛（KF）",
      },
    ],
  },
  {
    title: "网盘 / 老站",
    sites: [
      {
        name: "忧郁的loli",
        url: "https://www.mmgal.com",
        desc: "老牌汉化硬盘版资源站（已停止更新）",
      },
    ],
  },
  {
    title: "游戏信息",
    sites: [
      {
        name: "VNDB",
        url: "https://vndb.org",
        desc: "国际视觉小说数据库，与应用内 VNDB 补全同源",
      },
    ],
  },
];

function openSite(s: Site) {
  openUrl(s.url).catch(() => {});
}
</script>

<template>
  <div class="overlay" @click.self="emit('close')">
    <div class="modal res">
      <div class="head">
        <h2 style="display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0">
          <Icon name="external-link" :size="15" /> galgame 资源站
        </h2>
        <button class="btn icon-btn ghost" @click="emit('close')"><Icon name="close" :size="15" /></button>
      </div>

      <div class="body">
        <p class="hint">
          收录一些较稳定的社区 / 补丁 / 资源站，点击「打开」用默认浏览器访问。部分站点需要注册或积分。
        </p>

        <div v-for="g in GROUPS" :key="g.title">
          <div class="section-title">{{ g.title }}</div>
          <div class="sites">
            <div v-for="s in g.sites" :key="s.name" class="site">
              <div style="flex: 1; min-width: 0">
                <div class="nm">{{ s.name }}</div>
                <div class="sub muted">{{ s.desc }}</div>
                <div class="url">{{ s.url }}</div>
              </div>
              <button class="btn small" @click="openSite(s)">
                <Icon name="external-link" :size="13" /> 打开
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="foot">
        <span class="muted" style="margin-right: auto">链接为社区公开站点，资源版权归原作者所有</span>
        <button class="btn ghost" @click="emit('close')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.res {
  width: min(600px, 92vw);
}
.sites {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin: 6px 0 16px;
}
.site {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 12px;
  background: var(--surface-2);
  border: 1px solid var(--border);
  border-radius: 10px;
}
.site:hover {
  border-color: var(--border-strong);
}
.site .nm {
  font-size: 13.5px;
  font-weight: 600;
  color: var(--text);
}
.site .sub {
  font-size: 12px;
  margin-top: 2px;
}
.site .url {
  font-size: 11px;
  color: var(--text-faint);
  margin-top: 2px;
  word-break: break-all;
  font-variant-numeric: tabular-nums;
}
</style>
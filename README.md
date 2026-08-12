# GAL 启动器

Windows 桌面的 **galgame 本地图书馆启动器**：扫描本地游戏目录 → 封面墙管理 → 一键 / 转区启动 → 游玩时长统计 → VNDB 元数据补全 → 汉化 / R18 补丁管理 → 内置资源解包与自动分类。

官网：[gal-launcher.pages.dev](https://gal-launcher.pages.dev)（含下载与更新日志）

## 特性

- **扫描导入**：自动识别引擎与启动文件，批量入库
- **封面墙管理**：搜索、排序（最近游玩 / 标题 / 评分 / 收藏）、收藏与隐藏
- **启动**：一键启动 / Locale Emulator 转区启动 / 多启动文件支持
- **游玩统计**：后台自动累计时长、次数、上次游玩时间
- **元数据**：VNDB 封面与资料自动补全，支持批量补全与手动搜索
- **补丁管理**：汉化 / R18 / 修正补丁安装、一键回滚、自动备份
- **内置解包**：XP3 / PFS / ESC-ARC / NSA / PAC；解包后自动按「立绘 / CG / 背景 / 语音 / BGM / 视频 / 脚本 / 文档 / 数据」分类整理到子文件夹；内置不支持的格式可桥接外部工具（GARbro 等）
- **整库备份 / 恢复**：数据库 + 封面 + 补丁备份一键完成
- **资源站导航**：内置常用社区 / 补丁站快捷入口
- **更新提示**：非打扰式检测新版本，可「不再提示」

## 下载

Windows x64 安装版 / MSI：见 [GitHub Releases](https://github.com/netori/gal-launcher/releases) 或 [官网](https://gal-launcher.pages.dev)。

## 技术栈

Tauri 2 · Rust · Vue 3 + TypeScript · SQLite（rusqlite，WAL 模式）

## 开发

要求：Rust 1.9x + Node 18+

```bash
npm install
npm run tauri dev      # 开发（请用这个命令，debug exe 依赖 dev server）
npm run tauri build    # 打包（产物在 src-tauri/target/release/）
```

检查：

```bash
npx vue-tsc --noEmit    # 前端类型
cargo test --lib        # 后端测试
```

官网子工程在 [`website/`](website/)（独立的 Vite + Vue3 + TS，部署到 Cloudflare Pages，含 GitHub release 同源代理下载）。

## 反馈

Bug 与建议 → [GitHub Issues](https://github.com/netori/gal-launcher/issues)

## 声明

应用本体开源免费。内置的资源站导航仅为社区公开站点索引，资源版权归原作者所有；解包功能仅用于本人合法自持的游戏。

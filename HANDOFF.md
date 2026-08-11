# GAL 启动器 — 会话交接文档

> 生成时间：2026-08-11 ｜ 项目路径：`E:\开发项目\GAL启动器`
> 用途：给下一个 Claude 会话快速接上下文。读完本文件 + `src-tauri/src/*.rs` 与前端关键文件即可动手。

## 1. 项目是什么

Windows 桌面 **galgame 本地图书馆启动器**（Tauri 2 + Vue3/TS + SQLite）。
扫描本地 galgame 目录 → 识别引擎/启动文件 → 封面墙管理 → 一键/转区启动 → 时长统计 →
VNDB 元数据补全 → 汉化/R18 补丁管理 → **内置解包**（XP3/PFS/ESC/NSA/PAC 已实机验证）。
本会话完成了**产品 UI 重设计**（去紫粉渐变→暖琥珀）与 **logo 全面落地**（顶栏/空状态/设置页/favicon 已用真实手柄 logo），并做了一轮**增加 UI 复杂度**的质感提升（纹理/深度/排版，未引入土味装饰）。

## 2. 技术栈 / 运行

- **Rust 1.95 + Tauri 2**，后端在 `src-tauri/`；前端 Vue3 + Vite，根目录 `src/`
- SQLite：`rusqlite` bundled，WAL 模式
- 数据目录：`C:\Users\20905\AppData\Roaming\com.gal.launcher\gal_launcher.db`（含 covers/、assets/、backups/）
- 运行：
  - 开发：`npm run tauri dev`（**必须用这个**；debug exe 没有 dev server 会连 localhost 失败）
  - 打包：`npm run tauri build`（release；产物在 `src-tauri/target/release/`）
- 前端类型检查：`npx vue-tsc --noEmit`；后端：`cargo check` / `cargo test`
- 本机工具：`python`（真实解释器，带 Pillow/numpy 用于图片处理；`python3` 是 WindowsApps 残桩无 pip，别用）

## 3. 代码地图

| 文件 | 职责 |
|---|---|
| `src-tauri/src/lib.rs` | 入口、模块、命令注册 |
| `src-tauri/src/commands.rs` | Tauri 命令层（含新加的 `search_unpack_tools` 外部解包工具检测） |
| `src-tauri/src/db.rs` | SQLite schema + 迁移 + 仓储 |
| `src-tauri/src/models.rs` | 前后端共享结构 |
| `src-tauri/src/scanner.rs` | 扫描识别 |
| `src-tauri/src/launcher.rs` | 启动 + LE 转区 + 后台时长统计 |
| `src-tauri/src/asset.rs` | 解包分发 + `extract_external`（`tool <archive> <outdir>` 桥） |
| `src-tauri/src/esc.rs` `nsa.rs` `pac.rs` | 各解包解码器 |
| `src-tauri/src/ge.rs` | **PGD 图片解码（⚠️ 见 §7）** |
| `src-tauri/src/vndb.rs` | VNDB API 客户端 |
| `src/style.css` | **重设计核心**：暖琥珀令牌 + @font-face(Fraunces/Outfit) + 表面/组件样式 + 动效体系 |
| `src/components/Icon.vue` | 统一内联 SVG 图标集 |
| `src/components/BrandLogo.vue` | 顶栏品牌图标（当前为**琥珀内联 SVG**，见 §5 决策点） |
| `src/components/GameCard.vue` `DetailDrawer.vue` `AssetDialog.vue` 等 | 各界面 |
| `src/assets/fonts/` | **Fraunces-Variable.woff2（衬线展示字）+ Outfit-Variable.woff2（无衬线 UI）**，本地打包 |
| `src/assets/brand-logo.png` | 128px 应用内真实 logo（顶栏/空状态/设置署名）；`public/logo.png` 为 favicon；旧的 `src/assets/logo.png` 已无引用 |

## 4. 已实现且验证

- 扫描导入 / 封面墙+搜索排序 / 多启动文件 / 隐藏·删除·打开目录 / LE 转区 / 后台时长统计
- VNDB 封面与元数据 / 补丁管理 / 资源解包（XP3/PFS/ESC/NSA/PAC 实机验证）
- **UI 重设计（暖琥珀方向）**：`vue-tsc` 通过、浏览器截图验证渲染正确
  - 砍掉紫粉渐变，单强调色 `--accent #d97e3d`；暖褐黑底 `#16120f` + 表面暖阶；阴影带底色；语义色暖化
  - 品牌字 Fraunces（拉丁衬线，中文回退雅黑），UI 字 Outfit；数字全部 `tabular-nums`
  - 主按钮琥珀底+深暖字（对比 5.96:1）；卡片暖阴影/星标琥珀光/激活 chip 琥珀底；封面墙 1560px 最大宽居中
  - 背景：暖琥珀双光极光 + 中央暖高光 + 细噪点 + 暗角（body::before 噪点，z-index 0 压内容之下）
  - 全部旧紫粉/冷色硬编码已清（补丁标签、格式徽章、占位渐变、扫描候选色板→暖大地色系）
- **图标**：用户最终选定 `E:\重要内容填充\galgame启动器图标\img_1786411428128.png`（**游戏手柄款**），**保持原色不换色**，已生成全套替换 `src-tauri/icons/`（含多尺寸 icon.ico）。⚠️ **需重编译才生效**（见 §5）
- **动效体系**（早前会话）：统一缓动令牌、卡片进场 stagger、抽屉/模态框进出场、toast 堆叠、按压反馈、prefers-reduced-motion 降级
- **外部解包工具预设**：设置里有预设下拉（GARbro/GalArc/arc_unpacker/Dumper2）+「检测」按钮 → 后端 `search_unpack_tools` 在常见位置自动找 exe；选中预设时出现「下载」按钮（`openUrl` 调 opener 插件打官方下载页，GARbro/GalArc/arc_unpacker 有 GitHub Releases 链接，Dumper2 查无稳定官方源故不配链接并提示手动指定）
- **galgame 资源站（市场）**：工具栏新增「资源站」按钮 → ResourceDialog（新组件），按 汉化/补丁 · 资源/社区 · 网盘/老站 · 游戏信息 四类列出经核实的站点（2DFan、鲲补丁 moyu.moe、御爱同萌 ai2.moe、翼梦舞城 otomedream、鲲 kungal、Nyaa、绯月 bbs.kfpromax、忧郁的loli mmgal、VNDB），每个一行名/描述/URL +「打开」（openUrl）；Esc 关闭；站点数组在组件顶部，想增减直接改
- **Taste Skill 家族已装**（全局 `~/.claude/skills/`）：design-taste-frontend、high-end-visual-design、minimalist-ui、industrial-brutalist-ui、redesign-existing-projects、stitch-design-taste、image-to-code（新版会话可用 `/redesign-skill` 等）
- **解包文件自动分类**：`classify_asset` 规则强化（新增 表情/界面/文档/数据 分类，补齐「其他音频」；识别 krkr/NScripter/NeXAS 惯例、v 开头语音、tlg/pgd/qnt 等图片格式）；解包后自动按分类**物理整理**到 `<分类>/` 子文件夹（`sort_extracted` + `prune_empty_dirs`），画廊列表与导出按物理布局读分类（`category_from_rel`）。顺带修掉 Windows 下 `to_string_lossy` 反斜杠导致画廊误分类的潜伏 bug。前端 AssetDialog 的 `CATEGORIES` 已同步
- **交互组件排版优化**：select 统一自定义箭头+focus 环、checkbox/radio 琥珀 `accent-color`、`.section-title` 加琥珀刻度（flex 对齐）、对话框引导段统一 `.hint`（13px/行高1.7/下边距14px）、`input[type=text]` focus 加焦点环；详情抽屉副标题 `.subline`、补丁警告行 `.warn`（琥珀）、hero 封面加细框+投影、KV 标签 `--text-faint→--text-dim`。**顺手清掉 6 处残留冷色**（AssetDialog tab.on / DetailDrawer .tag 紫色、PickExeDialog .row.exe 青色、.row:hover 的 #9682ff 兜底）——上次 HANDOFF 说紫粉已清干净但组件 scoped 样式里漏了
- **键盘支持**：新增 `src/composables/useCloseOnEscape.ts`（capture + stopImmediatePropagation + `active` 谓词，子对话框先注册能挡住父抽屉的 Esc，避免连关两层）；所有模态/抽屉/右键菜单/确认框按 Esc 关闭；封面墙 `←/→/↑/↓` 移动焦点（焦点在卡片上时，Enter 打开）
- **emoji → SVG**：LaunchDialog「📁」、PickExeDialog 目录行「📁」、App 空结果「🔍」全换 Icon 组件（新增 `upload`/`download` 两个描边图标）
- **整库备份/恢复**：新增 `src-tauri/src/backup.rs`（`export`/`restore`，2 个单测）；备份 zip = VACUUM INTO 的 db 一致快照 + covers/ + backups/（补丁备份），**排除可再生的 assets/ 解包缓存**；恢复先解压到 staging → 校验 games 表 → 热切换连接（内存连接换下旧句柄 → 替换文件 → `db::init` 重开，兼容老库补列）；命令 `export_backup` / `import_backup`，设置页新增「整库备份/恢复」区块，恢复后 App 刷新库

## 5. ⚠️ 重点交接：logo 已全部落地 + UI 复杂度已提升

**（A）logo 现状（本次已落地，无需再问颜色统一问题）**
- 用户选定 logo #2（img_1786411428128.png，**游戏手柄款**，原色）已替换 `src-tauri/icons/*`（含 icon.ico）。
- **应用内已统一用真实 logo**（紫粉原色，用户明确不换色），新增资源 `src/assets/brand-logo.png`（128px）：
  - 顶栏 BrandLogo.vue：琥珀 "G" SVG → 真实 logo 图（圆角裁切 + 细描边 + 中性深影，不再带琥珀光）
  - 空状态（无游戏时）：🎮 emoji → `.glyph-logo` 真实 logo
  - 设置对话框底部：品牌署名条（logo + GAL 启动器 + v0.1.0，置于脚部左侧）
  - favicon：`index.html` 改用 `/logo.png`（public/logo.png，64px）
  - 旧的 `src/assets/logo.png`（64px 缩略）已无引用，可删
- **任务栏/窗口图标还没重编译**：重跑 `npm run tauri dev`（或 `touch src-tauri/build.rs`）生效。

**（B）UI 复杂度提升（本次已做，方向与红线遵守情况）**
- 复杂度走的是**质感/深度/排版**，未引入多色渐变、霓虹、玻璃堆叠、新增 uppercase 小标签：
  - 顶栏：平直底边框 → 90° 渐隐发丝线（`.toolbar::after`）+ 顶部内高光 + 略增模糊
  - 封面卡：底座改**分层阴影**（内顶部高光 + 接触影 + 环境影）；hover 再加深分层 + 琥珀描边；**新增跟随鼠标的柔光**（GameCard `@mousemove` 写 `--mx/--my`，CSS `.card::before` radial-gradient，`prefers-reduced-motion` 下关闭）
  - 主按钮：顶部白色内高光 + 加深琥珀投影
  - chip 激活态：内圈发光 + 柔和外影
  - 模态框：标题改 **Fraunces**、头部底改渐隐发丝线、阴影分层 + 顶部高光
  - 抽屉：左侧平直边 → 渐隐发丝线、标题 Fraunces、分层阴影
  - toast：前导状态圆点（ok/err 着色 + 光晕）、分层阴影
  - 搜索框：focus 加琥珀焦点环
- 验证：`npx vue-tsc --noEmit` 通过；Claude Preview 截图确认顶栏/空状态/设置页渲染正确。卡片 hover 柔光需在真机（有库数据）里看效果。
- **若继续加深**：封面墙排版节奏、卡片信息密度（引擎 chip/时长）、抽屉内文件画像分组。守住红线即可。

## 6. 内置解包格式状态（全部在用户真实库上验证过）

| 格式 | 引擎/示例 | 状态 | 验证 |
|---|---|---|---|
| XP3 | 吉里吉里2/Z | ✅ 内置（xp3 crate） | 库内 2 个 krkr 游戏 |
| PFS | Artemis | ✅ 内置 | 单测+真实 |
| ESC-ARC1/2 | Escude（廃村少女×2） | ✅ 内置 | 真实 etc.bin/script |
| NSA | NScripter（时散） | ✅ 内置 | arc.nsa 3504 条目全解 |
| PAC | NeXAS 变体 | ✅ 内置 | ev.pac 499、st.pac 2732 条目全解 |
| PGD 图 | PAC 里的 GE 图 | ⚠️ 见 §7 | — |
| custom PKG | Yamiyo ni Odore | ❌ 无内置 | 走外部工具 |

## 7. PGD 图片解码（已搁置，用户暂停）

- 现象：PAC 解出的 `.PGD` 转 PNG 大部分花掉。当前 ge.rs 已按 pgd.cpp 思路实现，**实测 33/36 张能正确解码**（EV001A01/EV003A01 等肉眼正常），3 张（EV010A01/EV017ZA01/EV020A01）报 `LZ 字面越界`。用户**主动暂停此项**，别擅自继续。
- 若恢复：参考 weimingtom/ToolTLVN `pgd2png/pgd.cpp`（pgd_uncompress32 / _pgd3_ge_process_24/32 / ge2png）；PGD3 差分图需基图，画廊不列。相关探针测试 `ge::tests::real_ge_from_assets`（--ignored）保留可复用。

## 8. 其它已知边界 / 待办

- NSA comp=4（NBZ/bzip2）暂跳过；PAC 压缩类型（`u32@4 != 0`）未支持；Yamiyo 的 PKG 未内置
- 外部工具桥：设置里 `unpack_tool`，内置不认识的格式自动 `tool <archive> <outdir>`；有预设+自动检测
- 手动清理：`🖼 资源 → 🗑 清空解包缓存`
- AppData 老库迁移：db.rs `ensure_columns` 已能补列
- **前端旧色清理已做**，但若下一会话继续动组件，先 `grep -rn "a06bf5\|ff6fb0\|4cf0b0\|6bb4ff" src` 确认无残留

## 9. 测试

- `cargo test --lib` 全绿（约 11 个单测）
- 若干 `#[ignore]` 真实文件测试直接引 `F:\game\gal\...`，调试利器，别删
- 前端：`npx vue-tsc --noEmit`

## 10. 给下一会话的开工建议

1. 先读本文件，`npm run tauri dev` 看现状（注意：图标需重编译才显示新版，应用内 logo 已全落地）
2. **主线已完成**：logo 全落地 + UI 复杂度质感提升（见 §5）。下一轮可选方向：
   - 继续加深：封面墙排版节奏、卡片信息密度、抽屉文件画像分组（守红线）
   - 真机复核卡片 hover 柔光与分层阴影
3. 之后可选：NBZ、PAC(pack!=0)、Yamiyo PKG、PGD（用户暂停，勿擅动）
4. 产出后跑 `npx vue-tsc --noEmit` + Claude Preview 截图验证（`.claude/launch.json` 的 `gal-web`，端口 1421；主 dev 在 1420）

## 11. GitHub 仓库与版本发布流程（2026-08-11 新增）

- **仓库（公开，更新检查依赖它）**：https://github.com/netori/gal-launcher
  - 分支 `main`；`gh` 已登录（netori）；本仓库提交身份：`netori <netori@users.noreply.github.com>`（仅仓库级配置）
- **更新检查机制**（本期新增）：
  - 启动后 ~1.5s 静默查 `https://api.github.com/repos/netori/gal-launcher/releases/latest`；网络失败/限流一律静默，绝不打扰
  - 后端 `commands::check_update` / `dismiss_update`（仓库常量 `UPDATE_REPO` 在 commands.rs，改仓库记得同步）
  - 版本比较：`vX.Y.Z` 标签 vs `tauri.conf.json` 当前版本；不低于当前或已被「不再提示」→ None
  - 前端：左下角非打扰横幅（下载更新=浏览器打开 exe/msi 直链或 release 页 / 不再提示=写 settings `dismissed_update` 按版本记录 / ×=只关本次）
- **发布新版本流程**：
  1. 改版本号三处：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml` 的 `version`（release 标签用 `vX.Y.Z`）
  2. `npm run tauri build`
  3. `gh release create vX.Y.Z --title "vX.Y.Z" --notes "更新内容" "src-tauri/target/release/bundle/nsis/GAL Launcher_0.1.0_x64-setup.exe" "src-tauri/target/release/bundle/msi/GAL Launcher_0.1.0_x64_en-US.msi"`（注意：文件名里的版本号是 tauri.conf.json 里的，若版本变了路径里就是新版本号）
  4. 旧版用户下次启动即收到左上横幅；`不再提示` 只针对该版本，发新版本会再次提示
- **国内网络注意**：`api.github.com` 可能不稳定，更新检查会静默失败不影响使用；资源站导航里 2DFan 用的是大陆中转域名（fan2d.top）
- **⚠️ uploads.github.com 曾被 DNS 污染导致无法上传 release 资产**：全域名曾指向代理 IP 172.28.255.154（Clash 类工具 rules 没覆盖该域 → 直连 DNS NXDOMAIN），已通过开代理解决，**v0.1.0 release 的两个安装包附件已上传成功**。若再遇「no such host」，多半是代理规则又漏了该域或代理没开。首个 v0.1.0 release 已建且带完整资产

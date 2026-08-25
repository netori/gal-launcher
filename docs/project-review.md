# GAL 启动器 — 项目审查报告

> 审查范围：全部 `src/`（Vue3 前端）、`src-tauri/src/*.rs`（Rust 后端）、`website/`（官网）、构建配置（tauri.conf.json / Cargo.toml / vite.config.ts）、Git 状态。
> 验证手段：`npx vue-tsc --noEmit` ✅ · `npm run build`（vite 生产构建）✅ · `cargo check` ✅（8 个 dead-code 警告）· `cargo test --lib` ✅（15 passed / 10 ignored）· 浏览器渲染主应用空态与官网并截图（vision 工具审阅）· 配色测量。

---

## 一、总评

项目工程质量整体**高于同类个人项目平均水平**：

- 架构清晰：命令层 / 仓储层 / 平台能力分离，`commands.rs` 里锁外做 IO、锁内写库的节奏控制得当；
- 边界处理认真：zip 路径穿越防护、备份 staged 校验、热切换数据库连接、WAL、孤儿会话回收、`prefers-reduced-motion` 降级等都有考虑；
- 测试意识好：11+ 个单测覆盖 scanner/patcher/backup/asset/util，全部通过；
- 视觉方向明确：暖琥珀暗色主题统一（实测主色 `#1a1512` 占 95.7%），空状态与官网 hero 渲染干净、无错位。

主要问题集中在**数据层一致性、长任务并发模型、安全配置、发布自动化**四块，按优先级列出如下。

---

## 二、🔴 高优先级（真实缺陷 / 风险）

### 1. SQLite 外键约束从未开启 → `ON DELETE CASCADE` 全部失效

`schema`（`db.rs`）里 `game_files / play_sessions / patches / patch_backups / authorized_roots` 都声明了 `REFERENCES games(id) ON DELETE CASCADE`，但 `db::init()` 只设置了 `journal_mode = WAL`，**没有 `PRAGMA foreign_keys = ON`**。

SQLite 外键检查默认关闭且是**每连接**设置：删除游戏（`remove_from_library` / `delete_game`）时级联删除**不会触发**，子表行（文件画像、游玩记录、补丁与补丁备份映射）永久成为孤儿，库随读写无限膨胀；未来若按 `game_id` 聚合统计还会读出幽灵数据。

修复（一行 + 线程连接同步）：

```rust
conn.pragma_update(None, "journal_mode", "WAL")?;
conn.pragma_update(None, "foreign_keys", "ON")?;   // ← 加上
```

另外 `finish_session`、`install_patch` 等后台线程**另开连接**写库，这些连接同样要开 `foreign_keys`（写路径上的每条连接都开）。

### 2. 多写连接无 busy_timeout → 游玩时长统计可能静默丢失

主连接（`Mutex<Connection>`）+ 后台 `watch_session` 线程另开的连接会对 `games`/`play_sessions` 并发写。WAL 下读者不阻塞写者，但**两个写者**仍会冲突：rusqlite 默认 `busy_timeout = 0`，撞上即报 `SQLITE_BUSY`，而 `finish_session` 里 `let _ = conn.execute(...)` **把错误吞掉了** —— 一次"刚关掉游戏、恰好同时有别的写操作"就可能白记时长。

建议：每条连接 `PRAGMA busy_timeout = 5000`；`finish_session` 对写失败至少 `eprintln!`/记日志，别静默吞。

### 3. 长耗时 / 网络命令都是同步阻塞命令（Tauri 异步运行时被占用）

`scan_directory`（全盘遍历）、`import_games`（深度 10 遍历每个目录）、`fetch_missing_covers`（逐游戏两次 HTTP + `thread::sleep` 250ms×N）、`search_vndb`/`apply_vndb_metadata`（HTTP）、`check_update`（8s 超时 HTTP）、`extract_assets`（大包全量解压）全部是 `#[tauri::command]` 同步函数。

Tauri 2 中命令跑在异步运行时上，长时间阻塞会**卡住并发的其它命令**（含事件/窗口操作），社区明确建议重活走 `tauri::async_runtime::spawn_blocking` 或改 async + 分片：
- https://github.com/tauri-apps/tauri/discussions/10329（Running CPU-bound blocking work in a command）
- https://dev.to/hiyoyok/rust-async-in-tauri-v2-what-tripped-me-up-and-how-i-fixed-it-1662

建议改造顺序：`check_update`（启动 1.5s 后就跑，最常碰）→ `extract_assets`（顺带加进度事件，用户体感最明显）→ `fetch_missing_covers`（见 #5）。

### 4. CSP 为 null + WebView 直接加载远程图片

- `tauri.conf.json`：`"security": { "csp": null }` —— 无内容安全策略。
- `MetadataDialog.vue` L90：`<img :src="h.imageUrl">` 直接把 `s.vndb.org` 的封面**远程加载进 WebView**。

当前风险有限（其余图片都走 data URI），但"本地文件管理工具 + 可执行任意路径上 exe"的组合值得更严配置。建议：设置一条最小 CSP（`default-src 'self'; img-src 'self' data: https://s.vndb.org; style-src 'self' 'unsafe-inline'`，字体自包不需要外源，注意补 Tauri 的 `ipc:` / `asset:` 段），或把 VNDB 封面也改为 `download_cover` 后走本地读取。

---

## 三、🟡 中优先级

### 5. `fetch_missing_covers` 无进度、无取消、会撞 VNDB 限流

- 每游戏 **2 次请求**（search + fetch），连续跑，250ms 间隔只加在游戏间；约 100 个缺封面的游戏就超 VNDB 的 **200 req / 5 分钟** 配额（`vndb.rs` 头注释自己也写着），失败只能等全部跑完看聚合列表。
- 前端 `App.vue onImported` 导入后自动触发一次全库补全，用户无进度、无法中止。

建议：改为按批跑+断点续跑（每批 ≤40 游戏）、用 `tauri::Emitter` 推进度事件、前端给进度条+取消按钮；失败项落库（如 `meta_fetch_status`）而不是每次重扫全库。

### 6. 版本号不一致

- `package.json` / `tauri.conf.json` / `Cargo.toml` 是 **0.1.1**；
- `SettingsDialog.vue` L295 硬编码 **v0.1.0**，发布新版本时极易漏改。

建议：前端用 `@tauri-apps/api/app.getVersion()`（需在 capabilities 加 `core:app:default`），或由构建注入常量。

### 7. 封面缩略图缓存只增不减

`util.rs read_cover_thumb` 以 `sha1(path|mtime|size|size)` 为键落盘，源图变化即生成新文件，旧文件永久残留（注释也承认"孤儿文件"）；整库备份 `backup.rs` 又把整个 `covers/` 打进去，孤儿会随备份越滚越大。

建议：缩略图文件名里编码 mtime 后，启动时做一次"仅保留被当前库引用 + 最近 N 天"的清理，或把 thumbs 目录排除出备份。

### 8. 缺少窗口状态持久化 + 单实例保护

- 无 `tauri-plugin-window-state`，窗口位置/大小每次重置；
- 无 `tauri-plugin-single-instance`，双开时两个进程都持有写连接，是 #2 冲突的高发场景（尤其"系统级隐藏目录/批量操作"这类连环写）。

都有官方插件，接入成本很低。

### 9. `lock().unwrap()` 中毒即 panic

`commands.rs lock()` 对 `Mutex<Connection>` 直接 `.unwrap()`：任何持锁期间 panic（如某命令 `expect`）会把连接锁永久毒化，之后**所有**命令 panic。改为 `unwrap_or_else(|p| p.into_inner())` 即可。

### 10. `ge.rs` 8 个 dead-code 警告

PGD 解码按用户要求搁置，整个模块（`decode/write_png/lz_uncompress/process24/process32/u32le`）编译时一直报警。建议模块头加 `#![allow(dead_code)]` + 注释"PGD 解码暂停中"，或整体 `#[cfg(feature = "pgd")]` 门控，保持 `cargo check` 干净（CI 里 `-D warnings` 才不会被自家警告绊倒，见 #14）。

---

## 四、🟢 低优先级 / 清理

11. **未使用资产**：`src/assets/logo.png`（HANDOFF §5 已注明可删）、`src/assets/vue.svg`、`public/vite.svg`、`public/tauri.svg`（Vite 模板残留）→ 删掉减包体。
12. **文案**：`MetadataDialog.vue` L72 搜索框提示"梵文或中英文都行"的"梵文"疑为"日文"笔误；`README.md` "Rust 1.9x"建议写成具体版本。
13. **本地安装依赖的坑**：本机 `node_modules` 缺失时 `npx vue-tsc` 会现场拉取 **vue-tsc@3.3.10**（与 TS 5.6 不兼容，报 `ERR_PACKAGE_PATH_NOT_EXPORTED`），README 的检查命令建议加一句"先 `npm ci`"。
14. **无 lint / 格式化门禁**：TS 侧没有 eslint/prettier，Rust 侧没有 `cargo fmt` 检查。加 `eslint` 最保守配置 + `cargo fmt --check` 即可，收益远大于成本。
15. **发布全靠手动**（HANDOFF §11 流程）：建议加 GitHub Actions——`main` 推送跑 `vue-tsc` + `cargo test`；打 `v*` tag 时 `npm run tauri build` 并 `gh release create` 自动附 NSIS/MSI。官网 `_worker.js` 的代理下载逻辑已就位，自动化后整条链路闭环。
16. **更新分发可升级**：当前是"横幅 + 浏览器下载安装包"，后续可换 `tauri-plugin-updater`（应用内下载+校验+替换），用户体感更好；不急。
17. **联动小问题**：`delete_game` 先删库记录再送回收站，`trash::delete` 失败时记录已删、文件还在——已有文案提示，可接受；但若想更稳，可先回收站后删库（失败则不删）。

---

## 五、视觉审查结论（基于实际截图 + vision 工具）

| 项目 | 结论 |
|---|---|
| 空状态页 | 居中 logo + 标题 + 说明 + 主按钮，层级清晰，无错位；vision 检测无重叠 |
| 顶栏 | 搜索框/视图 chips/双下拉/五个按钮在 1280px 下布局舒展，`flex-wrap` 兜底窄窗 |
| 配色 | 实测主色 `#1a1512`（95.7%）+ 表层 `#47372f`，暖暗底符合设计令牌；琥珀主按钮对比度足够 |
| 官网 | hero 排版、特性卡、下载区均正常渲染；`正在获取最新版本…` 在本地 dev 下会回退直连 GitHub API（线上走 `_worker.js` 代理，正常） |
| 已知视觉取舍 | 空状态 logo 与"想玩"状态色 `#6bb4ff` 是仅剩的冷色元素——前者用户明确保留原色，后者若想统一可换暖色语义色（低优先） |

---

## 六、建议实施顺序

1. **一天内**：#1 外键 + #2 busy_timeout + #9 锁中毒（全是后端几行改动，`cargo test` 兜底）；
2. **一周内**：#3 改造 `check_update`/`extract_assets` 为 async + 进度事件；#8 加 window-state / single-instance 插件；#6 版本号注入；
3. **两周内**：#5 批量补封面改造（分批+进度+取消）；#4 CSP；#7 缩略图清理；
4. **随缘**：#10-#17 的清理与自动化。

---

## 七、修复记录（本次已实施）

| 项 | 修复内容 | 文件 |
|---|---|---|
| #1 外键失效 | `init()`/`open_worker()` 里 `PRAGMA foreign_keys=ON`；启动时 `reap_orphan_children` 清理历史孤儿行 | `src-tauri/src/db.rs` |
| #2 busy_timeout | 所有写连接 `busy_timeout(5s)`（`configure_conn` 统一配置）；`finish_session` 失败改为 `eprintln!` 记录，不再静默吞 | `src-tauri/src/db.rs` |
| #9 锁中毒 | `lock()` 改 `unwrap_or_else(into_inner)` | `src-tauri/src/commands.rs` |
| #3 阻塞命令 | `scan_directory / import_games / fetch_missing_covers / apply_vndb_metadata / check_update / extract_assets / search_unpack_tools` 全部转 async + `spawn_blocking`（长任务不再占用 Tauri 异步运行时）；`fetch_missing_covers` 改独立连接写库，不再握 AppState 锁 | `src-tauri/src/commands.rs` |
| 后台写连接 | `install_patch / uninstall_patch` 的独立连接统一走 `db::open_worker`（带 busy_timeout/外键） | `src-tauri/src/commands.rs` |
| #7 缩略图孤儿 | 新增 `util::cleanup_old_thumbs`，启动后后台清理 >30 天未修改的缩略图 | `src-tauri/src/util.rs`、`src-tauri/src/lib.rs` |
| #10 dead-code 警告 | `ge.rs` 模块级 `#![allow(dead_code)]`（附暂停说明），`cargo check` 归零警告 | `src-tauri/src/ge.rs` |
| #6 版本号 | 设置页脚改为从 `package.json` 构建期注入，同步三处版本时不再漏改 | `src/components/SettingsDialog.vue` |
| #12 文案 | MetadataDialog 搜索框提示"梵文"→"日文原名 / 罗马音 / 中文译名" | `src/components/MetadataDialog.vue` |
| #11 清理 | 删除未引用模板资产：`src/assets/logo.png`、`src/assets/vue.svg`、`public/vite.svg`、`public/tauri.svg` | — |

**未做（留待后续）**：#5 分批/进度/取消 UI、#13-#16 lint/CI/updater。

## 八、第二轮修复（#4 CSP + #8 插件）

| 项 | 修复内容 | 文件 |
|---|---|---|
| #4 CSP | `tauri.conf.json` 由 `csp: null` 改为最小安全策略：`default-src 'self'` + `connect-src ipc: http://ipc.localhost` + `img-src 'self' asset: http://asset.localhost data: https://s.vndb.org` + `style-src 'self' 'unsafe-inline'`（Vue 内联样式必需）+ `font-src`/`media-src`。说明：VNDB 搜索结果缩略图仍允许 `https://s.vndb.org`（该 CDN 只出静态图片，风险极低；应用持久封面本就下载到本地走 data URI）；脚本不放开 `unsafe-inline`/`eval` | `src-tauri/tauri.conf.json` |
| #8 窗口状态 | 新增 `tauri-plugin-window-state`，记住窗口位置/大小并在下次启动恢复 | `src-tauri/Cargo.toml`、`src-tauri/src/lib.rs` |
| #8 单实例 | 新增 `tauri-plugin-single-instance`，双开时聚焦已有主窗口；避免两个进程各持写连接撞 `SQLITE_BUSY` | `src-tauri/Cargo.toml`、`src-tauri/src/lib.rs` |

**平台坑（已处理）**：`window-state` 与 `single-instance` 两个 crate 顶层都有 `#![cfg(not(any(target_os = "android", target_os = "ios")))]` —— 移动端整 crate 为空、没有 `init`/`Builder`。注册统一走 `lib.rs` 里 cfg 门控的辅助函数（`window_state_plugin()` / `single_instance_plugin()`，移动端返回空占位插件），桌面与 Android 双端编译均为 0 警告。

**验证**：`cargo check`（桌面）exit 0 无警告 · `cargo check --target aarch64-linux-android`（需在 PATH 加入 `Android\Sdk\ndk\26.1.*\toolchains\llvm\prebuilt\windows-x86_64\bin`）exit 0 无警告 · `cargo test --lib` 15 passed · `npm run build` exit 0。

**仍留待后续**：#5 批量补封面分批/进度/取消 UI、#13-#16 lint/CI/updater 等自动化。

## 九、第三轮修复（#5 进度/取消 UI + #13 lint + #15 CI）

| 项 | 修复内容 | 文件 |
|---|---|---|
| #5 进度事件 | `fetch_missing_covers` 每处理完一个游戏发 `covers-progress` 事件（`{processed,total,current,done}`），前端进度条 + 当前游戏名 | `src-tauri/src/commands.rs` |
| #5 取消 | `AppState` 增加 `fetch_cancel: Arc<AtomicBool>`；新命令 `cancel_fetch_covers` 置位，循环下一轮即停；`CoverBatch` 增加 `cancelled` 字段 | `src-tauri/src/commands.rs`、`src-tauri/src/lib.rs` |
| #5 前端 | 新增共享单例 `useCoverFetch`（进度 + 取消 + 事件订阅）；App.vue 右下角进度横幅（可取消，样式沿用 update-banner 体系），导入后自动补全接入同一任务；设置页按钮改走同一实例 | `src/composables/useCoverFetch.ts`、`src/App.vue`、`src/style.css`、`src/components/SettingsDialog.vue`、`src/api.ts` |
| #13 ESLint | 新增 flat config（eslint + typescript-eslint + eslint-plugin-vue + globals.browser），规则与项目现状对齐（类型检查由 vue-tsc 兜底，模板排版不强制重排）；`npm run lint` 零告警 | `eslint.config.js`、`package.json` |
| #15 CI | `.github/workflows/ci.yml`：main 推送/PR 跑 lint + vue-tsc + build + cargo test；`.github/workflows/release.yml`：打 `v*` 标签自动 `tauri build` 产出 NSIS/MSI 并 `gh release create` 发布（供应用内更新横幅直链下载） | `.github/workflows/ci.yml`、`.github/workflows/release.yml` |

**验证**：`npm run lint` exit 0 · `vue-tsc` exit 0 · `npm run build` exit 0 · `cargo check`（桌面）0 警告 · `cargo check --target aarch64-linux-android` 0 警告 · `cargo test --lib` 15 passed。

**仍未做**：#16 `tauri-plugin-updater` 应用内自动更新（需要自建更新端点 + 签名密钥，属于部署侧工作，建议后续单独做）。

## 十、UI 美观性优化（第四轮）

按「UI 美观性建议」清单实施（跳过 #1 评分常驻，其余全做）：

| 项 | 内容 | 文件 |
|---|---|---|
| 卡片列宽 | 封面墙最小列宽 170→184px（窄窗 4 列变 3 列，信息区更舒展）；引擎 chip 加 `flex-shrink:0` + `max-width:62%`，与时长 chip 同行不挤压 | `src/style.css` `.grid`、`src/components/GameCard.vue` |
| 游玩进度条 | 详情抽屉新增「已游玩进度」（实际时长/VNDB 估时，琥珀渐变条 + 光晕，缺估时自动隐藏），设计同官网 mock 的 `.d-stat` | `src/components/DetailDrawer.vue` |
| 文件占比条 | 「文件画像」从数字 tag 墙改为占比条列表（宽度=该类数/总数，下限 4%），库存构成一目了然 | `src/components/DetailDrawer.vue` |
| 状态色统一 | 「想玩」唯一冷色 `#6bb4ff` → 暖杏 `#d9a25e`，全色板暖色化 | `src/api.ts` `STATUS_META` |
| 顶栏收纳 | 「失效检测」「资源站」收进右上「⋯」溢出菜单（复用 `.ctx` 样式 + 右上锚定 + `is-open` 激活态）；Esc/点击遮罩关闭 | `src/App.vue`、`src/style.css` |
| Toast 让位 | 批量操作条出现时 toast 上移（`.toast-wrap.lifted`），不再重叠 | `src/App.vue`、`src/style.css` |
| 官网同步 | AppMock 卡片补齐真实元素：评分常驻左上（`★ 8.4`）、引擎/时长 chips（`吉里吉里 14h`） | `website/src/components/AppMock.vue` |
| hero 优化 | 抽屉 hero 改用 `read_cover(800)` 缩略图（省内存），容器固定 3:4 + `object-fit:cover` 与封面墙裁切一致，hover 微放大 | `src/components/DetailDrawer.vue`、`src/style.css` |

**未改**：#7 空状态标题——检查后发现 `.empty h2` 已在用 `var(--font-display)`，无需改动。

**验证**：`npm run lint` / `vue-tsc` / `npm run build`（主应用 + 官网）全部 exit 0；浏览器实测「⋯」菜单弹出定位正确、官网预览卡片新元素渲染正常（vision 工具确认无重叠）。注：浏览器工具的键盘投递缺少 `key` 属性，Esc 关闭无法在该环境复验，但逻辑与此前在真机验证过的右键菜单同一套实现。
# 提示词：GAL 启动器 Android 移植

> 用法：把下面整段复制给执行移植的 AI（Claude Code / Cursor / 其他 Agent）。执行前先读项目根目录 `HANDOFF.md`、`README.md` 与本文件，再动手。
> 生成日期：2026-08-12。

---

## 角色与目标

你是资深 Rust + Tauri 2 + Android 工程师。请把位于 `E:\开发项目\GAL启动器` 的 Windows 桌面项目「GAL 启动器」移植到 Android 手机端，**最大程度复用现有代码**（Rust 后端核心 + Vue3 前端），并且**不允许破坏 Windows 桌面版**。移植 ≠ 重写：Tauri 2 官方支持 Android，应走同一框架的移动端通道。

## 项目是什么

Windows 桌面 galgame 本地图书馆启动器。扫描本地游戏目录 → 识别引擎/启动文件 → 封面墙管理（搜索/排序/收藏/隐藏）→ 一键/转区启动 → 后台时长统计 → VNDB 元数据补全 → 汉化/R18 补丁管理（zip/文件夹覆盖式+自动备份回滚）→ 内置资源解包（XP3/PFS/ESC-ARC/NSA/PAC，自动分类整理）→ 整库备份/恢复 → GitHub 更新检查。官网 `website/`（Cloudflare Pages）。

当前版本 v0.1.1。仓库 https://github.com/netori/gal-launcher 。

## 现有技术栈与代码地图

- 后端 Rust：`src-tauri/src/`，16 个文件。核心：
  - `db.rs` SQLite（rusqlite bundled，WAL 模式）、`models.rs` 共享结构、`commands.rs` Tauri 命令层（约 30 个命令）、`scanner.rs` 扫描/引擎识别
  - `launcher.rs` **Windows 绑定**：`Command::new` 拉起 exe + Locale Emulator 转区启动 + `child.wait()` 后台线程结算时长
  - `vndb.rs` VNDB API（ureq）、`patcher.rs` 补丁安装/回滚、`asset.rs`+`esc.rs`/`nsa.rs`/`pac.rs`/`ge.rs` 解包器
  - `backup.rs` 整库备份恢复、`util.rs`（含 `set_hidden_attr`，已有 `#[cfg(not(windows))]` 空实现）
- 前端 Vue3+TS+Vite：`src/`，`App.vue`(490 行) + `store.ts` + `api.ts`（Tauri invoke 桥）+ 14 个组件（GameCard/DetailDrawer/ScanDialog/PatchDialog/AssetDialog/MetadataDialog/SettingsDialog/FolderPickerDialog/PickExeDialog/LaunchDialog/ResourceDialog 等）
- 依赖注意：`zip` 必须 8.x；`windows-sys` 只被 util.rs 的 cfg(windows) 分支用；`trash` 用于删除；`tauri-plugin-dialog` 用于原生对话框；`tauri-plugin-opener` 用于打开外链
- 运行：`npm run tauri dev`；后端测试 `cargo test --lib`；前端类型 `npx vue-tsc --noEmit`

## 总体技术路线

1. `npm run tauri android init` 生成 `src-tauri/gen/android/`，接入 Tauri 2 官方 Android 通道（WebView 内跑同一套 Vue 前端，Rust 后端编译为 .so）。
2. **复用**：db/models/vndb/解包算法（esc/nsa/pac/asset 的解包逻辑）/补丁复制逻辑/备份/scanner 的引擎识别算法/util 的 data-URI 与 base64 工具/check_update —— 这些是纯 Rust 跨平台，应当一行业务逻辑都不改，只动文件 I/O 层。
3. **隔离**：所有 Windows 专属代码用 `#[cfg(target_os = "windows")]` 包住，非 Windows 平台给空实现或移动端替代实现（参考 util.rs 已有的写法）。
4. **前端**：桌面布局移动化，但 api.ts 的命令签名与 store 尽量不动。

## ⚠️ 开工前必须拍板的三个平台决策

### 决策 1：文件访问模型（最关键，决定后端改造量）

Android 10+ 分区存储，现有代码拿绝对路径直接 `std::fs` 在手机上默认失效。两条路线：

- **A. All-Files-Access（建议默认）**：manifest 声明 `MANAGE_EXTERNAL_STORAGE`，引导用户到系统设置打开「所有文件访问」。改造量最小——扫描/补丁/解包/删除几乎原样可跑，`std::fs` 直接读真实路径。代价：Play Store 上架受限，仅适合走 GitHub Releases + 官网直链分发（本项目正是如此）；Android 11 以下无需此权限。给一个首次启动的引导页/提示，未授权时功能禁用并跳转设置。
- **B. SAF 文档树**：`ACTION_OPEN_DOCUMENT_TREE` 选目录 → persistable URI 权限 → 全部文件操作改走 content resolver / DocumentFile。Play 合规，但要把命令层所有路径参数换成 SAF 句柄、`read_image`/遍历/覆盖/备份都要重写，工作量是 A 的 3 倍以上，`trash` 还要换 `DocumentsContract.deleteDocument`。

**默认走 A**，并在 `db.rs` 里加一张表记录「已授权的游戏根目录」和「是否已授予 All-Files-Access」；保留 SAF 为后续可选增强。如果你要上架 Play，再讨论 B。

### 决策 2：游戏启动方式（"转区启动"在 Android 不存在）

Windows 的 exe 启动 + Locale Emulator 转区在 Android 上无意义。改为**运行时（runtime）**模型，每款游戏存一个 `launch_method`：
- `apk`：通过包名 `Intent` 拉起已安装的 Android 版游戏（如 Ren'Py 打包的 APK）
- `file`：`ACTION_VIEW` + MIME/文件类型打开（由系统或用户选的 App 接管，如模拟器）
- `command`：自定义（仅高级用户，如 Termux 里跑脚本）

`launcher.rs` 的 `spawn_child`/`watch_session` 整块 `#[cfg(windows)]` 保留；移动端新写 `launch_android`（发 Intent 或提示用户已装运行时）。前端 `engine_needs_locale` 提示改为「该引擎在移动端需要模拟器/对应运行时」的提示。

### 决策 3：游玩时长统计

Windows 是 `child.wait()` 等进程退出结算。Android 拉起的不是子进程，无法等待。三选一或组合：
- 前台服务（`FOREGROUND_SERVICE`）在「正在游玩」期间计时，配合屏幕常亮；退出/切走时结算写入 `sessions` 表
- 需要精确前台检测时加 `PACKAGE_USAGE_STATS`（需特殊权限授权，作为可选增强）
- 提供手动「开始/结束」计时兜底

保持 DB schema 与桌面版一致（未来可做桌面↔手机数据迁移）。

## 后端改造清单

### 保持不动（纯跨平台，禁止改动逻辑）
`db.rs`、`models.rs`、`vndb.rs`、`esc.rs`、`nsa.rs`、`pac.rs`、`ge.rs`（⚠️ 用户已暂停 PGD 解码，勿碰）、`asset.rs` 的解包与分类逻辑、`patcher.rs` 的安装/回滚逻辑（仅换文件 I/O 来源）、`backup.rs`、`util.rs` 的 data-URI/base64/时间/`contains_ci`、`scanner.rs` 的引擎识别与文件画像算法、`commands.rs` 的 `check_update`/`dismiss_update`/设置读写。

### Windows 专属，必须 `#[cfg]` 隔离
| 位置 | 现状 | 移动端处理 |
|---|---|---|
| `launcher.rs` 全部 | `Command` 启动 + LE 转区 + `watch_session` | 整体 `#[cfg(windows)]`；移动端新写 Intent 启动与前台服务计时 |
| `commands.rs::reveal_in_explorer` | `explorer /select,` | `#[cfg(windows)]`；Android 替代为「分享」Intent 或直接移除该命令 |
| `util.rs::set_hidden_attr` | 已有 `#[cfg(not(windows))]` 空实现 | 保留现状即可 |
| `commands.rs::delete_game` | 依赖 `trash` crate | 先验证 trash 对 Android 的支持；不支持则 `#[cfg]` 分平台，Android 走 `std::fs::remove_dir_all`（All-Files-Access 下可用） |

### 文件 I/O 抽象（决策 1 为 A 时的最小改动）
- 命令层 `scan_directory`/`import_games`/`get_game_files`/`read_image`/补丁/解包的路径参数，在 Android 下来自「已授权根目录」（真实路径，All-Files-Access），所以**命令签名与 std::fs 基本不用改**；要改的是 UI 侧选目录的方式与权限引导。
- 写一层薄薄的 `platform_fs` 帮助函数（`#[cfg]` 内）：Android 上校验目录在已授权列表内、处理 `/storage/emulated/0` 的路径规范化；Windows 上直接透传。

### 新增移动能力
- AndroidManifest：`INTERNET`、`MANAGE_EXTERNAL_STORAGE`、`FOREGROUND_SERVICE`（决策 3 需要）、`PACKAGE_USAGE_STATS`（可选）
- 前台服务：长扫描、解包、时长计时时保活；进程被系统回收时能恢复
- 生命周期：Android 杀后台时 app 状态（当前抽屉/对话框）需要可恢复；扫描/解包任务用 WorkManager 或前台服务而非裸线程
- 首次启动引导：申请/检测文件访问权限

## 前端改造清单

- `api.ts` 命令签名**保持不变**，只可能在移动端新增命令（如 `request_all_files_access`/`check_files_access`/`get_authorized_roots`）
- `App.vue`：封面墙改响应式网格（手机 2 列）/列表切换；顶栏工具栏改**底部导航**（库/资源/设置）；「全部/收藏/隐藏」视图 chips 在小屏收敛
- `DetailDrawer.vue`：右侧抽屉 → 全屏页或底部 sheet；`ResourceDialog` 等对话框 → 全屏化，适配竖屏
- 右键菜单/悬停效果 → **长按**；键盘快捷键（Esc/方向键）保留但降级为增强项
- `FolderPickerDialog` 在 Android 改为系统文档树选择器或直接「选择一个已授权根目录」；`PickExeDialog` 的 zip/exe 文件选择在 Android 改 MIME 文件选择
- 触控目标 ≥44dp、适配 safe-area 与刘海屏、状态栏沉浸
- **保持暖琥珀设计令牌与 Fraunces/Outfit 字体**（`src/style.css`），不许换设计语言
- 图片展示 `read_image` 的 data-URI 方案跨平台复用
- 更新检查横幅逻辑已跨平台，移动端复用到设置页

## 里程碑与验收标准

| 里程碑 | 内容 | 验收 |
|---|---|---|
| **M0 骨架** | `tauri android init`，Rust 加 `aarch64-linux-android`/`armv7-linux-androideabi`/`x86_64-linux-android` target；真机或模拟器能跑起空应用 | 应用安装、启动、显示前端 |
| **M1 数据** | cfg 隔离 Windows 专属；All-Files-Access 授权引导；扫描/导入/封面墙在移动端跑通 | 手机上选到真实游戏目录 → 扫描识别 → 封面墙展示，时长/引擎字段正确 |
| **M2 启动** | 运行时模型 + Intent 启动 + 前台服务计时 | 点「开始游玩」能拉起对应 App 并计时，退出后时长入库 |
| **M3 生态** | VNDB 元数据、补丁管理、资源解包在移动端打通 | 与桌面版同库同格式；解包分类正确；补丁可装可回滚 |
| **M4 发布** | UI 移动化打磨、生命周期健壮、签名 APK/AAB、接入 GitHub release 与官网下载区 | `cargo test --lib` 通过；`npx vue-tsc --noEmit` 通过；APK 安装可分发；官网下载链接可用 |

## 全局验收红线

1. **Windows 桌面版不能坏**：共享代码一律 `#[cfg]` 分平台，不允许把桌面功能删掉或用 if-else 写死在移动分支里；改完要能 `npm run tauri build` 桌面打包 + `cargo test --lib` 全绿。
2. **DB schema 与桌面版保持一致**，为将来桌面↔手机迁移留路。
3. 版本号三处同步：`package.json`、`src-tauri/tauri.conf.json`、`src-tauri/Cargo.toml`；Android 另加 versionCode 递增。
4. 涉及删除/覆盖一律二次确认（沿用现有交互）。
5. 解包功能仅面向用户本地合法自持游戏，加免责文案。
6. `zip` 依赖保持 8.x（1.3 被 yanked）。
7. 设计红线：保持暖琥珀主题与 Fraunces/Outfit 字体，不做多色渐变/霓虹/玻璃堆叠。
8. 每步产出后自己验证：能跑就真机/模拟器跑截图确认，别只编译。

## 参考资料（执行者先读）

- `E:\开发项目\GAL启动器\HANDOFF.md` —— 最完整的交接文档（含代码地图、解包格式状态、发布流程）
- `E:\开发项目\GAL启动器\README.md`
- `E:\开发项目\GAL启动器\src-tauri\src\*.rs` 与 `src\*.vue`/`api.ts`/`store.ts`
- `website/` 官网子工程（发布时给下载区加 Android APK/AAB）
- GitHub 发布流程：`gh release create vX.Y.Z ...`（见 HANDOFF §11），上传 APK/AAB 资产

## 先回答再动手

开工前先输出：① 你对文件访问模型（决策 1）的取舍与理由；② 你确认的 Windows 专属代码完整清单（哪些要 cfg、哪些要替换）；③ 你的里程碑拆分与第一步计划。确认后再进入 M0。

# 手机 galgame 生态调研（2026-08-12）

> 目的：确定 GAL 启动器 Android 移植的「引擎→运行时」适配模型。结论已落到移植计划 M2。

## 核心结论

**手机上的 galgame 绝大多数是 PC 引擎的移植/播放**，靠专用的「运行时 App」打开游戏数据文件夹，而不是像 Windows 那样直接运行 exe。启动器的职责 = **识别引擎 → 映射到正确的运行时 App → 拉起运行时（尽力传游戏路径）→ 开始计时**。

关键事实：
1. 引擎识别（`scanner.rs` 已有）与运行时**一一对应**，启动模型应由引擎驱动。
2. 打开目标不是 exe，而是主资源：krkr=`data.xp3`（常被改名「运行点我.xp3」「dtcn.xp3」）、NScripter=游戏数据文件夹、RPG Maker=`Game.exe`、Ren'Py=APK、Tyrano=`index.html`。
3. **这些运行时都没有稳定可靠的「外部深链直接拉起指定游戏」接口**：Tyranor/Kirikiroid2 用自己的文件浏览器手动选目录/文件，JoiPlay 手动「添加游戏」。→ v1 采取「拉起运行时 App + 尽力传参，不成则引导用户点一下」。

## 引擎 → Android 运行时映射表

| PC 引擎（scanner 识别名） | 文件特征 | Android 运行时（包名） | 打开目标 |
|---|---|---|---|
| KiriKiri 吉里吉里 (krkr2/KiriKiri Z) | `.xp3`（`data.xp3`）、`startup.tjs`、`krkr.eXe`、`data/` 文件夹 | **Tyranor** `com.akira.tyranoemu`（新一代、推荐，兼容性广）、**Kirikiroid2** `org.tvp.kirikiri2`（老、高版本安卓兼容差、建议配合补丁） | 主 xp3 / exe / data 文件夹 |
| NScripter / ONScripter | `nscript.dat` / `0.txt`、`arc.nsa`、`default.ttf` | ONScripter 系列（new JH / old JH / MiNE / onscripter-EN）、**Tyranor**（也支持 ONS） | 游戏数据文件夹 |
| Ren'Py | `renpy/`、`lib/`、`.rpyc` | **原生 APK**（官方 Ren'Py Android 构建，直接装）或 **JoiPlay** `cyou.joiplay.joiplay` Ren'Py 插件 | APK / 游戏文件夹 |
| RPG Maker (2000/2003/XP/VX/VX Ace/MV/MZ) | `Game.exe`、`RPGVX*.exe`、`www/`(MV/MZ) | **JoiPlay** RPG Maker 插件（主力）、MaldiVes（仅 MV）、Tyranor | Game.exe |
| Wolf RPG Editor | `Game.exe` + wolf 资源 | **JoiPlay** Wolf RPG 插件 | Game.exe |
| TyranoBuilder / TyranoScript | `index.html`、`www/`、`data/` | **JoiPlay**、**Tyranor** | index.html |
| Artemis Engine | `.pfs` | **Tyranor**（兼容模式） | 文件夹 |
| VN Maker | — | Tyranor | — |
| Flash / Godot / HTML5 | `.swf` / 其它 | JoiPlay（Ruffle 插件 / Godot 实验性 / HTML5） | 对应入口文件 |
| 任意 Windows exe / 其它 | `.exe` | **Winlator**（完整 Windows 模拟）、PPSSPP(PSP) | 容器内 |
| VNDS | `.vnds` | VNDS Interpreter | — |

## 各运行时的启动方式细节

- **Tyranor（推荐主力，兼容 KRKR/ONS/Artemis/RPG Maker 等 20+ 引擎）**
  - 首次启动必须授予「所有文件访问」权限（正是本移植决策 1 的 All-Files-Access 模型，用户已习惯）
  - 添加游戏：底部「添加」→「选择路径」→ 选游戏文件夹 → 首页点卡片「启动游戏」；支持把游戏放 `Tyranor/` 根目录自动扫描
  - 支持 zip 直读；新版本还内嵌 ONS 扫描
- **Kirikiroid2**（老牌 krkr 专用，v1.3.9/1.4.0）
  - 打开 App 内文件浏览器 → 选游戏目录 → 点 `data.xp3`（或改名 xp3 / `name.exe` / `data` 文件夹）
  - 注意：Play 商店版是带广告的旧版，社区推荐去 GitHub（zeas2/Kirikiroid2）下 debloated 版；SD 卡需 root；高安卓有兼容问题
- **JoiPlay**（RPG Maker / Ren'Py / Wolf / Tyrano / Flash / HTML5）
  - 首页「+」→「添加游戏」→ 选游戏目录里的 `Game.exe`（RPG 类）或 `index.html`（HTML 类）→ 运行
  - Ren'Py / RPG Maker / Wolf 需要装对应插件（动态模块/独立插件）
- **ONScripter 家族**：选游戏文件夹（需含 `nscript.dat` + `arc.nsa`）
- **Winlator**：Windows 全模拟，作为兜底

## 对移植设计的启示

1. **M2 的 `launch_method` 默认按引擎映射**（上表），每游戏可覆盖（`runtime`/`apk`/`command`）。
2. **`detect_launch_target()`**：为 krkr 找主 xp3（名含 启动/运行/开始/start/data 优先，否则根目录首个 xp3）；RPG Maker 找 `Game.exe`；Tyrano 找 `index.html`；NScripter 定位文件夹。
3. **`is_app_installed(pkg)`** 检测运行时是否已装，未装返回「请先安装 Tyranor/JoiPlay…」+ 引导。
4. **外部拉起走「拉起 App + 尽力传参」**：优先级为 ① ACTION_VIEW + file URI（文件型运行时，如某些会注册文件关联）→ ② 仅拉起 App 并 toast 引导。不做不稳定的深链深度集成。
5. **补丁管理在手机上照样重要且更常用**：krkr 汉化/R18 补丁 = 往游戏目录拷 `.xp3`（覆盖式）；NScripter 换 `nscript.dat`/`arc.nsa`；RPG Maker 替换 `Game.exe`/`www` —— 全部命中现有 `install_replace` 覆盖式 + 自动备份。
6. **参考实现**：Pegasus frontend（Android 前端）用 `am start -n pkg/activity` 拉起 App，对模拟器用 intent extra 传路径（如 Dolphin `-e AutoStartFile`），见 [pegasus-android-appdb/specials.txt](https://github.com/mmatyas/pegasus-android-appdb/blob/master/specials.txt)。

## 来源

- 鲲 Galgame 论坛《如何在安卓系统上玩galgame》：https://www.kungal.com/topic/1642 （引擎→模拟器总表）
- 莱姆Lime《安卓如何使用Tyranor游玩Galgame》：https://limeblogs.github.io/2026/02/06/galgameandroid/ （krkr/ONS 文件特征与对比）
- Galgame Wiki Tyranor 词条：https://www.galgame.it/wiki/tool/emulators/tyranor.html （包名 com.akira.tyranoemu、添加流程）
- GitHub zeas2/Kirikiroid2：https://github.com/zeas2/Kirikiroid2/ （org.tvp.kirikiri2、data.xp3/name.exe/data 打开方式、补丁站 bbs.avgfun.net）
- JoiPlay 官网/文档：https://joiplay.net/ （cyou.joiplay.joiplay、添加 Game.exe/index.html、插件体系）
- VN Paths《How to Play Visual Novels on Android》：https://vnpaths.com/how-to-play-visual-novels-on-android/
- 柚哩神社 / 宅方社 / ACG 网等中文教程（krkr2 使用、mt 管理器/ZArchiver 解压流程）

/** 全站静态文案与数据（唯一数据源） */

export const REPO_URL = "https://github.com/netori/gal-launcher";

export const NAV_LINKS = [
  { href: "#features", label: "特性" },
  { href: "#preview", label: "预览" },
  { href: "#download", label: "下载" },
  { href: "#changelog", label: "更新" },
  { href: "#resources", label: "资源" },
  { href: "#faq", label: "FAQ" },
];

export interface Feature {
  icon: string;
  title: string;
  desc: string;
}

export const FEATURES: Feature[] = [
  {
    icon: "scan",
    title: "智能扫描与引擎识别",
    desc: "扫描本地游戏目录，自动识别引擎与启动文件，入库即可启动。",
  },
  {
    icon: "grid",
    title: "封面墙管理",
    desc: "搜索、排序、收藏与隐藏，按最近游玩 / 标题 / 评分 / 收藏整理收藏。",
  },
  {
    icon: "play",
    title: "一键启动与转区",
    desc: "一键启动，或经 Locale Emulator 转区启动，支持多启动文件。",
  },
  {
    icon: "chart",
    title: "后台游玩统计",
    desc: "自动累计游玩时长、次数与上次游玩时间，无需手动记录。",
  },
  {
    icon: "book",
    title: "VNDB 元数据补全",
    desc: "自动补全封面与资料，支持批量补全与手动搜索。",
  },
  {
    icon: "tag",
    title: "补丁管理",
    desc: "汉化 / R18 / 修正补丁的安装、一键回滚与自动备份。",
  },
  {
    icon: "box",
    title: "内置资源解包",
    desc: "支持 XP3 / PFS / ESC-ARC / NSA / PAC，解包后自动按类型分类。",
  },
  {
    icon: "download",
    title: "整库备份恢复",
    desc: "数据库、封面与补丁备份一键完成，迁移无忧。",
  },
];

export interface ResourceLink {
  name: string;
  domain: string;
  url: string;
  desc: string;
}

export const RESOURCES: ResourceLink[] = [
  {
    name: "2DFan",
    domain: "fan2d.top",
    url: "https://fan2d.top",
    desc: "中文 galgame 资讯与评分社区（大陆中转）",
  },
  {
    name: "御爱同萌",
    domain: "www.ai2.moe",
    url: "https://www.ai2.moe",
    desc: "补丁与资源交流社区",
  },
  {
    name: "鲲Galgame",
    domain: "www.kungal.com",
    url: "https://www.kungal.com",
    desc: "Galgame 资讯与讨论社区",
  },
  {
    name: "鲲补丁",
    domain: "www.moyu.moe",
    url: "https://www.moyu.moe",
    desc: "补丁聚合站",
  },
  {
    name: "Nyaa",
    domain: "nyaa.si",
    url: "https://nyaa.si",
    desc: "资源索引站",
  },
  {
    name: "VNDB",
    domain: "vndb.org",
    url: "https://vndb.org",
    desc: "英文视觉小说数据库",
  },
];

export interface FaqItem {
  q: string;
  a: string;
}

export const FAQS: FaqItem[] = [
  {
    q: "如何转区启动？",
    a: "在游戏详情里选择「转区启动」，需系统装有 Locale Emulator；未安装时会提示配置其路径。",
  },
  {
    q: "解包出来的文件看不懂分类怎么办？",
    a: "解包后会自动按立绘 / CG / 背景 / 语音 / BGM / 视频 / 脚本 / 文档 / 数据归类到子文件夹；个别内置不支持的格式可桥接 GARbro 等外部工具。",
  },
  {
    q: "备份存在哪里？",
    a: "整库备份默认保存在应用数据目录，可在设置中自定义保存位置；恢复时选择备份文件即可。",
  },
  {
    q: "支持哪些平台？",
    a: "当前提供 Windows 桌面版。macOS / Linux 等其他平台与历史版本，请关注 GitHub Releases。",
  },
  {
    q: "如何反馈 Bug？",
    a: "到 GitHub Issues 提交，附上系统版本、复现步骤与日志即可；也欢迎直接提功能建议。",
  },
];

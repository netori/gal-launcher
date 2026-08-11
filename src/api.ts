import { invoke } from "@tauri-apps/api/core";

export interface Game {
  id: number;
  title: string;
  sourceDir: string;
  launchPath: string;
  launchCandidates: string[];
  launchSet: boolean;
  engine: string;
  coverPath: string | null;
  description: string | null;
  rating: number | null;
  vndbId: string | null;
  tags: string[];
  developer: string | null;
  released: string | null;
  lengthMinutes: number | null;
  addedAt: number;
  lastPlayed: number | null;
  totalSeconds: number;
  playCount: number;
  hidden: boolean;
  favorite: boolean;
}

export interface Candidate {
  title: string;
  sourceDir: string;
  launchPath: string;
  launchCandidates: string[];
  engine: string;
  coverPath: string | null;
  fileCount: number;
  alreadyImported: boolean;
  note: string;
}

export interface FileInfo {
  id: number;
  relPath: string;
  kind: string;
  size: number;
}

export interface Patch {
  id: number;
  gameId: number;
  name: string;
  kind: string;
  sourcePath: string;
  installMethod: string;
  installed: boolean;
  installedAt: number | null;
  backupDir: string | null;
  note: string;
}

export interface VnSearchHit {
  vndbId: string;
  title: string;
  imageUrl: string | null;
  rating: number | null;
  votecount: number;
}

export interface Settings {
  localeEmulatorPath: string | null;
  gameRoot: string | null;
  unpackTool: string | null;
}

export const api = {
  scanDirectory: (root: string) => invoke<Candidate[]>("scan_directory", { root }),
  importGames: (candidates: Candidate[]) =>
    invoke<number>("import_games", { candidates }),
  listGames: (showHidden: boolean) =>
    invoke<Game[]>("list_games", { showHidden }),
  getGameFiles: (gameId: number) =>
    invoke<FileInfo[]>("get_game_files", { gameId }),
  toggleFavorite: (gameId: number) =>
    invoke<Game>("toggle_favorite", { gameId }),
  setHidden: (gameId: number, hidden: boolean) =>
    invoke<Game>("set_hidden", { gameId, hidden }),
  removeFromLibrary: (gameId: number) =>
    invoke<void>("remove_from_library", { gameId }),
  deleteGame: (gameId: number) => invoke<void>("delete_game", { gameId }),
  setHiddenAttr: (path: string, hidden: boolean) =>
    invoke<void>("set_hidden_attr", { path, hidden }),
  readImage: (path: string) => invoke<string>("read_image", { path }),
  launchGame: (gameId: number, useLocale: boolean, launchPath?: string) =>
    invoke<Game>("launch_game", { gameId, useLocale, launchPath }),
  setLaunchFile: (gameId: number, launchPath: string) =>
    invoke<Game>("set_launch_file", { gameId, launchPath }),
  saveSetting: (key: string, value: string) =>
    invoke<void>("save_setting", { key, value }),
  getSettings: () => invoke<Settings>("get_settings"),
  /** 在常见位置查找外部解包工具：参数为候选 exe 文件名，返回「exe 文件名 → 路径」。 */
  searchUnpackTools: (exes: string[]) =>
    invoke<Record<string, string>>("search_unpack_tools", { exes }),

  // VNDB 元数据
  searchVndb: (query: string) => invoke<VnSearchHit[]>("search_vndb", { query }),
  applyVndbMetadata: (gameId: number, vndbId: string, useVndbTitle: boolean) =>
    invoke<Game>("apply_vndb_metadata", { gameId, vndbId, useVndbTitle }),
  setGameTitle: (gameId: number, title: string) =>
    invoke<Game>("set_game_title", { gameId, title }),
  fetchMissingCovers: () =>
    invoke<{ updated: number; failed: string[] }>("fetch_missing_covers"),
  reveal: (path: string) => invoke<void>("reveal_in_explorer", { path }),

  // 补丁
  addPatch: (input: {
    gameId: number;
    name: string;
    kind: string;
    sourcePath: string;
    installMethod: string;
  }) => invoke<Patch>("add_patch", { input }),
  getPatches: (gameId: number) => invoke<Patch[]>("list_patches", { gameId }),
  installPatch: (patchId: number) =>
    invoke<Patch>("install_patch", { patchId }),
  uninstallPatch: (patchId: number) =>
    invoke<Patch>("uninstall_patch", { patchId }),
  removePatch: (patchId: number) =>
    invoke<void>("remove_patch", { patchId }),

  // M3 资源解包
  listAssetArchives: (gameId: number) =>
    invoke<ArchiveInfo[]>("list_asset_archives", { gameId }),
  extractAssets: (gameId: number, archiveRel: string) =>
    invoke<AssetEntry[]>("extract_assets", { gameId, archiveRel }),
  listExtractedAssets: (gameId: number) =>
    invoke<AssetEntry[]>("list_extracted_assets", { gameId }),
  exportAssets: (gameId: number, dest: string, category?: string) =>
    invoke<number>("export_assets", { gameId, dest, category }),
  clearAssetCache: (gameId: number) =>
    invoke<number>("clear_asset_cache", { gameId }),

  // 整库备份 / 恢复
  exportBackup: (dest: string) =>
    invoke<{ path: string; fileCount: number }>("export_backup", { dest }),
  importBackup: (src: string) =>
    invoke<{ games: number; covers: number; backups: number }>("import_backup", { src }),
};

export interface ArchiveInfo {
  relPath: string;
  absPath: string;
  format: string;
  sizeBytes: number;
  extractedCount: number;
}

export interface AssetEntry {
  rel: string;
  absPath: string;
  size: number;
  category: string;
}

/** 显示用什么引擎转区时的提示文案。 */
export const engineNeedsLocale = (engine: string) =>
  /吉里|Kiri|RPG|NScrip|WOLF|Artemis|Ren/i.test(engine);
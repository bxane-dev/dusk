import { invoke } from "@tauri-apps/api/core";
import type {
  Achievement,
  AutoScreenshotScanResult,
  CollectionMembership,
  CollectionRecord,
  GameRecord,
  LaunchResult,
  ScanResult,
  ScreenshotRecord,
  SaveBackupRecord,
  SaveConfig,
  Stats,
} from "../types";

export const api = {
  listGames: () => invoke<GameRecord[]>("list_games"),
  scanGames: () => invoke<ScanResult>("scan_games"),
  chooseExecutable: () => invoke<string | null>("choose_executable"),
  chooseGameInstaller: () => invoke<string | null>("choose_game_installer"),
  runGameInstaller: (installerPath: string) =>
    invoke<void>("run_game_installer", { installerPath }),
  openExternalTarget: (target: "creator" | "steam" | "epic" | "gog" | "itch") =>
    invoke<void>("open_external_target", { target }),
  addManualGame: (title: string, exePath: string) =>
    invoke<GameRecord>("add_manual_game", { title, exePath }),
  setFavorite: (gameId: string, favorite: boolean) =>
    invoke<void>("set_favorite", { gameId, favorite }),
  renameGame: (gameId: string, title: string) =>
    invoke<void>("rename_game", { gameId, title }),
  removeGame: (gameId: string) => invoke<void>("remove_game", { gameId }),
  chooseCover: (gameId: string) => invoke<boolean>("choose_cover", { gameId }),
  launchGame: (gameId: string) =>
    invoke<LaunchResult>("launch_game", { gameId }),
  openGameFolder: (gameId: string) =>
    invoke<void>("open_game_folder", { gameId }),
  importScreenshots: (gameId: string) =>
    invoke<number>("import_screenshots", { gameId }),
  scanScreenshots: () =>
    invoke<AutoScreenshotScanResult>("scan_screenshots"),
  listScreenshots: (gameId?: string) =>
    invoke<ScreenshotRecord[]>("list_screenshots", { gameId: gameId ?? null }),
  deleteScreenshot: (screenshotId: number) =>
    invoke<void>("delete_screenshot", { screenshotId }),
  listCollections: () => invoke<CollectionRecord[]>("list_collections"),
  createCollection: (name: string) =>
    invoke<CollectionRecord>("create_collection", { name }),
  deleteCollection: (collectionId: string) =>
    invoke<void>("delete_collection", { collectionId }),
  setCollectionMembership: (
    collectionId: string,
    gameId: string,
    included: boolean,
  ) =>
    invoke<void>("set_collection_membership", {
      collectionId,
      gameId,
      included,
    }),
  collectionMemberships: () =>
    invoke<CollectionMembership[]>("collection_memberships"),
  getStats: () => invoke<Stats>("get_stats"),
  listAchievements: () => invoke<Achievement[]>("list_achievements"),
  chooseSaveFolder: (gameId: string) =>
    invoke<SaveConfig | null>("choose_save_folder", { gameId }),
  getSaveConfig: (gameId: string) =>
    invoke<SaveConfig | null>("get_save_config", { gameId }),
  clearSaveConfig: (gameId: string) =>
    invoke<void>("clear_save_config", { gameId }),
  createSaveBackup: (gameId: string) =>
    invoke<SaveBackupRecord>("create_save_backup", { gameId }),
  listSaveBackups: (gameId: string) =>
    invoke<SaveBackupRecord[]>("list_save_backups", { gameId }),
  restoreSaveBackup: (gameId: string, backupId: string) =>
    invoke<SaveBackupRecord>("restore_save_backup", { gameId, backupId }),
  deleteSaveBackup: (gameId: string, backupId: string) =>
    invoke<void>("delete_save_backup", { gameId, backupId }),
  dataDirectory: () => invoke<string>("data_directory"),
};

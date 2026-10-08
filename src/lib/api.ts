import { invoke } from "@tauri-apps/api/core";
import type {
  Achievement,
  AutoScreenshotScanResult,
  CollectionMembership,
  CollectionRecord,
  CloudUploadResult,
  CloudSyncAllResult,
  GameRecord,
  LaunchResult,
  ProfileRecord,
  ProfileSaveFileState,
  ScanResult,
  ScreenshotRecord,
  SaveBackupRecord,
  SaveConfig,
  Stats,
} from "../types";

export const api = {
  setAccountScope: (userId: string | null) => invoke<void>("set_account_scope", { accountUserId: userId }),
  exportAccountState: () => invoke<Record<string, unknown>>("export_account_state"),
  importAccountState: (state: Record<string, unknown>) => invoke<void>("import_account_state", { state }),
  listProfiles: () => invoke<ProfileRecord[]>("list_profiles"),
  getActiveProfile: () => invoke<ProfileRecord>("get_active_profile"),
  createProfile: (name: string) =>
    invoke<ProfileRecord>("create_profile", { name }),
  renameProfile: (profileId: string, name: string) =>
    invoke<ProfileRecord>("rename_profile", { profileId, name }),
  setActiveProfile: (profileId: string) =>
    invoke<ProfileRecord>("set_active_profile", { profileId }),
  deleteProfile: (profileId: string) =>
    invoke<ProfileRecord>("delete_profile", { profileId }),
  listGames: () => invoke<GameRecord[]>("list_games"),
  scanGames: () => invoke<ScanResult>("scan_games"),
  refreshMissingCovers: () =>
    invoke<{ attempted: number; updated: number; remaining: number }>("refresh_missing_covers"),
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
  getProfileSaveFileState: (gameId: string) =>
    invoke<ProfileSaveFileState>("get_profile_save_file_state", { gameId }),
  saveProfileFiles: (gameId: string) =>
    invoke<ProfileSaveFileState>("save_profile_files", { gameId }),
  loadProfileFiles: (gameId: string) =>
    invoke<ProfileSaveFileState>("load_profile_files", { gameId }),
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
  uploadSaveBackupToCloud: (input: {
    backupId: string;
    supabaseUrl: string;
    publishableKey: string;
    accessToken: string;
    authUserId: string;
  }) => invoke<CloudUploadResult>("upload_save_backup_to_cloud", input),
  uploadAllSaveBackupsToCloud: (input: {
    supabaseUrl: string;
    publishableKey: string;
    accessToken: string;
    authUserId: string;
  }) => invoke<CloudSyncAllResult>("upload_all_save_backups_to_cloud", input),
  uploadCloudManifest: (input: {
    supabaseUrl: string;
    publishableKey: string;
    accessToken: string;
    authUserId: string;
  }) => invoke<void>("upload_cloud_manifest", input),
  discordRpcEnable: (clientId: string, state?: string) =>
    invoke<void>("discord_rpc_enable", { clientId, state: state ?? null }),
  discordRpcUpdate: (state: string) => invoke<void>("discord_rpc_update", { state }),
  discordRpcDisable: () => invoke<void>("discord_rpc_disable"),
  dataDirectory: () => invoke<string>("data_directory"),
};

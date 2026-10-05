import { check, type DownloadEvent } from "@tauri-apps/plugin-updater";

export interface UpdateCheckResult {
  available: boolean;
  version?: string;
  body?: string | null;
  date?: string | null;
}

export async function checkForDuskUpdate(): Promise<UpdateCheckResult> {
  const update = await check();
  if (!update) {
    return { available: false };
  }

  const result: UpdateCheckResult = {
    available: true,
    version: update.version,
    body: update.body,
    date: update.date,
  };

  await update.close();
  return result;
}

export async function installDuskUpdate(
  onProgress?: (event: DownloadEvent) => void,
): Promise<void> {
  const update = await check();
  if (!update) return;

  await update.downloadAndInstall((event) => {
    onProgress?.(event);
  });
}

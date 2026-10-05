export interface GameRecord {
  id: string;
  title: string;
  exePath: string | null;
  installPath: string;
  source: "steam" | "epic" | "manual" | string;
  sourceId: string | null;
  favorite: boolean;
  coverPath: string | null;
  coverDataUrl: string | null;
  addedAt: string;
  lastPlayed: string | null;
  totalSeconds: number;
  launchCount: number;
}

export interface ScanResult {
  found: number;
  added: number;
  updated: number;
  steamFound: number;
  epicFound: number;
  gogFound: number;
  emulatorFound: number;
  warnings: string[];
}

export interface LaunchResult {
  started: boolean;
  tracking: boolean;
  message: string;
}

export interface Stats {
  gameCount: number;
  favoriteCount: number;
  playedGameCount: number;
  totalSeconds: number;
  launchCount: number;
  last7DaysSeconds: number;
  screenshotCount: number;
  topGame: string | null;
}

export interface Achievement {
  id: string;
  title: string;
  description: string;
  unlocked: boolean;
  current: number;
  target: number;
}

export interface ScreenshotRecord {
  id: number;
  gameId: string;
  path: string;
  dataUrl: string | null;
  createdAt: string;
}

export interface CollectionRecord {
  id: string;
  name: string;
  gameCount: number;
}

export interface CollectionMembership {
  collectionId: string;
  gameId: string;
}

export interface AutoScreenshotScanResult {
  found: number;
  imported: number;
  steamImported: number;
  matchedImported: number;
  skippedDuplicates: number;
}

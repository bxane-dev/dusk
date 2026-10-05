import {
  Archive,
  Check,
  Clock3,
  Database,
  FolderOpen,
  Gamepad2,
  HardDrive,
  Heart,
  Home,
  ImagePlus,
  Images,
  Layers3,
  Library,
  Moon,
  Pencil,
  Play,
  Plus,
  RefreshCw,
  RotateCcw,
  ScanSearch,
  Save,
  Search,
  Settings,
  Sparkles,
  Star,
  Trash2,
  Trophy,
  X,
} from "lucide-react";
import {
  type FormEvent,
  type MouseEvent,
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "./lib/api";
import { checkForDuskUpdate, installDuskUpdate } from "./lib/updater";
import { useControllerNavigation } from "./lib/useControllerNavigation";
import type {
  Achievement,
  CollectionMembership,
  CollectionRecord,
  GameRecord,
  ScreenshotRecord,
  SaveBackupRecord,
  SaveConfig,
  Stats,
} from "./types";

type View =
  | "home"
  | "library"
  | "favorites"
  | "screenshots"
  | "achievements"
  | "settings";

type SortMode = "name" | "recent" | "playtime";
type ThemeName = "night" | "oled" | "slate";
type AccentName = "violet" | "ember" | "cyan";

const EMPTY_STATS: Stats = {
  gameCount: 0,
  favoriteCount: 0,
  playedGameCount: 0,
  totalSeconds: 0,
  launchCount: 0,
  last7DaysSeconds: 0,
  screenshotCount: 0,
  topGame: null,
};

function cx(...values: Array<string | false | null | undefined>) {
  return values.filter(Boolean).join(" ");
}

function readableError(error: unknown) {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  try {
    return JSON.stringify(error);
  } catch {
    return "Unknown error";
  }
}

function formatDuration(totalSeconds: number) {
  if (!totalSeconds) return "0m";
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  if (hours >= 100) return String(hours) + "h";
  if (hours > 0) return String(hours) + "h " + String(minutes) + "m";
  return String(Math.max(1, minutes)) + "m";
}

function formatBytes(bytes: number) {
  if (bytes <= 0) return "0 B";
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  return (unit === 0 ? String(Math.round(value)) : value.toFixed(value >= 10 ? 1 : 2)) + " " + units[unit];
}

function formatDate(value: string | null) {
  if (!value) return "Never";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "Unknown";
  return new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "short",
  }).format(date);
}

function sourceLabel(source: string) {
  if (source === "steam") return "Steam";
  if (source === "epic") return "Epic";
  if (source === "gog") return "GOG";
  if (source === "emulator") return "Emulator";
  if (source === "manual") return "Manual";
  return source;
}

function NavItem(props: {
  active: boolean;
  icon: ReactNode;
  label: string;
  count?: number;
  onClick: () => void;
}) {
  return (
    <button className={cx("nav-item", props.active && "active")} onClick={props.onClick}>
      <span className="nav-icon">{props.icon}</span>
      <span>{props.label}</span>
      {typeof props.count === "number" && <span className="nav-count">{props.count}</span>}
    </button>
  );
}

function Cover({ game }: { game: GameRecord }) {
  if (game.coverDataUrl) {
    return (
      <div className="cover">
        <img src={game.coverDataUrl} alt="" />
      </div>
    );
  }

  const initials = game.title
    .split(" ")
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part.slice(0, 1).toUpperCase())
    .join("");

  return (
    <div className="cover cover-placeholder">
      <span>{initials || "D"}</span>
      <small>{sourceLabel(game.source)}</small>
    </div>
  );
}

function GameCard(props: {
  game: GameRecord;
  onOpen: () => void;
  onPlay: () => void;
  onFavorite: () => void;
  playing: boolean;
}) {
  function play(event: MouseEvent<HTMLButtonElement>) {
    event.stopPropagation();
    props.onPlay();
  }

  function favorite(event: MouseEvent<HTMLButtonElement>) {
    event.stopPropagation();
    props.onFavorite();
  }

  return (
    <article
      className="game-card"
      data-controller-game-id={props.game.id}
      role="button"
      tabIndex={0}
      onClick={props.onOpen}
      onKeyDown={(event) => {
        if (event.key === "Enter" || event.key === " ") props.onOpen();
      }}
    >
      <div className="game-art">
        <Cover game={props.game} />
        <div className="game-overlay">
          <button
            className="round-action primary"
            onClick={play}
            aria-label={"Play " + props.game.title}
            disabled={props.playing}
          >
            {props.playing ? <RefreshCw className="spin" size={18} /> : <Play size={18} />}
          </button>
          <button
            className={cx("round-action", props.game.favorite && "selected")}
            onClick={favorite}
            aria-label={props.game.favorite ? "Remove favorite" : "Add favorite"}
          >
            <Star size={17} fill={props.game.favorite ? "currentColor" : "none"} />
          </button>
        </div>
      </div>
      <div className="game-card-copy">
        <strong title={props.game.title}>{props.game.title}</strong>
        <div className="game-meta">
          <span>{sourceLabel(props.game.source)}</span>
          <span>•</span>
          <span>{formatDuration(props.game.totalSeconds)}</span>
        </div>
      </div>
    </article>
  );
}

function EmptyState(props: {
  icon: ReactNode;
  title: string;
  copy: string;
  action?: ReactNode;
}) {
  return (
    <div className="empty-state">
      <div className="empty-icon">{props.icon}</div>
      <h3>{props.title}</h3>
      <p>{props.copy}</p>
      {props.action}
    </div>
  );
}

function AddGameModal(props: {
  onClose: () => void;
  onAdded: () => Promise<void>;
}) {
  const [title, setTitle] = useState("");
  const [path, setPath] = useState("");
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");

  async function browse() {
    try {
      const picked = await api.chooseExecutable();
      if (picked) setPath(picked);
    } catch (err) {
      setError(readableError(err));
    }
  }

  async function submit(event: FormEvent) {
    event.preventDefault();
    setError("");
    if (!title.trim() || !path.trim()) {
      setError("Choose an executable and enter a game title.");
      return;
    }

    setSaving(true);
    try {
      await api.addManualGame(title.trim(), path.trim());
      await props.onAdded();
      props.onClose();
    } catch (err) {
      setError(readableError(err));
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="modal-backdrop" onMouseDown={props.onClose}>
      <form
        className="modal"
        onSubmit={submit}
        onMouseDown={(event) => event.stopPropagation()}
      >
        <div className="modal-head">
          <div>
            <span className="eyebrow">Manual library entry</span>
            <h2>Add game</h2>
          </div>
          <button type="button" className="icon-button" onClick={props.onClose}>
            <X size={18} />
          </button>
        </div>

        <label>
          <span>Game title</span>
          <input
            autoFocus
            value={title}
            onChange={(event) => setTitle(event.target.value)}
            placeholder="Game name"
          />
        </label>

        <label>
          <span>Executable</span>
          <div className="path-picker">
            <input value={path} readOnly placeholder="Choose a .exe file" />
            <button type="button" className="button secondary" onClick={browse}>
              Browse
            </button>
          </div>
        </label>

        {error && <div className="inline-error">{error}</div>}

        <div className="modal-actions">
          <button type="button" className="button ghost" onClick={props.onClose}>
            Cancel
          </button>
          <button className="button primary" disabled={saving}>
            {saving ? "Adding…" : "Add to Dusk"}
          </button>
        </div>
      </form>
    </div>
  );
}

function GameDetail(props: {
  game: GameRecord;
  collections: CollectionRecord[];
  memberships: CollectionMembership[];
  onClose: () => void;
  onPlay: (game: GameRecord) => Promise<void>;
  onRefresh: () => Promise<void>;
  onToast: (message: string, type?: "ok" | "error") => void;
}) {
  const [busy, setBusy] = useState(false);
  const [saveConfig, setSaveConfig] = useState<SaveConfig | null>(null);
  const [saveBackups, setSaveBackups] = useState<SaveBackupRecord[]>([]);
  const [saveLoading, setSaveLoading] = useState(true);

  const membershipSet = useMemo(() => {
    return new Set(
      props.memberships
        .filter((item) => item.gameId === props.game.id)
        .map((item) => item.collectionId),
    );
  }, [props.memberships, props.game.id]);

  async function refreshSaveData() {
    setSaveLoading(true);
    try {
      const values = await Promise.all([
        api.getSaveConfig(props.game.id),
        api.listSaveBackups(props.game.id),
      ]);
      setSaveConfig(values[0]);
      setSaveBackups(values[1]);
    } catch (error) {
      props.onToast(readableError(error), "error");
    } finally {
      setSaveLoading(false);
    }
  }

  useEffect(() => {
    void refreshSaveData();
  }, [props.game.id]);

  async function configureSaveFolder() {
    setBusy(true);
    try {
      const config = await api.chooseSaveFolder(props.game.id);
      if (config) {
        setSaveConfig(config);
        props.onToast("Save folder configured.");
        await refreshSaveData();
      }
    } catch (error) {
      props.onToast(readableError(error), "error");
    } finally {
      setBusy(false);
    }
  }

  async function backupSaves() {
    setBusy(true);
    try {
      const backup = await api.createSaveBackup(props.game.id);
      props.onToast(
        "Save backup created: " +
          String(backup.fileCount) +
          " files · " +
          formatBytes(backup.totalBytes) +
          ".",
      );
      await refreshSaveData();
    } catch (error) {
      props.onToast(readableError(error), "error");
    } finally {
      setBusy(false);
    }
  }

  async function restoreBackup(backup: SaveBackupRecord) {
    const okay = window.confirm(
      "Restore this save backup from " +
        formatDate(backup.createdAt) +
        "? Dusk will create a safety backup of your current saves first.",
    );
    if (!okay) return;

    setBusy(true);
    try {
      const safety = await api.restoreSaveBackup(props.game.id, backup.id);
      props.onToast(
        "Save restored. Safety backup created with " +
          String(safety.fileCount) +
          " files.",
      );
      await refreshSaveData();
    } catch (error) {
      props.onToast(readableError(error), "error");
    } finally {
      setBusy(false);
    }
  }

  async function deleteBackup(backup: SaveBackupRecord) {
    const okay = window.confirm(
      "Delete this Dusk backup? Your current live save files will not be changed.",
    );
    if (!okay) return;

    setBusy(true);
    try {
      await api.deleteSaveBackup(props.game.id, backup.id);
      props.onToast("Save backup deleted.");
      await refreshSaveData();
    } catch (error) {
      props.onToast(readableError(error), "error");
    } finally {
      setBusy(false);
    }
  }

  async function clearSaveFolder() {
    const okay = window.confirm(
      "Stop managing this save folder? Existing Dusk backups will be kept.",
    );
    if (!okay) return;

    setBusy(true);
    try {
      await api.clearSaveConfig(props.game.id);
      props.onToast("Save folder disconnected.");
      await refreshSaveData();
    } catch (error) {
      props.onToast(readableError(error), "error");
    } finally {
      setBusy(false);
    }
  }

  async function run(action: () => Promise<unknown>, success?: string) {
    setBusy(true);
    try {
      await action();
      if (success) props.onToast(success);
      await props.onRefresh();
    } catch (error) {
      props.onToast(readableError(error), "error");
    } finally {
      setBusy(false);
    }
  }

  async function rename() {
    const title = window.prompt("Rename game", props.game.title);
    if (!title || !title.trim() || title.trim() === props.game.title) return;
    await run(() => api.renameGame(props.game.id, title.trim()), "Game renamed.");
  }

  async function remove() {
    const okay = window.confirm(
      'Remove "' + props.game.title + '" from Dusk? The game files will not be deleted.',
    );
    if (!okay) return;
    await run(() => api.removeGame(props.game.id), "Removed from Dusk.");
    props.onClose();
  }

  return (
    <div className="detail-backdrop" onMouseDown={props.onClose}>
      <aside className="detail-panel" onMouseDown={(event) => event.stopPropagation()}>
        <div className="detail-art">
          <Cover game={props.game} />
          <button className="icon-button detail-close" onClick={props.onClose}>
            <X size={19} />
          </button>
        </div>

        <div className="detail-body">
          <span className="source-pill">{sourceLabel(props.game.source)}</span>
          <h2>{props.game.title}</h2>
          <p className="detail-subtitle">
            {formatDuration(props.game.totalSeconds)} played · {props.game.launchCount} tracked{" "}
            {props.game.launchCount === 1 ? "session" : "sessions"}
          </p>

          <button className="button primary wide" onClick={() => props.onPlay(props.game)}>
            <Play size={17} fill="currentColor" />
            Play
          </button>

          <div className="detail-actions">
            <button
              className="detail-action"
              disabled={busy}
              onClick={() =>
                run(
                  () => api.setFavorite(props.game.id, !props.game.favorite),
                  props.game.favorite ? "Removed from favorites." : "Added to favorites.",
                )
              }
            >
              <Heart size={17} fill={props.game.favorite ? "currentColor" : "none"} />
              {props.game.favorite ? "Favorited" : "Favorite"}
            </button>
            <button
              className="detail-action"
              disabled={busy}
              onClick={() => run(() => api.openGameFolder(props.game.id))}
            >
              <FolderOpen size={17} />
              Folder
            </button>
            <button
              className="detail-action"
              disabled={busy}
              onClick={() =>
                run(async () => {
                  const changed = await api.chooseCover(props.game.id);
                  if (changed) props.onToast("Cover updated.");
                })
              }
            >
              <ImagePlus size={17} />
              Cover
            </button>
            <button
              className="detail-action"
              disabled={busy}
              onClick={() =>
                run(async () => {
                  const count = await api.importScreenshots(props.game.id);
                  props.onToast(
                    count === 0
                      ? "No screenshots imported."
                      : "Imported " + String(count) + " screenshot" + (count === 1 ? "." : "s."),
                  );
                })
              }
            >
              <Images size={17} />
              Shots
            </button>
          </div>

          <section className="detail-section">
            <h3>Collections</h3>
            {props.collections.length === 0 ? (
              <p className="muted">Create a collection from the sidebar first.</p>
            ) : (
              <div className="collection-checks">
                {props.collections.map((collection) => {
                  const included = membershipSet.has(collection.id);
                  return (
                    <button
                      key={collection.id}
                      disabled={busy}
                      className={cx("collection-check", included && "included")}
                      onClick={() =>
                        run(() =>
                          api.setCollectionMembership(
                            collection.id,
                            props.game.id,
                            !included,
                          ),
                        )
                      }
                    >
                      <span>{collection.name}</span>
                      {included && <Check size={16} />}
                    </button>
                  );
                })}
              </div>
            )}
          </section>

          <section className="detail-section">
            <div className="detail-section-heading">
              <h3>Save backups</h3>
              {saveConfig && (
                <button className="text-action" disabled={busy} onClick={() => void backupSaves()}>
                  <Save size={14} />
                  Back up now
                </button>
              )}
            </div>

            {saveLoading ? (
              <p className="muted">Loading save backup settings…</p>
            ) : !saveConfig ? (
              <div className="save-empty">
                <Archive size={18} />
                <div>
                  <strong>No save folder configured</strong>
                  <p>
                    Select the exact folder this game uses for saves. Dusk will not guess or
                    modify save locations automatically.
                  </p>
                </div>
                <button
                  className="button secondary"
                  disabled={busy}
                  onClick={() => void configureSaveFolder()}
                >
                  Choose folder
                </button>
              </div>
            ) : (
              <div className="save-manager">
                <div className="save-config-row">
                  <div>
                    <span>Managed save folder</span>
                    <code title={saveConfig.savePath}>{saveConfig.savePath}</code>
                  </div>
                  <div className="save-config-actions">
                    <button
                      className="text-action"
                      disabled={busy}
                      onClick={() => void configureSaveFolder()}
                    >
                      <FolderOpen size={14} />
                      Change
                    </button>
                    <button
                      className="text-action danger"
                      disabled={busy}
                      onClick={() => void clearSaveFolder()}
                    >
                      <X size={14} />
                      Disconnect
                    </button>
                  </div>
                </div>

                {saveBackups.length === 0 ? (
                  <p className="muted">
                    No restore points yet. Create one before changing mods, settings, or game files.
                  </p>
                ) : (
                  <div className="backup-list">
                    {saveBackups.map((backup) => (
                      <div className="backup-row" key={backup.id}>
                        <div className="backup-copy">
                          <strong>
                            {backup.kind === "pre-restore" ? "Safety backup" : "Manual backup"}
                          </strong>
                          <span>
                            {formatDate(backup.createdAt)} · {backup.fileCount} files ·{" "}
                            {formatBytes(backup.totalBytes)}
                          </span>
                        </div>
                        <div className="backup-actions">
                          <button
                            className="icon-button"
                            disabled={busy}
                            title="Restore this backup"
                            onClick={() => void restoreBackup(backup)}
                          >
                            <RotateCcw size={15} />
                          </button>
                          <button
                            className="icon-button"
                            disabled={busy}
                            title="Delete this backup"
                            onClick={() => void deleteBackup(backup)}
                          >
                            <Trash2 size={15} />
                          </button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}
          </section>

          <section className="detail-section">
            <h3>Details</h3>
            <dl className="detail-list">
              <div>
                <dt>Last played</dt>
                <dd>{formatDate(props.game.lastPlayed)}</dd>
              </div>
              <div>
                <dt>Installed at</dt>
                <dd title={props.game.installPath}>{props.game.installPath}</dd>
              </div>
              <div>
                <dt>Executable</dt>
                <dd title={props.game.exePath || ""}>
                  {props.game.exePath || "Not detected"}
                </dd>
              </div>
            </dl>
          </section>

          <div className="detail-footer">
            <button className="text-action" onClick={rename} disabled={busy}>
              <Pencil size={15} />
              Rename
            </button>
            <button className="text-action danger" onClick={remove} disabled={busy}>
              <Trash2 size={15} />
              Remove from Dusk
            </button>
          </div>
        </div>
      </aside>
    </div>
  );
}

export default function App() {
  const [view, setView] = useState<View>("home");
  const [games, setGames] = useState<GameRecord[]>([]);
  const [stats, setStats] = useState<Stats>(EMPTY_STATS);
  const [achievements, setAchievements] = useState<Achievement[]>([]);
  const [collections, setCollections] = useState<CollectionRecord[]>([]);
  const [memberships, setMemberships] = useState<CollectionMembership[]>([]);
  const [screenshots, setScreenshots] = useState<ScreenshotRecord[]>([]);
  const [dataDirectory, setDataDirectory] = useState("");

  const [selectedGameId, setSelectedGameId] = useState<string | null>(null);
  const [collectionFilter, setCollectionFilter] = useState("all");
  const [query, setQuery] = useState("");
  const [sourceFilter, setSourceFilter] = useState("all");
  const [sortMode, setSortMode] = useState<SortMode>("name");
  const [addOpen, setAddOpen] = useState(false);
  const [loading, setLoading] = useState(true);
  const [scanning, setScanning] = useState(false);
  const [screenshotScanning, setScreenshotScanning] = useState(false);
  const [playingId, setPlayingId] = useState<string | null>(null);
  const [toast, setToast] = useState<{ message: string; type: "ok" | "error" } | null>(null);
  const [updateBusy, setUpdateBusy] = useState(false);
  const [availableUpdate, setAvailableUpdate] = useState<string | null>(null);
  const [consoleMode, setConsoleMode] = useState(
    () => localStorage.getItem("dusk-console-mode") === "true",
  );

  const [theme, setTheme] = useState<ThemeName>(
    () => (localStorage.getItem("dusk-theme") as ThemeName) || "night",
  );
  const [accent, setAccent] = useState<AccentName>(
    () => (localStorage.getItem("dusk-accent") as AccentName) || "violet",
  );

  const showToast = useCallback((message: string, type: "ok" | "error" = "ok") => {
    setToast({ message, type });
    window.setTimeout(() => setToast(null), 3600);
  }, []);

  const refreshCore = useCallback(
    async (initial = false) => {
      if (initial) setLoading(true);
      try {
        const values = await Promise.all([
          api.listGames(),
          api.getStats(),
          api.listAchievements(),
          api.listCollections(),
          api.collectionMemberships(),
        ]);
        setGames(values[0]);
        setStats(values[1]);
        setAchievements(values[2]);
        setCollections(values[3]);
        setMemberships(values[4]);
      } catch (error) {
        showToast(readableError(error), "error");
      } finally {
        if (initial) setLoading(false);
      }
    },
    [showToast],
  );

  const refreshScreenshots = useCallback(async () => {
    try {
      setScreenshots(await api.listScreenshots());
    } catch (error) {
      showToast(readableError(error), "error");
    }
  }, [showToast]);

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      await refreshCore(true);
      try {
        const result = await api.scanScreenshots();
        if (!cancelled && result.imported > 0) {
          await Promise.all([refreshScreenshots(), refreshCore(false)]);
          showToast(
            "Dusk automatically found " +
              String(result.imported) +
              " new screenshot" +
              (result.imported === 1 ? "." : "s."),
          );
        }
      } catch {
        // Screenshot discovery is best-effort and must not block Dusk startup.
      }
    })();

    void api.dataDirectory().then(setDataDirectory).catch(() => undefined);

    return () => {
      cancelled = true;
    };
  }, [refreshCore, refreshScreenshots, showToast]);

  useEffect(() => {
    if (view === "screenshots") void refreshScreenshots();
  }, [view, refreshScreenshots]);

  useEffect(() => {
    const timer = window.setInterval(() => {
      void refreshCore(false);
    }, 30000);
    return () => window.clearInterval(timer);
  }, [refreshCore]);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    document.documentElement.dataset.accent = accent;
    localStorage.setItem("dusk-theme", theme);
    localStorage.setItem("dusk-accent", accent);
  }, [theme, accent]);

  async function setConsoleModeEnabled(enabled: boolean) {
    try {
      await getCurrentWindow().setFullscreen(enabled);
      setConsoleMode(enabled);
      localStorage.setItem("dusk-console-mode", String(enabled));

      if (enabled) {
        window.setTimeout(() => {
          const first = document.querySelector<HTMLElement>(
            '.game-card, .nav-item, button:not(:disabled)',
          );
          first?.focus();
        }, 180);
      }
    } catch (error) {
      showToast("Could not change fullscreen mode: " + readableError(error), "error");
    }
  }

  function controllerBack() {
    if (selectedGameId) {
      setSelectedGameId(null);
      return;
    }
    if (addOpen) {
      setAddOpen(false);
      return;
    }
    if (view !== "home") {
      setView("home");
      setCollectionFilter("all");
    }
  }

  function controllerPlay(gameId: string) {
    const game = games.find((item) => item.id === gameId);
    if (game) void play(game);
  }

  const { connected: controllerConnected } = useControllerNavigation({
    onBack: controllerBack,
    onToggleConsole: () => void setConsoleModeEnabled(!consoleMode),
    onPlayGame: controllerPlay,
  });

  useEffect(() => {
    if (!consoleMode) return;
    void getCurrentWindow().setFullscreen(true).catch(() => undefined);
  }, []);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.key === "F11") {
        event.preventDefault();
        void setConsoleModeEnabled(!consoleMode);
      } else if (event.key === "Escape" && consoleMode) {
        event.preventDefault();
        controllerBack();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [consoleMode, selectedGameId, addOpen, view, games]);


  const selectedGame = games.find((game) => game.id === selectedGameId) || null;

  const membershipLookup = useMemo(() => {
    const map = new Map<string, Set<string>>();
    for (const membership of memberships) {
      const set = map.get(membership.collectionId) || new Set<string>();
      set.add(membership.gameId);
      map.set(membership.collectionId, set);
    }
    return map;
  }, [memberships]);

  const visibleGames = useMemo(() => {
    let next = [...games];

    if (view === "favorites") next = next.filter((game) => game.favorite);

    if (collectionFilter !== "all") {
      const allowed = membershipLookup.get(collectionFilter) || new Set<string>();
      next = next.filter((game) => allowed.has(game.id));
    }

    if (sourceFilter !== "all") {
      next = next.filter((game) => game.source === sourceFilter);
    }

    const needle = query.trim().toLocaleLowerCase();
    if (needle) {
      next = next.filter((game) =>
        (game.title + " " + game.source).toLocaleLowerCase().includes(needle),
      );
    }

    next.sort((a, b) => {
      if (sortMode === "playtime") return b.totalSeconds - a.totalSeconds;
      if (sortMode === "recent") {
        return new Date(b.lastPlayed || 0).getTime() - new Date(a.lastPlayed || 0).getTime();
      }
      return a.title.localeCompare(b.title);
    });

    return next;
  }, [games, view, collectionFilter, membershipLookup, sourceFilter, query, sortMode]);

  const recentGames = useMemo(() => {
    return [...games]
      .filter((game) => Boolean(game.lastPlayed))
      .sort(
        (a, b) =>
          new Date(b.lastPlayed || 0).getTime() - new Date(a.lastPlayed || 0).getTime(),
      )
      .slice(0, 6);
  }, [games]);

  const favoriteGames = useMemo(() => games.filter((game) => game.favorite).slice(0, 6), [games]);

  async function scan() {
    setScanning(true);
    try {
      const result = await api.scanGames();
      const screenshotResult = await api.scanScreenshots();
      await Promise.all([refreshCore(false), refreshScreenshots()]);
      const detail =
        String(result.added) +
        " new · " +
        String(result.updated) +
        " refreshed · " +
        String(result.steamFound) +
        " Steam · " +
        String(result.epicFound) +
        " Epic · " +
        String(result.gogFound) +
        " GOG · " +
        String(result.emulatorFound) +
        " emulators" +
        (screenshotResult.imported > 0
          ? " · " + String(screenshotResult.imported) + " screenshots"
          : "");
      showToast("Scan complete: " + String(result.found) + " found (" + detail + ").");
      if (result.warnings.length > 0 && result.found === 0) {
        window.setTimeout(() => showToast(result.warnings[0], "error"), 500);
      }
    } catch (error) {
      showToast(readableError(error), "error");
    } finally {
      setScanning(false);
    }
  }

  async function scanScreenshotsNow() {
    setScreenshotScanning(true);
    try {
      const result = await api.scanScreenshots();
      await Promise.all([refreshScreenshots(), refreshCore(false)]);
      if (result.imported > 0) {
        showToast(
          "Found " +
            String(result.imported) +
            " new screenshot" +
            (result.imported === 1 ? "." : "s.") +
            " Steam: " +
            String(result.steamImported) +
            " · matched folders: " +
            String(result.matchedImported) +
            ".",
        );
      } else if (result.found > 0) {
        showToast("Screenshot scan complete. No new screenshots; existing files were already known.");
      } else {
        showToast("Screenshot scan complete. No matching screenshots were found.");
      }
    } catch (error) {
      showToast(readableError(error), "error");
    } finally {
      setScreenshotScanning(false);
    }
  }

  async function play(game: GameRecord) {
    setPlayingId(game.id);
    try {
      const result = await api.launchGame(game.id);
      showToast(result.message, result.started ? "ok" : "error");
    } catch (error) {
      showToast(readableError(error), "error");
    } finally {
      setPlayingId(null);
    }
  }

  async function toggleFavorite(game: GameRecord) {
    try {
      await api.setFavorite(game.id, !game.favorite);
      await refreshCore(false);
    } catch (error) {
      showToast(readableError(error), "error");
    }
  }

  async function createCollection() {
    const name = window.prompt("Collection name");
    if (!name || !name.trim()) return;
    try {
      const collection = await api.createCollection(name.trim());
      await refreshCore(false);
      setCollectionFilter(collection.id);
      setView("library");
      showToast("Created " + collection.name + ".");
    } catch (error) {
      showToast(readableError(error), "error");
    }
  }

  async function deleteCollection(collection: CollectionRecord) {
    const okay = window.confirm(
      'Delete the "' + collection.name + '" collection? Games stay in your library.',
    );
    if (!okay) return;
    try {
      await api.deleteCollection(collection.id);
      if (collectionFilter === collection.id) setCollectionFilter("all");
      await refreshCore(false);
      showToast("Collection deleted.");
    } catch (error) {
      showToast(readableError(error), "error");
    }
  }

  async function checkUpdates() {
    setUpdateBusy(true);
    try {
      const result = await checkForDuskUpdate();
      if (result.available && result.version) {
        setAvailableUpdate(result.version);
        showToast("Dusk " + result.version + " is available.");
      } else {
        setAvailableUpdate(null);
        showToast("Dusk is up to date.");
      }
    } catch (error) {
      showToast(
        "Updater is not configured for this build yet: " + readableError(error),
        "error",
      );
    } finally {
      setUpdateBusy(false);
    }
  }

  async function installUpdate() {
    setUpdateBusy(true);
    try {
      await installDuskUpdate();
    } catch (error) {
      showToast(readableError(error), "error");
      setUpdateBusy(false);
    }
  }

  async function deleteScreenshot(screenshot: ScreenshotRecord) {
    if (!window.confirm("Delete this imported screenshot from Dusk?")) return;
    try {
      await api.deleteScreenshot(screenshot.id);
      await Promise.all([refreshScreenshots(), refreshCore(false)]);
      showToast("Screenshot deleted.");
    } catch (error) {
      showToast(readableError(error), "error");
    }
  }

  const activeCollection = collections.find((item) => item.id === collectionFilter);
  const libraryTitle =
    view === "favorites"
      ? "Favorites"
      : collectionFilter !== "all"
        ? activeCollection?.name || "Collection"
        : "Library";

  return (
    <div className={cx("app-shell", consoleMode && "console-mode", controllerConnected && "controller-connected")}>
      <aside className="sidebar">
        <div className="brand">
          <div className="brand-mark">
            <Moon size={19} fill="currentColor" />
          </div>
          <div>
            <strong>Dusk</strong>
            <span>Game launcher</span>
          </div>
        </div>

        <nav className="nav-list">
          <NavItem
            active={view === "home"}
            icon={<Home size={18} />}
            label="Home"
            onClick={() => {
              setView("home");
              setCollectionFilter("all");
            }}
          />
          <NavItem
            active={view === "library" && collectionFilter === "all"}
            icon={<Library size={18} />}
            label="Library"
            count={stats.gameCount}
            onClick={() => {
              setView("library");
              setCollectionFilter("all");
            }}
          />
          <NavItem
            active={view === "favorites"}
            icon={<Heart size={18} />}
            label="Favorites"
            count={stats.favoriteCount}
            onClick={() => {
              setView("favorites");
              setCollectionFilter("all");
            }}
          />
          <NavItem
            active={view === "screenshots"}
            icon={<Images size={18} />}
            label="Screenshots"
            count={stats.screenshotCount}
            onClick={() => setView("screenshots")}
          />
          <NavItem
            active={view === "achievements"}
            icon={<Trophy size={18} />}
            label="Achievements"
            onClick={() => setView("achievements")}
          />
        </nav>

        <div className="sidebar-divider" />

        <div className="sidebar-section-head">
          <span>Collections</span>
          <button className="mini-icon" onClick={createCollection} title="New collection">
            <Plus size={15} />
          </button>
        </div>

        <div className="collection-nav">
          {collections.length === 0 ? (
            <p className="sidebar-empty">No collections yet.</p>
          ) : (
            collections.map((collection) => (
              <div className="collection-row" key={collection.id}>
                <button
                  className={cx(
                    "collection-nav-item",
                    view === "library" && collectionFilter === collection.id && "active",
                  )}
                  onClick={() => {
                    setCollectionFilter(collection.id);
                    setView("library");
                  }}
                >
                  <Layers3 size={15} />
                  <span>{collection.name}</span>
                  <small>{collection.gameCount}</small>
                </button>
                <button
                  className="collection-delete"
                  onClick={() => void deleteCollection(collection)}
                  title={"Delete " + collection.name}
                >
                  <X size={13} />
                </button>
              </div>
            ))
          )}
        </div>

        <div className="sidebar-bottom">
          <NavItem
            active={view === "settings"}
            icon={<Settings size={18} />}
            label="Settings"
            onClick={() => setView("settings")}
          />
          <div className="local-chip">
            <HardDrive size={14} />
            <span>Local-first</span>
          </div>
        </div>
      </aside>

      <main className="main">
        <header className="topbar">
          <div className="search-box">
            <Search size={17} />
            <input
              value={query}
              onChange={(event) => setQuery(event.target.value)}
              onFocus={() => {
                if (view === "home") setView("library");
              }}
              placeholder="Search your games"
            />
            {query && (
              <button onClick={() => setQuery("")} aria-label="Clear search">
                <X size={15} />
              </button>
            )}
          </div>

          <div className="top-actions">
            <button className="button secondary" onClick={() => setAddOpen(true)}>
              <Plus size={16} />
              Add game
            </button>
            <button className="button primary" onClick={() => void scan()} disabled={scanning}>
              {scanning ? <RefreshCw className="spin" size={16} /> : <ScanSearch size={16} />}
              {scanning ? "Scanning…" : "Scan PC"}
            </button>
          </div>
        </header>

        <div className="content">
          {loading ? (
            <div className="loading-screen">
              <div className="brand-mark">
                <Moon size={23} fill="currentColor" />
              </div>
              <RefreshCw className="spin" size={20} />
              <span>Loading your local library…</span>
            </div>
          ) : (
            <>
              {view === "home" && (
                <div className="page">
                  <section className="hero">
                    <div>
                      <span className="eyebrow">Local library</span>
                      <h1>Your games. One place.</h1>
                      <p>
                        Dusk scans installed launchers, starts games locally, and records only
                        the sessions it can actually observe.
                      </p>
                      <div className="hero-actions">
                        <button className="button primary" onClick={() => void scan()} disabled={scanning}>
                          <ScanSearch size={16} />
                          Scan this PC
                        </button>
                        <button className="button secondary" onClick={() => setAddOpen(true)}>
                          <Plus size={16} />
                          Add manually
                        </button>
                      </div>
                    </div>
                    <div className="hero-stat">
                      <span>Total playtime</span>
                      <strong>{formatDuration(stats.totalSeconds)}</strong>
                      <small>
                        {stats.topGame ? "Most played: " + stats.topGame : "No tracked sessions yet"}
                      </small>
                    </div>
                  </section>

                  <section className="stat-grid">
                    <div className="stat-card">
                      <Library size={18} />
                      <span>Games</span>
                      <strong>{stats.gameCount}</strong>
                    </div>
                    <div className="stat-card">
                      <Clock3 size={18} />
                      <span>Last 7 days</span>
                      <strong>{formatDuration(stats.last7DaysSeconds)}</strong>
                    </div>
                    <div className="stat-card">
                      <Play size={18} />
                      <span>Tracked sessions</span>
                      <strong>{stats.launchCount}</strong>
                    </div>
                    <div className="stat-card">
                      <Images size={18} />
                      <span>Screenshots</span>
                      <strong>{stats.screenshotCount}</strong>
                    </div>
                  </section>

                  <section className="content-section">
                    <div className="section-heading">
                      <div>
                        <span className="eyebrow">Continue</span>
                        <h2>Recently played</h2>
                      </div>
                      <button
                        className="text-action"
                        onClick={() => {
                          setSortMode("recent");
                          setView("library");
                        }}
                      >
                        See library
                      </button>
                    </div>

                    {recentGames.length === 0 ? (
                      <EmptyState
                        icon={<Gamepad2 size={24} />}
                        title="No tracked sessions yet"
                        copy="Scan your PC or add a game manually, then launch it through Dusk."
                        action={
                          <button className="button primary" onClick={() => void scan()}>
                            Scan for games
                          </button>
                        }
                      />
                    ) : (
                      <div className="game-grid">
                        {recentGames.map((game) => (
                          <GameCard
                            key={game.id}
                            game={game}
                            playing={playingId === game.id}
                            onOpen={() => setSelectedGameId(game.id)}
                            onPlay={() => void play(game)}
                            onFavorite={() => void toggleFavorite(game)}
                          />
                        ))}
                      </div>
                    )}
                  </section>

                  {favoriteGames.length > 0 && (
                    <section className="content-section">
                      <div className="section-heading">
                        <div>
                          <span className="eyebrow">Pinned</span>
                          <h2>Favorites</h2>
                        </div>
                      </div>
                      <div className="game-grid">
                        {favoriteGames.map((game) => (
                          <GameCard
                            key={game.id}
                            game={game}
                            playing={playingId === game.id}
                            onOpen={() => setSelectedGameId(game.id)}
                            onPlay={() => void play(game)}
                            onFavorite={() => void toggleFavorite(game)}
                          />
                        ))}
                      </div>
                    </section>
                  )}
                </div>
              )}

              {(view === "library" || view === "favorites") && (
                <div className="page">
                  <div className="page-heading">
                    <div>
                      <span className="eyebrow">
                        {collectionFilter === "all" ? "Games" : "Collection"}
                      </span>
                      <h1>{libraryTitle}</h1>
                      <p>
                        {visibleGames.length} {visibleGames.length === 1 ? "game" : "games"} shown
                      </p>
                    </div>
                    <div className="library-controls">
                      <select
                        value={sourceFilter}
                        onChange={(event) => setSourceFilter(event.target.value)}
                        aria-label="Filter by source"
                      >
                        <option value="all">All sources</option>
                        <option value="steam">Steam</option>
                        <option value="epic">Epic</option>
                        <option value="gog">GOG</option>
                        <option value="emulator">Emulators</option>
                        <option value="manual">Manual</option>
                      </select>
                      <select
                        value={sortMode}
                        onChange={(event) => setSortMode(event.target.value as SortMode)}
                        aria-label="Sort games"
                      >
                        <option value="name">Name</option>
                        <option value="recent">Recently played</option>
                        <option value="playtime">Playtime</option>
                      </select>
                    </div>
                  </div>

                  {visibleGames.length === 0 ? (
                    <EmptyState
                      icon={view === "favorites" ? <Heart size={25} /> : <Library size={25} />}
                      title={games.length === 0 ? "Your library is empty" : "No games match these filters"}
                      copy={
                        games.length === 0
                          ? "Dusk can detect Steam, Epic, GOG, and common emulators, or you can point it at any game executable."
                          : "Change the search, source, or collection filter."
                      }
                      action={
                        games.length === 0 ? (
                          <div className="empty-actions">
                            <button className="button primary" onClick={() => void scan()}>
                              <ScanSearch size={16} />
                              Scan PC
                            </button>
                            <button className="button secondary" onClick={() => setAddOpen(true)}>
                              <Plus size={16} />
                              Add game
                            </button>
                          </div>
                        ) : undefined
                      }
                    />
                  ) : (
                    <div className="game-grid large">
                      {visibleGames.map((game) => (
                        <GameCard
                          key={game.id}
                          game={game}
                          playing={playingId === game.id}
                          onOpen={() => setSelectedGameId(game.id)}
                          onPlay={() => void play(game)}
                          onFavorite={() => void toggleFavorite(game)}
                        />
                      ))}
                    </div>
                  )}
                </div>
              )}

              {view === "screenshots" && (
                <div className="page">
                  <div className="page-heading">
                    <div>
                      <span className="eyebrow">Local gallery</span>
                      <h1>Screenshots</h1>
                      <p>Automatically detected and manually imported local screenshots.</p>
                    </div>
                    <div className="top-actions">
                      <button
                        className="button primary"
                        disabled={screenshotScanning}
                        onClick={() => void scanScreenshotsNow()}
                      >
                        {screenshotScanning ? (
                          <RefreshCw className="spin" size={16} />
                        ) : (
                          <ScanSearch size={16} />
                        )}
                        {screenshotScanning ? "Scanning…" : "Scan screenshots"}
                      </button>
                      <button className="button secondary" onClick={() => void refreshScreenshots()}>
                        <RefreshCw size={16} />
                        Refresh
                      </button>
                    </div>
                  </div>

                  {screenshots.length === 0 ? (
                    <EmptyState
                      icon={<Images size={25} />}
                      title="No screenshots found"
                      copy="Dusk checks Steam screenshot folders, common per-game screenshot folders, Windows Screenshots, and Xbox Game Bar Captures. You can also import images manually from a game."
                    />
                  ) : (
                    <div className="screenshot-grid">
                      {screenshots.map((screenshot) => {
                        const game = games.find((item) => item.id === screenshot.gameId);
                        return (
                          <article className="screenshot-card" key={screenshot.id}>
                            {screenshot.dataUrl ? (
                              <img src={screenshot.dataUrl} alt="" />
                            ) : (
                              <div className="image-unavailable">Preview unavailable</div>
                            )}
                            <div className="screenshot-meta">
                              <div>
                                <strong>{game?.title || "Unknown game"}</strong>
                                <span>{formatDate(screenshot.createdAt)}</span>
                              </div>
                              <button
                                className="icon-button"
                                onClick={() => void deleteScreenshot(screenshot)}
                                title="Delete imported screenshot"
                              >
                                <Trash2 size={16} />
                              </button>
                            </div>
                          </article>
                        );
                      })}
                    </div>
                  )}
                </div>
              )}

              {view === "achievements" && (
                <div className="page">
                  <div className="page-heading">
                    <div>
                      <span className="eyebrow">Dusk milestones</span>
                      <h1>Achievements</h1>
                      <p>
                        Launcher milestones calculated from real Dusk library and session data,
                        not achievements imported from game platforms.
                      </p>
                    </div>
                  </div>

                  <div className="achievement-grid">
                    {achievements.map((achievement) => {
                      const percent = Math.min(
                        100,
                        Math.round((achievement.current / achievement.target) * 100),
                      );
                      return (
                        <article
                          key={achievement.id}
                          className={cx("achievement-card", achievement.unlocked && "unlocked")}
                        >
                          <div className="achievement-icon">
                            {achievement.unlocked ? <Trophy size={21} /> : <Sparkles size={21} />}
                          </div>
                          <div>
                            <div className="achievement-title">
                              <h3>{achievement.title}</h3>
                              {achievement.unlocked && <span>Unlocked</span>}
                            </div>
                            <p>{achievement.description}</p>
                            <div className="progress-track">
                              <div style={{ width: String(percent) + "%" }} />
                            </div>
                            <small>
                              {achievement.current} / {achievement.target}
                            </small>
                          </div>
                        </article>
                      );
                    })}
                  </div>
                </div>
              )}

              {view === "settings" && (
                <div className="page settings-page">
                  <div className="page-heading">
                    <div>
                      <span className="eyebrow">Dusk</span>
                      <h1>Settings</h1>
                      <p>Appearance and local data information.</p>
                    </div>
                  </div>

                  <section className="settings-card">
                    <div className="settings-card-head">
                      <Gamepad2 size={20} />
                      <div>
                        <h3>Console mode</h3>
                        <p>Fullscreen layout designed for controller navigation.</p>
                      </div>
                    </div>

                    <div className="setting-row">
                      <div>
                        <strong>{controllerConnected ? "Controller connected" : "Controller navigation"}</strong>
                        <span>
                          D-pad or left stick moves focus · A selects · X launches the focused game ·
                          B goes back · Menu toggles fullscreen.
                        </span>
                      </div>
                      <button
                        className={cx("button", consoleMode ? "primary" : "secondary")}
                        onClick={() => void setConsoleModeEnabled(!consoleMode)}
                      >
                        <Gamepad2 size={16} />
                        {consoleMode ? "Exit console mode" : "Enter console mode"}
                      </button>
                    </div>
                  </section>

                  <section className="settings-card">
                    <div className="settings-card-head">
                      <Moon size={20} />
                      <div>
                        <h3>Appearance</h3>
                        <p>Stored locally in this app.</p>
                      </div>
                    </div>

                    <div className="setting-row">
                      <div>
                        <strong>Theme</strong>
                        <span>Choose the base surface style.</span>
                      </div>
                      <div className="segmented">
                        {(["night", "oled", "slate"] as ThemeName[]).map((item) => (
                          <button
                            key={item}
                            className={theme === item ? "active" : ""}
                            onClick={() => setTheme(item)}
                          >
                            {item}
                          </button>
                        ))}
                      </div>
                    </div>

                    <div className="setting-row">
                      <div>
                        <strong>Accent</strong>
                        <span>Used for primary actions and highlights.</span>
                      </div>
                      <div className="accent-options">
                        {(["violet", "ember", "cyan"] as AccentName[]).map((item) => (
                          <button
                            key={item}
                            className={cx("accent-swatch", item, accent === item && "active")}
                            onClick={() => setAccent(item)}
                            aria-label={item + " accent"}
                          >
                            {accent === item && <Check size={14} />}
                          </button>
                        ))}
                      </div>
                    </div>
                  </section>

                  <section className="settings-card">
                    <div className="settings-card-head">
                      <RefreshCw size={20} />
                      <div>
                        <h3>Updates</h3>
                        <p>Secure updates from official Dusk GitHub Releases.</p>
                      </div>
                    </div>
                    <div className="setting-row">
                      <div>
                        <strong>
                          {availableUpdate ? "Dusk " + availableUpdate + " available" : "Automatic updater"}
                        </strong>
                        <span>
                          {availableUpdate
                            ? "Download and install the new version."
                            : "Check for a newer signed Dusk release."}
                        </span>
                      </div>
                      {availableUpdate ? (
                        <button
                          className="button primary"
                          disabled={updateBusy}
                          onClick={() => void installUpdate()}
                        >
                          {updateBusy ? "Updating…" : "Install update"}
                        </button>
                      ) : (
                        <button
                          className="button secondary"
                          disabled={updateBusy}
                          onClick={() => void checkUpdates()}
                        >
                          {updateBusy ? "Checking…" : "Check for updates"}
                        </button>
                      )}
                    </div>
                  </section>

                  <section className="settings-card">
                    <div className="settings-card-head">
                      <Database size={20} />
                      <div>
                        <h3>Local data</h3>
                        <p>Dusk does not require a cloud account.</p>
                      </div>
                    </div>
                    <div className="data-path">
                      <span>Application data directory</span>
                      <code>{dataDirectory || "Unavailable"}</code>
                    </div>
                    <div className="settings-note">
                      <HardDrive size={17} />
                      <p>
                        Playtime, collections, imported screenshot references, and Dusk settings
                        stay on this PC. Dusk only records a session when it can observe the launched
                        process.
                      </p>
                    </div>
                  </section>

                  <section className="settings-card">
                    <div className="settings-card-head">
                      <ScanSearch size={20} />
                      <div>
                        <h3>Game detection</h3>
                        <p>Current automatic scanners.</p>
                      </div>
                    </div>
                    <div className="scanner-list">
                      <div>
                        <Check size={16} />
                        <strong>Steam</strong>
                        <span>Reads local app manifests and library folders.</span>
                      </div>
                      <div>
                        <Check size={16} />
                        <strong>Epic Games</strong>
                        <span>Reads installed Epic manifest files.</span>
                      </div>
                      <div>
                        <Check size={16} />
                        <strong>GOG</strong>
                        <span>Reads GOG game install records from the Windows registry.</span>
                      </div>
                      <div>
                        <Check size={16} />
                        <strong>Emulators</strong>
                        <span>Detects common installed emulators such as Dolphin, PCSX2, RetroArch, Ryujinx, Cemu, PPSSPP, and DuckStation.</span>
                      </div>
                      <div>
                        <Plus size={16} />
                        <strong>Manual games</strong>
                        <span>Any local Windows executable can be added explicitly.</span>
                      </div>
                    </div>
                  </section>

                  <section className="about-card">
                    <div className="brand-mark">
                      <Moon size={20} fill="currentColor" />
                    </div>
                    <div>
                      <strong>Dusk 0.3.0</strong>
                      <span>Open code · github.com/bxane-dev/dusk</span>
                    </div>
                  </section>
                </div>
              )}
            </>
          )}
        </div>
      </main>

      {addOpen && (
        <AddGameModal
          onClose={() => setAddOpen(false)}
          onAdded={() => refreshCore(false)}
        />
      )}

      {selectedGame && (
        <GameDetail
          game={selectedGame}
          collections={collections}
          memberships={memberships}
          onClose={() => setSelectedGameId(null)}
          onPlay={play}
          onRefresh={() => refreshCore(false)}
          onToast={showToast}
        />
      )}

      {controllerConnected && (
        <div className="controller-bar" aria-live="polite">
          <span><kbd>A</kbd> Select</span>
          <span><kbd>X</kbd> Play</span>
          <span><kbd>B</kbd> Back</span>
          <span><kbd>Menu</kbd> {consoleMode ? "Exit fullscreen" : "Fullscreen"}</span>
        </div>
      )}

      {toast && <div className={cx("toast", toast.type)}>{toast.message}</div>}
    </div>
  );
}

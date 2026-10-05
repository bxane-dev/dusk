use chrono::{DateTime, Utc};
use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{
    collections::{hash_map::DefaultHasher, HashMap, HashSet},
    env,
    fs,
    path::{Path, PathBuf},
    hash::{Hash, Hasher},
    process::Command,
    sync::OnceLock,
    thread,
    time::{Duration, Instant},
};
use tauri::{AppHandle, Manager};
use uuid::Uuid;
use walkdir::WalkDir;

static SCHEMA_STATE: OnceLock<Result<(), String>> = OnceLock::new();

#[derive(Debug, Clone)]
struct DiscoveredGame {
    id: String,
    title: String,
    exe_path: Option<String>,
    install_path: String,
    source: String,
    source_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GameRecord {
    id: String,
    title: String,
    exe_path: Option<String>,
    install_path: String,
    source: String,
    source_id: Option<String>,
    favorite: bool,
    cover_path: Option<String>,
    cover_data_url: Option<String>,
    added_at: String,
    last_played: Option<String>,
    total_seconds: i64,
    launch_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScanResult {
    found: usize,
    added: usize,
    updated: usize,
    steam_found: usize,
    epic_found: usize,
    gog_found: usize,
    emulator_found: usize,
    device_found: usize,
    warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LaunchResult {
    started: bool,
    tracking: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stats {
    game_count: i64,
    favorite_count: i64,
    played_game_count: i64,
    total_seconds: i64,
    launch_count: i64,
    last_7_days_seconds: i64,
    screenshot_count: i64,
    top_game: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Achievement {
    id: String,
    title: String,
    description: String,
    unlocked: bool,
    current: i64,
    target: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenshotRecord {
    id: i64,
    game_id: String,
    path: String,
    data_url: Option<String>,
    created_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AutoScreenshotScanResult {
    found: usize,
    imported: usize,
    steam_imported: usize,
    matched_imported: usize,
    skipped_duplicates: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CollectionRecord {
    id: String,
    name: String,
    game_count: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CollectionMembership {
    collection_id: String,
    game_id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveConfig {
    game_id: String,
    save_path: String,
    configured_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveBackupRecord {
    id: String,
    game_id: String,
    backup_path: String,
    created_at: String,
    file_count: i64,
    total_bytes: i64,
    kind: String,
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Could not resolve Dusk data directory: {error}"))?;
    fs::create_dir_all(&dir).map_err(|error| format!("Could not create data directory: {error}"))?;
    Ok(dir)
}

fn database_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("dusk.db"))
}

fn open_database(app: &AppHandle) -> Result<Connection, String> {
    let path = database_path(app)?;
    let connection =
        Connection::open(path).map_err(|error| format!("Could not open Dusk database: {error}"))?;

    connection
        .busy_timeout(Duration::from_secs(2))
        .map_err(|error| format!("Could not configure Dusk database timeout: {error}"))?;
    connection
        .execute_batch("PRAGMA foreign_keys = ON;")
        .map_err(|error| format!("Could not configure Dusk database: {error}"))?;

    let schema = SCHEMA_STATE.get_or_init(|| initialize_schema(&connection));
    if let Err(error) = schema {
        return Err(error.clone());
    }

    Ok(connection)
}

fn initialize_schema(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            PRAGMA journal_mode = WAL;

            CREATE TABLE IF NOT EXISTS games (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                exe_path TEXT,
                install_path TEXT NOT NULL,
                source TEXT NOT NULL,
                source_id TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                hidden INTEGER NOT NULL DEFAULT 0,
                cover_path TEXT,
                added_at TEXT NOT NULL,
                last_played TEXT,
                total_seconds INTEGER NOT NULL DEFAULT 0,
                launch_count INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT NOT NULL,
                duration_seconds INTEGER NOT NULL,
                FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS screenshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id TEXT NOT NULL,
                path TEXT NOT NULL UNIQUE,
                source_path TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE COLLATE NOCASE,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS collection_games (
                collection_id TEXT NOT NULL,
                game_id TEXT NOT NULL,
                PRIMARY KEY(collection_id, game_id),
                FOREIGN KEY(collection_id) REFERENCES collections(id) ON DELETE CASCADE,
                FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS save_configs (
                game_id TEXT PRIMARY KEY,
                save_path TEXT NOT NULL,
                configured_at TEXT NOT NULL,
                FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
            );

            CREATE TABLE IF NOT EXISTS save_backups (
                id TEXT PRIMARY KEY,
                game_id TEXT NOT NULL,
                backup_path TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL,
                file_count INTEGER NOT NULL,
                total_bytes INTEGER NOT NULL,
                kind TEXT NOT NULL DEFAULT 'manual',
                FOREIGN KEY(game_id) REFERENCES games(id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_save_backups_game_created
            ON save_backups(game_id, created_at DESC);

            CREATE INDEX IF NOT EXISTS idx_games_last_played ON games(last_played);
            CREATE INDEX IF NOT EXISTS idx_sessions_game_id ON sessions(game_id);
            CREATE INDEX IF NOT EXISTS idx_screenshots_game_id ON screenshots(game_id);
            "#,
        )
        .map_err(|error| format!("Could not initialize Dusk database: {error}"))?;

    // Existing Dusk databases predate automatic screenshot source tracking.
    let _ = connection.execute("ALTER TABLE screenshots ADD COLUMN source_path TEXT", []);
    connection
        .execute_batch(
            r#"
            CREATE UNIQUE INDEX IF NOT EXISTS idx_screenshots_source_path
            ON screenshots(source_path)
            WHERE source_path IS NOT NULL;

            CREATE TABLE IF NOT EXISTS ignored_screenshot_sources (
                source_path TEXT PRIMARY KEY,
                ignored_at TEXT NOT NULL
            );
            "#,
        )
        .map_err(|error| format!("Could not initialize screenshot tracking: {error}"))
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn row_to_game(row: &rusqlite::Row<'_>) -> rusqlite::Result<GameRecord> {
    let cover_path: Option<String> = row.get(7)?;

    Ok(GameRecord {
        id: row.get(0)?,
        title: row.get(1)?,
        exe_path: row.get(2)?,
        install_path: row.get(3)?,
        source: row.get(4)?,
        source_id: row.get(5)?,
        favorite: row.get::<_, i64>(6)? != 0,
        cover_path,
        cover_data_url: None,
        added_at: row.get(8)?,
        last_played: row.get(9)?,
        total_seconds: row.get(10)?,
        launch_count: row.get(11)?,
    })
}

fn get_game(connection: &Connection, id: &str) -> Result<GameRecord, String> {
    connection
        .query_row(
            r#"
            SELECT id, title, exe_path, install_path, source, source_id, favorite,
                   cover_path, added_at, last_played, total_seconds, launch_count
            FROM games
            WHERE id = ?1 AND hidden = 0
            "#,
            params![id],
            row_to_game,
        )
        .optional()
        .map_err(|error| format!("Could not read game: {error}"))?
        .ok_or_else(|| "Game not found.".to_string())
}

fn upsert_discovered(connection: &Connection, game: &DiscoveredGame) -> Result<bool, String> {
    let existed: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM games WHERE id = ?1)",
            params![game.id],
            |row| row.get(0),
        )
        .map_err(|error| format!("Could not check game: {error}"))?;

    connection
        .execute(
            r#"
            INSERT INTO games (
                id, title, exe_path, install_path, source, source_id, added_at, hidden
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0)
            ON CONFLICT(id) DO UPDATE SET
                title = excluded.title,
                exe_path = COALESCE(excluded.exe_path, games.exe_path),
                install_path = excluded.install_path,
                source = excluded.source,
                source_id = excluded.source_id,
                hidden = 0
            "#,
            params![
                game.id,
                game.title,
                game.exe_path,
                game.install_path,
                game.source,
                game.source_id,
                now()
            ],
        )
        .map_err(|error| format!("Could not save discovered game: {error}"))?;

    Ok(existed)
}

fn quoted_vdf_value(contents: &str, key: &str) -> Option<String> {
    let pattern = format!(r#""{}"\s+"([^"]+)""#, regex::escape(key));
    Regex::new(&pattern)
        .ok()?
        .captures(contents)?
        .get(1)
        .map(|value| value.as_str().replace(r"\\", r"\"))
}

fn find_best_executable(root: &Path, game_name: &str) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }

    let normalized_name: String = game_name
        .to_ascii_lowercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect();

    let mut candidates: Vec<(i64, PathBuf)> = Vec::new();

    for entry in WalkDir::new(root)
        .max_depth(5)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()).map(|value| value.eq_ignore_ascii_case("exe")) != Some(true) {
            continue;
        }

        let filename = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let blocked = [
            "unins",
            "uninstall",
            "crash",
            "report",
            "vc_redist",
            "vcredist",
            "dxsetup",
            "setup",
            "unitycrashhandler",
            "dotnet",
        ];
        if blocked.iter().any(|word| filename.contains(word)) {
            continue;
        }

        let normalized_file: String = filename
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect();

        let mut score = 100_i64 - entry.depth() as i64 * 8;
        if !normalized_name.is_empty()
            && (normalized_file.contains(&normalized_name)
                || normalized_name.contains(&normalized_file))
        {
            score += 150;
        }
        if filename.contains("launcher") {
            score -= 20;
        }
        if filename.contains("win64") || filename.contains("x64") {
            score += 10;
        }

        candidates.push((score, path.to_path_buf()));
    }

    candidates
        .into_iter()
        .max_by_key(|(score, _)| *score)
        .map(|(_, path)| path)
}


fn find_best_executable_bounded(
    root: &Path,
    game_name: &str,
    started: Instant,
    budget: Duration,
) -> Option<PathBuf> {
    if !root.exists() {
        return None;
    }

    let normalized_name: String = game_name
        .to_ascii_lowercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect();

    let mut best: Option<(i64, PathBuf)> = None;

    for entry in WalkDir::new(root)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .take(2000)
    {
        if started.elapsed() >= budget {
            break;
        }
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        if path
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| value.eq_ignore_ascii_case("exe"))
            != Some(true)
        {
            continue;
        }

        let filename = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let blocked = [
            "unins",
            "uninstall",
            "crash",
            "report",
            "vc_redist",
            "vcredist",
            "dxsetup",
            "setup",
            "unitycrashhandler",
            "dotnet",
        ];
        if blocked.iter().any(|word| filename.contains(word)) {
            continue;
        }

        let normalized_file: String = filename
            .chars()
            .filter(|character| character.is_ascii_alphanumeric())
            .collect();

        let mut score = 100_i64 - entry.depth() as i64 * 8;
        if !normalized_name.is_empty()
            && (normalized_file.contains(&normalized_name)
                || normalized_name.contains(&normalized_file))
        {
            score += 150;
        }
        if filename.contains("launcher") {
            score -= 20;
        }
        if filename.contains("win64") || filename.contains("x64") {
            score += 10;
        }

        if best.as_ref().map(|(current, _)| score > *current).unwrap_or(true) {
            best = Some((score, path.to_path_buf()));
        }
    }

    best.map(|(_, path)| path)
}

fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(program_files_x86) = env::var("PROGRAMFILES(X86)") {
        roots.push(PathBuf::from(program_files_x86).join("Steam"));
    }
    if let Ok(program_files) = env::var("PROGRAMFILES") {
        roots.push(PathBuf::from(program_files).join("Steam"));
    }
    roots.sort();
    roots.dedup();
    roots.into_iter().filter(|path| path.exists()).collect()
}

fn scan_steam() -> (Vec<DiscoveredGame>, Vec<String>) {
    let mut games = Vec::new();
    let mut warnings = Vec::new();
    let mut libraries: HashSet<PathBuf> = HashSet::new();

    for root in steam_roots() {
        libraries.insert(root.clone());

        let library_file = root.join("steamapps").join("libraryfolders.vdf");
        if let Ok(contents) = fs::read_to_string(&library_file) {
            if let Ok(regex) = Regex::new(r#""path"\s+"([^"]+)""#) {
                for captures in regex.captures_iter(&contents) {
                    if let Some(value) = captures.get(1) {
                        let decoded = value.as_str().replace(r"\\", r"\");
                        libraries.insert(PathBuf::from(decoded));
                    }
                }
            }
        }
    }

    if libraries.is_empty() {
        warnings.push("Steam installation was not found in the standard Windows locations.".into());
        return (games, warnings);
    }

    let mut seen_ids = HashSet::new();

    for library in libraries {
        let steamapps = library.join("steamapps");
        let entries = match fs::read_dir(&steamapps) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let filename = path
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or_default();

            if !filename.starts_with("appmanifest_") || !filename.ends_with(".acf") {
                continue;
            }

            let contents = match fs::read_to_string(&path) {
                Ok(contents) => contents,
                Err(_) => continue,
            };

            let app_id = quoted_vdf_value(&contents, "appid").or_else(|| {
                filename
                    .strip_prefix("appmanifest_")
                    .and_then(|value| value.strip_suffix(".acf"))
                    .map(ToOwned::to_owned)
            });
            let title = quoted_vdf_value(&contents, "name");
            let install_dir = quoted_vdf_value(&contents, "installdir");

            let (app_id, title, install_dir) = match (app_id, title, install_dir) {
                (Some(app_id), Some(title), Some(install_dir)) => (app_id, title, install_dir),
                _ => continue,
            };

            if !seen_ids.insert(app_id.clone()) {
                continue;
            }

            let install_path = steamapps.join("common").join(&install_dir);
            let exe_path = find_best_executable(&install_path, &title)
                .map(|value| value.to_string_lossy().into_owned());

            games.push(DiscoveredGame {
                id: format!("steam:{app_id}"),
                title,
                exe_path,
                install_path: install_path.to_string_lossy().into_owned(),
                source: "steam".into(),
                source_id: Some(app_id),
            });
        }
    }

    (games, warnings)
}

fn scan_epic() -> (Vec<DiscoveredGame>, Vec<String>) {
    let mut games = Vec::new();
    let mut warnings = Vec::new();

    let program_data = match env::var("PROGRAMDATA") {
        Ok(value) => PathBuf::from(value),
        Err(_) => {
            warnings.push("PROGRAMDATA is unavailable, so Epic games could not be scanned.".into());
            return (games, warnings);
        }
    };

    let manifests = program_data
        .join("Epic")
        .join("EpicGamesLauncher")
        .join("Data")
        .join("Manifests");

    if !manifests.exists() {
        return (games, warnings);
    }

    let entries = match fs::read_dir(&manifests) {
        Ok(entries) => entries,
        Err(error) => {
            warnings.push(format!("Could not read Epic manifests: {error}"));
            return (games, warnings);
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("item") {
            continue;
        }

        let contents = match fs::read_to_string(&path) {
            Ok(contents) => contents,
            Err(_) => continue,
        };
        let json: serde_json::Value = match serde_json::from_str(&contents) {
            Ok(json) => json,
            Err(_) => continue,
        };

        let title = match json.get("DisplayName").and_then(|value| value.as_str()) {
            Some(value) if !value.trim().is_empty() => value.trim().to_string(),
            _ => continue,
        };
        let install_location = match json.get("InstallLocation").and_then(|value| value.as_str()) {
            Some(value) if !value.trim().is_empty() => PathBuf::from(value),
            _ => continue,
        };

        let source_id = json
            .get("CatalogItemId")
            .and_then(|value| value.as_str())
            .or_else(|| json.get("AppName").and_then(|value| value.as_str()))
            .unwrap_or(&title)
            .to_string();

        let launch_executable = json
            .get("LaunchExecutable")
            .and_then(|value| value.as_str())
            .filter(|value| !value.trim().is_empty());

        let exe = launch_executable
            .map(|value| install_location.join(value))
            .filter(|value| value.exists())
            .or_else(|| find_best_executable(&install_location, &title));

        games.push(DiscoveredGame {
            id: format!("epic:{source_id}"),
            title,
            exe_path: exe.map(|value| value.to_string_lossy().into_owned()),
            install_path: install_location.to_string_lossy().into_owned(),
            source: "epic".into(),
            source_id: Some(source_id),
        });
    }

    (games, warnings)
}


#[cfg(target_os = "windows")]
fn parse_gog_registry_output(output: &str) -> Vec<DiscoveredGame> {
    let mut games = Vec::new();
    let mut current_key = String::new();
    let mut values: HashMap<String, String> = HashMap::new();

    let flush = |key: &str, values: &mut HashMap<String, String>, games: &mut Vec<DiscoveredGame>| {
        if key.is_empty() {
            values.clear();
            return;
        }

        let title = values
            .get("gamename")
            .or_else(|| values.get("displayname"))
            .cloned();

        let install_path = values
            .get("path")
            .or_else(|| values.get("installlocation"))
            .cloned();

        let Some(title) = title else {
            values.clear();
            return;
        };
        let Some(install_path) = install_path else {
            values.clear();
            return;
        };

        let install = PathBuf::from(&install_path);
        if !install.exists() {
            values.clear();
            return;
        }

        let source_id = values
            .get("gameid")
            .cloned()
            .or_else(|| key.rsplit('\\').next().map(ToOwned::to_owned))
            .unwrap_or_else(|| title.clone());

        let exe_path = values
            .get("exe")
            .map(PathBuf::from)
            .filter(|value| value.exists())
            .or_else(|| find_best_executable(&install, &title))
            .map(|value| value.to_string_lossy().into_owned());

        games.push(DiscoveredGame {
            id: format!("gog:{source_id}"),
            title,
            exe_path,
            install_path: install.to_string_lossy().into_owned(),
            source: "gog".into(),
            source_id: Some(source_id),
        });

        values.clear();
    };

    for raw_line in output.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("HKEY_") {
            flush(&current_key, &mut values, &mut games);
            current_key = line.to_string();
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let name = parts[0].to_ascii_lowercase();
        let value = parts[2..].join(" ");
        values.insert(name, value);
    }

    flush(&current_key, &mut values, &mut games);
    games
}

#[cfg(target_os = "windows")]
fn scan_gog() -> (Vec<DiscoveredGame>, Vec<String>) {
    let roots = [
        r"HKLM\SOFTWARE\WOW6432Node\GOG.com\Games",
        r"HKLM\SOFTWARE\GOG.com\Games",
        r"HKCU\SOFTWARE\GOG.com\Games",
    ];

    let mut games = Vec::new();
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();

    for root in roots {
        let output = Command::new("reg")
            .args(["query", root, "/s"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let text = String::from_utf8_lossy(&output.stdout);
                for game in parse_gog_registry_output(&text) {
                    if seen.insert(game.id.clone()) {
                        games.push(game);
                    }
                }
            }
            Ok(_) => {}
            Err(error) => warnings.push(format!("Could not query GOG registry data: {error}")),
        }
    }

    (games, warnings)
}

#[cfg(not(target_os = "windows"))]
fn scan_gog() -> (Vec<DiscoveredGame>, Vec<String>) {
    (Vec::new(), Vec::new())
}

#[cfg(target_os = "windows")]
fn resolve_windows_executable(name: &str, candidates: &[PathBuf]) -> Option<PathBuf> {
    for candidate in candidates {
        if candidate.is_file() {
            return Some(candidate.clone());
        }
    }

    let output = Command::new("where").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

#[cfg(target_os = "windows")]
fn scan_emulators() -> (Vec<DiscoveredGame>, Vec<String>) {
    let mut games = Vec::new();

    let program_files = env::var("PROGRAMFILES").ok().map(PathBuf::from);
    let program_files_x86 = env::var("PROGRAMFILES(X86)").ok().map(PathBuf::from);
    let local_app_data = env::var("LOCALAPPDATA").ok().map(PathBuf::from);
    let app_data = env::var("APPDATA").ok().map(PathBuf::from);

    let specs: Vec<(&str, &str, Vec<PathBuf>)> = vec![
        (
            "Dolphin Emulator",
            "Dolphin.exe",
            [
                program_files.as_ref().map(|p| p.join("Dolphin").join("Dolphin.exe")),
                local_app_data.as_ref().map(|p| p.join("Dolphin").join("Dolphin.exe")),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        (
            "PCSX2",
            "pcsx2-qt.exe",
            [
                program_files.as_ref().map(|p| p.join("PCSX2").join("pcsx2-qt.exe")),
                local_app_data.as_ref().map(|p| p.join("PCSX2").join("pcsx2-qt.exe")),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        (
            "RetroArch",
            "retroarch.exe",
            [
                program_files.as_ref().map(|p| p.join("RetroArch-Win64").join("retroarch.exe")),
                program_files_x86.as_ref().map(|p| p.join("RetroArch-Win64").join("retroarch.exe")),
                app_data.as_ref().map(|p| p.join("RetroArch").join("retroarch.exe")),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        (
            "Ryujinx",
            "Ryujinx.exe",
            [
                local_app_data.as_ref().map(|p| p.join("Ryujinx").join("Ryujinx.exe")),
                app_data.as_ref().map(|p| p.join("Ryujinx").join("Ryujinx.exe")),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        (
            "Cemu",
            "Cemu.exe",
            [
                program_files.as_ref().map(|p| p.join("Cemu").join("Cemu.exe")),
                local_app_data.as_ref().map(|p| p.join("Cemu").join("Cemu.exe")),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        (
            "PPSSPP",
            "PPSSPPWindows64.exe",
            [
                program_files.as_ref().map(|p| p.join("PPSSPP").join("PPSSPPWindows64.exe")),
                program_files_x86.as_ref().map(|p| p.join("PPSSPP").join("PPSSPPWindows64.exe")),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        (
            "DuckStation",
            "duckstation-qt-x64-ReleaseLTCG.exe",
            [
                local_app_data.as_ref().map(|p| p.join("DuckStation").join("duckstation-qt-x64-ReleaseLTCG.exe")),
                program_files.as_ref().map(|p| p.join("DuckStation").join("duckstation-qt-x64-ReleaseLTCG.exe")),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
    ];

    for (title, executable_name, candidates) in specs {
        if let Some(executable) = resolve_windows_executable(executable_name, &candidates) {
            let install_path = executable
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .to_string_lossy()
                .into_owned();

            let source_id = title
                .to_ascii_lowercase()
                .replace(' ', "-");

            games.push(DiscoveredGame {
                id: format!("emulator:{source_id}"),
                title: title.to_string(),
                exe_path: Some(executable.to_string_lossy().into_owned()),
                install_path,
                source: "emulator".into(),
                source_id: Some(source_id),
            });
        }
    }

    (games, Vec::new())
}

#[cfg(not(target_os = "windows"))]
fn scan_emulators() -> (Vec<DiscoveredGame>, Vec<String>) {
    (Vec::new(), Vec::new())
}


fn likely_game_folder_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    ![
        "windows",
        "program files",
        "program files (x86)",
        "programdata",
        "users",
        "$recycle.bin",
        "system volume information",
    ]
    .iter()
    .any(|blocked| lower == *blocked)
}

fn common_device_game_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        for letter in b'C'..=b'Z' {
            let drive = PathBuf::from(format!("{}:\\", letter as char));
            if !drive.exists() {
                continue;
            }

            for relative in ["Games", "Game", "PortableGames", "Portable Games"] {
                let candidate = drive.join(relative);
                if candidate.is_dir() {
                    roots.push(candidate);
                }
            }
        }

        if let Ok(profile) = env::var("USERPROFILE") {
            let profile = PathBuf::from(profile);
            for relative in ["Games", "Desktop\\Games", "Documents\\Games"] {
                let candidate = profile.join(relative);
                if candidate.is_dir() {
                    roots.push(candidate);
                }
            }
        }
    }

    roots.sort();
    roots.dedup();
    roots
}

fn scan_common_device_game_folders() -> (Vec<DiscoveredGame>, Vec<String>) {
    let started = Instant::now();
    let budget = Duration::from_secs(12);
    let mut games = Vec::new();
    let mut warnings = Vec::new();
    let mut seen_paths = HashSet::new();
    let mut inspected_folders = 0_usize;

    for root in common_device_game_roots() {
        if started.elapsed() >= budget || inspected_folders >= 250 {
            warnings.push(
                "Automatic device-folder scan stopped at its safety limit; launcher scans still completed."
                    .into(),
            );
            break;
        }

        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(_) => continue,
        };

        for entry in entries.filter_map(Result::ok) {
            if started.elapsed() >= budget || inspected_folders >= 250 {
                break;
            }

            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let title = entry.file_name().to_string_lossy().trim().to_string();
            if title.is_empty() || !likely_game_folder_name(&title) {
                continue;
            }

            inspected_folders += 1;

            let canonical = path.canonicalize().unwrap_or(path.clone());
            let canonical_key = canonical.to_string_lossy().to_ascii_lowercase();
            if !seen_paths.insert(canonical_key.clone()) {
                continue;
            }

            let Some(executable) =
                find_best_executable_bounded(&canonical, &title, started, budget)
            else {
                continue;
            };

            let mut hasher = DefaultHasher::new();
            canonical_key.hash(&mut hasher);
            let source_id = format!("{:016x}", hasher.finish());
            games.push(DiscoveredGame {
                id: format!("device:{source_id}"),
                title,
                exe_path: Some(executable.to_string_lossy().into_owned()),
                install_path: canonical.to_string_lossy().into_owned(),
                source: "device".into(),
                source_id: Some(source_id),
            });
        }
    }

    (games, warnings)
}

#[tauri::command]
fn list_games(app: AppHandle) -> Result<Vec<GameRecord>, String> {
    let connection = open_database(&app)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, title, exe_path, install_path, source, source_id, favorite,
                   cover_path, added_at, last_played, total_seconds, launch_count
            FROM games
            WHERE hidden = 0
            ORDER BY title COLLATE NOCASE ASC
            "#,
        )
        .map_err(|error| format!("Could not prepare game list: {error}"))?;

    let rows = statement
        .query_map([], row_to_game)
        .map_err(|error| format!("Could not read game list: {error}"))?;

    let mut games = Vec::new();
    for row in rows {
        games.push(row.map_err(|error| format!("Could not decode game: {error}"))?);
    }
    Ok(games)
}

fn scan_games_blocking(app: AppHandle) -> Result<ScanResult, String> {
    let (steam_games, mut warnings) = scan_steam();
    let (epic_games, epic_warnings) = scan_epic();
    let (gog_games, gog_warnings) = scan_gog();
    let (emulator_games, emulator_warnings) = scan_emulators();
    let (device_games, device_warnings) = scan_common_device_game_folders();

    warnings.extend(epic_warnings);
    warnings.extend(gog_warnings);
    warnings.extend(emulator_warnings);
    warnings.extend(device_warnings);

    let steam_found = steam_games.len();
    let epic_found = epic_games.len();
    let gog_found = gog_games.len();
    let emulator_found = emulator_games.len();
    let device_found = device_games.len();

    let all_games: Vec<DiscoveredGame> = steam_games
        .into_iter()
        .chain(epic_games)
        .chain(gog_games)
        .chain(emulator_games)
        .chain(device_games)
        .collect();

    let connection = open_database(&app)?;
    let mut added = 0;
    let mut updated = 0;

    for game in &all_games {
        if upsert_discovered(&connection, game)? {
            updated += 1;
        } else {
            added += 1;
        }
    }

    Ok(ScanResult {
        found: all_games.len(),
        added,
        updated,
        steam_found,
        epic_found,
        gog_found,
        emulator_found,
        device_found,
        warnings,
    })
}

#[tauri::command]
async fn scan_games(app: AppHandle) -> Result<ScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || scan_games_blocking(app))
        .await
        .map_err(|error| format!("Game scan worker failed: {error}"))?
}

#[tauri::command]
fn choose_executable() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Windows executable", &["exe"])
        .pick_file()
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn add_manual_game(app: AppHandle, title: String, exe_path: String) -> Result<GameRecord, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Game title cannot be empty.".into());
    }

    let executable = PathBuf::from(exe_path.trim());
    if !executable.is_file() {
        return Err("The selected executable does not exist.".into());
    }

    let install_path = executable
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_string_lossy()
        .into_owned();

    let game = DiscoveredGame {
        id: format!("manual:{}", Uuid::new_v4()),
        title: title.to_string(),
        exe_path: Some(executable.to_string_lossy().into_owned()),
        install_path,
        source: "manual".into(),
        source_id: None,
    };

    let connection = open_database(&app)?;
    upsert_discovered(&connection, &game)?;
    get_game(&connection, &game.id)
}

#[tauri::command]
fn set_favorite(app: AppHandle, game_id: String, favorite: bool) -> Result<(), String> {
    let connection = open_database(&app)?;
    connection
        .execute(
            "UPDATE games SET favorite = ?1 WHERE id = ?2",
            params![favorite as i64, game_id],
        )
        .map_err(|error| format!("Could not update favorite: {error}"))?;
    Ok(())
}

#[tauri::command]
fn rename_game(app: AppHandle, game_id: String, title: String) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Game title cannot be empty.".into());
    }
    let connection = open_database(&app)?;
    connection
        .execute(
            "UPDATE games SET title = ?1 WHERE id = ?2",
            params![title, game_id],
        )
        .map_err(|error| format!("Could not rename game: {error}"))?;
    Ok(())
}

#[tauri::command]
fn remove_game(app: AppHandle, game_id: String) -> Result<(), String> {
    let connection = open_database(&app)?;
    connection
        .execute("UPDATE games SET hidden = 1 WHERE id = ?1", params![game_id])
        .map_err(|error| format!("Could not remove game from library: {error}"))?;
    Ok(())
}

#[tauri::command]
fn choose_cover(app: AppHandle, game_id: String) -> Result<bool, String> {
    let source = match rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
        .pick_file()
    {
        Some(path) => path,
        None => return Ok(false),
    };

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();

    let covers_dir = app_data_dir(&app)?.join("covers");
    fs::create_dir_all(&covers_dir)
        .map_err(|error| format!("Could not create cover directory: {error}"))?;

    let destination = covers_dir.join(format!("{}.{}", game_id.replace(':', "_"), extension));
    fs::copy(&source, &destination)
        .map_err(|error| format!("Could not copy cover image: {error}"))?;

    let connection = open_database(&app)?;
    connection
        .execute(
            "UPDATE games SET cover_path = ?1 WHERE id = ?2",
            params![destination.to_string_lossy().into_owned(), game_id],
        )
        .map_err(|error| format!("Could not save cover image: {error}"))?;

    Ok(true)
}

#[cfg(target_os = "windows")]
fn process_running(executable_name: &str) -> bool {
    let filter = format!("IMAGENAME eq {executable_name}");
    let output = Command::new("tasklist")
        .args(["/FI", &filter, "/FO", "CSV", "/NH"])
        .output();

    match output {
        Ok(output) => String::from_utf8_lossy(&output.stdout)
            .to_ascii_lowercase()
            .contains(&executable_name.to_ascii_lowercase()),
        Err(_) => false,
    }
}

#[cfg(not(target_os = "windows"))]
fn process_running(_executable_name: &str) -> bool {
    false
}

fn record_session(app: &AppHandle, game_id: &str, started_at: DateTime<Utc>, duration: Duration) {
    let seconds = duration.as_secs() as i64;
    if seconds <= 0 {
        return;
    }

    let ended_at = Utc::now();
    if let Ok(connection) = open_database(app) {
        let _ = connection.execute(
            r#"
            INSERT INTO sessions (game_id, started_at, ended_at, duration_seconds)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                game_id,
                started_at.to_rfc3339(),
                ended_at.to_rfc3339(),
                seconds
            ],
        );

        let _ = connection.execute(
            r#"
            UPDATE games
            SET total_seconds = total_seconds + ?1,
                last_played = ?2,
                launch_count = launch_count + 1
            WHERE id = ?3
            "#,
            params![seconds, ended_at.to_rfc3339(), game_id],
        );
    }
}

#[cfg(target_os = "windows")]
fn launch_steam_game(
    app: AppHandle,
    game: GameRecord,
) -> Result<LaunchResult, String> {
    let app_id = game
        .source_id
        .clone()
        .ok_or_else(|| "Steam App ID is missing.".to_string())?;
    let uri = format!("steam://rungameid/{app_id}");

    Command::new("cmd")
        .args(["/C", "start", "", &uri])
        .spawn()
        .map_err(|error| format!("Could not ask Steam to launch the game: {error}"))?;

    let executable_name = game
        .exe_path
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|value| value.to_str())
        .map(ToOwned::to_owned);

    let Some(executable_name) = executable_name else {
        return Ok(LaunchResult {
            started: true,
            tracking: false,
            message: "Launched through Steam. Dusk could not identify the game executable, so this session will not be counted.".into(),
        });
    };

    let game_id = game.id.clone();
    thread::spawn(move || {
        let discovery_deadline = Instant::now() + Duration::from_secs(90);
        while Instant::now() < discovery_deadline {
            if process_running(&executable_name) {
                let started_at = Utc::now();
                let timer = Instant::now();
                let mut consecutive_misses = 0;

                loop {
                    thread::sleep(Duration::from_secs(5));
                    if process_running(&executable_name) {
                        consecutive_misses = 0;
                    } else {
                        consecutive_misses += 1;
                        if consecutive_misses >= 2 {
                            break;
                        }
                    }
                }

                record_session(&app, &game_id, started_at, timer.elapsed());
                return;
            }
            thread::sleep(Duration::from_secs(2));
        }
    });

    Ok(LaunchResult {
        started: true,
        tracking: true,
        message: "Steam launch requested. Dusk will start counting once the game process appears.".into(),
    })
}

#[cfg(not(target_os = "windows"))]
fn launch_steam_game(_app: AppHandle, _game: GameRecord) -> Result<LaunchResult, String> {
    Err("Steam launching is currently implemented for Windows.".into())
}

fn launch_direct_game(app: AppHandle, game: GameRecord) -> Result<LaunchResult, String> {
    let exe_path = game
        .exe_path
        .clone()
        .ok_or_else(|| "No executable is known for this game.".to_string())?;
    let executable = PathBuf::from(&exe_path);

    if !executable.exists() {
        return Err("The game executable no longer exists. Rescan or add the game again.".into());
    }

    let working_directory = executable
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(&game.install_path));

    let mut child = Command::new(&executable)
        .current_dir(working_directory)
        .spawn()
        .map_err(|error| format!("Could not launch game: {error}"))?;

    let game_id = game.id.clone();
    thread::spawn(move || {
        let started_at = Utc::now();
        let timer = Instant::now();
        if child.wait().is_ok() {
            record_session(&app, &game_id, started_at, timer.elapsed());
        }
    });

    Ok(LaunchResult {
        started: true,
        tracking: true,
        message: "Game launched. Dusk is tracking this session.".into(),
    })
}

#[tauri::command]
fn launch_game(app: AppHandle, game_id: String) -> Result<LaunchResult, String> {
    let connection = open_database(&app)?;
    let game = get_game(&connection, &game_id)?;
    drop(connection);

    if game.source == "steam" {
        launch_steam_game(app, game)
    } else {
        launch_direct_game(app, game)
    }
}

#[cfg(target_os = "windows")]
#[tauri::command]
fn open_game_folder(app: AppHandle, game_id: String) -> Result<(), String> {
    let connection = open_database(&app)?;
    let game = get_game(&connection, &game_id)?;

    Command::new("explorer")
        .arg(&game.install_path)
        .spawn()
        .map_err(|error| format!("Could not open game folder: {error}"))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
#[tauri::command]
fn open_game_folder(_app: AppHandle, _game_id: String) -> Result<(), String> {
    Err("Opening game folders is currently implemented for Windows.".into())
}


fn is_screenshot_image(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|value| value.to_str())
            .map(|value| value.to_ascii_lowercase())
            .as_deref(),
        Some("png") | Some("jpg") | Some("jpeg") | Some("webp") | Some("gif") | Some("bmp")
    )
}

fn screenshot_created_at(path: &Path) -> String {
    fs::metadata(path)
        .ok()
        .and_then(|metadata| metadata.modified().ok())
        .map(|modified| DateTime::<Utc>::from(modified).to_rfc3339())
        .unwrap_or_else(now)
}

enum ScreenshotStoreOutcome {
    Imported,
    Duplicate,
    Ignored,
    Failed,
}

fn store_screenshot_source(
    app: &AppHandle,
    connection: &Connection,
    game_id: &str,
    source: &Path,
    allow_ignored: bool,
) -> ScreenshotStoreOutcome {
    if !source.is_file() || !is_screenshot_image(source) {
        return ScreenshotStoreOutcome::Failed;
    }

    let source_path = source.to_string_lossy().into_owned();

    if allow_ignored {
        let _ = connection.execute(
            "DELETE FROM ignored_screenshot_sources WHERE source_path = ?1",
            params![source_path],
        );
    } else {
        let ignored: bool = connection
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM ignored_screenshot_sources WHERE source_path = ?1)",
                params![source_path],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if ignored {
            return ScreenshotStoreOutcome::Ignored;
        }
    }

    let exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM screenshots WHERE source_path = ?1)",
            params![source_path],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if exists {
        return ScreenshotStoreOutcome::Duplicate;
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();

    let screenshots_dir = match app_data_dir(app) {
        Ok(path) => path.join("screenshots").join(game_id.replace(':', "_")),
        Err(_) => return ScreenshotStoreOutcome::Failed,
    };

    if fs::create_dir_all(&screenshots_dir).is_err() {
        return ScreenshotStoreOutcome::Failed;
    }

    let destination = screenshots_dir.join(format!("{}.{}", Uuid::new_v4(), extension));
    if fs::copy(source, &destination).is_err() {
        return ScreenshotStoreOutcome::Failed;
    }

    let inserted = connection.execute(
        "INSERT OR IGNORE INTO screenshots (game_id, path, source_path, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            game_id,
            destination.to_string_lossy().into_owned(),
            source_path,
            screenshot_created_at(source)
        ],
    );

    match inserted {
        Ok(1) => ScreenshotStoreOutcome::Imported,
        Ok(_) => {
            let _ = fs::remove_file(destination);
            ScreenshotStoreOutcome::Duplicate
        }
        Err(_) => {
            let _ = fs::remove_file(destination);
            ScreenshotStoreOutcome::Failed
        }
    }
}

fn image_files_in_directory(directory: &Path, max_depth: usize) -> Vec<PathBuf> {
    if !directory.is_dir() {
        return Vec::new();
    }

    WalkDir::new(directory)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file() && is_screenshot_image(entry.path()))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

fn normalized_match_text(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .collect()
}

fn scan_screenshots_blocking(app: AppHandle) -> Result<AutoScreenshotScanResult, String> {
    let connection = open_database(&app)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, title, install_path, source, source_id
            FROM games
            WHERE hidden = 0
            "#,
        )
        .map_err(|error| format!("Could not prepare screenshot game scan: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })
        .map_err(|error| format!("Could not load games for screenshot scan: {error}"))?;

    let games: Vec<(String, String, String, String, Option<String>)> = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("Could not decode games for screenshot scan: {error}"))?;
    drop(statement);

    let mut found = 0;
    let mut imported = 0;
    let mut steam_imported = 0;
    let mut matched_imported = 0;
    let mut skipped_duplicates = 0;
    let mut seen_sources = HashSet::new();

    // Steam keeps screenshots under userdata/<account>/760/remote/<appid>/screenshots.
    for (game_id, _title, _install_path, source, source_id) in &games {
        if source != "steam" {
            continue;
        }
        let Some(app_id) = source_id.as_deref() else {
            continue;
        };

        for steam_root in steam_roots() {
            let userdata = steam_root.join("userdata");
            let accounts = match fs::read_dir(&userdata) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for account in accounts.filter_map(Result::ok) {
                let directory = account
                    .path()
                    .join("760")
                    .join("remote")
                    .join(app_id)
                    .join("screenshots");

                for source_path in image_files_in_directory(&directory, 1) {
                    let key = source_path.to_string_lossy().into_owned();
                    if !seen_sources.insert(key) {
                        continue;
                    }
                    found += 1;
                    match store_screenshot_source(&app, &connection, game_id, &source_path, false) {
                        ScreenshotStoreOutcome::Imported => {
                            imported += 1;
                            steam_imported += 1;
                        }
                        ScreenshotStoreOutcome::Duplicate | ScreenshotStoreOutcome::Ignored => {
                            skipped_duplicates += 1;
                        }
                        ScreenshotStoreOutcome::Failed => {}
                    }
                }
            }
        }
    }

    // Match screenshots inside common per-game screenshot folders.
    for (game_id, _title, install_path, _source, _source_id) in &games {
        let root = PathBuf::from(install_path);
        for name in ["screenshots", "Screenshots", "ScreenShots", "captures", "Captures"] {
            let directory = root.join(name);
            for source_path in image_files_in_directory(&directory, 2) {
                let key = source_path.to_string_lossy().into_owned();
                if !seen_sources.insert(key) {
                    continue;
                }
                found += 1;
                match store_screenshot_source(&app, &connection, game_id, &source_path, false) {
                    ScreenshotStoreOutcome::Imported => {
                        imported += 1;
                        matched_imported += 1;
                    }
                    ScreenshotStoreOutcome::Duplicate | ScreenshotStoreOutcome::Ignored => {
                        skipped_duplicates += 1;
                    }
                    ScreenshotStoreOutcome::Failed => {}
                }
            }
        }
    }

    // Windows and Xbox Game Bar commonly save to these folders. To avoid false
    // associations, only import when the filename contains a sufficiently specific
    // normalized game title.
    if let Ok(profile) = env::var("USERPROFILE") {
        let profile = PathBuf::from(profile);
        let shared_directories = [
            profile.join("Pictures").join("Screenshots"),
            profile.join("Videos").join("Captures"),
        ];

        for directory in shared_directories {
            for source_path in image_files_in_directory(&directory, 2) {
                let key = source_path.to_string_lossy().into_owned();
                if !seen_sources.insert(key) {
                    continue;
                }

                let filename = source_path
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or_default();
                let normalized_file = normalized_match_text(filename);

                let matched_game = games.iter().find(|(_id, title, _path, _source, _source_id)| {
                    let normalized_title = normalized_match_text(title);
                    normalized_title.len() >= 5 && normalized_file.contains(&normalized_title)
                });

                let Some((game_id, _title, _path, _source, _source_id)) = matched_game else {
                    continue;
                };

                found += 1;
                match store_screenshot_source(&app, &connection, game_id, &source_path, false) {
                    ScreenshotStoreOutcome::Imported => {
                        imported += 1;
                        matched_imported += 1;
                    }
                    ScreenshotStoreOutcome::Duplicate | ScreenshotStoreOutcome::Ignored => {
                        skipped_duplicates += 1;
                    }
                    ScreenshotStoreOutcome::Failed => {}
                }
            }
        }
    }

    Ok(AutoScreenshotScanResult {
        found,
        imported,
        steam_imported,
        matched_imported,
        skipped_duplicates,
    })
}

#[tauri::command]
async fn scan_screenshots(app: AppHandle) -> Result<AutoScreenshotScanResult, String> {
    tauri::async_runtime::spawn_blocking(move || scan_screenshots_blocking(app))
        .await
        .map_err(|error| format!("Screenshot scan worker failed: {error}"))?
}

#[tauri::command]
fn import_screenshots(app: AppHandle, game_id: String) -> Result<usize, String> {
    let selected = match rfd::FileDialog::new()
        .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif", "bmp"])
        .pick_files()
    {
        Some(files) => files,
        None => return Ok(0),
    };

    let connection = open_database(&app)?;
    let mut imported = 0;

    for source in selected {
        if matches!(
            store_screenshot_source(&app, &connection, &game_id, &source, true),
            ScreenshotStoreOutcome::Imported
        ) {
            imported += 1;
        }
    }

    Ok(imported)
}

#[tauri::command]
fn list_screenshots(app: AppHandle, game_id: Option<String>) -> Result<Vec<ScreenshotRecord>, String> {
    let connection = open_database(&app)?;

    let sql = if game_id.is_some() {
        "SELECT id, game_id, path, created_at FROM screenshots WHERE game_id = ?1 ORDER BY created_at DESC"
    } else {
        "SELECT id, game_id, path, created_at FROM screenshots ORDER BY created_at DESC"
    };

    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("Could not prepare screenshot query: {error}"))?;

    let decode = |row: &rusqlite::Row<'_>| -> rusqlite::Result<ScreenshotRecord> {
        let path: String = row.get(2)?;
        Ok(ScreenshotRecord {
            id: row.get(0)?,
            game_id: row.get(1)?,
            data_url: None,
            path,
            created_at: row.get(3)?,
        })
    };

    let mut screenshots = Vec::new();

    if let Some(game_id) = game_id {
        let rows = statement
            .query_map(params![game_id], decode)
            .map_err(|error| format!("Could not load screenshots: {error}"))?;
        for row in rows {
            screenshots.push(row.map_err(|error| format!("Could not decode screenshot: {error}"))?);
        }
    } else {
        let rows = statement
            .query_map([], decode)
            .map_err(|error| format!("Could not load screenshots: {error}"))?;
        for row in rows {
            screenshots.push(row.map_err(|error| format!("Could not decode screenshot: {error}"))?);
        }
    }

    Ok(screenshots)
}

#[tauri::command]
fn delete_screenshot(app: AppHandle, screenshot_id: i64) -> Result<(), String> {
    let connection = open_database(&app)?;
    let screenshot: Option<(String, Option<String>)> = connection
        .query_row(
            "SELECT path, source_path FROM screenshots WHERE id = ?1",
            params![screenshot_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(|error| format!("Could not find screenshot: {error}"))?;

    if let Some((path, source_path)) = screenshot {
        let _ = fs::remove_file(path);
        if let Some(source_path) = source_path {
            let _ = connection.execute(
                "INSERT OR REPLACE INTO ignored_screenshot_sources (source_path, ignored_at) VALUES (?1, ?2)",
                params![source_path, now()],
            );
        }
    }

    connection
        .execute("DELETE FROM screenshots WHERE id = ?1", params![screenshot_id])
        .map_err(|error| format!("Could not delete screenshot: {error}"))?;

    Ok(())
}

#[tauri::command]
fn list_collections(app: AppHandle) -> Result<Vec<CollectionRecord>, String> {
    let connection = open_database(&app)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT c.id, c.name, COUNT(cg.game_id)
            FROM collections c
            LEFT JOIN collection_games cg ON cg.collection_id = c.id
            GROUP BY c.id, c.name
            ORDER BY c.name COLLATE NOCASE ASC
            "#,
        )
        .map_err(|error| format!("Could not prepare collections: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok(CollectionRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                game_count: row.get(2)?,
            })
        })
        .map_err(|error| format!("Could not read collections: {error}"))?;

    let mut collections = Vec::new();
    for row in rows {
        collections.push(row.map_err(|error| format!("Could not decode collection: {error}"))?);
    }
    Ok(collections)
}

#[tauri::command]
fn create_collection(app: AppHandle, name: String) -> Result<CollectionRecord, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Collection name cannot be empty.".into());
    }

    let id = Uuid::new_v4().to_string();
    let connection = open_database(&app)?;
    connection
        .execute(
            "INSERT INTO collections (id, name, created_at) VALUES (?1, ?2, ?3)",
            params![id, name, now()],
        )
        .map_err(|error| {
            if error.to_string().contains("UNIQUE") {
                "A collection with that name already exists.".to_string()
            } else {
                format!("Could not create collection: {error}")
            }
        })?;

    Ok(CollectionRecord {
        id,
        name: name.to_string(),
        game_count: 0,
    })
}

#[tauri::command]
fn delete_collection(app: AppHandle, collection_id: String) -> Result<(), String> {
    let connection = open_database(&app)?;
    connection
        .execute("DELETE FROM collections WHERE id = ?1", params![collection_id])
        .map_err(|error| format!("Could not delete collection: {error}"))?;
    Ok(())
}

#[tauri::command]
fn set_collection_membership(
    app: AppHandle,
    collection_id: String,
    game_id: String,
    included: bool,
) -> Result<(), String> {
    let connection = open_database(&app)?;
    if included {
        connection
            .execute(
                "INSERT OR IGNORE INTO collection_games (collection_id, game_id) VALUES (?1, ?2)",
                params![collection_id, game_id],
            )
            .map_err(|error| format!("Could not add game to collection: {error}"))?;
    } else {
        connection
            .execute(
                "DELETE FROM collection_games WHERE collection_id = ?1 AND game_id = ?2",
                params![collection_id, game_id],
            )
            .map_err(|error| format!("Could not remove game from collection: {error}"))?;
    }
    Ok(())
}

#[tauri::command]
fn collection_memberships(app: AppHandle) -> Result<Vec<CollectionMembership>, String> {
    let connection = open_database(&app)?;
    let mut statement = connection
        .prepare("SELECT collection_id, game_id FROM collection_games")
        .map_err(|error| format!("Could not prepare collection memberships: {error}"))?;

    let rows = statement
        .query_map([], |row| {
            Ok(CollectionMembership {
                collection_id: row.get(0)?,
                game_id: row.get(1)?,
            })
        })
        .map_err(|error| format!("Could not load collection memberships: {error}"))?;

    let mut memberships = Vec::new();
    for row in rows {
        memberships.push(row.map_err(|error| format!("Could not decode membership: {error}"))?);
    }
    Ok(memberships)
}

#[tauri::command]
fn get_stats(app: AppHandle) -> Result<Stats, String> {
    let connection = open_database(&app)?;

    let game_count = connection
        .query_row("SELECT COUNT(*) FROM games WHERE hidden = 0", [], |row| row.get(0))
        .unwrap_or(0);
    let favorite_count = connection
        .query_row(
            "SELECT COUNT(*) FROM games WHERE hidden = 0 AND favorite = 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let played_game_count = connection
        .query_row(
            "SELECT COUNT(*) FROM games WHERE hidden = 0 AND launch_count > 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_seconds = connection
        .query_row(
            "SELECT COALESCE(SUM(total_seconds), 0) FROM games WHERE hidden = 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let launch_count = connection
        .query_row(
            "SELECT COALESCE(SUM(launch_count), 0) FROM games WHERE hidden = 0",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let last_7_days_seconds = connection
        .query_row(
            "SELECT COALESCE(SUM(duration_seconds), 0) FROM sessions WHERE started_at >= datetime('now', '-7 days')",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let screenshot_count = connection
        .query_row("SELECT COUNT(*) FROM screenshots", [], |row| row.get(0))
        .unwrap_or(0);

    let top_game = connection
        .query_row(
            r#"
            SELECT title
            FROM games
            WHERE hidden = 0 AND total_seconds > 0
            ORDER BY total_seconds DESC
            LIMIT 1
            "#,
            [],
            |row| row.get(0),
        )
        .optional()
        .unwrap_or(None);

    Ok(Stats {
        game_count,
        favorite_count,
        played_game_count,
        total_seconds,
        launch_count,
        last_7_days_seconds,
        screenshot_count,
        top_game,
    })
}

#[tauri::command]
fn list_achievements(app: AppHandle) -> Result<Vec<Achievement>, String> {
    let stats = get_stats(app.clone())?;
    let connection = open_database(&app)?;

    let night_sessions: i64 = connection
        .query_row(
            r#"
            SELECT COUNT(*)
            FROM sessions
            WHERE CAST(strftime('%H', started_at) AS INTEGER) BETWEEN 0 AND 4
            "#,
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let achievements = vec![
        Achievement {
            id: "first-launch".into(),
            title: "First Light".into(),
            description: "Finish your first tracked game session.".into(),
            unlocked: stats.launch_count >= 1,
            current: stats.launch_count.min(1),
            target: 1,
        },
        Achievement {
            id: "collector".into(),
            title: "Collector".into(),
            description: "Build a library of 10 games.".into(),
            unlocked: stats.game_count >= 10,
            current: stats.game_count.min(10),
            target: 10,
        },
        Achievement {
            id: "ten-hours".into(),
            title: "Settled In".into(),
            description: "Track 10 hours of playtime in Dusk.".into(),
            unlocked: stats.total_seconds >= 36_000,
            current: (stats.total_seconds / 3600).min(10),
            target: 10,
        },
        Achievement {
            id: "hundred-hours".into(),
            title: "After Dark".into(),
            description: "Track 100 hours of playtime in Dusk.".into(),
            unlocked: stats.total_seconds >= 360_000,
            current: (stats.total_seconds / 3600).min(100),
            target: 100,
        },
        Achievement {
            id: "variety".into(),
            title: "No Main".into(),
            description: "Play 5 different games.".into(),
            unlocked: stats.played_game_count >= 5,
            current: stats.played_game_count.min(5),
            target: 5,
        },
        Achievement {
            id: "snapshots".into(),
            title: "Memory Card".into(),
            description: "Import 10 screenshots into Dusk.".into(),
            unlocked: stats.screenshot_count >= 10,
            current: stats.screenshot_count.min(10),
            target: 10,
        },
        Achievement {
            id: "night-shift".into(),
            title: "Night Shift".into(),
            description: "Finish a tracked session that started between midnight and 05:00.".into(),
            unlocked: night_sessions >= 1,
            current: night_sessions.min(1),
            target: 1,
        },
    ];

    Ok(achievements)
}


fn validate_save_directory(path: &Path) -> Result<PathBuf, String> {
    if !path.exists() {
        return Err("The selected save folder does not exist.".into());
    }
    if !path.is_dir() {
        return Err("The selected save path is not a folder.".into());
    }

    let canonical = path
        .canonicalize()
        .map_err(|error| format!("Could not resolve save folder: {error}"))?;

    if canonical.file_name().is_none() {
        return Err("A drive root cannot be used as a save folder.".into());
    }

    let protected_roots = [
        env::var("USERPROFILE").ok(),
        env::var("WINDIR").ok(),
        env::var("PROGRAMFILES").ok(),
        env::var("PROGRAMFILES(X86)").ok(),
        env::var("APPDATA").ok(),
        env::var("LOCALAPPDATA").ok(),
    ];

    for protected in protected_roots.into_iter().flatten() {
        if let Ok(protected) = PathBuf::from(protected).canonicalize() {
            if canonical == protected {
                return Err(
                    "Choose the game's specific save folder, not a broad system or profile folder."
                        .into(),
                );
            }
        }
    }

    Ok(canonical)
}

fn copy_directory_tree(source: &Path, destination: &Path) -> Result<(i64, i64), String> {
    fs::create_dir_all(destination)
        .map_err(|error| format!("Could not create backup directory: {error}"))?;

    let mut file_count = 0_i64;
    let mut total_bytes = 0_i64;

    for entry in WalkDir::new(source)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let source_path = entry.path();
        let relative = source_path
            .strip_prefix(source)
            .map_err(|error| format!("Could not map save path: {error}"))?;

        if relative.as_os_str().is_empty() {
            continue;
        }

        let target = destination.join(relative);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)
                .map_err(|error| format!("Could not create backup folder: {error}"))?;
            continue;
        }

        if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)
                    .map_err(|error| format!("Could not create backup folder: {error}"))?;
            }

            let copied = fs::copy(source_path, &target)
                .map_err(|error| format!("Could not copy save file: {error}"))?;

            file_count += 1;
            total_bytes += copied as i64;
        }
    }

    Ok((file_count, total_bytes))
}

fn clear_directory_contents(path: &Path) -> Result<(), String> {
    let entries = fs::read_dir(path)
        .map_err(|error| format!("Could not read configured save folder: {error}"))?;

    for entry in entries {
        let entry = entry.map_err(|error| format!("Could not inspect save folder: {error}"))?;
        let target = entry.path();

        if target.is_dir() {
            fs::remove_dir_all(&target)
                .map_err(|error| format!("Could not clear save folder: {error}"))?;
        } else {
            fs::remove_file(&target)
                .map_err(|error| format!("Could not clear save file: {error}"))?;
        }
    }

    Ok(())
}

fn save_config_for_game(connection: &Connection, game_id: &str) -> Result<SaveConfig, String> {
    connection
        .query_row(
            "SELECT game_id, save_path, configured_at FROM save_configs WHERE game_id = ?1",
            params![game_id],
            |row| {
                Ok(SaveConfig {
                    game_id: row.get(0)?,
                    save_path: row.get(1)?,
                    configured_at: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("Could not read save configuration: {error}"))?
        .ok_or_else(|| "No save folder is configured for this game.".to_string())
}

fn create_save_backup_internal(
    app: &AppHandle,
    connection: &Connection,
    game_id: &str,
    kind: &str,
) -> Result<SaveBackupRecord, String> {
    let config = save_config_for_game(connection, game_id)?;
    let source = validate_save_directory(Path::new(&config.save_path))?;

    let id = Uuid::new_v4().to_string();
    let backup_root = app_data_dir(app)?
        .join("save-backups")
        .join(game_id.replace(':', "_"))
        .join(&id);

    let (file_count, total_bytes) = copy_directory_tree(&source, &backup_root)?;
    let created_at = now();

    connection
        .execute(
            r#"
            INSERT INTO save_backups
                (id, game_id, backup_path, created_at, file_count, total_bytes, kind)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                id,
                game_id,
                backup_root.to_string_lossy().into_owned(),
                created_at,
                file_count,
                total_bytes,
                kind
            ],
        )
        .map_err(|error| format!("Could not record save backup: {error}"))?;

    Ok(SaveBackupRecord {
        id,
        game_id: game_id.to_string(),
        backup_path: backup_root.to_string_lossy().into_owned(),
        created_at,
        file_count,
        total_bytes,
        kind: kind.to_string(),
    })
}

#[tauri::command]
fn choose_save_folder(app: AppHandle, game_id: String) -> Result<Option<SaveConfig>, String> {
    // Ensure the game exists before associating a filesystem location with it.
    let connection = open_database(&app)?;
    let _ = get_game(&connection, &game_id)?;

    let Some(selected) = rfd::FileDialog::new().pick_folder() else {
        return Ok(None);
    };

    let save_path = validate_save_directory(&selected)?;
    let configured_at = now();

    connection
        .execute(
            r#"
            INSERT INTO save_configs (game_id, save_path, configured_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(game_id) DO UPDATE SET
                save_path = excluded.save_path,
                configured_at = excluded.configured_at
            "#,
            params![
                game_id,
                save_path.to_string_lossy().into_owned(),
                configured_at
            ],
        )
        .map_err(|error| format!("Could not save backup configuration: {error}"))?;

    Ok(Some(SaveConfig {
        game_id,
        save_path: save_path.to_string_lossy().into_owned(),
        configured_at,
    }))
}

#[tauri::command]
fn get_save_config(app: AppHandle, game_id: String) -> Result<Option<SaveConfig>, String> {
    let connection = open_database(&app)?;
    connection
        .query_row(
            "SELECT game_id, save_path, configured_at FROM save_configs WHERE game_id = ?1",
            params![game_id],
            |row| {
                Ok(SaveConfig {
                    game_id: row.get(0)?,
                    save_path: row.get(1)?,
                    configured_at: row.get(2)?,
                })
            },
        )
        .optional()
        .map_err(|error| format!("Could not load save configuration: {error}"))
}

#[tauri::command]
fn clear_save_config(app: AppHandle, game_id: String) -> Result<(), String> {
    let connection = open_database(&app)?;
    connection
        .execute("DELETE FROM save_configs WHERE game_id = ?1", params![game_id])
        .map_err(|error| format!("Could not clear save configuration: {error}"))?;
    Ok(())
}

fn create_save_backup_blocking(app: AppHandle, game_id: String) -> Result<SaveBackupRecord, String> {
    let connection = open_database(&app)?;
    create_save_backup_internal(&app, &connection, &game_id, "manual")
}

#[tauri::command]
async fn create_save_backup(app: AppHandle, game_id: String) -> Result<SaveBackupRecord, String> {
    tauri::async_runtime::spawn_blocking(move || create_save_backup_blocking(app, game_id))
        .await
        .map_err(|error| format!("Save backup worker failed: {error}"))?
}

#[tauri::command]
fn list_save_backups(app: AppHandle, game_id: String) -> Result<Vec<SaveBackupRecord>, String> {
    let connection = open_database(&app)?;
    let mut statement = connection
        .prepare(
            r#"
            SELECT id, game_id, backup_path, created_at, file_count, total_bytes, kind
            FROM save_backups
            WHERE game_id = ?1
            ORDER BY created_at DESC
            "#,
        )
        .map_err(|error| format!("Could not prepare backup list: {error}"))?;

    let rows = statement
        .query_map(params![game_id], |row| {
            Ok(SaveBackupRecord {
                id: row.get(0)?,
                game_id: row.get(1)?,
                backup_path: row.get(2)?,
                created_at: row.get(3)?,
                file_count: row.get(4)?,
                total_bytes: row.get(5)?,
                kind: row.get(6)?,
            })
        })
        .map_err(|error| format!("Could not load save backups: {error}"))?;

    let mut backups = Vec::new();
    for row in rows {
        backups.push(row.map_err(|error| format!("Could not decode save backup: {error}"))?);
    }
    Ok(backups)
}

fn restore_save_backup_blocking(
    app: AppHandle,
    game_id: String,
    backup_id: String,
) -> Result<SaveBackupRecord, String> {
    let connection = open_database(&app)?;
    let config = save_config_for_game(&connection, &game_id)?;
    let destination = validate_save_directory(Path::new(&config.save_path))?;

    let backup_path: String = connection
        .query_row(
            "SELECT backup_path FROM save_backups WHERE id = ?1 AND game_id = ?2",
            params![backup_id, game_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not load save backup: {error}"))?
        .ok_or_else(|| "Backup not found.".to_string())?;

    let source = PathBuf::from(backup_path);
    if !source.is_dir() {
        return Err("The backup files are missing from disk.".into());
    }

    // Always preserve the current saves before a destructive restore.
    let safety_backup = create_save_backup_internal(&app, &connection, &game_id, "pre-restore")?;

    clear_directory_contents(&destination)?;
    if let Err(error) = copy_directory_tree(&source, &destination) {
        return Err(format!(
            "Restore failed after creating safety backup {}: {}",
            safety_backup.id, error
        ));
    }

    Ok(safety_backup)
}

#[tauri::command]
async fn restore_save_backup(
    app: AppHandle,
    game_id: String,
    backup_id: String,
) -> Result<SaveBackupRecord, String> {
    tauri::async_runtime::spawn_blocking(move || {
        restore_save_backup_blocking(app, game_id, backup_id)
    })
    .await
    .map_err(|error| format!("Save restore worker failed: {error}"))?
}

#[tauri::command]
fn delete_save_backup(app: AppHandle, game_id: String, backup_id: String) -> Result<(), String> {
    let connection = open_database(&app)?;
    let backup_path: Option<String> = connection
        .query_row(
            "SELECT backup_path FROM save_backups WHERE id = ?1 AND game_id = ?2",
            params![backup_id, game_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| format!("Could not locate save backup: {error}"))?;

    if let Some(backup_path) = backup_path {
        let path = PathBuf::from(backup_path);
        if path.is_dir() {
            fs::remove_dir_all(&path)
                .map_err(|error| format!("Could not delete backup files: {error}"))?;
        }
    }

    connection
        .execute(
            "DELETE FROM save_backups WHERE id = ?1 AND game_id = ?2",
            params![backup_id, game_id],
        )
        .map_err(|error| format!("Could not delete backup record: {error}"))?;

    Ok(())
}

#[tauri::command]
fn data_directory(app: AppHandle) -> Result<String, String> {
    Ok(app_data_dir(&app)?.to_string_lossy().into_owned())
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if let Err(error) = open_database(app.handle()) {
                let diagnostic = format!(
                    "Dusk startup database initialization warning at {}\n{}\n",
                    now(),
                    error
                );
                let log_path = app
                    .path()
                    .app_data_dir()
                    .unwrap_or_else(|_| env::temp_dir().join("Dusk"))
                    .join("startup-error.log");
                if let Some(parent) = log_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                let _ = fs::write(log_path, diagnostic);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_games,
            scan_games,
            choose_executable,
            add_manual_game,
            set_favorite,
            rename_game,
            remove_game,
            choose_cover,
            launch_game,
            open_game_folder,
            import_screenshots,
            scan_screenshots,
            list_screenshots,
            delete_screenshot,
            list_collections,
            create_collection,
            delete_collection,
            set_collection_membership,
            collection_memberships,
            get_stats,
            list_achievements,
            choose_save_folder,
            get_save_config,
            clear_save_config,
            create_save_backup,
            list_save_backups,
            restore_save_backup,
            delete_save_backup,
            data_directory,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Dusk");
}

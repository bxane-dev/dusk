# Changelog

All notable Dusk changes are documented here.

## 1.6.1 - 2026-10-06

Account-screen and desktop hotfix.

- Restored minimize, maximize/restore, close, and drag controls while the login/register screen is open.
- Added a persistent Continue as guest path for users who want a local-only Dusk library without an account.
- Added a Settings path from guest mode back to sign-in.
- Fixed desktop Supabase configuration fallback so account login works even when release environment variables are missing.
- Kept only the public Supabase publishable key in the desktop client; privileged server credentials remain server-side.
- Cropped the approved Dusk emblem for UI, Windows icons, shortcuts, installer, and uninstaller so the logo is clearly visible.
- Disabled cloud-sync actions while running as a guest.

## 1.6.0 - 2026-10-05

Dusk accounts and cross-device account state.

- Added the official violet Dusk desktop logo to the title bar, sidebar, account screen, loading screen, executable, shortcuts, and About panel.
- Added a custom dark-violet NSIS installer with branded header/sidebar artwork and matching installer/uninstaller icons.
- Added a deterministic Windows branding pipeline that generates icon and installer assets from one approved logo source.
- Added a custom Dusk login and registration screen.
- Registration now uses email, username, and password; login uses username and password.
- Added Supabase-backed Dusk accounts with unique usernames and persistent sessions.
- Replaced anonymous per-install cloud identities with authenticated account-owned save storage.
- Added cross-device sync for owner profiles, library membership, favorites, playtime, launch counts, collections, and active profile.
- Added safe local merge behavior so device-specific executable paths and save-folder paths are never copied blindly from another PC.
- Added account sign-out from Settings.
- Added private per-account state storage with Row Level Security.
- Added a username login Edge Function that resolves usernames server-side without exposing privileged keys to the desktop client.
- Preserved private physical save-backup uploads under the authenticated user's Storage namespace.

## 1.5.0 - 2026-10-05

Installer hub and bxane branding.

- Added a safe local game-installer flow for user-selected .exe and .msi files.
- Added one-click browser links for Steam, Epic Games, GOG, and itch.io.
- Added a persistent clickable bxane creator pill linking to https://guns.lol/bxane.
- Physical owner save-file vaults: each profile stores actual save files separately for every configured game.
- Added Save to profile / Load profile files controls with pre-load safety backups.
- Updated package authorship/publisher branding to bxane.
- Dusk does not automatically download or install games from unofficial redistribution sites.
- Preserves automatic game discovery, Downloads scanning, hidden scan helper processes, custom window controls, and rapid-click stability protections from earlier releases.

## 1.0.4 - 2026-10-05

Desktop window and scan UX hardening.

- Replaced the native frame with a Dusk custom title bar.
- The default window is larger and centered on screen.
- The top title area can be dragged to move the window.
- Added working minimize, maximize/restore, and close controls.
- Double-clicking the draggable title area toggles maximize/restore.
- Internal Windows helper commands used for scanning and process tracking now run with CREATE_NO_WINDOW, eliminating flashing console windows.
- Downloads and Downloads/Games are now included in the default bounded device scan roots.
- Loose installer executables in Downloads are not blindly imported; Dusk scans contained game folders conservatively.
- Preserves v1.0.3 automatic background scanning and v1.0.2 rapid-click stability protections.

## 1.0.3 - 2026-10-05

Automatic discovery and responsiveness hardening.

- Dusk now automatically scans for games shortly after startup.
- Automatic rescans run about every 10 minutes while Dusk remains open and when the app becomes active after a stale scan.
- Added bounded discovery for common game folders across available Windows drives and common profile game folders.
- Device-folder scans run on blocking workers and use strict time, folder-count, depth, and entry limits.
- Added a Settings toggle for automatic device scanning plus last-scan status.
- Added a Device source/filter for games found outside supported launcher manifests.
- Manual and automatic scans share one hard lock so repeated clicks cannot stack scan jobs.
- Preserved the v1.0.2 anti-freeze changes: coalesced refreshes, click deduplication, SQLite busy handling, lazy local media loading, and background filesystem work.

## 1.0.2 - 2026-10-05

Stability and responsiveness hotfix.

- Database schema/migration initialization now runs once per process instead of on every command.
- Added SQLite busy timeout handling for short-lived concurrent access.
- Heavy game scans, screenshot scans, save backups, and save restores now run on blocking workers instead of tying up command handling.
- Rapid repeated clicks are deduplicated for scans, launches, favorites, collections, update checks, screenshot deletion, fullscreen changes, and game-detail actions.
- Core library refreshes and screenshot refreshes are coalesced instead of stacking overlapping requests.
- Favorites update optimistically without forcing a full database/library reload on every click.
- Repeated screenshot-page visits no longer reload all screenshot previews unnecessarily.
- File picker and add-game actions now have hard re-entry locks.
- Controller polling no longer recreates its animation loop when connection state changes.
- Toast timers no longer let older notifications clear newer ones.
- Media previews now load directly from Dusk's scoped local asset storage instead of sending full image files as base64 through IPC, sharply reducing memory and UI pressure.

## 1.0.1 - 2026-10-05

Startup hotfix.

- Fixed a Windows startup exit caused by registering the Tauri updater plugin in builds that did not contain updater signing/public-key configuration.
- Updater runtime registration is now injected only for properly signed updater builds.
- Added a CI smoke test that launches the built Windows executable and verifies it remains running.
- Pinned Tauri updater JavaScript/Rust versions to matching releases.

## 1.0.0 - 2026-10-05

First stable release.

### Library and launching
- Steam installed-game discovery from local manifests and library folders.
- Epic Games discovery from local manifests.
- GOG discovery from Windows install records.
- Common emulator detection.
- Manual Windows executable entries.
- Native launching and tracked play sessions.
- Search, filtering, sorting, favorites, and collections.

### Media and personalization
- Custom game covers.
- Automatic screenshot discovery for Steam, game folders, Windows screenshots, and Xbox Game Bar captures.
- Duplicate-safe local screenshot gallery.
- Night, OLED, and Slate themes.
- Violet, Ember, and Cyan accents.

### Saves and safety
- Per-game save-folder configuration.
- Manual restore points.
- Pre-restore safety snapshots.
- Protection from selecting broad system/profile folders as save roots.

### Controller and console mode
- Standard Gamepad API navigation.
- D-pad / stick directional focus.
- Controller select, back, and launch shortcuts.
- Native fullscreen console mode.
- Larger controller-friendly UI and focus states.

### Desktop and release
- Tauri 2 + React + TypeScript + Rust.
- Local SQLite persistence.
- NSIS Windows installer.
- Windows CI for frontend and Rust builds.
- Optional signed updater artifacts through Tauri.
- Optional Windows Authenticode signing when a certificate is configured.


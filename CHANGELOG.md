# Changelog

All notable Dusk changes are documented here.

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


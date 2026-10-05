# Changelog

All notable Dusk changes are documented here.

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


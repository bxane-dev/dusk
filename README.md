# Dusk

Dusk is a local-first Windows game launcher built with Tauri, Rust, React, TypeScript, and SQLite.

**Current stable version: 1.0.3**

It scans real local game installations, launches games, tracks sessions it can observe, and keeps the library database on the user's PC. Dusk does not ship with fake games or seeded playtime.

## Features

- Steam library scanning from local app manifests and library folders
- Epic Games scanning from local manifest files
- GOG scanning from local Windows install records
- Common emulator detection (Dolphin, PCSX2, RetroArch, Ryujinx, Cemu, PPSSPP, DuckStation)
- Automatic background game scanning on startup and at safe intervals
- Bounded common game-folder discovery across available Windows drives
- Manual game entries for any local Windows executable
- Native game launching
- Steam launch through the Steam protocol
- Playtime and session tracking when the game process can be observed
- Favorites
- Search, source filtering, and sorting
- Custom collections
- Custom cover images
- Automatic screenshot discovery from Steam, game folders, Windows Screenshots, and Xbox Game Bar Captures
- Imported local screenshot gallery with duplicate protection
- Per-game save-folder backups and restore points
- Automatic pre-restore safety snapshots
- Controller navigation using the standard Gamepad API
- Fullscreen console mode with controller focus and shortcuts
- Dusk achievements calculated from real local Dusk data
- Local stats dashboard
- Night, OLED, and Slate themes
- Violet, Ember, and Cyan accents
- SQLite persistence
- No cloud account required

## Stack

- Tauri 2
- Rust
- React 19
- TypeScript
- Vite
- SQLite via rusqlite

## Run in development

Requirements:

- Node.js
- Rust stable
- Windows WebView2 / Tauri Windows prerequisites

Commands:

    npm install
    npm run tauri dev

## Build the Windows installer

    npm install
    npm run tauri build

The Tauri config currently targets an NSIS Windows installer.

## Data

Dusk stores its SQLite database, copied cover images, and imported screenshots in the Tauri application data directory for dev.bxane.dusk.

Playtime is only recorded for sessions Dusk can actually track. If a Steam game's executable cannot be detected, Dusk can still request the Steam launch but explicitly reports that the session cannot be counted.

## Automatic detection

Current scanners:

- Steam
- Epic Games
- GOG
- Common emulator installations

Detected emulators are added as launchable library entries. ROM-specific launching will be handled separately; any other Windows game can still be added manually by selecting its executable.

## Screenshot detection

Dusk automatically checks:

- Steam userdata screenshot folders and matches them by Steam App ID
- Common `Screenshots` / `Captures` folders inside known game install directories
- `Pictures\Screenshots`
- `Videos\Captures` used by Xbox Game Bar

For shared Windows folders, Dusk only imports an image when the filename contains a sufficiently specific known game title. Imported source paths are tracked so repeated scans do not create duplicates. If an automatically found screenshot is removed from Dusk, its original source file is left untouched and Dusk remembers not to re-import it automatically.

## Save-game backups

Save backups are opt-in per game. Dusk does not guess save locations.

From a game's details panel:

1. Choose the exact save folder used by that game.
2. Create a restore point with **Back up now**.
3. Review backup time, file count, and size.
4. Restore any snapshot when needed.
5. Delete old Dusk snapshots without changing the live save folder.

Before every restore, Dusk first creates a `pre-restore` safety backup of the current saves. Restore operations replace the contents of the configured save folder with the selected snapshot.

Dusk rejects drive roots and broad Windows, Program Files, user-profile, and AppData roots as backup targets. Backups stay local under Dusk's application data directory.

## Controller and console mode

Dusk supports standard gamepads through the browser Gamepad API used by the Tauri webview.

Default controls:

- D-pad / left stick: move focus
- A / Cross: select the focused control
- X / Square: launch the focused game card
- B / Circle: go back / close the current panel
- Menu / Start: toggle fullscreen console mode
- F11: keyboard shortcut for console mode

Console mode switches the Tauri window into native fullscreen, collapses Dusk to an icon rail, increases card sizing and focus visibility, and shows an on-screen controller hint bar. The setting is stored locally.

## CI and releases

GitHub Actions checks the React build and Rust code on Windows.

The Windows release workflow can be started manually from GitHub Actions or from the dedicated `installer-build` branch. It publishes a stable NSIS installer. If updater-signing and Windows certificate secrets are configured, the same workflow also publishes signed updater artifacts and Authenticode-signs the Windows build.

## Stable release

Stable installers are published on GitHub Releases:

- Repository: https://github.com/bxane-dev/dusk
- Releases: https://github.com/bxane-dev/dusk/releases
- Changelog: `CHANGELOG.md`
- Signing setup: `RELEASE_SIGNING.md`

The release workflow never commits private signing material. Missing signing secrets cause the corresponding signing layer to be skipped rather than fabricated.

## Contributing

Contributions, forks, fixes, and feature work are welcome. See CONTRIBUTING.md.

## License

Dusk uses the Dusk Open Code License (DOCL) v1.0.

You may use, modify, fork, redistribute, and build on the code, including commercially. Public projects using a substantial portion of Dusk must credit bxane-dev and link back to the original source.

- Author: https://github.com/bxane-dev
- Source: https://github.com/bxane-dev/dusk

See LICENSE for the full terms.

DOCL is a custom open-code license and is not presented as an OSI-approved open-source license.

# Dusk

Dusk is a local-first Windows game launcher built with Tauri, Rust, React, TypeScript, and SQLite.

It scans real local game installations, launches games, tracks sessions it can observe, and keeps the library database on the user's PC. Dusk does not ship with fake games or seeded playtime.

## Features

- Steam library scanning from local app manifests and library folders
- Epic Games scanning from local manifest files
- Manual game entries for any local Windows executable
- Native game launching
- Steam launch through the Steam protocol
- Playtime and session tracking when the game process can be observed
- Favorites
- Search, source filtering, and sorting
- Custom collections
- Custom cover images
- Imported local screenshot gallery
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

Anything else can be added manually by selecting its executable.

## CI and releases

GitHub Actions checks the React build and Rust code on Windows.

The Windows release workflow can be started manually from GitHub Actions. It builds the Tauri installer and creates a draft GitHub Release.

## Contributing

Contributions, forks, fixes, and feature work are welcome. See CONTRIBUTING.md.

## License

Dusk uses the Dusk Open Code License (DOCL) v1.0.

You may use, modify, fork, redistribute, and build on the code, including commercially. Public projects using a substantial portion of Dusk must credit bxane-dev and link back to the original source.

- Author: https://github.com/bxane-dev
- Source: https://github.com/bxane-dev/dusk

See LICENSE for the full terms.

DOCL is a custom open-code license and is not presented as an OSI-approved open-source license.

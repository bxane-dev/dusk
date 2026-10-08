# Changelog

All notable Dusk changes are documented here.

## 1.7.4 - 2026-10-09

Build fix for profile-picture persistence and artwork fallback.

- Fixed a Rust compile collision caused by two functions using the same normalized-match helper name.
- Enabled reqwest query support required by the fallback-only MediaWiki artwork lookup.
- Includes persistent Dusk profile pictures from v1.7.3.
- Includes local-first portrait artwork and non-Steam web fallback from v1.7.2.
- Keeps scanner deduplication, stronger executable detection, DuskLauncher.exe Discord mitigation, and optional Discord Rich Presence.

## 1.7.3 - 2026-10-09

Persistent Dusk profile pictures.

- Dusk now saves each account avatar's stable private-storage path in the authenticated account metadata.
- Profile pictures persist reliably across app restarts, sign-outs, and later sign-ins.
- Signed avatar URLs are regenerated from the saved private object path instead of rediscovering the file on every launch.
- Existing avatars from earlier Dusk versions are detected once and migrated automatically to the persistent avatar-path format.
- Avatar signed URLs now last up to seven days while the underlying private avatar remains permanently stored until the user removes it.
- Removing an avatar deletes the private storage object and clears the saved account metadata path.
- Keeps the local-first artwork lookup and scanner fixes from v1.7.2.

## 1.7.2 - 2026-10-09

Fallback-only web artwork lookup for non-Steam games.

- Keeps Dusk local-first: manual covers are preserved, launcher-native/local portrait art is preferred, and web lookup only runs when those sources fail.
- Added non-Steam web artwork fallback using title plus detected publisher/company metadata when available.
- Uses Wikipedia/MediaWiki as a keyless metadata source instead of arbitrary image scraping.
- Requires strong title matching, video-game context, portrait-oriented artwork, and extra publisher/company agreement for weaker title matches.
- Rejects ambiguous, landscape, square, tiny, or non-HTTPS image results instead of guessing.
- Caches accepted fallback artwork locally so Dusk does not re-download it on every launch.
- Steam keeps its official portrait library-art fallback, but only after local Steam cache and local install-folder artwork fail.
- Existing scanner deduplication, portrait-art filtering, and manual-cover preservation from v1.7.1 remain included.

## 1.7.1 - 2026-10-08

Game library cleanup and portrait artwork hotfix.

- Replaced wide Steam header artwork with portrait Steam library artwork suited to Dusk game cards.
- Prefer locally cached Steam 600x900 library artwork before downloading an official Steam portrait fallback.
- Local artwork detection now favors covers, posters, key art, box art, capsules, and vertical images while rejecting wide headers, heroes, banners, logos, and icons.
- Added scanner deduplication by executable/install location with launcher manifests preferred over generic device-folder matches.
- Existing stale device-scan duplicates that match Steam, Epic, or GOG installs are hidden on the next scan.
- Added cover-origin tracking so user-selected manual covers are preserved while old automatic artwork can refresh to the improved portrait format.
- Keeps the stronger executable filtering and expanded Windows library scanning from v1.7.0.

## 1.7.0 - 2026-10-08

Stronger installed-game detection and automatic per-game artwork.

- Expanded automatic Windows game-folder discovery to include common Xbox, EA, Ubisoft, portable, and custom game-library locations.
- Tightened executable selection so Dusk is less likely to pick uninstallers, anti-cheat helpers, redistributables, updaters, launch helpers, or service executables as the game binary.
- Added automatic local artwork discovery using installed cover, poster, key art, capsule, header, hero, banner, and library images when available.
- Steam games now automatically use the game's own official Steam header artwork when no user-selected cover already exists.
- Existing manually selected covers are preserved and never overwritten by automatic artwork.
- Detection remains local-first; artwork falls back gracefully when a launcher or game does not expose usable assets.

## 1.6.11 - 2026-10-08

Optional Dusk Discord Rich Presence.

- Added optional Discord Rich Presence support using Discord IPC.
- Rich Presence is disabled by default and never connects unless the user enables it.
- Added a Discord Rich Presence card in Settings with a Discord Application ID field.
- When enabled, Dusk identifies itself intentionally as the Dusk desktop launcher instead of relying on generic process detection.
- Presence states include Browsing library, Viewing screenshots, Viewing achievements, Adjusting Dusk settings, and Launching a game.
- Dusk uses a non-game-style Watching activity type rather than publishing itself as a played game.
- Disabling Rich Presence clears the activity and closes the Discord IPC connection.
- Dusk still cannot override Discord's separate Registered Games process scanner; users can independently disable Dusk there in Discord.
- Includes the v1.6.10 live window/taskbar crescent icon and DuskLauncher.exe process identity changes.

## 1.6.10 - 2026-10-08

Windows app icon and Discord detection cleanup.

- Explicitly sets the live Dusk window/taskbar icon from the official crescent PNG at runtime.
- Enabled Tauri PNG decoding and the window set-icon permission for the packaged app.
- Keeps the crescent embedded in the executable, shortcuts, installer, and uninstaller.
- Renamed the main Windows process binary from dusk.exe to DuskLauncher.exe while keeping the installed product, shortcut, and UI name as Dusk.
- Updated CI and release startup tests for the new executable filename.
- Dusk has no Discord RPC integration and does not publish Discord activity itself; the executable rename is intended to reduce Discord's automatic false game detection.

## 1.6.9 - 2026-10-08

Official transparent crescent branding.

- Replaced the previous Dusk desktop mark with the user-approved crescent logo.
- The in-app logo now uses a smooth transparent SVG instead of a raster tile.
- Removed the artificial dark/violet background behind the in-app logo.
- Added official violet, cyan, and red transparent SVG variants under the brand folder.
- Added generated violet, cyan, and red transparent PNG variants to the repository.
- Regenerated Windows icon sizes and icon.ico from the same crescent geometry.
- Regenerated installer and uninstaller header/sidebar artwork with the new crescent.
- Added subtle cyan and red installer accent rails while keeping violet as Dusk's primary color.
- Added a Brand Assets workflow that regenerates and commits raster/Windows assets when the vector branding changes.
- App, title-bar, login/register screen, executable, shortcuts, installer, and uninstaller now share the same crescent identity.

## 1.6.8 - 2026-10-07

Profile UI cleanup.

- Removed the Profiles item from the main sidebar navigation.
- Removed the standalone Profiles page.
- Removed the bottom owner-profile dropdown and add-profile button from the sidebar.
- Existing owner-profile data, libraries, saves, backups, and switching logic are preserved.
- Profile management remains available in Settings, so no existing data is deleted.

## 1.6.7 - 2026-10-07

Custom Dusk account avatars.

- Added user-uploaded profile avatars for signed-in Dusk accounts.
- Avatar uploads accept image files up to 150 MB.
- Added a private Supabase Storage bucket dedicated to Dusk avatars.
- Bucket-level file-size and image MIME-type restrictions enforce the avatar upload rules server-side.
- Each user can only read, upload, replace, and delete the avatar stored under their own authenticated user ID.
- Avatar images are displayed as round profile pictures using a centered square crop with object-fit cover.
- Added upload, change, and remove avatar controls to Dusk account Settings.
- Added the signed-in account avatar, display name, and username to the Dusk sidebar.
- If no avatar exists, Dusk falls back to the account's initial.
- Avatar/name changes update the sidebar immediately without restarting Dusk.
- Avatar files remain private and are displayed through short-lived signed URLs.

## 1.6.6 - 2026-10-07

Password recovery for Dusk accounts.

- Added a visible Forgot password? action to the Dusk sign-in screen.
- Users can enter the email attached to their Dusk account and request a Supabase recovery email.
- Recovery requests use generic messaging so Dusk does not reveal whether an email address has an account.
- Added a hosted dark-violet Dusk password-reset page.
- The recovery page validates the temporary recovery session before changing the password.
- New passwords must be at least 8 characters and must be confirmed before submission.
- After resetting, the user signs in normally with the same Dusk username and the new password.
- The reset page does not expose the Supabase service-role key and does not store passwords.
- The hosted recovery flow avoids using the old localhost page as the intended reset destination.

## 1.6.5 - 2026-10-07

Editable Dusk accounts and owner profiles.

- Added editable Dusk account display names and usernames.
- Added editable account email addresses.
- Added secure password changes that require the current password.
- Passwords remain managed by Supabase Auth and are never stored by Dusk.
- Username changes keep username login working with the new username.
- Email changes update the authenticated Dusk account immediately without the old localhost confirmation redirect.
- Registration now asks for a display name as well as email, username, and password.
- Added a dedicated account-profile editor in Settings with separate Profile and Security sections.
- Added rename controls for local owner profiles on both the Profiles page and Settings.
- Owner-profile renames sync through Dusk account state.
- Account-table writes are restricted to the validated server-side account-update endpoint; the desktop client only receives read access.
- Includes the persistent per-profile achievements introduced in the 1.6.4 code line.

## 1.6.4 - 2026-10-07

Persistent, profile-owned achievements.

- Achievements are now saved in Dusk instead of being display-only calculations.
- Achievement progress is stored separately for each owner profile.
- Saved progress uses a high-water mark, so earned progress never moves backwards if games, screenshots, or local history are removed later.
- Once an achievement is unlocked, it stays unlocked.
- Achievement progress and unlock state are included in Dusk account sync and merge across devices.
- Cross-device merges keep the highest progress/unlocked state instead of overwriting it with lower progress.
- Existing profiles automatically create saved achievement records from their current Dusk stats the next time achievements are loaded or account state is synced.
- Deleting a profile also removes only that profile's saved achievement records.

## 1.6.3 - 2026-10-07

Profiles are now directly available from the main Dusk navigation.

- Added a dedicated Profiles page to the sidebar.
- Added visible cards for every owner profile.
- Shows the active profile, profile count, current library size, per-profile game count, backup count, and last-used date.
- Switch profiles directly from the Profiles page.
- Create new profiles from the page header or the dedicated new-profile card.
- Delete non-required profiles directly from the page.
- Open the active profile's library with one click.
- Preserved the compact profile switcher in the sidebar and the advanced profile controls in Settings.
- Profiles continue to keep owner libraries, collections, save folders, backup vaults, and synced account state separate.

## 1.6.2 - 2026-10-07

Registration, updater, and logo hotfix.

- Removed the broken browser-based email confirmation redirect that pointed to localhost.
- New Dusk accounts are verified server-side and signed into the desktop app immediately.
- Added a Dusk-branded Supabase confirmation email template to the repository for future confirmation flows.
- Replaced the optional Tauri updater-plugin runtime path with a built-in GitHub Releases updater.
- Dusk now checks the latest release, compares versions, downloads the official Windows x64 installer, verifies the GitHub SHA-256 digest when available, launches the installer, and closes the current app.
- Removed the misleading "plugin updater not found" failure path.
- Fixed the official Dusk logo pipeline to use the full approved emblem instead of cropping away the left side of the D.
- Kept guest mode, account sync, custom window controls, and the branded installer.

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


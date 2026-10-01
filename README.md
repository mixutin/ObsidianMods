<div align="center">

# ◆ Obsidian Mods

### A modern mod platform for Minecraft Dungeons II

Discover, install, update and manage mods from one native app — with a real **F8 menu rendered inside the game**.

[Website](https://mixutin.github.io/ObsidianMods/) ·
[Discover Mods](https://mixutin.github.io/ObsidianMods/discover.html) ·
[Download Manager](https://mixutin.github.io/ObsidianMods/manager.html) ·
[GitHub Releases](https://github.com/mixutin/ObsidianMods/releases)

**⭐ If Obsidian Mods helps you, please star the repository. It genuinely motivates me to keep building more Minecraft Dungeons II mods, compatibility fixes and manager features.**

[![GitHub stars](https://img.shields.io/github/stars/mixutin/ObsidianMods?style=social)](https://github.com/mixutin/ObsidianMods/stargazers)

![CI](https://github.com/mixutin/ObsidianMods/actions/workflows/ci.yml/badge.svg)
![Catalog](https://github.com/mixutin/ObsidianMods/actions/workflows/catalog.yml/badge.svg)
![Windows](https://img.shields.io/badge/Windows-10%20%2F%2011-6f42c1)
![Linux](https://img.shields.io/badge/Linux-AppImage%20%7C%20DEB%20%7C%20RPM-6f42c1)
![License](https://img.shields.io/github/license/mixutin/ObsidianMods)

</div>

---

## What is Obsidian Mods?

Obsidian Mods is a Minecraft Dungeons II mod ecosystem built around three pieces:

- **Obsidian Mods Manager** — a native Rust desktop app for Windows and Linux.
- **Obsidian Mods Website** — Discover, mod pages, downloads and one-click installs.
- **Obsidian Native Runtime** — a binary runtime loaded inside Dungeons II that renders the F8 menu directly in the game's DirectX swap chain.

The manager handles the annoying parts — game detection, package verification, backups, updates, profiles and loader state — so installing a mod does not mean manually editing game folders.

## Highlights

| Feature | Status |
| --- | --- |
| Discover catalog | ✅ |
| \`obsidianmods://\` one-click installs | ✅ |
| SHA-256 package verification | ✅ |
| Native Obsidian DLL mods | ✅ |
| PAK / UTOC / UCAS mods | ✅ |
| UE4SS compatibility | ✅ |
| Enable / disable / uninstall | ✅ |
| Automatic backups | ✅ |
| Saved mod profiles | ✅ |
| Modded / Vanilla mode | ✅ |
| Native F8 in-game menu | ✅ |
| Runtime mod-health reporting | ✅ |
| Windows manager | ✅ |
| Linux manager | ✅ |
| Automatic mod updates | 🚧 |
| Native Cutscene FPS Unlock | 🧪 Experimental |

## In-game F8 menu

Obsidian Native Runtime renders directly into Dungeons II using the game's Direct3D swap chain.

Press **F8** to open the menu and see:

- active native mods
- disabled mods
- loader failures
- renderer/runtime information
- current launch mode
- present FPS and frame-time telemetry
- compatibility problems

The menu does **not** require UE4SS to initialize, so it can still report that UE4SS failed rather than disappearing with it.

## Manager

The desktop manager currently supports:

- automatic Minecraft Dungeons II Steam detection
- Windows and Linux
- Obsidian native plugins
- UE4SS mods
- PAK / UTOC / UCAS mods
- website-backed Discover catalog
- verified one-click installation
- \`obsidianmods://install/<id>\` deep links
- mod enable / disable
- uninstall with rollback backups
- saved mod profiles
- Modded / Vanilla launch mode
- update detection
- native runtime installation and updates
- direct game launch

### Downloads

| Platform | Package |
| --- | --- |
| Windows | Setup \`.exe\` |
| Windows | Portable \`.zip\` |
| Linux | AppImage |
| Debian / Ubuntu / Mint / Pop!_OS | \`.deb\` |
| Fedora / Nobara / RPM-family | \`.rpm\` |
| Linux universal | \`.tar.gz\` |

Use the [Manager page](https://mixutin.github.io/ObsidianMods/manager.html) or [latest GitHub release](https://github.com/mixutin/ObsidianMods/releases/latest).

## Architecture

\`\`\`text
                    Obsidian Mods Website
                           │
                     catalog.json
                           │
             ┌─────────────┴─────────────┐
             │                           │
          Discover                  Desktop Manager
                                         │
                                 SHA-256 verification
                                         │
                       ┌─────────────────┼─────────────────┐
                       │                 │                 │
                 Native plugins      PAK / IoStore      UE4SS
                       │
                Obsidian Runtime
                       │
               Direct3D 11 / 12
                       │
                 F8 in-game menu
\`\`\`

## Binary-only mod distribution

The public platform is open, but individual Obsidian mod implementations do not have to be.

Public mod releases may contain only:

- compiled DLLs
- Lua bytecode
- PAK / IoStore containers
- package metadata
- required third-party license notices
- SHA-256 checksums

Individual Obsidian mod source and the native runtime implementation are maintained in a private build repository. Binary assets are published through [\`mixutin/ObsidianMods-Builds\`](https://github.com/mixutin/ObsidianMods-Builds).

See the [package format documentation](https://mixutin.github.io/ObsidianMods/package-format.html).

## Catalog security

Discover installs are deliberately stricter than manual installs.

- catalog assets must use HTTPS
- catalog builds must come from the official \`ObsidianMods-Builds\` GitHub Releases path
- every package includes a SHA-256 hash
- the manager verifies the complete archive before extraction
- archive download and expanded-size limits protect against oversized packages
- catalog CI downloads published assets and validates their hashes
- mods can be marked **stable**, **experimental**, or **temporarily blocked**

Manual ZIP installs are still available when a user deliberately chooses a local package.

## Repository layout

\`\`\`text
src/                  Rust desktop manager
docs/                 GitHub Pages website + catalog
scripts/              Catalog validation tooling
assets/               Packaging and desktop integration
.github/workflows/     CI, releases, Pages and catalog checks
\`\`\`

## Development

\`\`\`bash
cargo check
cargo run
\`\`\`

List detected mods without opening the GUI:

\`\`\`bash
cargo run -- --list
\`\`\`

Validate the public catalog:

\`\`\`bash
python3 scripts/validate_catalog.py
\`\`\`

## Disclaimer

Obsidian Mods is a community project and is not affiliated with Mojang Studios or Microsoft.

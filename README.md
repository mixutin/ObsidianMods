# Obsidian Mods

Obsidian Mods is a Minecraft Dungeons II mod distribution site and cross-platform mod manager.

## What is public

- The Obsidian Mods website and catalog format
- The desktop manager source
- Release automation and packaging
- Binary-only mod release assets and SHA-256 hashes

## What is not public

Individual Obsidian mod implementation source is kept in a private build repository. Public mod releases contain only the files needed to run the mod.

## Manager

The Rust desktop manager currently supports:

- Minecraft Dungeons II Steam auto-detection
- Windows and Linux
- UE4SS and PAK/UTOC/UCAS mods
- Discover catalog backed by the Obsidian Mods website
- SHA-256 verified one-click installation
- obsidianmods://install/<id> website deep links
- Enable, disable, uninstall and automatic backups
- Saved mod profiles
- Modded / Vanilla launch mode
- F8 always-on-top mod-health overlay
- Direct game launch

## Distribution targets

GitHub Actions builds:

- Windows x64 installer with obsidianmods:// protocol registration
- Windows x64 portable ZIP
- Linux x64 AppImage
- Debian/Ubuntu .deb
- Fedora/RHEL-compatible .rpm
- Portable Linux .tar.gz

The AppImage is the distro-independent Linux build.

## Catalog security

Catalog packages must use HTTPS and publish a SHA-256 hash. The desktop manager downloads to its local cache, verifies the complete archive, and only then installs it.

## In-game overlay

The manager can launch an F8 overlay alongside Minecraft Dungeons II. It is a separate always-on-top process rather than an injected game component, so the Obsidian UI does not depend on UE4SS successfully initializing. The overlay reports managed mod state and loader failures while the game is running.

Experimental mod implementation source stays in the private build repository. Public mod releases contain only packaged builds, metadata and checksums. The public website never contains individual mod source code.

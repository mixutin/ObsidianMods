# Contributing to Obsidian Mods

Thanks for helping improve Minecraft Dungeons II modding.

## Manager and website changes

1. Fork the repository.
2. Create a focused branch.
3. Run `cargo fmt --all`.
4. Run `cargo check --locked`.
5. Run `python3 scripts/validate_catalog.py`.
6. Open a pull request describing what changed and why.

## Catalog changes

Catalog entries must:
- use a unique lowercase kebab-case ID
- point to an official ObsidianMods-Builds GitHub Release asset
- include the exact SHA-256
- declare loader and game-build compatibility
- use stable, experimental, or blocked status honestly

## Mod source

This public repository does not require individual mod implementation source. Public mod releases may contain binary/runtime assets, metadata and required license notices.

## Compatibility reports

Include your game build, manager version, runtime version, OS, Proton version when relevant, and the exact mod version.

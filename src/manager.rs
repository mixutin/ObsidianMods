use crate::model::{Catalog, CatalogMod, GamePaths, InstalledMod, ModKind, ModProfile};
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write, copy};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use sysinfo::System;
use walkdir::WalkDir;

const APP_ID: &str = "1912410";
const GAME_NAME: &str = "Minecraft Dungeons II";
const CATALOG_URL: &str = "https://mixutin.github.io/ObsidianMods/catalog.json";

#[derive(Default, Deserialize)]
struct ModMeta {
    id: Option<String>,
    name: Option<String>,
    version: Option<String>,
    author: Option<String>,
    description: Option<String>,
}

pub struct ModManager {
    pub paths: GamePaths,
    pub data_root: PathBuf,
}

impl ModManager {
    pub fn detect() -> Result<Self> {
        let game =
            detect_game_dir().ok_or_else(|| anyhow!("Minecraft Dungeons II was not found"))?;
        let win64 = game.join("Dungeons/Binaries/Win64");
        let ue4ss_mods = win64.join("ue4ss/Mods");
        let ue4ss_mods_txt = ue4ss_mods.join("mods.txt");
        let paks = game.join("Dungeons/Content/Paks");
        let paks_enabled = paks.join("~mods");
        let paks_disabled = paks.join("~mods_disabled");
        let data_root = dirs::data_local_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")))
            .join("ObsidianMods");

        fs::create_dir_all(&data_root)?;
        fs::create_dir_all(&paks_enabled)?;
        fs::create_dir_all(&paks_disabled)?;

        let manager = Self {
            paths: GamePaths {
                game,
                win64,
                ue4ss_mods,
                ue4ss_mods_txt,
                paks_enabled,
                paks_disabled,
            },
            data_root,
        };

        Ok(manager)
    }

    pub fn scan(&self) -> Result<Vec<InstalledMod>> {
        let mut mods = Vec::new();
        mods.extend(self.scan_ue4ss()?);
        mods.extend(self.scan_paks(&self.paths.paks_enabled, true)?);
        mods.extend(self.scan_paks(&self.paths.paks_disabled, false)?);
        mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(mods)
    }

    pub fn profiles(&self) -> Result<Vec<ModProfile>> {
        let dir = self.data_root.join("profiles");
        fs::create_dir_all(&dir)?;
        let mut profiles = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            let text = fs::read_to_string(&path)?;
            if let Ok(profile) = serde_json::from_str::<ModProfile>(&text) {
                profiles.push(profile);
            }
        }

        profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(profiles)
    }

    pub fn save_profile(&self, name: &str, mods: &[InstalledMod]) -> Result<()> {
        let name = name.trim();
        if name.is_empty() {
            return Err(anyhow!("Profile name cannot be empty"));
        }

        let dir = self.data_root.join("profiles");
        fs::create_dir_all(&dir)?;
        let profile = ModProfile {
            name: name.to_string(),
            mods: mods
                .iter()
                .map(|item| (item.id.clone(), item.enabled))
                .collect(),
        };

        let path = dir.join(format!("{}.json", slugify(name)));
        fs::write(path, serde_json::to_string_pretty(&profile)?)?;
        Ok(())
    }

    pub fn apply_profile(&self, profile: &ModProfile, mods: &[InstalledMod]) -> Result<()> {
        for item in mods {
            if let Some(enabled) = profile.mods.get(&item.id) {
                if *enabled != item.enabled {
                    self.set_enabled(item, *enabled)?;
                }
            }
        }
        Ok(())
    }

    pub fn delete_profile(&self, profile: &ModProfile) -> Result<()> {
        let path = self
            .data_root
            .join("profiles")
            .join(format!("{}.json", slugify(&profile.name)));
        if path.is_file() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    pub fn fetch_catalog(&self) -> Result<Catalog> {
        let response = ureq::get(CATALOG_URL)
            .set("User-Agent", "ObsidianModsManager/0.2")
            .call()
            .context("Could not reach the Obsidian Mods catalog")?;
        let text = response.into_string()?;
        let catalog: Catalog = serde_json::from_str(&text)?;
        if catalog.schema != 1 {
            return Err(anyhow!("Unsupported catalog schema {}", catalog.schema));
        }
        Ok(catalog)
    }

    pub fn install_catalog_mod(&self, item: &CatalogMod) -> Result<Vec<String>> {
        if item.status.as_deref() == Some("blocked") {
            return Err(anyhow!(
                "{} is temporarily blocked: {}",
                item.name,
                item.status_note
                    .as_deref()
                    .unwrap_or("the current game build is not supported")
            ));
        }
        if !item.download_url.starts_with("https://") {
            return Err(anyhow!("Refusing a non-HTTPS mod download"));
        }
        if item.loader.eq_ignore_ascii_case("ue4ss") && !self.ue4ss_installed() {
            return Err(anyhow!(
                "This mod requires UE4SS, but UE4SS is not installed"
            ));
        }

        let downloads = self.data_root.join("downloads");
        fs::create_dir_all(&downloads)?;
        let archive = downloads.join(format!("{}-{}.zip", item.id, item.version));
        let response = ureq::get(&item.download_url)
            .set("User-Agent", "ObsidianModsManager/0.2")
            .call()
            .with_context(|| format!("Could not download {}", item.name))?;
        let mut reader = response.into_reader();
        let mut file = File::create(&archive)?;
        let mut hasher = Sha256::new();
        let mut buf = [0u8; 64 * 1024];

        loop {
            let n = reader.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n])?;
        }
        file.flush()?;

        let actual = hex::encode(hasher.finalize());
        if !actual.eq_ignore_ascii_case(item.sha256.trim()) {
            let _ = fs::remove_file(&archive);
            return Err(anyhow!(
                "SHA-256 mismatch for {} (expected {}, got {})",
                item.name,
                item.sha256,
                actual
            ));
        }

        self.install_archive(&archive)
    }

    fn scan_ue4ss(&self) -> Result<Vec<InstalledMod>> {
        let mut out = Vec::new();
        if !self.paths.ue4ss_mods.is_dir() {
            return Ok(out);
        }

        let states = read_ue4ss_states(&self.paths.ue4ss_mods_txt);
        let builtins = [
            "shared",
            "Keybinds",
            "ConsoleEnablerMod",
            "ConsoleCommandsMod",
            "CheatManagerEnablerMod",
            "LineTraceMod",
            "SplitScreenMod",
            "BPML_GenericFunctions",
            "BPModLoaderMod",
            "ObsidianRuntime",
        ];

        for entry in fs::read_dir(&self.paths.ue4ss_mods)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let folder = entry.file_name().to_string_lossy().to_string();
            if builtins.iter().any(|x| x.eq_ignore_ascii_case(&folder)) {
                continue;
            }

            let root = entry.path();
            let meta = read_meta(&root);
            out.push(InstalledMod {
                id: meta.id.unwrap_or_else(|| slugify(&folder)),
                name: meta.name.unwrap_or_else(|| folder.clone()),
                version: meta.version,
                author: meta.author,
                description: meta.description,
                kind: ModKind::Ue4ss,
                enabled: states.get(&folder).copied().unwrap_or(true),
                root: root.clone(),
                files: vec![root],
            });
        }
        Ok(out)
    }

    fn scan_paks(&self, dir: &Path, enabled: bool) -> Result<Vec<InstalledMod>> {
        let mut groups: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
        if !dir.is_dir() {
            return Ok(Vec::new());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path
                .extension()
                .and_then(|x| x.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !matches!(ext.as_str(), "pak" | "utoc" | "ucas") {
                continue;
            }
            let stem = path
                .file_stem()
                .and_then(|x| x.to_str())
                .unwrap_or("Unknown");
            let key = stem.trim_end_matches("_P").to_string();
            groups.entry(key).or_default().push(path);
        }
        Ok(groups
            .into_iter()
            .map(|(name, files)| InstalledMod {
                id: slugify(&name),
                name,
                version: None,
                author: None,
                description: None,
                kind: ModKind::Pak,
                enabled,
                root: dir.to_path_buf(),
                files,
            })
            .collect())
    }

    pub fn install_archive(&self, archive_path: &Path) -> Result<Vec<String>> {
        if archive_path
            .extension()
            .and_then(|x| x.to_str())
            .map(|x| x.eq_ignore_ascii_case("zip"))
            != Some(true)
        {
            return Err(anyhow!(
                "For now, Obsidian Mods Manager installs .zip mod packages"
            ));
        }

        let cache = self
            .data_root
            .join("cache")
            .join(format!("install-{}", epoch()));
        fs::create_dir_all(&cache)?;
        extract_zip(archive_path, &cache)?;

        let mut installed = Vec::new();
        let mut ue4ss_roots = Vec::<PathBuf>::new();
        for entry in WalkDir::new(&cache).into_iter().filter_map(Result::ok) {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }

            let filename = p.file_name().and_then(|x| x.to_str());
            if filename == Some("main.lua") {
                if let Some(scripts) = p.parent() {
                    if scripts.file_name().and_then(|x| x.to_str()) == Some("Scripts") {
                        if let Some(root) = scripts.parent() {
                            ue4ss_roots.push(root.to_path_buf());
                        }
                    }
                }
            }

            if filename == Some("main.dll") {
                if let Some(dlls) = p.parent() {
                    if dlls.file_name().and_then(|x| x.to_str()) == Some("dlls") {
                        if let Some(root) = dlls.parent() {
                            ue4ss_roots.push(root.to_path_buf());
                        }
                    }
                }
            }
        }
        ue4ss_roots.sort();
        ue4ss_roots.dedup();

        for root in ue4ss_roots {
            let name = root
                .file_name()
                .and_then(|x| x.to_str())
                .unwrap_or("ObsidianMod")
                .to_string();
            let dest = self.paths.ue4ss_mods.join(&name);
            if dest.exists() {
                self.backup_path(&dest, &name)?;
                fs::remove_dir_all(&dest)?;
            }
            copy_dir(&root, &dest)?;
            self.set_ue4ss_state(&name, true)?;
            installed.push(format!("{name} (UE4SS)"));
        }

        let shared = cache.join("shared");
        if shared.is_dir() {
            for entry in WalkDir::new(&shared).into_iter().filter_map(Result::ok) {
                if !entry.file_type().is_file() {
                    continue;
                }
                let rel = entry.path().strip_prefix(&shared)?;
                let dest = self.paths.ue4ss_mods.join("shared").join(rel);
                if dest.exists() {
                    self.backup_path(
                        &dest,
                        &format!("shared-{}", slugify(&rel.to_string_lossy())),
                    )?;
                }
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(entry.path(), &dest)?;
            }
        }

        let pak_files: Vec<PathBuf> = WalkDir::new(&cache)
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| e.into_path())
            .filter(|p| {
                p.is_file()
                    && matches!(
                        p.extension()
                            .and_then(|x| x.to_str())
                            .unwrap_or("")
                            .to_lowercase()
                            .as_str(),
                        "pak" | "utoc" | "ucas"
                    )
            })
            .collect();
        if !pak_files.is_empty() {
            fs::create_dir_all(&self.paths.paks_enabled)?;
            for source in pak_files {
                let name = source
                    .file_name()
                    .ok_or_else(|| anyhow!("Invalid mod filename"))?;
                let dest = self.paths.paks_enabled.join(name);
                if dest.exists() {
                    self.backup_path(&dest, &name.to_string_lossy())?;
                }
                fs::copy(&source, &dest)?;
                installed.push(name.to_string_lossy().to_string());
            }
        }

        let _ = fs::remove_dir_all(&cache);
        if installed.is_empty() {
            return Err(anyhow!(
                "No UE4SS mod (Lua/C++) or PAK/UTOC/UCAS mod was found in this archive"
            ));
        }
        Ok(installed)
    }

    pub fn set_enabled(&self, item: &InstalledMod, enabled: bool) -> Result<()> {
        match item.kind {
            ModKind::Ue4ss => self.set_ue4ss_state(
                item.root
                    .file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or(&item.name),
                enabled,
            ),
            ModKind::Pak => {
                let dest_dir = if enabled {
                    &self.paths.paks_enabled
                } else {
                    &self.paths.paks_disabled
                };
                fs::create_dir_all(dest_dir)?;
                for file in &item.files {
                    let name = file
                        .file_name()
                        .ok_or_else(|| anyhow!("Invalid mod filename"))?;
                    fs::rename(file, dest_dir.join(name))?;
                }
                Ok(())
            }
        }
    }

    pub fn uninstall(&self, item: &InstalledMod) -> Result<()> {
        self.backup_mod(item)?;
        match item.kind {
            ModKind::Ue4ss => {
                if item.root.is_dir() {
                    fs::remove_dir_all(&item.root)?;
                }
                let folder = item
                    .root
                    .file_name()
                    .and_then(|x| x.to_str())
                    .unwrap_or(&item.name);
                self.remove_ue4ss_state(folder)?;
            }
            ModKind::Pak => {
                for file in &item.files {
                    if file.exists() {
                        fs::remove_file(file)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn backup_mod(&self, item: &InstalledMod) -> Result<()> {
        let base = self.backup_dir(&item.name)?;
        match item.kind {
            ModKind::Ue4ss => copy_dir(&item.root, &base.join("mod"))?,
            ModKind::Pak => {
                fs::create_dir_all(&base)?;
                for file in &item.files {
                    if file.exists() {
                        fs::copy(file, base.join(file.file_name().unwrap()))?;
                    }
                }
            }
        }
        Ok(())
    }

    fn backup_path(&self, path: &Path, name: &str) -> Result<()> {
        let base = self.backup_dir(name)?;
        if path.is_dir() {
            copy_dir(path, &base.join("previous"))?;
        } else if path.is_file() {
            fs::create_dir_all(&base)?;
            fs::copy(path, base.join(path.file_name().unwrap()))?;
        }
        Ok(())
    }

    fn backup_dir(&self, name: &str) -> Result<PathBuf> {
        let safe: String = name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || "-_.".contains(c) {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let dir = self
            .data_root
            .join("backups")
            .join(format!("{}-{}", epoch(), safe));
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    fn set_ue4ss_state(&self, name: &str, enabled: bool) -> Result<()> {
        fs::create_dir_all(&self.paths.ue4ss_mods)?;
        let old = fs::read_to_string(&self.paths.ue4ss_mods_txt).unwrap_or_default();
        let state_line = format!("{name} : {}", if enabled { 1 } else { 0 });
        let mut lines: Vec<String> = old
            .lines()
            .filter(|line| {
                line.trim()
                    .split_once(':')
                    .map(|(left, _)| !left.trim().eq_ignore_ascii_case(name))
                    .unwrap_or(true)
            })
            .map(str::to_string)
            .collect();

        if name.eq_ignore_ascii_case("ObsidianRuntime") {
            lines.insert(0, state_line);
        } else if let Some(pos) = lines
            .iter()
            .position(|line| line.trim().starts_with("; Built-in keybinds"))
        {
            lines.insert(pos, state_line);
        } else {
            lines.push(state_line);
        }

        fs::write(
            &self.paths.ue4ss_mods_txt,
            format!("{}\n", lines.join("\n")),
        )?;
        Ok(())
    }

    fn remove_ue4ss_state(&self, name: &str) -> Result<()> {
        let old = fs::read_to_string(&self.paths.ue4ss_mods_txt).unwrap_or_default();
        let kept: Vec<&str> = old
            .lines()
            .filter(|line| {
                line.trim()
                    .split_once(':')
                    .map(|(left, _)| !left.trim().eq_ignore_ascii_case(name))
                    .unwrap_or(true)
            })
            .collect();
        fs::write(&self.paths.ue4ss_mods_txt, format!("{}\n", kept.join("\n")))?;
        Ok(())
    }

    pub fn ue4ss_installed(&self) -> bool {
        self.paths.win64.join("ue4ss/UE4SS.dll").is_file()
    }

    pub fn vanilla_mode(&self) -> bool {
        self.paths
            .win64
            .join("dwmapi.obsidian-disabled.dll")
            .is_file()
    }

    pub fn set_vanilla_mode(&self, enabled: bool) -> Result<()> {
        let proxy = self.paths.win64.join("dwmapi.dll");
        let disabled_proxy = self.paths.win64.join("dwmapi.obsidian-disabled.dll");
        let disabled_paks = self
            .paths
            .paks_enabled
            .with_file_name("~mods.obsidian-disabled");

        if enabled {
            if proxy.is_file() && !disabled_proxy.exists() {
                fs::rename(&proxy, &disabled_proxy)?;
            }
            if self.paths.paks_enabled.is_dir() && !disabled_paks.exists() {
                fs::rename(&self.paths.paks_enabled, &disabled_paks)?;
            }
            fs::create_dir_all(&self.paths.paks_enabled)?;
        } else {
            if disabled_proxy.is_file() {
                if proxy.exists() {
                    return Err(anyhow!("Cannot restore UE4SS: dwmapi.dll already exists"));
                }
                fs::rename(&disabled_proxy, &proxy)?;
            }
            if disabled_paks.is_dir() {
                if self.paths.paks_enabled.is_dir() {
                    let empty = fs::read_dir(&self.paths.paks_enabled)?.next().is_none();
                    if empty {
                        fs::remove_dir(&self.paths.paks_enabled)?;
                    } else {
                        return Err(anyhow!(
                            "Cannot restore PAK mods because ~mods contains new files"
                        ));
                    }
                }
                fs::rename(&disabled_paks, &self.paths.paks_enabled)?;
            }
        }
        Ok(())
    }

    pub fn open_game_folder(&self) -> Result<()> {
        open_path(&self.paths.game)
    }

    pub fn open_mods_folder(&self) -> Result<()> {
        fs::create_dir_all(&self.paths.paks_enabled)?;
        open_path(&self.paths.paks_enabled)
    }

    pub fn launch_game(&self) -> Result<()> {
        if !overlay_running() {
            if let Ok(exe) = std::env::current_exe() {
                let _ = Command::new(exe).arg("--overlay").spawn();
            }
        }
        open_uri(&format!("steam://rungameid/{APP_ID}"))
    }
}

fn read_meta(root: &Path) -> ModMeta {
    for name in ["obsidian-mod.json", "mod.json"] {
        if let Ok(text) = fs::read_to_string(root.join(name)) {
            if let Ok(meta) = serde_json::from_str::<ModMeta>(&text) {
                return meta;
            }
        }
    }
    ModMeta::default()
}

fn read_ue4ss_states(path: &Path) -> BTreeMap<String, bool> {
    let mut out = BTreeMap::new();
    let Ok(text) = fs::read_to_string(path) else {
        return out;
    };
    for line in text.lines() {
        if let Some((name, value)) = line.trim().split_once(':') {
            out.insert(name.trim().to_string(), value.trim() != "0");
        }
    }
    out
}

fn detect_game_dir() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let mut steam_roots = vec![
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        home.join(".local/share/Steam"),
        home.join(".steam/steam"),
    ];

    if cfg!(target_os = "windows") {
        if let Ok(p) = std::env::var("ProgramFiles(x86)") {
            steam_roots.push(PathBuf::from(p).join("Steam"));
        }
        if let Ok(p) = std::env::var("ProgramFiles") {
            steam_roots.push(PathBuf::from(p).join("Steam"));
        }
    }

    let mut libraries = Vec::new();
    for root in steam_roots {
        libraries.push(root.clone());
        let vdf = root.join("steamapps/libraryfolders.vdf");
        if let Ok(text) = fs::read_to_string(vdf) {
            for line in text.lines() {
                if line.contains("\"path\"") {
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() > 3 {
                        let raw = parts[3].replace("\\\\", "\\");
                        libraries.push(PathBuf::from(raw));
                    }
                }
            }
        }
    }

    libraries.sort();
    libraries.dedup();
    for lib in libraries {
        let game = lib.join("steamapps/common").join(GAME_NAME);
        if game
            .join("Dungeons/Binaries/Win64/Dungeons-Win64-Shipping.exe")
            .is_file()
        {
            return Some(game);
        }
    }
    None
}

fn extract_zip(path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(path).with_context(|| format!("Could not open {}", path.display()))?;
    let mut archive = zip::ZipArchive::new(file)?;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let Some(rel) = entry.enclosed_name() else {
            continue;
        };
        let out = dest.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out)?;
        } else {
            if let Some(parent) = out.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut target = File::create(&out)?;
            copy(&mut entry, &mut target)?;
        }
    }
    Ok(())
}

fn copy_dir(from: &Path, to: &Path) -> Result<()> {
    fs::create_dir_all(to)?;
    for entry in WalkDir::new(from) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(from)?;
        let dest = to.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&dest)?;
        } else {
            if let Some(parent) = dest.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &dest)?;
        }
    }
    Ok(())
}

fn slugify(value: &str) -> String {
    let mut out = String::new();
    let mut dash = false;

    for c in value.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            dash = false;
        } else if !dash && !out.is_empty() {
            out.push('-');
            dash = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "mod".to_string()
    } else {
        out
    }
}

fn epoch() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
fn overlay_running() -> bool {
    let system = System::new_all();
    system.processes().values().any(|process| {
        process
            .cmd()
            .iter()
            .any(|arg| arg.to_string_lossy() == "--overlay")
    })
}

fn open_path(path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(path).spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    Ok(())
}

fn open_uri(uri: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", "", uri]).spawn()?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("xdg-open").arg(uri).spawn()?;
    }
    Ok(())
}

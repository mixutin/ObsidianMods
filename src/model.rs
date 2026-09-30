use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ModKind {
    Ue4ss,
    Pak,
}

impl ModKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ue4ss => "UE4SS",
            Self::Pak => "PAK",
        }
    }
}

#[derive(Clone, Debug)]
pub struct InstalledMod {
    pub name: String,
    pub version: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
    pub kind: ModKind,
    pub enabled: bool,
    pub root: PathBuf,
    pub files: Vec<PathBuf>,
}

#[derive(Clone, Debug)]
pub struct GamePaths {
    pub game: PathBuf,
    pub win64: PathBuf,
    pub ue4ss_mods: PathBuf,
    pub ue4ss_mods_txt: PathBuf,
    pub paks_enabled: PathBuf,
    pub paks_disabled: PathBuf,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Catalog {
    pub schema: u32,
    pub generated_at: String,
    pub mods: Vec<CatalogMod>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct CatalogMod {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub summary: String,
    pub description: String,
    pub category: String,
    pub loader: String,
    pub game_build: String,
    pub download_url: String,
    pub sha256: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub featured: bool,
}

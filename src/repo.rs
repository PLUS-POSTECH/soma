use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::error::Result as SomaResult;

#[derive(Deserialize, Serialize)]
pub struct Repository {
    pub name: String,
    pub manifest: Manifest,
}

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    pub name: String,
    pub executable: Vec<FileEntry>,
    pub readonly: Vec<FileEntry>,
    pub binary: BinaryConfig,
}

#[derive(Deserialize, Serialize)]
pub struct FileEntry {
    pub path: String,
    pub public: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub struct BinaryConfig {
    pub os: String,
    pub entry: String,
}

fn read_file_contents(path: impl AsRef<Path>) -> SomaResult<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn load_manifest(manifest_path: impl AsRef<Path>) -> SomaResult<Manifest> {
    Ok(toml::from_slice(&read_file_contents(manifest_path)?)?)
}

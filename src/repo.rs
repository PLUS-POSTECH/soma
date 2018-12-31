use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_derive::{Deserialize, Serialize};

use crate::error::Result as SomaResult;
use crate::repo::backend::Backend;

pub mod backend;

pub type RepositoryList = HashMap<String, Backend>;

#[derive(Deserialize, Serialize)]
pub struct Repository {
    name: String,
    manifest: Manifest,
}

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    name: String,
    executable: Vec<FileEntry>,
    readonly: Vec<FileEntry>,
    binary: BinaryConfig,
}

#[derive(Deserialize, Serialize)]
struct FileEntry {
    path: String,
    public: Option<bool>,
}

#[derive(Deserialize, Serialize)]
struct BinaryConfig {
    os: String,
    entry: String,
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

use std::fs::File;
use std::io::Read;

use failure::Error;
use serde_derive::Deserialize;

#[derive(Deserialize)]
pub struct Manifest {
    pub name: String,
    pub executable: Vec<FileEntry>,
    pub readonly: Vec<FileEntry>,
    pub binary: BinaryConfig,
}

#[derive(Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub public: Option<bool>,
}

#[derive(Deserialize)]
pub struct BinaryConfig {
    pub os: String,
    pub entry: String,
}

fn read_file_contents(file_name: &str) -> Result<Vec<u8>, Error> {
    let mut file = File::open(file_name)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn load_manifest(manifest_file_name: &str) -> Result<Manifest, Error> {
    Ok(toml::from_slice(&read_file_contents(manifest_file_name)?)?)
}

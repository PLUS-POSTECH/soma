use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error as SomaError, Result as SomaResult};
use crate::repo::backend::Backend;

pub mod backend;

pub const MANIFEST_FILE_NAME: &'static str = "soma.toml";

pub type RepositoryIndex = BTreeMap<String, Repository>;

pub trait BTreeMapExt<K, V> {
    fn unique_insert(&mut self, key: K, value: V) -> SomaResult<Option<V>>;
}

impl BTreeMapExt<String, Repository> for RepositoryIndex {
    fn unique_insert(&mut self, key: String, value: Repository) -> SomaResult<Option<Repository>> {
        if self.contains_key(&key) {
            Err(SomaError::DuplicateRepositoryError)?
        } else {
            Ok(self.insert(key, value))
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Repository {
    name: String,
    local_path: PathBuf,
    backend: Backend,
}

impl Repository {
    pub fn new(name: String, local_path: PathBuf, backend: Backend) -> Repository {
        Repository {
            name,
            local_path,
            backend,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn local_path(&self) -> &PathBuf {
        &self.local_path
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }
}

#[derive(Deserialize)]
pub struct Manifest {
    name: String,
    work_dir: Option<PathBuf>,
    executable: Vec<FileEntry>,
    readonly: Vec<FileEntry>,
    binary: BinaryConfig,
}

#[derive(Serialize)]
pub struct SolidManifest {
    name: String,
    work_dir: PathBuf,
    executable: Vec<SolidFileEntry>,
    readonly: Vec<SolidFileEntry>,
    binary: BinaryConfig,
}

impl Manifest {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn executable(&self) -> &Vec<FileEntry> {
        &self.executable
    }

    pub fn readonly(&self) -> &Vec<FileEntry> {
        &self.readonly
    }

    pub fn solidify(&self) -> SomaResult<SolidManifest> {
        let work_dir = match &self.work_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(format!("/home/{}", self.name)),
        };
        let executable = self
            .executable
            .iter()
            .map(|file| file.solidify(&work_dir))
            .collect::<SomaResult<Vec<_>>>()?;
        let readonly = self
            .readonly
            .iter()
            .map(|file| file.solidify(&work_dir))
            .collect::<SomaResult<Vec<_>>>()?;

        Ok(SolidManifest {
            name: self.name.clone(),
            work_dir,
            executable,
            readonly,
            binary: self.binary.clone(),
        })
    }
}

// target_path is defined as String instead of PathBuf to support Windows
#[derive(Deserialize)]
pub struct FileEntry {
    path: PathBuf,
    public: Option<bool>,
    target_path: Option<String>,
}

#[derive(Serialize)]
pub struct SolidFileEntry {
    path: PathBuf,
    public: bool,
    target_path: String,
}

impl FileEntry {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn public(&self) -> bool {
        self.public.unwrap_or(false)
    }

    pub fn target_path(&self) -> &Option<String> {
        &self.target_path
    }

    pub fn solidify(&self, work_dir: impl AsRef<Path>) -> SomaResult<SolidFileEntry> {
        let target_path = match &self.target_path {
            Some(path) => path.clone(),
            None => {
                let work_dir = work_dir
                    .as_ref()
                    .to_str()
                    .ok_or(SomaError::InvalidUnicodeError)?;
                let file_name = self
                    .path
                    .file_name()
                    .ok_or(SomaError::FileNameNotFoundError)?
                    .to_str()
                    .ok_or(SomaError::InvalidUnicodeError)?;
                // manual string concatenation to support Windows
                format!("{}/{}", work_dir, file_name)
            }
        };
        Ok(SolidFileEntry {
            path: self.path.clone(),
            public: self.public.unwrap_or(false),
            target_path,
        })
    }
}

#[derive(Clone, Deserialize, Serialize)]
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

use std::collections::BTreeMap;
use std::fs::{remove_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use fs_extra::dir::{copy, CopyOptions};
use hyper::client::connect::Connect;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

use crate::docker;
use crate::error::{Error as SomaError, Result as SomaResult};
use crate::repo::backend::Backend;
use crate::template::{render_files_from_template, RenderingInput, Templates};
use crate::{Environment, Printer};

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

#[derive(Deserialize, Serialize)]
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

    pub fn build_image(
        &self,
        env: &Environment<impl Connect + 'static, impl Printer>,
        problem_name: &str,
    ) -> SomaResult<()> {
        let work_dir = tempdir()?;
        let work_dir_path = work_dir.path();
        let repo_path = self.local_path();
        let image_name = format!("soma/{}", problem_name);

        remove_dir_all(&work_dir)?;
        let mut copy_options = CopyOptions::new();
        copy_options.copy_inside = true;
        copy(&repo_path, &work_dir, &copy_options)?;

        let manifest = load_manifest(work_dir_path.join(MANIFEST_FILE_NAME))?;

        let rendering_input = RenderingInput::new(env, self.name(), manifest);

        render_files_from_template(Templates::Binary, &rendering_input, work_dir_path)?;

        docker::build(&image_name, work_dir_path)?;
        work_dir.close()?;
        Ok(())
    }
}

#[derive(Deserialize, Serialize)]
pub struct Manifest {
    name: String,
    executable: Vec<FileEntry>,
    readonly: Vec<FileEntry>,
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
}

#[derive(Deserialize, Serialize)]
pub struct FileEntry {
    path: String,
    public: Option<bool>,
}

impl FileEntry {
    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn public(&self) -> bool {
        self.public.unwrap_or(false)
    }
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

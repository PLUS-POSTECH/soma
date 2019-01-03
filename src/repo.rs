use std::collections::BTreeMap;
use std::fs::{remove_dir_all, File};
use std::io::Read;
use std::path::{Path, PathBuf};

use fs_extra::dir::{copy, CopyOptions};
use futures::future::Future;
use hyper::client::connect::Connect;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;
use tokio::runtime::current_thread::Runtime;

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

        let manifest = load_manifest(work_dir_path.join(MANIFEST_FILE_NAME))?
            .convert_to_docker_entry(&format!("/home/{}", problem_name))?;

        let rendering_input = RenderingInput::new(env.username(), self.name(), manifest);

        render_files_from_template(Templates::Binary, &rendering_input, work_dir_path)?;

        docker::build(&image_name, work_dir_path)?;
        work_dir.close()?;
        Ok(())
    }

    pub fn run_container(
        &self,
        env: &Environment<impl Connect + 'static, impl Printer>,
        problem_name: &str,
        runtime: &mut Runtime,
    ) -> SomaResult<String> {
        let image_name = format!("soma/{}", problem_name);
        let container_run = docker::create(env, &image_name)
            .and_then(|container_name| docker::start(env, &container_name).map(|_| container_name));
        runtime.block_on(container_run)
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

    pub fn convert_to_docker_entry(&self, default_path: impl AsRef<Path>) -> SomaResult<Manifest> {
        let executable = self.executable().iter();
        let readonly = self.readonly().iter();
        let map_file_entry = |file_entry: &FileEntry| -> SomaResult<FileEntry> {
            let new_file_entry = file_entry
                .target_path
                .as_ref()
                .unwrap_or(
                    &default_path.as_ref().join(
                        file_entry
                            .path
                            .file_name()
                            .ok_or(SomaError::InvalidManifestError)?,
                    ),
                )
                .clone();
            Ok(FileEntry {
                path: file_entry.path.clone(),
                public: file_entry.public,
                target_path: Some(new_file_entry),
            })
        };
        let new_executable: SomaResult<Vec<FileEntry>> = executable.map(map_file_entry).collect();
        let new_readonly: SomaResult<Vec<FileEntry>> = readonly.map(map_file_entry).collect();

        Ok(Manifest {
            name: self.name().clone(),
            executable: new_executable?,
            readonly: new_readonly?,
            binary: self.binary.clone(),
        })
    }
}

#[derive(Deserialize, Serialize)]
pub struct FileEntry {
    path: PathBuf,
    public: Option<bool>,
    target_path: Option<PathBuf>,
}

impl FileEntry {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn public(&self) -> bool {
        self.public.unwrap_or(false)
    }

    pub fn target_path(&self) -> &Option<PathBuf> {
        &self.target_path
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

use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use fs_extra::dir::{copy, CopyOptions};
use git2::{BranchType, ObjectType, Repository as GitRepository, ResetType};
use remove_dir_all::remove_dir_all;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

pub use self::repo_manager::RepositoryManager;

mod repo_manager;

pub const MANIFEST_FILE_NAME: &str = "soma.toml";

#[derive(Clone, Deserialize, Serialize)]
pub enum Backend {
    GitBackend(String),
    // On Windows, this path corresponds to extended length path
    // which means we can only join backslash-delimited paths to it
    LocalBackend(PathBuf),
}

impl Backend {}

impl fmt::Display for Backend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Backend::GitBackend(url) => write!(f, "Git: {}", url),
            Backend::LocalBackend(path) => write!(f, "Local: {}", path.to_string_lossy()),
        }
    }
}

pub struct Repository<'a> {
    name: String,
    backend: Backend,
    manager: &'a RepositoryManager<'a>,
}

impl<'a> Repository<'a> {
    fn new(name: String, backend: Backend, manager: &'a RepositoryManager<'a>) -> Repository<'a> {
        Repository {
            name,
            backend,
            manager,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn backend(&self) -> &Backend {
        &self.backend
    }

    pub fn manager(&self) -> &'a RepositoryManager {
        &self.manager
    }

    pub fn local_path(&self) -> PathBuf {
        self.manager.repo_path(&self.name)
    }

    pub fn update(&self) -> SomaResult<()> {
        let local_path = self.local_path();

        match &self.backend {
            Backend::GitBackend(url) => {
                let git_repo = GitRepository::open(&local_path)
                    .or_else(|_| GitRepository::clone(url, &local_path))?;
                git_repo
                    .find_remote("origin")?
                    .fetch(&["master"], None, None)?;

                let origin_master = git_repo.find_branch("origin/master", BranchType::Remote)?;
                let head_commit = origin_master.get().peel(ObjectType::Commit)?;
                git_repo.reset(&head_commit, ResetType::Hard, None)?;

                Ok(())
            }

            Backend::LocalBackend(path) => {
                if local_path.exists() {
                    remove_dir_all(&local_path)?;
                }

                let mut copy_options = CopyOptions::new();
                copy_options.copy_inside = true;
                copy(&path, &local_path, &copy_options)?;

                Ok(())
            }
        }
    }
}

#[derive(Deserialize)]
pub struct Manifest {
    name: String,
    work_dir: Option<PathBuf>,
    binary: BinaryConfig,
}

#[derive(Serialize)]
pub struct SolidManifest {
    name: String,
    work_dir: PathBuf,
    binary: SolidBinaryConfig,
}

impl Manifest {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn binary(&self) -> &BinaryConfig {
        &self.binary
    }

    pub fn solidify(&self) -> SomaResult<SolidManifest> {
        let work_dir = match &self.work_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(format!("/home/{}", self.name)),
        };

        let binary = self.binary.solidify(&work_dir)?;

        Ok(SolidManifest {
            name: self.name.clone(),
            work_dir,
            binary,
        })
    }
}

#[derive(Deserialize)]
pub struct BinaryConfig {
    os: String,
    cmd: String,
    executable: Vec<FileEntry>,
    readonly: Vec<FileEntry>,
}

#[derive(Serialize)]
struct SolidBinaryConfig {
    os: String,
    cmd: String,
    file_entries: Vec<SolidFileEntry>,
}

impl BinaryConfig {
    pub fn executable(&self) -> &Vec<FileEntry> {
        &self.executable
    }

    pub fn readonly(&self) -> &Vec<FileEntry> {
        &self.readonly
    }

    fn solidify(&self, work_dir: impl AsRef<Path>) -> SomaResult<SolidBinaryConfig> {
        let executable = self
            .executable
            .iter()
            .map(|file| file.solidify(&work_dir, PermissionOption::Executable));
        let readonly = self
            .readonly
            .iter()
            .map(|file| file.solidify(&work_dir, PermissionOption::ReadOnly));
        let file_entries = executable.chain(readonly).collect::<SomaResult<Vec<_>>>()?;

        Ok(SolidBinaryConfig {
            os: self.os.clone(),
            entry: self.entry.clone(),
            file_entries,
        })
    }
}

enum PermissionOption {
    Executable,
    ReadOnly,
    // Reserved: FetchOnly, ReadWrite, WithPermission
}

impl PermissionOption {
    pub fn unix_permissions(&self) -> Option<u16> {
        match self {
            PermissionOption::Executable => Some(0o550),
            PermissionOption::ReadOnly => Some(0o440),
        }
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
struct SolidFileEntry {
    path: PathBuf,
    public: bool,
    target_path: String,
    permissions: u16,
}

impl FileEntry {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn public(&self) -> bool {
        self.public.unwrap_or(false)
    }

    fn solidify(
        &self,
        work_dir: impl AsRef<Path>,
        permission_option: PermissionOption,
    ) -> SomaResult<SolidFileEntry> {
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
            permissions: permission_option.unix_permissions().unwrap_or(440),
        })
    }
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

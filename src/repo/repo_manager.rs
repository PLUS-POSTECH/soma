use std::collections::BTreeMap;
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use remove_dir_all::remove_dir_all;

use crate::data_dir::{DirectoryManager, Registration};
use crate::error::{Error as SomaError, Result as SomaResult};
use crate::repo::{Backend, Repository};

const INDEX_FILE_NAME: &'static str = "index";

fn index_path<'a>(registration: &Registration<'a, RepositoryManager<'a>>) -> PathBuf {
    registration.root_path().join(INDEX_FILE_NAME)
}

pub struct RepositoryManager<'a> {
    registration: Registration<'a, RepositoryManager<'a>>,
    index: BTreeMap<String, Backend>,
    dirty: bool,
}

impl<'a> Deref for RepositoryManager<'a> {
    type Target = Registration<'a, Self>;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.registration
    }
}

impl<'a> DirectoryManager<'a> for RepositoryManager<'a> {
    const DIR: &'static str = "repositories";

    fn new(registration: Registration<'a, Self>) -> SomaResult<Self> {
        let index_path = index_path(&registration);
        let index = if index_path.exists() {
            let file = File::open(index_path.as_path())?;
            serde_cbor::from_reader(file)?
        } else {
            BTreeMap::new()
        };

        Ok(RepositoryManager {
            registration,
            index,
            dirty: false,
        })
    }
}

impl<'a> RepositoryManager<'a> {
    pub(super) fn repo_path(&self, repo_name: impl AsRef<Path>) -> PathBuf {
        self.root_path().join(repo_name)
    }

    pub fn add_repo(&mut self, repo_name: String, backend: Backend) -> SomaResult<()> {
        if self.repo_exists(&repo_name) {
            Err(SomaError::DuplicateRepositoryError)?;
        } else {
            self.index.insert(repo_name.clone(), backend.clone());
            self.dirty = true;
        }

        Ok(())
    }

    pub fn remove_repo(&mut self, repo_name: &str) -> SomaResult<()> {
        let local_path = self.repo_path(repo_name);
        if local_path.is_dir() {
            remove_dir_all(local_path)?;
        }

        self.index
            .remove(repo_name)
            .ok_or(SomaError::RepositoryNotFoundError)?;
        self.dirty = true;

        Ok(())
    }

    pub fn get_repo(&self, repo_name: &str) -> SomaResult<Repository> {
        let repository = match self.index.get(repo_name) {
            Some(backend) => Repository::new(repo_name.to_owned(), backend.clone(), self),
            None => Err(SomaError::RepositoryNotFoundError)?,
        };
        Ok(repository)
    }

    pub fn list_repo(&'a self) -> impl Iterator<Item = Repository<'a>> {
        self.index
            .iter()
            .map(move |(name, backend)| Repository::new(name.clone(), backend.clone(), self))
    }

    pub fn repo_exists(&self, repo_name: &str) -> bool {
        self.index.contains_key(repo_name)
    }
}

impl<'a> Drop for RepositoryManager<'a> {
    fn drop(&mut self) {
        if self.dirty {
            let path = index_path(&self);
            if let Ok(mut file) = File::create(path) {
                if serde_cbor::to_writer(&mut file, &self.index).is_err() {
                    eprintln!("failed to write the repository index to the file");
                }
            } else {
                eprintln!("failed to create the repository index file");
            }
        }
    }
}

use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use remove_dir_all::remove_dir_all;
use serde::{Deserialize, Serialize};

use crate::data_dir::{DirectoryManager, Registration};
use crate::prelude::*;
use crate::problem::Problem;
use crate::repository::backend::{Backend, BackendExt};
use crate::repository::{read_prob_list, ProblemIndex, Repository};
use crate::NameString;

const INDEX_FILE_NAME: &str = "index";

fn index_path<'a>(registration: &Registration<'a, RepositoryManager<'a>>) -> PathBuf {
    registration.root_path().join(INDEX_FILE_NAME)
}

#[derive(Clone, Deserialize, Serialize)]
struct Index {
    backend: Box<dyn Backend>,
    prob_list: Vec<ProblemIndex>,
}

pub struct RepositoryManager<'a> {
    registration: Registration<'a, RepositoryManager<'a>>,
    repo_index: BTreeMap<NameString, Index>,
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
        let repo_index = if index_path.exists() {
            let file = File::open(index_path.as_path())?;
            serde_cbor::from_reader(file)?
        } else {
            BTreeMap::new()
        };

        Ok(RepositoryManager {
            registration,
            repo_index,
            dirty: false,
        })
    }
}

impl<'a> RepositoryManager<'a> {
    pub(super) fn repo_path(&self, repo_name: impl AsRef<Path>) -> PathBuf {
        self.root_path().join(repo_name)
    }

    pub fn add_repo(
        &mut self,
        repo_name: impl AsRef<str>,
        backend: Box<dyn Backend>,
    ) -> SomaResult<()> {
        let repo_name = NameString::try_from(repo_name.as_ref())?;
        if self.repo_exists(&repo_name) {
            Err(SomaError::DuplicateRepository)?;
        } else {
            let temp_dir = tempfile::tempdir()?;
            backend.update_at(temp_dir.path())?;
            let prob_list = read_prob_list(temp_dir.path())?;

            self.repo_index
                .insert(repo_name.clone(), Index { backend, prob_list });
            self.dirty = true;
        }

        Ok(())
    }

    pub fn remove_repo(&mut self, repo_name: impl AsRef<str>) -> SomaResult<()> {
        let repo_name = NameString::try_from(repo_name.as_ref())?;
        let local_path = self.repo_path(&repo_name);
        if local_path.is_dir() {
            remove_dir_all(local_path)?;
        }

        self.repo_index
            .remove(&repo_name)
            .ok_or(SomaError::RepositoryNotFound)?;
        self.dirty = true;

        Ok(())
    }

    pub fn get_repo(&self, repo_name: impl AsRef<str>) -> SomaResult<Repository> {
        let repo_name = NameString::try_from(repo_name.as_ref())?;
        let repository = match self.repo_index.get(&repo_name) {
            Some(index) => Repository::new(
                repo_name.clone(),
                index.backend.clone(),
                index.prob_list.clone(),
                self,
            ),
            None => Err(SomaError::RepositoryNotFound)?,
        };
        Ok(repository)
    }

    // problem query is either a prob_name or a fully qualified name
    pub fn search_prob(&self, query: &str) -> SomaResult<Problem> {
        let mut result: Vec<_> = self
            .list_prob()
            .filter(|problem| {
                query == problem.prob_name() || query == problem.fully_qualified_name()
            })
            .collect();

        match result.len() {
            0 => Err(SomaError::ProblemNotFound)?,
            1 => Ok(result.swap_remove(0)),
            _ => Err(SomaError::ProblemQueryAmbiguous)?,
        }
    }

    pub fn list_repo(&'a self) -> impl Iterator<Item = Repository<'a>> {
        self.repo_index.iter().map(move |(name, index)| {
            Repository::new(
                name.clone(),
                index.backend.clone(),
                index.prob_list.clone(),
                self,
            )
        })
    }

    pub fn list_prob(&self) -> impl Iterator<Item = Problem> + '_ {
        self.repo_index.iter().flat_map(move |(repo_name, index)| {
            index.prob_list.iter().map(move |prob_index| {
                Problem::new(
                    repo_name.to_owned(),
                    prob_index.name.to_owned(),
                    self.repo_path(repo_name).join(&prob_index.path),
                )
            })
        })
    }

    pub fn repo_exists(&self, repo_name: impl AsRef<str>) -> bool {
        let repo_name = NameString::try_from(repo_name.as_ref());
        match repo_name {
            Ok(repo_name) => self.repo_index.contains_key(&repo_name),
            Err(_) => false,
        }
    }
}

impl<'a> Drop for RepositoryManager<'a> {
    fn drop(&mut self) {
        if self.dirty {
            let path = index_path(&self);
            if let Ok(mut file) = File::create(path) {
                if serde_cbor::to_writer(&mut file, &self.repo_index).is_err() {
                    eprintln!("Failed to save the repository index");
                }
            } else {
                eprintln!("Failed to open the repository index file");
            }
        }
    }
}

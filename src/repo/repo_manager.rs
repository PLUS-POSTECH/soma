use std::collections::BTreeMap;
use std::fs::File;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use remove_dir_all::remove_dir_all;
use serde::{Deserialize, Serialize};

use crate::data_dir::{DirectoryManager, Registration};
use crate::prelude::*;
use crate::repo::{Backend, Problem, ProblemIndex, Repository};

const INDEX_FILE_NAME: &'static str = "index";

fn index_path<'a>(registration: &Registration<'a, RepositoryManager<'a>>) -> PathBuf {
    registration.root_path().join(INDEX_FILE_NAME)
}

#[derive(Clone, Deserialize, Serialize)]
struct Index {
    backend: Backend,
    prob_list: Vec<ProblemIndex>,
}

pub struct RepositoryManager<'a> {
    registration: Registration<'a, RepositoryManager<'a>>,
    repo_index: BTreeMap<String, Index>,
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

    pub fn add_repo(&mut self, repo_name: String, backend: Backend) -> SomaResult<()> {
        if self.repo_exists(&repo_name) {
            Err(SomaError::DuplicateRepositoryError)?;
        } else {
            let temp_dir = tempfile::tempdir()?;
            backend.update_at(temp_dir.path())?;
            let prob_list = super::read_prob_list(temp_dir.path())?;

            self.repo_index.insert(
                repo_name.clone(),
                Index {
                    backend: backend.clone(),
                    prob_list,
                },
            );
            self.dirty = true;
        }

        Ok(())
    }

    pub fn remove_repo(&mut self, repo_name: &str) -> SomaResult<()> {
        let local_path = self.repo_path(repo_name);
        if local_path.is_dir() {
            remove_dir_all(local_path)?;
        }

        self.repo_index
            .remove(repo_name)
            .ok_or(SomaError::RepositoryNotFoundError)?;
        self.dirty = true;

        Ok(())
    }

    pub fn get_repo(&self, repo_name: &str) -> SomaResult<Repository> {
        let repository = match self.repo_index.get(repo_name) {
            Some(index) => Repository::new(
                repo_name.to_owned(),
                index.backend.clone(),
                index.prob_list.clone(),
                self,
            ),
            None => Err(SomaError::RepositoryNotFoundError)?,
        };
        Ok(repository)
    }

    pub fn search_prob(&self, query: &str) -> SomaResult<Problem> {
        let result: Vec<_> = self
            .repo_index
            .iter()
            .flat_map(|(repo_name, repo_index)| {
                repo_index
                    .prob_list
                    .iter()
                    .map(move |prob_index| (repo_name, &prob_index.name, &prob_index.path))
            })
            .filter(|(repo_name, prob_name, _)| {
                query == *prob_name || query == Problem::problem_id(repo_name, prob_name)
            })
            .collect();

        match result.len() {
            0 => Err(SomaError::ProblemNotFoundError)?,
            1 => {
                let (repo_name, prob_name, prob_path) = result[0];
                Ok(Problem {
                    repo_name: repo_name.to_owned(),
                    prob_name: prob_name.to_owned(),
                    path: self.repo_path(repo_name).join(prob_path),
                })
            }
            _ => Err(SomaError::MultipleProblemEntryError)?,
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

    pub fn repo_exists(&self, repo_name: &str) -> bool {
        self.repo_index.contains_key(repo_name)
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

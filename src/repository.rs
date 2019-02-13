use std::collections::HashSet;
use std::fmt;
use std::path::{Path, PathBuf};

use fs_extra::dir::{copy, CopyOptions};
use git2::{BranchType, ObjectType, Repository as GitRepository, ResetType};
use remove_dir_all::remove_dir_all;
use serde::{Deserialize, Serialize};

use crate::prelude::*;
use crate::problem::{read_manifest, MANIFEST_FILE_NAME};
use crate::read_file_contents;

pub use self::manager::RepositoryManager;

mod manager;

const LIST_FILE_NAME: &str = "soma-list.toml";

#[derive(Deserialize)]
struct ProblemList {
    problems: Vec<PathBuf>,
}

impl ProblemList {
    fn sanity_check(&self, repo_path: impl AsRef<Path>) -> SomaResult<()> {
        let hash_set = self
            .problems
            .iter()
            .map(|prob_relative_path| repo_path.as_ref().join(prob_relative_path).canonicalize())
            .collect::<Result<HashSet<_>, _>>()
            .map_err(|_| SomaError::InvalidSomaList)?;

        if hash_set.len() != self.problems.len() {
            Err(SomaError::InvalidSomaList)?;
        }

        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct ProblemIndex {
    name: String,
    path: PathBuf,
}

#[derive(Clone, Deserialize, Serialize)]
pub enum Backend {
    GitBackend(String),
    // On Windows, this path corresponds to extended length path
    // which means we can only join backslash-delimited paths to it
    LocalBackend(PathBuf),
}

impl Backend {
    fn update_at(&self, local_path: impl AsRef<Path>) -> SomaResult<()> {
        match &self {
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
                if local_path.as_ref().exists() {
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
    prob_list: Vec<ProblemIndex>,
    manager: &'a RepositoryManager<'a>,
}

impl<'a> Repository<'a> {
    fn new(
        name: String,
        backend: Backend,
        prob_list: Vec<ProblemIndex>,
        manager: &'a RepositoryManager<'a>,
    ) -> Repository<'a> {
        Repository {
            name,
            backend,
            prob_list,
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

    pub fn path(&self) -> PathBuf {
        self.manager.repo_path(&self.name)
    }

    pub fn update(&self) -> SomaResult<()> {
        self.backend.update_at(self.path())
    }

    pub fn prob_name_iter(&'a self) -> impl Iterator<Item = &'a String> {
        self.prob_list.iter().map(|prob_index| &prob_index.name)
    }
}

fn read_prob_manifest(
    repo_path: impl AsRef<Path>,
    prob_relative_path: impl AsRef<Path>,
) -> SomaResult<ProblemIndex> {
    let prob_path = repo_path.as_ref().join(&prob_relative_path);
    let manifest_path = prob_path.join(MANIFEST_FILE_NAME);
    if !manifest_path.exists() {
        Err(SomaError::InvalidRepository)?;
    }

    let manifest = read_manifest(manifest_path)?;
    Ok(ProblemIndex {
        name: manifest.name().to_owned(),
        path: prob_relative_path.as_ref().to_owned(),
    })
}

fn read_prob_list(repo_path: impl AsRef<Path>) -> SomaResult<Vec<ProblemIndex>> {
    let list_path = repo_path.as_ref().join(LIST_FILE_NAME);
    if list_path.exists() {
        let prob_list: ProblemList = toml::from_slice(&read_file_contents(list_path)?)?;
        prob_list.sanity_check(&repo_path)?;
        prob_list
            .problems
            .iter()
            .map(|prob_relative_path| read_prob_manifest(&repo_path, prob_relative_path))
            .collect()
    } else {
        Ok(vec![read_prob_manifest(&repo_path, "./")?])
    }
}

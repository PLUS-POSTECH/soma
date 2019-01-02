use std::fmt;
use std::fs::{create_dir_all, remove_dir_all};
use std::path::{Path, PathBuf};

use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use git2::{BranchType, ObjectType, Repository as GitRepository, ResetType};
use serde::{Deserialize, Serialize};

use crate::error::Result as SomaResult;

#[derive(Debug, Deserialize, Serialize)]
pub enum Backend {
    GitBackend(String),
    // On Windows, this path corresponds to extended length path
    // which means we can only join backslash-delimited paths to it
    LocalBackend(PathBuf),
}

impl Backend {
    pub fn update(&self, local_path: impl AsRef<Path>) -> SomaResult<()> {
        match self {
            Backend::GitBackend(url) => {
                let git_repo =
                    GitRepository::open(&local_path).or(GitRepository::clone(url, &local_path))?;
                git_repo
                    .find_remote("origin")?
                    .fetch(&["master"], None, None)?;

                // let oid = repo.refname_to_id("refs/remotes/origin/master")?;
                // let object = repo.find_object(oid, None).unwrap();
                let origin_master = git_repo.find_branch("origin/master", BranchType::Remote)?;
                let head_commit = origin_master.get().peel(ObjectType::Commit)?;
                git_repo.reset(&head_commit, ResetType::Hard, None)?;

                Ok(())
            }
            Backend::LocalBackend(path) => {
                remove_dir_all(&local_path)?;
                create_dir_all(&local_path)?;
                copy_items(&vec![&path], &local_path, &CopyOptions::new())?;

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

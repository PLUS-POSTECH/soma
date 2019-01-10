use std::fmt;
use std::path::{Path, PathBuf};

use fs_extra::dir::{copy, CopyOptions};
use git2::{BranchType, ObjectType, Repository as GitRepository, ResetType};
use remove_dir_all::remove_dir_all;
use serde::{Deserialize, Serialize};

use crate::error::Result as SomaResult;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

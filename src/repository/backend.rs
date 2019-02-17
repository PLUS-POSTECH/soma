use std::fmt::{self, Display};
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

use fs_extra::dir;
use git2::{BranchType, ObjectType, Repository as GitRepository, ResetType};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::prelude::*;

#[typetag::serde(tag = "type")]
pub trait Backend: BackendClone + Display {
    fn update_at_path(&self, local_path: &Path) -> SomaResult<()>;
}

pub trait BackendClone {
    fn clone_box(&self) -> Box<dyn Backend>;
}

impl<T> BackendClone for T
where
    T: 'static + Backend + Clone,
{
    fn clone_box(&self) -> Box<dyn Backend> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Backend> {
    fn clone(&self) -> Box<dyn Backend> {
        self.clone_box()
    }
}

pub trait BackendExt: Backend {
    fn update_at(&self, local_path: impl AsRef<Path>) -> SomaResult<()> {
        self.update_at_path(local_path.as_ref())
    }
}

impl<T> BackendExt for T where T: ?Sized + Backend {}

pub fn location_to_backend(repo_location: &str) -> SomaResult<(String, Box<dyn Backend>)> {
    let path = Path::new(repo_location);
    if path.is_dir() {
        // local backend
        Ok((
            path.file_name()
                .ok_or(SomaError::FileNameNotFound)?
                .to_str()
                .ok_or(SomaError::InvalidUnicode)?
                .to_lowercase(),
            Box::new(LocalBackend::new(path.canonicalize()?.to_owned())),
        ))
    } else {
        // git backend
        let parsed_url = Url::parse(repo_location).or(Err(SomaError::RepositoryNotFound))?;
        let last_name = parsed_url
            .path_segments()
            .ok_or(SomaError::RepositoryNotFound)?
            .last()
            .ok_or(SomaError::FileNameNotFound)?;
        let repo_name = if last_name.ends_with(".git") {
            &last_name[..last_name.len() - 4]
        } else {
            &last_name
        };
        Ok((
            repo_name.to_lowercase(),
            Box::new(GitBackend::new(repo_location.to_owned())),
        ))
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct GitBackend {
    url: String,
}

impl GitBackend {
    pub fn new(url: String) -> Self {
        GitBackend { url }
    }
}

#[typetag::serde]
impl Backend for GitBackend {
    fn update_at_path(&self, local_path: &Path) -> SomaResult<()> {
        let git_repo = GitRepository::open(local_path)
            .or_else(|_| GitRepository::clone(&self.url, local_path))?;
        git_repo
            .find_remote("origin")?
            .fetch(&["master"], None, None)?;

        let origin_master = git_repo.find_branch("origin/master", BranchType::Remote)?;
        let head_commit = origin_master.get().peel(ObjectType::Commit)?;
        git_repo.reset(&head_commit, ResetType::Hard, None)?;

        Ok(())
    }
}

impl Display for GitBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Git: {}", &self.url)
    }
}

// On Windows, this path corresponds to extended length path
// which means we can only join backslash-delimited paths to it
#[derive(Clone, Deserialize, Serialize)]
pub struct LocalBackend {
    origin: PathBuf,
}

impl LocalBackend {
    pub fn new(origin: PathBuf) -> Self {
        LocalBackend { origin }
    }
}

#[typetag::serde]
impl Backend for LocalBackend {
    fn update_at_path(&self, local_path: &Path) -> SomaResult<()> {
        if local_path.exists() {
            remove_dir_all(local_path)?;
        }

        let mut copy_options = dir::CopyOptions::new();
        copy_options.copy_inside = true;
        dir::copy(&self.origin, local_path, &copy_options)?;

        Ok(())
    }
}

impl Display for LocalBackend {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Local: {}", self.origin.to_string_lossy())
    }
}

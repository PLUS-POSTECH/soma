use std::path::PathBuf;

use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub enum Backend {
    GitBackend(PathBuf),
    LocalBackend(PathBuf),
}

impl Backend {
    pub fn path(&self) -> PathBuf {
        match self {
            Backend::GitBackend(path) => path.clone(),
            Backend::LocalBackend(path) => path.clone(),
        }
    }

    pub fn update(&self) {
        unimplemented!()
    }
}

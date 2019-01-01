use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Backend {
    GitBackend(String),
    // On Windows, this path corresponds to extended length path
    // which means we can only join backslash-delimited paths to it
    LocalBackend(PathBuf),
}

impl Backend {
    pub fn update(&self) {
        unimplemented!()
    }
}

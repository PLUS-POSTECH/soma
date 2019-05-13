use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use self::configs::{BinaryConfig, FileEntry, SolidBinaryConfig};
use crate::prelude::*;
use crate::{read_file_contents, NameString};

pub mod configs;

pub const MANIFEST_FILE_NAME: &str = "soma.toml";

#[derive(Debug)]
pub struct Problem {
    repo_name: NameString,
    prob_name: NameString,
    path: PathBuf,
}

impl Problem {
    pub fn new(repo_name: NameString, prob_name: NameString, path: PathBuf) -> Self {
        Problem {
            repo_name,
            prob_name,
            path,
        }
    }

    pub fn fully_qualified_name(&self) -> String {
        format!("{}.{}", &self.repo_name, &self.prob_name)
    }

    pub fn docker_image_name(&self, user_name: &str) -> String {
        format!("soma.{}/{}", user_name, self.fully_qualified_name())
    }

    pub fn repo_name(&self) -> &NameString {
        &self.repo_name
    }

    pub fn prob_name(&self) -> &NameString {
        &self.prob_name
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn load_manifest(&self) -> SomaResult<Manifest> {
        let manifest_path = self.path().join(MANIFEST_FILE_NAME);
        read_manifest(manifest_path)
    }
}

#[derive(Deserialize)]
pub struct Manifest {
    name: NameString,
    work_dir: Option<PathBuf>,
    binary: BinaryConfig,
}

#[derive(Serialize)]
pub struct SolidManifest {
    name: NameString,
    work_dir: PathBuf,
    binary: SolidBinaryConfig,
}

impl Manifest {
    pub fn name(&self) -> &NameString {
        &self.name
    }

    pub fn public_files(&self) -> Vec<&PathBuf> {
        let binary = &self.binary;
        let executables = binary.executable().iter();
        let readonly = binary.readonly().iter();

        executables
            .chain(readonly)
            .filter(|file_entry| file_entry.public())
            .map(FileEntry::path)
            .collect()
    }

    pub fn solidify(&self) -> SomaResult<SolidManifest> {
        let work_dir = match &self.work_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(format!("/home/{}", self.name)),
        };

        // TODO: More descriptive error
        if !work_dir.has_root() {
            Err(SomaError::InvalidManifest)?;
        }

        let binary = self.binary.solidify(&work_dir)?;

        Ok(SolidManifest {
            name: self.name.clone(),
            work_dir,
            binary,
        })
    }
}

impl SolidManifest {
    pub fn binary(&self) -> &SolidBinaryConfig {
        &self.binary
    }
}

pub fn read_manifest(path: impl AsRef<Path>) -> SomaResult<Manifest> {
    Ok(toml::from_slice(&read_file_contents(path)?)?)
}

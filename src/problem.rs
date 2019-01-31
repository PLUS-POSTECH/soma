use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use self::configs::*;
use crate::prelude::*;

mod configs;

pub const MANIFEST_FILE_NAME: &str = "soma.toml";

#[derive(Debug)]
pub struct Problem {
    repo_name: String,
    prob_name: String,
    path: PathBuf,
}

impl Problem {
    pub fn new(repo_name: String, prob_name: String, path: PathBuf) -> Self {
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

    pub fn repo_name(&self) -> &String {
        &self.repo_name
    }

    pub fn prob_name(&self) -> &String {
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
    name: String,
    work_dir: Option<PathBuf>,
    binary: BinaryConfig,
}

#[derive(Serialize)]
pub struct SolidManifest {
    name: String,
    work_dir: PathBuf,
    binary: SolidBinaryConfig,
}

impl Manifest {
    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn public_files(&self) -> Vec<&PathBuf> {
        let binary = &self.binary;
        let executables = binary.executable().iter();
        let readonly = binary.readonly().iter();

        executables
            .chain(readonly)
            .filter(|file_entry| file_entry.public())
            .map(|file_entry| file_entry.path())
            .collect()
    }

    pub fn solidify(&self) -> SomaResult<SolidManifest> {
        let work_dir = match &self.work_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(format!("/home/{}", self.name)),
        };

        let binary = self.binary.solidify(&work_dir)?;

        Ok(SolidManifest {
            name: self.name.clone(),
            work_dir,
            binary,
        })
    }
}

fn read_file_contents(path: impl AsRef<Path>) -> SomaResult<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub fn read_manifest(path: impl AsRef<Path>) -> SomaResult<Manifest> {
    Ok(toml::from_slice(&read_file_contents(path)?)?)
}

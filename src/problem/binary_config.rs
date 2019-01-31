use std::path::Path;

use serde::{Deserialize, Serialize};

use super::file::{FileEntry, FilePermissions, SolidFileEntry};
use crate::prelude::*;

#[derive(Deserialize)]
pub struct BinaryConfig {
    os: String,
    cmd: String,
    executable: Vec<FileEntry>,
    readonly: Vec<FileEntry>,
}

#[derive(Serialize)]
pub struct SolidBinaryConfig {
    os: String,
    cmd: String,
    file_entries: Vec<SolidFileEntry>,
}

impl BinaryConfig {
    pub fn executable(&self) -> &Vec<FileEntry> {
        &self.executable
    }

    pub fn readonly(&self) -> &Vec<FileEntry> {
        &self.readonly
    }

    pub fn solidify(&self, work_dir: impl AsRef<Path>) -> SomaResult<SolidBinaryConfig> {
        let executable = self
            .executable
            .iter()
            .map(|file| file.solidify(&work_dir, FilePermissions::Executable));
        let readonly = self
            .readonly
            .iter()
            .map(|file| file.solidify(&work_dir, FilePermissions::ReadOnly));
        let file_entries = executable.chain(readonly).collect::<SomaResult<Vec<_>>>()?;

        Ok(SolidBinaryConfig {
            os: self.os.clone(),
            cmd: self.cmd.clone(),
            file_entries,
        })
    }
}

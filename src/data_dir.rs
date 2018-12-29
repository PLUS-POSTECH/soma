use std::fs;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use fs2::FileExt;
use question::{Answer, Question};

use crate::error::{Error as SomaError, Result as SomaResult};

pub struct DataDirectory {
    path: PathBuf,
    lock: File,
}

impl DataDirectory {
    pub fn new() -> SomaResult<DataDirectory> {
        let path = {
            if let Some(dir) = std::env::var_os("SOMA_DATA_DIR") {
                dir.into()
            } else {
                let mut home = dirs::home_dir().ok_or(SomaError::DataDirectoryAccessError)?;
                home.push(".soma");
                home
            }
        };

        if !path.exists() {
            if let Answer::NO = Question::new(&format!(
                "It seems that you use soma for the first time. The data directory will be initialized at {:?}. [y/n]\n",
                path.as_os_str(),
            ))
                .confirm()
            {
                Err(SomaError::DataDirectoryAccessError)?;
            }
            fs::create_dir_all(&path)?;
        }

        let lock = File::create(path.join("soma.lock"))?;
        // TODO: use more fine-tuned lock
        lock.try_lock_exclusive()
            .or(Err(SomaError::DataDirectoryLockError))?;

        Ok(DataDirectory { path, lock })
    }

    pub fn root_path<'a>(&'a self) -> impl AsRef<Path> + 'a {
        self.path.as_path()
    }
}

impl Drop for DataDirectory {
    fn drop(&mut self) {
        self.lock.unlock().expect("failed to unlock the lockfile");
    }
}

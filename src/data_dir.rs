use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};

use fs2::FileExt;
use question::{Answer, Question};

use crate::error::{Error as SomaError, Result as SomaResult};
use crate::repo::backend::Backend;
use crate::repo::Repository;
use crate::repo::{BTreeMapExt, RepositoryIndex};

const SOMA_DATA_DIR_ENV_NAME: &str = "SOMA_DATA_DIR";
const SOMA_DATA_DIR_NAME: &str = ".soma";

const LOCK_FILE_NAME: &str = "soma.lock";

const REPOSITORY_DIR_NAME: &str = "repositories";
const REPOSITORY_INDEX_FILE_NAME: &str = "index";

pub struct DataDirectory {
    root_path: PathBuf,
    lock: File,
}

impl DataDirectory {
    pub fn new() -> SomaResult<DataDirectory> {
        let path = {
            if let Some(dir) = std::env::var_os(SOMA_DATA_DIR_ENV_NAME) {
                dir.into()
            } else {
                let mut home = dirs::home_dir().ok_or(SomaError::DataDirectoryAccessError)?;
                home.push(SOMA_DATA_DIR_NAME);
                home
            }
        };

        if !path.exists() {
            if let Answer::NO = Question::new(&format!(
                "It seems that you use Soma for the first time. The data directory will be initialized at {:?}. [y/n]\n",
                path.as_os_str(),
            ))
                .confirm()
            {
                Err(SomaError::DataDirectoryAccessError)?;
            }
            fs::create_dir_all(&path)?;
        }

        DataDirectory::initialize_and_lock(path)
    }

    pub fn at_path(path: impl AsRef<Path>) -> SomaResult<DataDirectory> {
        fs::create_dir_all(&path)?;
        DataDirectory::initialize_and_lock(path.as_ref().to_owned())
    }

    fn initialize_and_lock(path: PathBuf) -> SomaResult<DataDirectory> {
        let lock = File::create(path.join(LOCK_FILE_NAME))?;
        // TODO: use more fine-tuned lock
        lock.try_lock_exclusive()
            .or(Err(SomaError::DataDirectoryLockError))?;

        Ok(DataDirectory {
            root_path: path,
            lock,
        })
    }

    pub fn root_path(&self) -> PathBuf {
        self.root_path.clone()
    }

    pub fn repo_root_path(&self) -> PathBuf {
        self.root_path.join(REPOSITORY_DIR_NAME)
    }

    pub fn repo_index_path(&self) -> PathBuf {
        self.repo_root_path().join(REPOSITORY_INDEX_FILE_NAME)
    }

    pub fn repo_path(&self, repo_name: impl AsRef<Path>) -> PathBuf {
        self.repo_root_path().join(repo_name)
    }

    fn init_repo(&self, repo_name: impl AsRef<Path>) -> SomaResult<PathBuf> {
        fs::create_dir_all(self.repo_root_path())?;
        let new_repo_path = self.repo_path(repo_name);
        fs::create_dir(&new_repo_path).or(Err(SomaError::DuplicateRepositoryError))?;
        Ok(new_repo_path)
    }

    pub fn read_repo_index(&self) -> SomaResult<RepositoryIndex> {
        let path = self.repo_index_path();
        if path.exists() {
            let file = File::open(path.as_path())?;
            Ok(serde_cbor::from_reader(file)?)
        } else {
            Ok(BTreeMap::new())
        }
    }

    fn write_repo_index(&self, repository_list: RepositoryIndex) -> SomaResult<()> {
        fs::create_dir_all(self.repo_root_path())?;
        let path = self.repo_index_path();
        let mut file = File::create(path)?;
        serde_cbor::to_writer(&mut file, &repository_list)?;
        Ok(())
    }

    pub fn add_repo(&self, repo_name: String, backend: Backend) -> SomaResult<()> {
        let mut repo_index = self.read_repo_index()?;
        let local_path = self.init_repo(&repo_name)?;
        let repository = Repository::new(repo_name.clone(), local_path, backend);
        repo_index.unique_insert(repo_name, repository)?;

        self.write_repo_index(repo_index)?;

        Ok(())
    }
}

impl Drop for DataDirectory {
    fn drop(&mut self) {
        self.lock.unlock().expect("failed to unlock the lockfile");
    }
}

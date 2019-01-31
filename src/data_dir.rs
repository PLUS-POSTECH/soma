use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::marker::PhantomData;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use fs2::FileExt;

use crate::prelude::*;

const SOMA_DATA_DIR_ENV_NAME: &str = "SOMA_DATA_DIR";
const SOMA_DATA_DIR_NAME: &str = ".soma";

const LOCK_FILE_NAME: &str = "soma.lock";

pub struct DataDirectory {
    root_path: PathBuf,
    lock: File,
    manager_set: HashSet<&'static str>,
}

impl DataDirectory {
    pub fn new() -> SomaResult<Self> {
        let path = {
            if let Some(dir) = std::env::var_os(SOMA_DATA_DIR_ENV_NAME) {
                dir.into()
            } else {
                let mut home = dirs::home_dir().ok_or(SomaError::DataDirectoryAccessDenied)?;
                home.push(SOMA_DATA_DIR_NAME);
                home
            }
        };

        if !path.exists() {
            fs::create_dir_all(&path)?;
            println!("Created Soma data directory at: {}", path.to_string_lossy());
        }

        DataDirectory::initialize_and_lock(path)
    }

    pub fn at_path(path: impl AsRef<Path>) -> SomaResult<Self> {
        fs::create_dir_all(&path)?;
        DataDirectory::initialize_and_lock(path.as_ref().to_owned())
    }

    fn initialize_and_lock(path: PathBuf) -> SomaResult<Self> {
        let lock = File::create(path.join(LOCK_FILE_NAME))?;
        lock.try_lock_exclusive()
            .or(Err(SomaError::DataDirectoryLockFailed))?;

        Ok(DataDirectory {
            root_path: path,
            lock,
            manager_set: HashSet::new(),
        })
    }

    pub fn register<'a, T>(&'a mut self) -> SomaResult<T>
    where
        T: DirectoryManager<'a>,
    {
        if !self.manager_set.insert(T::DIR) {
            panic!("A manager should be registered only once");
        }

        let manager_root = self.root_path.join(T::DIR);
        fs::create_dir_all(&manager_root)?;
        Ok(T::new(Registration::new(self, manager_root))?)
    }
}

impl Drop for DataDirectory {
    fn drop(&mut self) {
        if self.lock.unlock().is_err() {
            eprintln!("Failed to unlock the data directory");
        }
    }
}

pub struct Registration<'a, T>
where
    T: DirectoryManager<'a>,
{
    data_dir: &'a DataDirectory,
    root_path: PathBuf,
    phantom: PhantomData<*const T>,
}

impl<'a, T> Registration<'a, T>
where
    T: DirectoryManager<'a>,
{
    fn new(data_dir: &'a DataDirectory, root_path: PathBuf) -> Self {
        Registration {
            data_dir,
            root_path,
            phantom: PhantomData,
        }
    }

    pub fn data_dir(&self) -> &'a DataDirectory {
        self.data_dir
    }

    pub fn root_path(&self) -> &PathBuf {
        &self.root_path
    }
}

pub trait DirectoryManager<'a>: Deref<Target = Registration<'a, Self>>
where
    Self: Sized + 'a,
{
    const DIR: &'static str;

    fn new(registration: Registration<'a, Self>) -> SomaResult<Self>;
}

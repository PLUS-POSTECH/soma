use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use hyper::client::connect::Connect;
use lazy_static::lazy_static;
use tempfile::TempDir;
use tokio::runtime::current_thread::Runtime;

use soma::data_dir::DataDirectory;
use soma::docker::connect_default;
use soma::{Environment, NameString};

pub use self::test_printer::TestPrinter;

mod test_printer;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub const SIMPLE_BOF_GIT: &str = "https://github.com/PLUS-POSTECH/simple-bof.git";
pub const BATA_LIST_GIT: &str = "https://github.com/PLUS-POSTECH/soma-bata-list.git";

lazy_static! {
    pub static ref SIMPLE_BOF_REPO_NAME: NameString = NameString::try_from("simple-bof").unwrap();
    pub static ref BATA_LIST_REPO_NAME: NameString =
        NameString::try_from("soma-bata-list").unwrap();
}

pub fn test_env(data_dir: &mut DataDirectory) -> Environment<impl Connect, TestPrinter> {
    Environment::new(
        format!("soma-test-{}", COUNTER.fetch_add(1, Ordering::SeqCst)),
        data_dir,
        connect_default().expect("Failed to connect to docker"),
        TestPrinter::new(),
    )
    .expect("Failed to create environment")
}

pub fn tempdir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

pub fn temp_data_dir() -> (TempDir, DataDirectory) {
    let tempdir = tempdir();
    let data_dir = DataDirectory::at_path(tempdir.path()).expect("Failed to create data directory");
    (tempdir, data_dir)
}

pub fn expect_dir_contents(directory: impl AsRef<Path>, file_names: &[impl AsRef<OsStr>]) {
    let dir_set: HashSet<OsString> = fs::read_dir(directory)
        .expect("Failed to read the directory")
        .filter_map(|dir| match dir {
            Ok(entry) => Some(entry.file_name()),
            Err(_) => None,
        })
        .collect();
    assert_eq!(
        file_names
            .iter()
            .map(|ostr| ostr.as_ref().to_owned())
            .collect::<HashSet<OsString>>(),
        dir_set
    );
}

pub fn dir_contents_exists(directory: impl AsRef<Path>, file_names: &[impl AsRef<OsStr>]) -> bool {
    let dir_set: HashSet<OsString> = fs::read_dir(directory)
        .expect("Failed to read the directory")
        .filter_map(|dir| match dir {
            Ok(entry) => Some(entry.file_name()),
            Err(_) => None,
        })
        .collect();
    file_names
        .iter()
        .map(|ostr| ostr.as_ref().to_owned())
        .collect::<HashSet<OsString>>()
        .is_subset(&dir_set)
}

pub fn default_runtime() -> Runtime {
    Runtime::new().expect("Failed to initialize tokio runtime")
}

pub fn error_downcast(error: failure::Error) -> Result<soma::error::Error, failure::Error> {
    error.downcast()
}

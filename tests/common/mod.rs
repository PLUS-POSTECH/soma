use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;

use hyper::client::connect::Connect;
use tempfile::TempDir;
use tokio::runtime::current_thread::Runtime;

use soma::data_dir::DataDirectory;
use soma::docker::connect_default;
use soma::Environment;

pub use self::test_printer::TestPrinter;

mod test_printer;

pub fn test_env(data_dir: &mut DataDirectory) -> Environment<impl Connect, TestPrinter> {
    Environment::new(
        "soma-test".to_owned(),
        data_dir,
        connect_default().expect("failed to connect to docker"),
        TestPrinter::new(),
    )
    .expect("failed to create environment")
}

pub fn tempdir() -> TempDir {
    tempfile::tempdir().expect("failed to create temporary directory")
}

pub fn temp_data_dir() -> (TempDir, DataDirectory) {
    let tempdir = tempdir();
    let data_dir = DataDirectory::at_path(tempdir.path()).expect("failed to create data directory");
    (tempdir, data_dir)
}

pub fn expect_dir_contents(directory: impl AsRef<Path>, file_names: &[impl AsRef<OsStr>]) {
    let dir_set: HashSet<OsString> = fs::read_dir(directory)
        .expect("failed to read the directory")
        .into_iter()
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
        .expect("failed to read the directory")
        .into_iter()
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
    Runtime::new().expect("failed to initialize tokio runtime")
}

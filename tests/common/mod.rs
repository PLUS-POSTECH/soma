use std::collections::HashSet;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::Path;

use hyper::client::connect::Connect;
use tempfile::TempDir;
use tokio::runtime::current_thread::Runtime;

use soma::data_dir::DataDirectory;
use soma::docker::{connect_default, SomaContainer, SomaImage};
use soma::Environment;

pub use self::test_printer::TestPrinter;

mod test_printer;

pub fn test_env(temp_dir: &TempDir) -> Environment<impl Connect, TestPrinter> {
    Environment::new(
        "soma-test".to_owned(),
        DataDirectory::at_path(temp_dir.path()).expect("failed to create data directory"),
        connect_default().expect("failed to connect to docker"),
        TestPrinter::new(),
    )
}

pub fn tempdir() -> TempDir {
    tempfile::tempdir().expect("failed to create temporary directory")
}

pub fn expect_dir_contents(directory: impl AsRef<Path>, file_names: &[impl AsRef<OsStr>]) {
    let dir_set: HashSet<OsString> = fs::read_dir(directory)
        .expect("failed to read the directory")
        .into_iter()
        .filter_map(|dir| match dir {
            Ok(entry) => {
                println!("{:?}", entry.file_name());
                Some(entry.file_name())
            }
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

pub fn expect_image_exists(images: &Vec<SomaImage>, image_name: &str, negate: bool) {
    assert_eq!(
        images.iter().any(|image| match &image.image().repo_tags {
            Some(tags) => tags
                .iter()
                .any(|tag| tag.starts_with(format!("{}:", image_name).as_str())),
            None => false,
        }),
        !negate
    )
}

pub fn expect_image_from_repo_exists(images: &Vec<SomaImage>, repo_name: &str, negate: bool) {
    assert_eq!(
        images
            .iter()
            .any(|image| image.repository_name() == repo_name),
        !negate
    )
}

pub fn expect_container_exists(containers: &Vec<SomaContainer>, container_id: &str, negate: bool) {
    assert_eq!(
        containers
            .iter()
            .any(|container| container.container().id == container_id),
        !negate
    )
}

pub fn expect_container_from_repo_exists(
    containers: &Vec<SomaContainer>,
    repo_name: &str,
    negate: bool,
) {
    assert_eq!(
        containers
            .iter()
            .any(|container| container.repository_name() == repo_name),
        !negate
    )
}

pub fn default_runtime() -> Runtime {
    Runtime::new().expect("failed to initialize tokio runtime")
}

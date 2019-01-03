use hyper::client::connect::Connect;
use tempfile::TempDir;

use soma::data_dir::DataDirectory;
use soma::docker::connect_default;
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

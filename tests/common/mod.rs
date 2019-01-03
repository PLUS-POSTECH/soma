use hyper::client::connect::Connect;
use tempfile::TempDir;

use soma::data_dir::DataDirectory;
use soma::docker::connect_default;
use soma::error::Result as SomaResult;
use soma::Environment;

pub use self::test_printer::TestPrinter;

mod test_printer;

pub fn test_env(temp_dir: &TempDir) -> SomaResult<Environment<impl Connect, TestPrinter>> {
    Ok(Environment::new(
        "soma-test".to_owned(),
        DataDirectory::at_path(temp_dir.path())?,
        connect_default()?,
        TestPrinter::new(),
    ))
}

use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "soma failed to use data directory")]
    DataDirectoryError,
}

pub type Result<T> = std::result::Result<T, failure::Error>;

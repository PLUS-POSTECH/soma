use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "soma failed to access the data directory")]
    DataDirectoryAccessError,
    #[fail(display = "another soma instance is using the data directory")]
    DataDirectoryLockError,
    #[fail(display = "soma found that the repository is invalid")]
    InvalidRepositoryError,
}

pub type Result<T> = std::result::Result<T, failure::Error>;

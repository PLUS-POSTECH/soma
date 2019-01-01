use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed to access the data directory")]
    DataDirectoryAccessError,
    #[fail(display = "another soma instance is using the data directory")]
    DataDirectoryLockError,
    #[fail(display = "a repository with the same name already exists")]
    DuplicateRepositoryError,
    #[fail(display = "the specified repository path is invalid")]
    InvalidRepositoryPathError,
}

pub type Result<T> = std::result::Result<T, failure::Error>;

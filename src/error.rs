use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "failed to access the data directory")]
    DataDirectoryAccessError,
    #[fail(display = "another Soma instance is using the data directory")]
    DataDirectoryLockError,
    #[fail(display = "image build process exited with non-zero status code")]
    DockerBuildFailError,
    #[fail(display = "a repository with the same name already exists")]
    DuplicateRepositoryError,
    #[fail(display = "the specified repository contains invalid path")]
    InvalidRepositoryPathError,
    #[fail(display = "some entry in the manifest is invalid")]
    InvalidManifestError,
    #[fail(display = "the specified repository is not found")]
    RepositoryNotFoundError,
}

pub type Result<T> = std::result::Result<T, failure::Error>;

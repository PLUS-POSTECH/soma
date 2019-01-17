use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to access the data directory")]
    DataDirectoryAccessError,
    #[fail(display = "Another Soma instance is using the data directory")]
    DataDirectoryLockError,
    #[fail(display = "Image build process exited with non-zero status code")]
    DockerBuildFailError,
    #[fail(display = "A repository with the same name already exists")]
    DuplicateRepositoryError,
    #[fail(display = "Failed to detect filename from the path")]
    FileNameNotFoundError,
    #[fail(display = "Some entry in the manifest is invalid")]
    InvalidManifestError,
    #[fail(display = "The specified file's path contains unsupported characters")]
    InvalidUnicodeError,
    #[fail(display = "There is no container running for the specified problem")]
    ProblemNotRunningError,
    #[fail(display = "There is an image or an container from the repository")]
    RepositoryInUseError,
    #[fail(display = "The specified repository is not found")]
    RepositoryNotFoundError,
}

pub type Result<T> = std::result::Result<T, failure::Error>;

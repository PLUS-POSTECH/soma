use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to access the data directory")]
    DataDirectoryAccessDenied,
    #[fail(display = "Another Soma instance is using the data directory")]
    DataDirectoryLockFailed,
    #[fail(display = "Failed to build docker image for a problem")]
    DockerBuildFailed,
    #[fail(display = "A repository with the same name already exists")]
    DuplicateRepository,
    #[fail(display = "Failed to detect filename from the path")]
    FileNameNotFound,
    #[fail(
        display = "The specified file does not exist, or you don't have enough permission to access it"
    )]
    FileUnreachable,
    #[fail(display = "Some entry in the manifest is invalid")]
    InvalidManifest,
    #[fail(display = "The provided repository does not contain 'soma.toml' or 'soma-list.toml'")]
    InvalidRepository,
    #[fail(display = "soma-list.toml contains a duplicate or inaccessible entry")]
    InvalidSomaList,
    #[fail(
        display = "The name doesn't satisfy docker name component rules, which allows lower case alphanumerics with non-boundary '_', '__', or (multiple) '-'(s)"
    )]
    InvalidName,
    #[fail(display = "The specified file's path contains unsupported characters")]
    InvalidUnicode,
    #[fail(display = "There is a container already running for the specified problem")]
    ProblemAlreadyRunning,
    #[fail(display = "The specified problem is not found")]
    ProblemNotFound,
    #[fail(display = "There is no container running for the specified problem")]
    ProblemNotRunning,
    #[fail(display = "The provided query returned multiple problems")]
    ProblemQueryAmbiguous,
    #[fail(display = "There is an image or an container from the repository")]
    RepositoryInUse,
    #[fail(display = "The specified repository is not found")]
    RepositoryNotFound,
    #[fail(
        display = "The repository contains changes that cannot be handled by update command; Please remove and add the repository manually"
    )]
    UnsupportedUpdate,
}

pub type Result<T> = std::result::Result<T, failure::Error>;

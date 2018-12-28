use failure::Fail;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Unknown error")]
    Unknown,
    #[fail(display = "error: {}", _0)]
    Message(String),
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::Unknown
    }
}

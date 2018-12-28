use std::io;

use failure::Fail;
use toml;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "I/O error: {}", _0)]
    Io(#[cause] io::Error),
    #[fail(display = "Toml deserialization error: {}", _0)]
    TomlDe(#[cause] toml::de::Error),
    #[fail(display = "Unknown error")]
    Unknown,
    #[fail(display = "error: {}", _0)]
    Message(String),
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Self {
        Error::TomlDe(e)
    }
}

impl From<()> for Error {
    fn from(_: ()) -> Self {
        Error::Unknown
    }
}

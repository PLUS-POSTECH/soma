use std::cell::{RefCell, RefMut};
use std::convert::From;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use bollard::Docker;
use clap::crate_version;
use hyper::client::connect::Connect;
use lazy_static::lazy_static;
use regex::Regex;
use serde::de::{self, Deserializer, Unexpected, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::data_dir::DataDirectory;
use crate::prelude::*;
use crate::repository::RepositoryManager;

pub mod data_dir;
pub mod docker;
pub mod error;
pub mod ops;
pub mod prelude;
pub mod problem;
pub mod repository;
pub mod template;

pub const VERSION: &str = crate_version!();

pub trait Printer {
    type Handle;

    fn get_current_handle(&mut self) -> Self::Handle;
    fn write_line_at(&mut self, handle: &Self::Handle, message: &str);
    fn write_line(&mut self, message: &str);
}

pub struct Environment<'a, C: 'static, P: Printer + 'static> {
    username: String,
    repo_manager: RepositoryManager<'a>,
    docker: Docker<C>,
    printer: RefCell<P>,
}

impl<'a, C, P> Environment<'a, C, P>
where
    C: Connect,
    P: Printer,
{
    pub fn new(
        username: String,
        data_dir: &'a mut DataDirectory,
        docker: Docker<C>,
        printer: P,
    ) -> SomaResult<Environment<'a, C, P>> {
        let repo_manager = data_dir.register::<RepositoryManager>()?;

        Ok(Environment {
            username,
            repo_manager,
            docker,
            printer: RefCell::new(printer),
        })
    }

    pub fn username(&self) -> &String {
        &self.username
    }

    pub fn printer(&self) -> RefMut<P> {
        self.printer.borrow_mut()
    }

    pub fn repo_manager(&self) -> &RepositoryManager<'a> {
        &self.repo_manager
    }

    pub fn repo_manager_mut(&mut self) -> &mut RepositoryManager<'a> {
        &mut self.repo_manager
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NameString {
    inner: String,
}

lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"^[a-z0-9]+((?:[._]|__|[-]*)[a-z0-9]+)*$").unwrap();
}

impl NameString {
    pub fn new(s: impl AsRef<str>) -> SomaResult<NameString> {
        let s = s.as_ref();
        if NAME_REGEX.is_match(s) {
            Ok(NameString {
                inner: s.to_owned(),
            })
        } else {
            Err(SomaError::InvalidName)?
        }
    }

    // TODO: Use TryFrom trait when Rust stabilizes it.
    pub fn try_from(s: String) -> SomaResult<NameString> {
        if NAME_REGEX.is_match(&s) {
            Ok(NameString { inner: s })
        } else {
            Err(SomaError::InvalidName)?
        }
    }
}

impl PartialEq<String> for NameString {
    fn eq(&self, other: &String) -> bool {
        &self.inner == other
    }
}

impl PartialEq<str> for NameString {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl PartialEq<NameString> for String {
    fn eq(&self, other: &NameString) -> bool {
        self == &other.inner
    }
}

impl PartialEq<NameString> for str {
    fn eq(&self, other: &NameString) -> bool {
        self == other.inner
    }
}

impl fmt::Display for NameString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl AsRef<str> for NameString {
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl AsRef<Path> for NameString {
    fn as_ref(&self) -> &Path {
        Path::new(&self.inner)
    }
}

impl From<NameString> for Box<str> {
    fn from(s: NameString) -> Box<str> {
        s.inner.into()
    }
}

impl From<NameString> for String {
    fn from(s: NameString) -> String {
        s.inner
    }
}

impl Serialize for NameString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_ref())
    }
}

impl<'de> Deserialize<'de> for NameString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(NameStringVisitor)
    }
}

struct NameStringVisitor;

impl<'de> Visitor<'de> for NameStringVisitor {
    type Value = NameString;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "a string satisfying docker name component rules")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let name = NameString::new(s);
        match name {
            Ok(name) => Ok(name),
            Err(_) => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
        }
    }
}

fn read_file_contents(path: impl AsRef<Path>) -> SomaResult<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_de_tokens_error, assert_tokens, Token};

    #[test]
    fn test_name_eq() {
        assert_eq!("asdf", &NameString::new("asdf").unwrap());
        assert_eq!(&NameString::new("asdf").unwrap(), "asdf");
        assert_eq!(String::from("asdf"), NameString::new("asdf").unwrap());
        assert_eq!(NameString::new("asdf").unwrap(), String::from("asdf"));
        assert_ne!("qwer", &NameString::new("asdf").unwrap());
        assert_ne!(&NameString::new("qwer").unwrap(), "asdf");
        assert_ne!(String::from("qwer"), NameString::new("asdf").unwrap());
        assert_ne!(NameString::new("qwer").unwrap(), String::from("asdf"));
    }

    #[test]
    fn test_name_serde() {
        assert_tokens(&NameString::new("asdf0").unwrap(), &[Token::Str("asdf0")]);
        assert_tokens(
            &NameString::new("asdf0.qwer").unwrap(),
            &[Token::Str("asdf0.qwer")],
        );
        assert_tokens(
            &NameString::new("asdf0_qwer").unwrap(),
            &[Token::Str("asdf0_qwer")],
        );
        assert_tokens(
            &NameString::new("asdf0__qwer").unwrap(),
            &[Token::Str("asdf0__qwer")],
        );
        assert_tokens(
            &NameString::new("asdf0-qwer").unwrap(),
            &[Token::Str("asdf0-qwer")],
        );
        assert_tokens(
            &NameString::new("asdf0--qwer").unwrap(),
            &[Token::Str("asdf0--qwer")],
        );
        assert_tokens(
            &NameString::new("asdf0---qwer").unwrap(),
            &[Token::Str("asdf0---qwer")],
        );
    }

    #[test]
    fn test_name_de_error() {
        assert_de_tokens_error::<NameString>(
            &[Token::Str("")],
            "invalid value: string \"\", expected a string satisfying docker name component rules",
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("ASDF")
            ], "invalid value: string \"ASDF\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("AS@DF")
            ], "invalid value: string \"AS@DF\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("asdf..qwer")
            ], "invalid value: string \"asdf..qwer\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("asdf___qwer")
            ], "invalid value: string \"asdf___qwer\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("asdf.")
            ], "invalid value: string \"asdf.\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("asdf_")
            ], "invalid value: string \"asdf_\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("asdf-")
            ], "invalid value: string \"asdf-\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str(".asdf")
            ], "invalid value: string \".asdf\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("_asdf")
            ], "invalid value: string \"_asdf\", expected a string satisfying docker name component rules"
        );
        assert_de_tokens_error::<NameString>(&[
                Token::Str("-asdf")
            ], "invalid value: string \"-asdf\", expected a string satisfying docker name component rules"
        );
    }
}

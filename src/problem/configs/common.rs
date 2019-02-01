use std::fmt;
use std::path::{Path, PathBuf};

use path_slash::PathBufExt;
use serde::de::{self, Deserializer, Unexpected, Visitor};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

use crate::prelude::*;

#[derive(Debug, PartialEq)]
pub enum FilePermissions {
    Custom(u16),
    Executable,
    ReadOnly,
    // Reserved: FetchOnly, ReadWrite
}

impl Serialize for FilePermissions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let permissions_string = match self {
            FilePermissions::Custom(permissions) => format!("{:o}", permissions),
            FilePermissions::Executable => "550".to_owned(),
            FilePermissions::ReadOnly => "440".to_owned(),
        };
        serializer.serialize_str(&permissions_string)
    }
}

impl<'de> Deserialize<'de> for FilePermissions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PermissionsString)
    }
}

struct PermissionsString;

impl<'de> Visitor<'de> for PermissionsString {
    type Value = FilePermissions;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a file permissions string in octal number format"
        )
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let permissions = u16::from_str_radix(s, 8);
        match permissions {
            // Support sticky bits later
            Ok(permissions) if permissions <= 0o777 => Ok(FilePermissions::Custom(permissions)),
            _ => Err(de::Error::invalid_value(Unexpected::Str(s), &self)),
        }
    }
}

// target_path is defined as String instead of PathBuf to support Windows
#[derive(Deserialize)]
pub struct FileEntry {
    path: PathBuf,
    public: Option<bool>,
    target_path: Option<String>,
}

#[derive(Serialize)]
pub struct SolidFileEntry {
    path: PathBuf,
    public: bool,
    target_path: String,
    permissions: FilePermissions,
}

impl FileEntry {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn public(&self) -> bool {
        self.public.unwrap_or(false)
    }

    pub fn solidify(
        &self,
        work_dir: impl AsRef<Path>,
        permissions: FilePermissions,
    ) -> SomaResult<SolidFileEntry> {
        let target_path = match &self.target_path {
            Some(path) => PathBuf::from_slash(path),
            None => {
                let file_name = self.path.file_name().ok_or(SomaError::FileNameNotFound)?;
                work_dir.as_ref().join(file_name)
            }
        };
        Ok(SolidFileEntry {
            path: self.path.clone(),
            public: self.public.unwrap_or(false),
            target_path: target_path.to_slash().ok_or(SomaError::InvalidUnicode)?,
            permissions,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, Token};

    #[test]
    fn test_file_permissions_ser() {
        assert_ser_tokens(&FilePermissions::Executable, &[Token::Str("550")]);
        assert_ser_tokens(&FilePermissions::ReadOnly, &[Token::Str("440")]);
        assert_ser_tokens(&FilePermissions::Custom(0o777), &[Token::Str("777")]);
    }

    #[test]
    fn test_file_permissions_de() {
        assert_de_tokens(&FilePermissions::Custom(0o550), &[Token::Str("550")]);
        assert_de_tokens(&FilePermissions::Custom(0o440), &[Token::Str("440")]);
        assert_de_tokens(&FilePermissions::Custom(0o777), &[Token::Str("777")]);
        assert_de_tokens_error::<FilePermissions>(&[
                Token::Str("1000")
            ], "invalid value: string \"1000\", expected a file permissions string in octal number format"
        );
    }
}

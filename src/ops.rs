use std::fs;
use std::path::Path;

use hyper::client::connect::Connect;
use url::Url;

use crate::error::{Error as SomaError, Result as SomaResult};
use crate::repo::backend::Backend;
use crate::repo::load_manifest;
use crate::repo::MANIFEST_FILE_NAME;
use crate::Environment;
use crate::Printer;

pub fn parse_repo_url(url: &str) -> SomaResult<(String, Backend)> {
    let path = Path::new(url);
    if path.is_dir() {
        // local backend
        Ok((
            format!(
                "#{}",
                path.file_name()
                    .ok_or(SomaError::InvalidRepositoryPathError)?
                    .to_str()
                    .ok_or(SomaError::InvalidRepositoryPathError)?
            ),
            Backend::LocalBackend(path.canonicalize()?.to_owned()),
        ))
    } else {
        // git backend
        let parsed_url = Url::parse(url).or(Err(SomaError::InvalidRepositoryPathError))?;
        let last_name = parsed_url
            .path_segments()
            .ok_or(SomaError::InvalidRepositoryPathError)?
            .last()
            .ok_or(SomaError::InvalidRepositoryPathError)?;
        let repo_name = if last_name.ends_with(".git") {
            &last_name[..last_name.len() - 4]
        } else {
            &last_name
        };
        Ok((repo_name.to_owned(), Backend::GitBackend(url.to_owned())))
    }
}

pub fn fetch(
    env: &Environment<impl Connect + 'static, impl Printer>,
    problem_name: &str,
) -> SomaResult<()> {
    let repo_path = env.data_dir().repo_path(problem_name);
    let repo_manifest_path = repo_path.join(MANIFEST_FILE_NAME);
    let manifest = load_manifest(repo_manifest_path)?;
    let executables = manifest.executable().iter();
    let readonly = manifest.readonly().iter();
    executables
        .chain(readonly)
        .filter(|file_entry| file_entry.public())
        .try_for_each(|file_entry| {
            let file_path = repo_path.join(file_entry.path());
            let file_name = file_path
                .file_name()
                .ok_or(SomaError::InvalidRepositoryPathError)?
                .to_str()
                .ok_or(SomaError::InvalidRepositoryPathError)?;

            env.printer()
                .write_line(&format!("Fetching '{}'", file_name));
            fs::copy(&file_path, file_name)?;
            Ok(())
        })
}

pub fn pull(
    env: &Environment<impl Connect + 'static, impl Printer>,
    repo_name: &str,
) -> SomaResult<()> {
    let repo_index = env.data_dir().read_repo_index()?;
    let repository = repo_index
        .get(repo_name)
        .ok_or(SomaError::RepositoryNotFoundError)?;
    let backend = repository.backend();
    let local_path = repository.local_path();
    backend.update(local_path)?;
    env.printer().write_line(&format!(
        "successfully updated repository: '{}'",
        &repo_name
    ));
    Ok(())
}

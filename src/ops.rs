use std::fs;
use std::fs::remove_dir_all;
use std::path::Path;

use fs_extra::dir::{copy, CopyOptions};
use futures::future::Future;
use handlebars::Handlebars;
use hyper::client::connect::Connect;
use tempfile::tempdir;
use tokio::runtime::current_thread::Runtime;
use url::Url;

use crate::docker;
use crate::error::{Error as SomaError, Result as SomaResult};
use crate::repo::backend::Backend;
use crate::repo::{load_manifest, Repository, MANIFEST_FILE_NAME};
use crate::template::{HandleBarsExt, RenderingContext, Templates};
use crate::Environment;
use crate::Printer;

pub fn location_to_backend(repo_location: &str) -> SomaResult<(String, Backend)> {
    let path = Path::new(repo_location);
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
        let parsed_url =
            Url::parse(repo_location).or(Err(SomaError::InvalidRepositoryPathError))?;
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
        Ok((
            repo_name.to_owned(),
            Backend::GitBackend(repo_location.to_owned()),
        ))
    }
}

pub fn add(
    env: &Environment<impl Connect + 'static, impl Printer>,
    repo_location: &str,
) -> SomaResult<()> {
    let (repo_name, backend) = location_to_backend(repo_location)?;
    env.data_dir().add_repo(repo_name.clone(), backend)?;
    env.printer()
        .write_line(&format!("successfully added a repository '{}'", &repo_name));
    Ok(())
}

pub fn fetch(
    env: &Environment<impl Connect + 'static, impl Printer>,
    problem_name: &str,
    cwd: impl AsRef<Path>,
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
                .ok_or(SomaError::InvalidRepositoryPathError)?;

            env.printer()
                .write_line(&format!("Fetching '{}'", file_name.to_string_lossy()));
            fs::copy(&file_path, cwd.as_ref().join(file_name))?;
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

fn build_image(
    repository: &Repository,
    env: &Environment<impl Connect + 'static, impl Printer>,
    problem_name: &str,
) -> SomaResult<()> {
    let work_dir = tempdir()?;
    let work_dir_path = work_dir.path();
    let repo_path = repository.local_path();
    let image_name = format!("soma/{}", problem_name);

    remove_dir_all(&work_dir)?;
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy(&repo_path, &work_dir, &copy_options)?;

    let manifest = load_manifest(work_dir_path.join(MANIFEST_FILE_NAME))?
        .convert_to_docker_entry(&format!("/home/{}", problem_name))?;

    let render_context = RenderingContext::new(env.username(), repository.name(), manifest);
    Handlebars::new().render_templates(Templates::Binary, &render_context, work_dir_path)?;

    docker::build(&image_name, work_dir_path)?;
    work_dir.close()?;

    Ok(())
}

fn run_container(
    env: &Environment<impl Connect + 'static, impl Printer>,
    problem_name: &str,
    runtime: &mut Runtime,
) -> SomaResult<String> {
    let image_name = format!("soma/{}", problem_name);
    let container_run = docker::create(env, &image_name)
        .and_then(|container_name| docker::start(env, &container_name).map(|_| container_name));
    runtime.block_on(container_run)
}

pub fn run(
    env: &Environment<impl Connect + 'static, impl Printer>,
    problem_name: &str,
    mut runtime: &mut Runtime,
) -> SomaResult<()> {
    let repo_name = problem_name;
    let repo_index = env.data_dir().read_repo_index()?;
    let repository = repo_index
        .get(repo_name)
        .ok_or(SomaError::RepositoryNotFoundError)?;

    build_image(&repository, &env, repo_name)?;
    env.printer().write_line(&format!(
        "successfully built image for problem: '{}'",
        &repo_name
    ));

    let container_name = run_container(&env, repo_name, &mut runtime)?;
    env.printer().write_line(&format!(
        "successfully started container: '{}'",
        &container_name
    ));
    Ok(())
}

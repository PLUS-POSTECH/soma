use std::fs;
use std::path::Path;

use fs_extra::dir::{copy, CopyOptions};
use futures::future::Future;
use handlebars::Handlebars;
use hyper::client::connect::Connect;
use remove_dir_all::remove_dir_all;
use tempfile::tempdir;
use tokio::runtime::current_thread::Runtime;
use url::Url;

use crate::docker;
use crate::prelude::*;
use crate::repo::{load_manifest, Backend, Repository, MANIFEST_FILE_NAME};
use crate::template::{HandleBarsExt, RenderingContext, Templates};
use crate::Environment;
use crate::Printer;

pub fn location_to_backend(repo_location: &str) -> SomaResult<(String, Backend)> {
    let path = Path::new(repo_location);
    if path.is_dir() {
        // local backend
        Ok((
            path.file_name()
                .ok_or(SomaError::FileNameNotFoundError)?
                .to_str()
                .ok_or(SomaError::InvalidUnicodeError)?
                .to_owned(),
            Backend::LocalBackend(path.canonicalize()?.to_owned()),
        ))
    } else {
        // git backend
        let parsed_url = Url::parse(repo_location).or(Err(SomaError::RepositoryNotFoundError))?;
        let last_name = parsed_url
            .path_segments()
            .ok_or(SomaError::RepositoryNotFoundError)?
            .last()
            .ok_or(SomaError::FileNameNotFoundError)?;
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
    env: &mut Environment<impl Connect, impl Printer>,
    repo_location: &str,
    repo_name: Option<&str>,
) -> SomaResult<()> {
    let (resolved_repo_name, backend) = location_to_backend(repo_location)?;
    let repo_name = match repo_name {
        Some(repo_name) => repo_name.to_owned(),
        None => resolved_repo_name,
    };

    env.repo_manager_mut()
        .add_repo(repo_name.clone(), backend)?;

    let repository = env.repo_manager().get_repo(&repo_name)?;
    repository.update()?;

    env.printer()
        .write_line(&format!("Repository added: '{}'", &repo_name))?;

    Ok(())
}

pub fn fetch(
    env: &Environment<impl Connect, impl Printer>,
    problem_name: &str,
    cwd: impl AsRef<Path>,
) -> SomaResult<()> {
    let repo_name = problem_name;
    let repository = env.repo_manager().get_repo(repo_name)?;
    let repo_path = repository.local_path();

    let manifest = load_manifest(repo_path.join(MANIFEST_FILE_NAME))?;
    let executables = manifest.executable().iter();
    let readonly = manifest.readonly().iter();

    executables
        .chain(readonly)
        .filter(|file_entry| file_entry.public())
        .try_for_each(|file_entry| {
            let file_path = repo_path.join(file_entry.path());
            let file_name = file_path
                .file_name()
                .ok_or(SomaError::FileNameNotFoundError)?;

            env.printer()
                .write_line(&format!("Fetching '{}'...", file_name.to_string_lossy()))?;
            fs::copy(&file_path, cwd.as_ref().join(file_name))?;
            Ok(())
        })
}

pub fn build(env: &Environment<impl Connect, impl Printer>, problem_name: &str) -> SomaResult<()> {
    let repo_name = problem_name;
    let repository = env.repo_manager().get_repo(repo_name)?;
    repository.update()?;
    env.printer()
        .write_line(&format!("Repository updated: '{}'", &repo_name))?;

    build_image(&repository, &env, repo_name)?;
    env.printer()
        .write_line(&format!("Built image for problem: '{}'", &repo_name))?;
    Ok(())
}

fn build_image(
    repository: &Repository,
    env: &Environment<impl Connect, impl Printer>,
    problem_name: &str,
) -> SomaResult<()> {
    let work_dir = tempdir()?;
    let work_dir_path = work_dir.path();
    let repo_path = repository.local_path();
    let image_name = docker::image_name(problem_name);

    remove_dir_all(&work_dir)?;
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy(&repo_path, &work_dir, &copy_options)?;

    let manifest = load_manifest(work_dir_path.join(MANIFEST_FILE_NAME))?.solidify()?;

    let context = RenderingContext::new(env.username(), repository.name(), manifest);
    Handlebars::new().render_templates(Templates::Binary, &context, work_dir_path)?;

    docker::build(&image_name, work_dir_path)?;
    work_dir.close()?;

    Ok(())
}

fn run_container(
    env: &Environment<impl Connect, impl Printer>,
    problem_name: &str,
    port: u32,
    runtime: &mut Runtime,
) -> SomaResult<String> {
    let image_name = docker::image_name(problem_name);
    let repo_name = problem_name;
    let port_str = port.to_string();

    let container_run = docker::create(env, repo_name, &image_name, &port_str)
        .and_then(|container_name| docker::start(env, &container_name).map(|_| container_name));
    runtime.block_on(container_run)
}

pub fn run(
    env: &Environment<impl Connect, impl Printer>,
    problem_name: &str,
    port: u32,
    mut runtime: &mut Runtime,
) -> SomaResult<String> {
    let container_name = run_container(&env, problem_name, port, &mut runtime)?;
    env.printer()
        .write_line(&format!("Container started: '{}'", &container_name))?;
    Ok(container_name)
}

pub fn remove(
    env: &mut Environment<impl Connect, impl Printer>,
    repo_name: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let image_list = runtime.block_on(docker::list_images(env))?;
    if docker::image_from_repo_exists(&image_list, repo_name) {
        Err(SomaError::RepositoryInUseError)?;
    }

    env.repo_manager_mut().remove_repo(repo_name)?;
    env.printer()
        .write_line(&format!("Repository removed: '{}'", &repo_name))?;

    Ok(())
}

pub fn clean(
    env: &Environment<impl Connect, impl Printer>,
    problem_name: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let container_list = runtime.block_on(docker::list_containers(env))?;
    if docker::container_from_repo_exists(&container_list, problem_name) {
        Err(SomaError::RepositoryInUseError)?;
    }

    let image_name = docker::image_name(problem_name);
    runtime.block_on(docker::remove_image(env, &image_name))?;
    env.printer()
        .write_line(&format!("Problem image cleaned: '{}'", &problem_name))?;

    Ok(())
}

pub fn stop(
    env: &Environment<impl Connect, impl Printer>,
    problem_name: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let container_list = runtime.block_on(docker::list_containers(env))?;
    if !docker::container_from_repo_exists(&container_list, problem_name) {
        Err(SomaError::ProblemNotRunningError)?;
    }

    let container_list = docker::containers_from_repo(container_list, problem_name);
    let states_to_stop = vec!["paused", "restarting", "running"];

    let containers_to_stop = container_list
        .iter()
        .filter(|container| states_to_stop.contains(&container.container().state.as_str()));

    for container in containers_to_stop {
        runtime.block_on(docker::stop(env, &container.container().id))?;
    }

    for container in container_list {
        runtime.block_on(docker::remove_container(env, &container.container().id))?;
    }

    env.printer()
        .write_line(&format!("Problem stopped: '{}'", &problem_name))?;

    Ok(())
}

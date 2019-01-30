use std::fs;
use std::path::Path;

use flate2::write::GzEncoder;
use flate2::Compression;
use fs_extra::dir::{copy, CopyOptions};
use futures::{Future, Stream};
use handlebars::Handlebars;
use hyper::client::connect::Connect;
use remove_dir_all::remove_dir_all;
use tempfile::tempdir;
use tokio::runtime::current_thread::Runtime;
use url::Url;

use crate::docker;
use crate::prelude::*;
use crate::repo::{Backend, Problem};
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
        .write_line(&format!("Repository added: '{}'", &repo_name));

    Ok(())
}

pub fn fetch(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    cwd: impl AsRef<Path>,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;
    let manifest = problem.load_manifest()?;

    let binary = manifest.binary();
    let executables = binary.executable().iter();
    let readonly = binary.readonly().iter();

    executables
        .chain(readonly)
        .filter(|file_entry| file_entry.public())
        .try_for_each(|file_entry| {
            let file_path = problem.path().join(file_entry.path());
            let file_name = file_path
                .file_name()
                .ok_or(SomaError::FileNameNotFoundError)?;

            env.printer()
                .write_line(&format!("Fetching '{}'...", file_name.to_string_lossy()));
            fs::copy(&file_path, cwd.as_ref().join(file_name))?;
            Ok(())
        })
}

pub fn build(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;
    let repo_name = problem.repo_name();

    let repository = env.repo_manager().get_repo(repo_name).unwrap();
    repository.update()?;
    env.printer()
        .write_line(&format!("Repository updated: '{}'", repo_name));

    runtime.block_on(docker::prune_images_from_prob(&env, &problem))?;
    build_image(&env, &problem, runtime)?;
    env.printer()
        .write_line(&format!("Built image for problem: '{}'", problem.id()));
    Ok(())
}

fn encode_context(path: impl AsRef<Path>) -> SomaResult<Vec<u8>> {
    let compressor = GzEncoder::new(Vec::new(), Compression::default());
    let mut tar = tar::Builder::new(compressor);
    tar.append_dir_all("", path)?;
    tar.finish()?;
    let compressor = tar.into_inner()?;
    Ok(compressor.finish()?)
}

fn build_image(
    env: &Environment<impl Connect, impl Printer>,
    problem: &Problem,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let image_name = problem.docker_image_name();

    env.printer().write_line("Preparing build context...");
    let work_dir = tempdir()?;
    let work_dir_path = work_dir.path();

    // remove the parent directory
    remove_dir_all(&work_dir)?;
    let mut copy_options = CopyOptions::new();
    copy_options.copy_inside = true;
    copy(problem.path(), &work_dir, &copy_options)?;

    env.printer().write_line("Rendering build files...");
    let manifest = problem.load_manifest()?.solidify()?;

    let context = RenderingContext::new(env.username(), problem.repo_name(), manifest);
    Handlebars::new().render_templates(Templates::Binary, &context, work_dir_path)?;

    env.printer().write_line("Encoding build context...");
    let build_context = encode_context(work_dir_path)?;

    work_dir.close()?;
    env.printer().write_line("Building image...");
    runtime.block_on(docker::build(&env, &image_name, build_context))?;
    Ok(())
}

pub fn run(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    port: u32,
    runtime: &mut Runtime,
) -> SomaResult<String> {
    let problem = env.repo_manager().search_prob(prob_query)?;
    let image_name = problem.docker_image_name();
    let port_str = &port.to_string();

    let containers = runtime.block_on(docker::list_containers(&env))?;
    if docker::container_from_prob_running(&containers, &problem) {
        Err(SomaError::ProblemAlreadyRunningError)?
    }

    runtime.block_on(docker::prune_containers_from_prob(&env, &problem))?;

    let labels = docker::image_labels(env, &problem);
    let container_run =
        docker::create(env, labels, &image_name, port_str).and_then(|container_name| {
            env.printer().write_line(&format!("Starting container..."));
            docker::start(env, &container_name).map(|_| container_name)
        });

    env.printer().write_line(&format!(
        "Creating container for problem: '{}'",
        problem.id()
    ));
    let container_name = runtime.block_on(container_run)?;
    env.printer()
        .write_line(&format!("Container started: '{}'", &container_name));

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
        .write_line(&format!("Repository removed: '{}'", &repo_name));

    Ok(())
}

pub fn clean(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;

    let container_list = runtime.block_on(docker::list_containers(env))?;
    if docker::container_from_prob_exists(&container_list, &problem) {
        Err(SomaError::RepositoryInUseError)?;
    }

    runtime.block_on(docker::remove_image(env, &problem.docker_image_name()))?;
    env.printer()
        .write_line(&format!("Problem image cleaned: '{}'", problem.id()));

    Ok(())
}

pub fn stop(
    env: &Environment<impl Connect, impl Printer>,
    prob_query: &str,
    runtime: &mut Runtime,
) -> SomaResult<()> {
    let problem = env.repo_manager().search_prob(prob_query)?;

    let container_list = runtime.block_on(docker::list_containers(env))?;
    if !docker::container_from_prob_exists(&container_list, &problem) {
        Err(SomaError::ProblemNotRunningError)?;
    }

    let container_list = docker::containers_from_prob(container_list, &problem);
    let states_to_stop = &["paused", "restarting", "running"];

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
        .write_line(&format!("Problem stopped: '{}'", problem.id()));

    Ok(())
}
